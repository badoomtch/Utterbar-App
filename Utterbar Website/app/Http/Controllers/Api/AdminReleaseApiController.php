<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Mail\UpdateAnnouncement;
use App\Models\Release;
use App\Models\UpdateSubscriber;
use App\Services\Releases\UpdateMetadataSigner;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Mail;
use Illuminate\Support\Facades\Storage;

/**
 * Headless release publishing for automated builders (the Tauri app's release
 * script). Equivalent to the admin form at /admin/releases/create but callable
 * without a browser session. Auth is a bearer token (admin.api middleware).
 *
 * Workflow from the builder side:
 *   1. Upload Utterbar-Setup.exe to R2 at releases/{version}/Utterbar-Setup.exe
 *   2. Compute SHA-256 locally
 *   3. POST /api/admin/releases with a JSON body (schema below)
 *   4. Optionally announce by setting announce=true in the body
 *
 * See LICENCE_APP_HANDOFF.md and Orliq/RELEASE.md for the builder-side guide.
 */
class AdminReleaseApiController extends Controller
{
    /**
     * POST /api/admin/releases
     */
    public function store(Request $request): JsonResponse
    {
        // Version: strict three-segment semver (X.Y.Z). Prerelease suffixes
        // are rejected because `/api/updater/check` cannot compare them —
        // use the beta channel for pre-GA builds. Admin form + this API
        // intentionally share the same regex so publishing rules stay
        // consistent across both entry points.
        //
        // SHA-256: REQUIRED. The desktop updater refuses to install any
        // release without a hash (see updater.rs install_update), so a
        // release row with null sha256 cannot be auto-installed. We
        // reject at publish rather than silently persist a dead row.
        $data = $request->validate([
            'version'            => ['required', 'string', 'max:32', 'regex:/^\d+\.\d+\.\d+$/'],
            'channel'            => ['nullable', 'in:stable,beta'],
            'r2_key'             => ['nullable', 'string', 'max:255'],
            'tag'                => ['nullable', 'string', 'max:64'],
            'sha256'             => ['required', 'string', 'regex:/^[A-Fa-f0-9]{64}$/'],
            'notes_markdown'     => ['required', 'string'],
            'announce'           => ['nullable', 'boolean'],
            'announce_subject'   => ['nullable', 'string', 'max:200'],
            // Base64-encoded Ed25519 signature (64 raw bytes = 88 chars b64) over
            // the canonical metadata blob. Required when an updater public key
            // is configured — the publish script signs locally and sends the
            // signature here so the server never needs the private key.
            'updater_signature'  => ['nullable', 'string', 'max:128'],
        ], [
            'version.regex' => 'Version must be three numeric segments, e.g. 0.6.2. Pre-release suffixes (-rc.1, -beta) are not supported; use the beta channel for pre-GA builds.',
            'sha256.required' => 'SHA-256 is required — releases without a hash cannot be auto-installed by the desktop updater.',
            'sha256.regex'  => 'SHA-256 must be 64 hex chars.',
        ]);

        $channel = $data['channel'] ?? 'stable';

        // Default r2_key convention: releases/{version}/Utterbar-Setup-{version}.exe
        // (the versioned filename is canon so a user dragging the installer
        // anywhere can still tell which build it is). publish-release.sh
        // always sends r2_key explicitly — this default is only hit by
        // manual POSTs that omitted it.
        $r2Key = $data['r2_key'] ?? "releases/{$data['version']}/Utterbar-Setup-{$data['version']}.exe";
        $r2Key = ltrim($r2Key, '/');

        if (Release::where('version', $data['version'])->where('channel', $channel)->exists()) {
            return response()->json([
                'error'   => 'version_exists',
                'message' => "A {$channel} release with version {$data['version']} already exists.",
            ], 409);
        }

        $disk = Storage::disk('r2');
        if (!$disk->exists($r2Key)) {
            return response()->json([
                'error'   => 'file_not_in_bucket',
                'message' => "No file at R2 key: {$r2Key}. Upload it first.",
                'meta'    => ['r2_key' => $r2Key],
            ], 422);
        }

        $size = $disk->size($r2Key);

        // Build the canonical download URL the same way UpdaterApiController
        // does (single source of truth: r2.url config). The updater signature
        // is computed over a metadata blob that includes this URL — keeping
        // both sides identical ensures the client's reconstruction matches.
        $cdnBase = (string) (config('filesystems.disks.r2.url') ?: env('CLOUDFLARE_R2_PUBLIC_URL', 'https://dl.utterbar.com'));
        $downloadUrl = rtrim($cdnBase, '/') . '/' . ltrim($r2Key, '/');

        // Updater signature handling. When an updater public key is
        // configured (production), a signature is REQUIRED — pushing an
        // unsigned release would mean clients refuse to install it,
        // creating a silent dead-row. Reject at publish time instead.
        // When no key is configured (local dev / tests), accept whatever
        // the operator sent (including nothing) so the test suite doesn't
        // require key material.
        $updaterSig = $data['updater_signature'] ?? null;
        $hasUpdaterKey = (bool) config('services.updater.public_key_b64');
        if ($hasUpdaterKey) {
            if (!is_string($updaterSig) || $updaterSig === '') {
                return response()->json([
                    'error'   => 'updater_signature_required',
                    'message' => 'Releases must include an updater_signature when UPDATER_PUBLIC_KEY_B64 is set. Use scripts/sign-update-metadata.sh.',
                ], 422);
            }
            $signedOk = UpdateMetadataSigner::verify($updaterSig, [
                'version'      => $data['version'],
                'channel'      => $channel,
                'size_bytes'   => (int) $size,
                'sha256'       => strtolower($data['sha256']),
                'download_url' => $downloadUrl,
            ]);
            if (!$signedOk) {
                return response()->json([
                    'error'   => 'updater_signature_invalid',
                    'message' => 'updater_signature does not verify against the canonical metadata blob. Check that the operator key matches UPDATER_PUBLIC_KEY_B64 and that nothing changed (sha256, r2_key, size) between signing and upload.',
                ], 422);
            }
        }

        $release = Release::create([
            'version'           => $data['version'],
            'channel'           => $channel,
            'r2_key'            => $r2Key,
            'file_size_bytes'   => $size,
            'sha256'            => strtolower($data['sha256']),
            'updater_signature' => $updaterSig,
            'tag'               => $data['tag'] ?? null,
            'notes_markdown'    => $data['notes_markdown'],
            'published_at'      => now(),
        ]);

        $announced = 0;
        if ($request->boolean('announce') && $channel === 'stable') {
            $subject = trim((string) ($data['announce_subject'] ?? '')) ?: "Utterbar {$release->version} is out";
            $announced = $this->blastAnnouncement($release, $subject);
            $release->announced_at = now();
            $release->save();
        }

        Log::info('release published via API', [
            'id' => $release->id, 'version' => $release->version, 'channel' => $release->channel,
            'announced_to' => $announced,
        ]);

        return response()->json([
            'ok'             => true,
            'id'             => $release->id,
            'version'        => $release->version,
            'channel'        => $release->channel,
            'download_url'   => $release->downloadUrl(),
            'size_bytes'     => $release->file_size_bytes,
            'announced_to'   => $announced,
            'admin_url'      => url('/admin/releases'),
        ], 201);
    }

