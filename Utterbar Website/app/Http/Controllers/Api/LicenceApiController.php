<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Models\Activation;
use App\Models\Licence;
use App\Services\Licence\LicenceKeys;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;

/**
 * Endpoints the Tauri app calls to fetch its signed licence blob, activate
 * itself on a device, release a device, and heartbeat. The contract is
 * described in full in LICENCE_APP_HANDOFF.md §4.
 *
 * All endpoints:
 *   - accept and return JSON
 *   - normalise the licence key (strip dashes, uppercase, strip "UB" prefix)
 *   - return errors as { error: slug, message: human, meta: mixed|null }
 *   - use key + fingerprint as auth — activation_token is an opaque cache value
 *     returned to the app and not required on subsequent calls.
 */
class LicenceApiController extends Controller
{
    /**
     * POST /api/licence/fetch
     * Exchange the user-pasted short key for the 96-byte signed blob.
     */
    public function fetch(Request $request): JsonResponse
    {
        $data = $request->validate([
            'key' => 'required|string|max:64',
        ]);

        $licence = $this->findLicenceByRawKey($data['key']);
        if (!$licence) {
            return $this->errorJson(404, 'licence_not_found', "We couldn't find that licence.");
        }
        if ($licence->status !== 'active') {
            return $this->errorJson(410, 'licence_revoked', 'This licence is no longer active.', [
                'reason' => $licence->revoked_reason,
            ]);
        }
        if (!$licence->signed_blob) {
            // Shouldn't happen — every licence is signed at mint time — but guard anyway.
            return $this->errorJson(500, 'licence_unsigned', 'Licence is missing its signed payload. Contact support.');
        }

        return response()->json([
            'signed_blob' => base64_encode($licence->signed_blob),
            'licence_id'  => $licence->id,
            'sku'         => $licence->sku,
            'issued_at'   => $licence->issued_at?->toIso8601ZuluString(),
            'max_devices' => (int) $licence->max_devices,
        ]);
    }

    /**
     * POST /api/licence/activate
     * Record the device + fingerprint, enforce 3-device cap. Idempotent on
     * (licence_id, fingerprint_hash) — re-activating the same device updates
     * last_heartbeat and returns the existing activation_token.
     */
    public function activate(Request $request): JsonResponse
    {
        $data = $request->validate([
            'key'         => 'required|string|max:64',
            'fingerprint' => 'required|string|regex:/^[a-f0-9]{64}$/',
            'device_name' => 'required|string|max:100',
            'platform'    => 'nullable|string|max:32',
            'app_version' => 'nullable|string|max:32',
        ]);

        $licence = $this->findLicenceByRawKey($data['key']);
        if (!$licence)                    return $this->errorJson(404, 'licence_not_found', "We couldn't find that licence.");
        if ($licence->status !== 'active') return $this->errorJson(410, 'licence_revoked', 'This licence is no longer active.', ['reason' => $licence->revoked_reason]);

        $fingerprint = strtolower($data['fingerprint']);

        // Serialise activation decisions per-licence so two concurrent activates
        // on different fingerprints can't both slip past the 3-device cap.
        $result = DB::transaction(function () use ($licence, $fingerprint, $data, $request) {
            // Lock the licence row for the duration.
            $licence = Licence::whereKey($licence->id)->lockForUpdate()->first();

            $existing = Activation::where('licence_id', $licence->id)
                ->where('fingerprint_hash', $fingerprint)
                ->first();

            if ($existing) {
                // Same device — update heartbeat + metadata, return 200.
                $existing->device_name    = $data['device_name'];
                $existing->platform       = $data['platform'] ?? $existing->platform;
                $existing->app_version    = $data['app_version'] ?? $existing->app_version;
                $existing->last_heartbeat = now();
                $existing->last_ip        = $request->ip();
                $existing->last_user_agent = substr((string) $request->userAgent(), 0, 255);
                $existing->save();
                return ['status' => 'existing', 'activation' => $existing];
            }

            // Cap check.
            $used = $licence->activations()->count();
            if ($used >= (int) $licence->max_devices) {
                return ['status' => 'cap', 'activation' => null];
            }

            // New activation.
            $activation = Activation::create([
                'licence_id'       => $licence->id,
                'fingerprint_hash' => $fingerprint,
                'device_name'      => $data['device_name'],
                'platform'         => $data['platform'] ?? 'windows',
                'app_version'      => $data['app_version'] ?? null,
                'activation_token' => Str::random(64),
                'first_seen'       => now(),
                'last_heartbeat'   => now(),
                'last_ip'          => $request->ip(),
                'last_user_agent'  => substr((string) $request->userAgent(), 0, 255),
            ]);
            return ['status' => 'new', 'activation' => $activation];
        });

        if ($result['status'] === 'cap') {
            $devices = $licence->activations()->get()->map->toPublicArray($fingerprint)->values();
            return $this->errorJson(409, 'device_limit_reached',
                "This licence is already in use on {$licence->max_devices} devices.",
                [
                    'devices'    => $devices,
                    'manage_url' => url('/account'),
                ]);
        }

        /** @var Activation $activation */
        $activation = $result['activation'];
        $devices = $licence->activations()->get()->map->toPublicArray($fingerprint)->values();

        Log::info('licence activate', [
            'licence_id' => $licence->id,
            'fingerprint_prefix' => substr($fingerprint, 0, 8),
            'device_name' => $activation->device_name,
            'new' => $result['status'] === 'new',
        ]);

        return response()->json([
            'activation_token' => $activation->activation_token,
            'devices'          => $devices,
            'remaining_slots'  => max(0, $licence->max_devices - $devices->count()),
        ]);
    }

    /**
     * POST /api/licence/deactivate
     * Remove a device slot. Auth is key + fingerprint — if you know both, you
     * own the device enough to deactivate it.
     */
    public function deactivate(Request $request): JsonResponse
    {
        $data = $request->validate([
            'key'         => 'required|string|max:64',
            'fingerprint' => 'required|string|regex:/^[a-f0-9]{64}$/',
        ]);

        $licence = $this->findLicenceByRawKey($data['key']);
        if (!$licence) return $this->errorJson(404, 'licence_not_found', "We couldn't find that licence.");

        $fingerprint = strtolower($data['fingerprint']);

        $deleted = Activation::where('licence_id', $licence->id)
            ->where('fingerprint_hash', $fingerprint)
            ->delete();

        if ($deleted === 0) {
            // Already deactivated (or never activated). Treat as success for UX.
            Log::info('licence deactivate (no-op)', [
                'licence_id' => $licence->id,
                'fingerprint_prefix' => substr($fingerprint, 0, 8),
            ]);
        } else {
            Log::info('licence deactivate', [
                'licence_id' => $licence->id,
                'fingerprint_prefix' => substr($fingerprint, 0, 8),
            ]);
        }

        $remaining = max(0, $licence->max_devices - $licence->activations()->count());

        return response()->json([
            'ok'              => true,
            'remaining_slots' => $remaining,
        ]);
    }

    /**
     * POST /api/licence/heartbeat
     * Called at most once per 30 days by the app. Updates last-seen and
     * signals revocations via 410.
     */
    public function heartbeat(Request $request): JsonResponse
    {
        $data = $request->validate([
            'key'         => 'required|string|max:64',
            'fingerprint' => 'required|string|regex:/^[a-f0-9]{64}$/',
            'app_version' => 'nullable|string|max:32',
        ]);

        $licence = $this->findLicenceByRawKey($data['key']);
        if (!$licence)                    return $this->errorJson(404, 'licence_not_found', "We couldn't find that licence.");
        if ($licence->status !== 'active') return $this->errorJson(410, 'licence_revoked', 'This licence is no longer active.', ['reason' => $licence->revoked_reason]);

        $fingerprint = strtolower($data['fingerprint']);

        $activation = Activation::where('licence_id', $licence->id)
            ->where('fingerprint_hash', $fingerprint)
            ->first();

        if ($activation) {
            $activation->last_heartbeat = now();
            $activation->last_ip        = $request->ip();
            if (!empty($data['app_version'])) {
                $activation->app_version = $data['app_version'];
            }
            $activation->save();
        } else {
            // Device unknown for this licence. Don't fail — app may have been
            // reinstalled and needs to /activate again. Tell it to re-activate.
            return $this->errorJson(409, 'activation_missing', 'This device isn\'t activated on the licence. Re-activate it.', []);
        }

        return response()->json([
            'status' => 'active',
        ]);
    }

    /* ───── helpers ───── */

    private function findLicenceByRawKey(string $raw): ?Licence
    {
        $canonical = LicenceKeys::normalise($raw);
        if ($canonical === null) return null;
        return Licence::where('key_lookup', $canonical)->first();
    }

    private function errorJson(int $status, string $slug, string $message, mixed $meta = null): JsonResponse
    {
        return response()->json([
            'error'   => $slug,
            'message' => $message,
            'meta'    => $meta,
        ], $status);
    }
}