    /**
     * DELETE /api/admin/releases/{id}?delete_file=1
     */
    public function destroy(Request $request, int $id): JsonResponse
    {
        $release = Release::find($id);
        if (!$release) {
            return response()->json(['error' => 'not_found', 'message' => 'No release with that id.'], 404);
        }

        $deleteFile = $request->boolean('delete_file');
        if ($deleteFile) {
            try {
                Storage::disk('r2')->delete($release->r2_key);
            } catch (\Throwable $e) {
                Log::warning('R2 delete failed during release destroy', ['key' => $release->r2_key, 'err' => $e->getMessage()]);
            }
        }

        $release->delete();

        return response()->json([
            'ok' => true,
            'deleted_file' => $deleteFile,
            'version' => $release->version,
        ]);
    }

    private function blastAnnouncement(Release $release, string $subject): int
    {
        $subs = UpdateSubscriber::active()->get();
        $sent = 0;
        foreach ($subs as $sub) {
            // Staged rotation: DB hash only advances on a successful send,
            // so a mid-blast SMTP failure never orphans a subscriber with
            // a dead unsubscribe link. See UpdateSubscriber::prepareRotation().
            $sub->prepareRotation();
            try {
                Mail::to($sub->email)->send(new UpdateAnnouncement(
                    subscriber:      $sub,
                    release:         $release->version,
                    announceSubject: $subject,
                    notesMd:         $release->notes_markdown,
                ));
                $sub->commitRotation();
                $sent++;
            } catch (\Throwable $e) {
                $sub->discardRotation();
                Log::warning('Release announcement email failed', [
                    'email' => $sub->email, 'err' => $e->getMessage(),
                ]);
            }
        }
        return $sent;
    }
}
