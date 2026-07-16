<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Mail\LicenceDelivery;
use App\Models\Activation;
use App\Models\Licence;
use App\Services\Licence\LicenceKeys;
use App\Services\Licence\LicenceSigner;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Carbon;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Mail;

/**
 * Admin-side licence management. Covers everything the artisan commands
 * (licence:list, licence:resend, licence:revoke, licence:unrevoke,
 * licence:remint) do, plus a browser-based minting form so an operator
 * can hand out a licence without punching commands into SSH.
 *
 * Every destructive action is a POST. Route protection is the 'admin.email'
 * middleware layered on top of 'auth' (see routes/web.php).
 */
class LicenceController extends Controller
{
    public function index(Request $request)
    {
        $q      = trim((string) $request->query('q', ''));
        $status = (string) $request->query('status', '');

        $licences = Licence::query()
            ->when($q !== '', function ($qb) use ($q) {
                // Search by email, display key, or stripe IDs. Normalise the
                // key_lookup so "UB-XXXX-..." input matches what's in the DB.
                $keyNormalised = strtoupper(preg_replace('/[\s\-]+/', '', $q) ?? '');
                if (str_starts_with($keyNormalised, 'UB')) {
                    $keyNormalised = substr($keyNormalised, 2);
                }
                $qb->where(function ($w) use ($q, $keyNormalised) {
                    $w->where('email', 'like', "%{$q}%")
                      ->orWhere('key_lookup', $keyNormalised)
                      ->orWhere('stripe_checkout_session_id', $q)
                      ->orWhere('stripe_customer_id', $q);
                });
            })
            ->when(in_array($status, ['active', 'revoked', 'refunded'], true), fn ($qb) => $qb->where('status', $status))
            ->withCount('activations')
            ->orderByDesc('id')
            ->paginate(25)
            ->withQueryString();

        return view('pages.admin.licences.index', [
            'licences' => $licences,
            'q'        => $q,
            'status'   => $status,
        ]);
    }

    public function create()
    {
        return view('pages.admin.licences.create', [
            'default' => [
                'email'       => '',
                'sku'         => config('services.licence.sku_string', 'utterbar-pro-lifetime'),
                'max_devices' => (int) config('services.licence.max_devices', 3),
                'send_email'  => true,
                'note'        => '',
            ],
        ]);
    }

    /**
     * Mint a licence without a Stripe session behind it. Used for
     * complimentary copies, press keys, or re-issuing a licence that got
     * lost before we had the webhook wired up. Mirrors the mint flow in
     * StripeWebhookController::handleCheckoutSessionCompleted so the row
     * is indistinguishable downstream.
     */
    public function store(Request $request): RedirectResponse
    {
        $data = $request->validate([
            'email'       => ['required', 'email:rfc', 'max:255'],
            'sku'         => ['required', 'string', 'max:64'],
            'max_devices' => ['required', 'integer', 'min:1', 'max:50'],
            'send_email'  => ['nullable', 'boolean'],
            'note'        => ['nullable', 'string', 'max:500'],
        ]);

        $email = strtolower(trim($data['email']));

        // Soft-warn on a probable duplicate, but let the operator push through
        // via the `confirm_duplicate` flag. An "issue a second licence for X"
        // is a legitimate but rare operation — don't make it easy to do by
        // accident.
        $existing = Licence::where('email', $email)->where('status', 'active')->exists();
        if ($existing && !$request->boolean('confirm_duplicate')) {
            return back()->withInput()->withErrors([
                'email' => "An active licence already exists for {$email}. Tick 'confirm duplicate' if you really want a second one.",
            ]);
        }

        $licence = DB::transaction(function () use ($data, $email) {
            $licence = Licence::create([
                'key_lookup'  => LicenceKeys::generateLookup(),
                'email'       => $email,
                'sku'         => $data['sku'],
                'max_devices' => (int) $data['max_devices'],
                'status'      => 'active',
                'issued_at'   => Carbon::now(),
            ]);
            $licence->signed_blob = LicenceSigner::sign($licence);
            $licence->save();
            return $licence;
        });

        Log::info('admin minted licence', [
            'licence_id' => $licence->id,
            'email'      => $licence->email,
            'note'       => $data['note'] ?? null,
            'actor'      => optional($request->user())->email,
        ]);

        $flash = ['type' => 'ok', 'text' => "Licence {$licence->displayKey()} minted for {$licence->email}."];

        if ($request->boolean('send_email')) {
            try {
                Mail::to($licence->email)->send(new LicenceDelivery($licence));
                $flash['text'] .= ' Licence email sent.';
            } catch (\Throwable $e) {
                Log::warning('Admin-mint licence email failed', ['licence' => $licence->id, 'err' => $e->getMessage()]);
                $flash['type'] = 'error';
                $flash['text'] .= ' BUT email delivery failed. Use Resend on the licence page.';
            }
        }

        return redirect()->route('admin.licences.show', $licence->id)->with('flash', $flash);
    }

    public function show(int $id)
    {
        $licence = Licence::with(['activations' => fn ($q) => $q->orderByDesc('last_heartbeat')])
            ->findOrFail($id);

        return view('pages.admin.licences.show', ['licence' => $licence]);
    }

    public function revoke(Request $request, int $id): RedirectResponse
    {
        $data = $request->validate([
            'reason' => ['nullable', 'string', 'max:255'],
        ]);

        $licence = Licence::findOrFail($id);
        if ($licence->status === 'revoked') {
            return redirect()->route('admin.licences.show', $licence->id)->with('flash', [
                'type' => 'error', 'text' => 'Already revoked.',
            ]);
        }

        $licence->status = 'revoked';
        $licence->revoked_reason = $data['reason'] ?? null;
        $licence->revoked_at = now();
        $licence->save();

        Log::info('admin revoked licence', [
            'licence_id' => $licence->id,
            'actor'      => optional($request->user())->email,
            'reason'     => $licence->revoked_reason,
        ]);

        return redirect()->route('admin.licences.show', $licence->id)->with('flash', [
            'type' => 'ok',
            'text' => 'Licence revoked. Devices will disable on next heartbeat.',
        ]);
    }

    public function unrevoke(Request $request, int $id): RedirectResponse
    {
        $licence = Licence::findOrFail($id);
        if ($licence->status !== 'revoked') {
            return redirect()->route('admin.licences.show', $licence->id)->with('flash', [
                'type' => 'error', 'text' => 'Not revoked. Nothing to undo.',
            ]);
        }

        $licence->status = 'active';
        $licence->revoked_reason = null;
        $licence->revoked_at = null;
        $licence->save();

        Log::info('admin unrevoked licence', [
            'licence_id' => $licence->id,
            'actor'      => optional($request->user())->email,
        ]);

        return redirect()->route('admin.licences.show', $licence->id)->with('flash', [
            'type' => 'ok', 'text' => 'Licence re-activated.',
        ]);
    }

    public function resend(Request $request, int $id): RedirectResponse
    {
        $licence = Licence::findOrFail($id);

        $data = $request->validate([
            'to' => ['nullable', 'email:rfc', 'max:255'],
        ]);
        $to = $data['to'] ?? $licence->email;

        try {
            Mail::to($to)->send(new LicenceDelivery($licence));
        } catch (\Throwable $e) {
            return redirect()->route('admin.licences.show', $licence->id)->with('flash', [
                'type' => 'error',
                'text' => "Email delivery failed: {$e->getMessage()}",
            ]);
        }

        return redirect()->route('admin.licences.show', $licence->id)->with('flash', [
            'type' => 'ok',
            'text' => "Licence email sent to {$to}.",
        ]);
    }

    /**
     * Re-sign a single licence with the current private key. Mostly useful
     * right after a key rotation; the bulk path is `artisan licence:remint --all`.
     */
    public function remint(Request $request, int $id): RedirectResponse
    {
        $licence = Licence::findOrFail($id);
        try {
            $licence->signed_blob = LicenceSigner::sign($licence);
            $licence->save();
        } catch (\Throwable $e) {
            return redirect()->route('admin.licences.show', $licence->id)->with('flash', [
                'type' => 'error', 'text' => "Re-sign failed: {$e->getMessage()}",
            ]);
        }

        return redirect()->route('admin.licences.show', $licence->id)->with('flash', [
            'type' => 'ok', 'text' => 'Licence re-signed with the current private key.',
        ]);
    }

    public function deactivateDevice(Request $request, int $id, string $fingerprint): RedirectResponse
    {
        $licence = Licence::findOrFail($id);

        $deleted = Activation::where('licence_id', $licence->id)
            ->where('fingerprint_hash', strtolower($fingerprint))
            ->delete();

        Log::info('admin deactivated device', [
            'licence_id' => $licence->id,
            'fingerprint' => substr(strtolower($fingerprint), 0, 12) . '...',
            'actor'      => optional($request->user())->email,
        ]);

        return redirect()->route('admin.licences.show', $licence->id)->with('flash', [
            'type' => $deleted ? 'ok' : 'error',
            'text' => $deleted
                ? 'Device deactivated. Slot is now free.'
                : 'Device not found (probably already deactivated).',
        ]);
    }

    /**
     * Download the signed licence as a binary file for managed fleet
     * deployment. The file content is the same 96-byte Ed25519-signed
     * blob the desktop app already verifies offline; the IT admin
     * drops it at `%APPDATA%\Utterbar\licence.bin` on managed machines
     * and Utterbar treats it as the active Pro licence at startup.
     *
     * No new format, no new trust model: this is the existing
     * `signed_blob` column served as `application/octet-stream` with
     * a predictable filename. Admin auth is enforced by the route
     * middleware (`auth` + `admin.email`) configured in routes/web.php.
     *
     * If the licence has no signed blob yet (a row created before the
     * signer ran, or a remint mid-flight), 404 the download rather
     * than serving zero bytes; the admin can hit the "Re-sign" action
     * on the detail page to populate it.
     */
    public function download(Request $request, int $id)
    {
        $licence = Licence::findOrFail($id);

        if (empty($licence->signed_blob)) {
            abort(404, 'Licence has no signed blob yet. Re-sign on the detail page first.');
        }

        Log::info('admin downloaded licence.bin', [
            'licence_id' => $licence->id,
            'actor'      => optional($request->user())->email,
            'bytes'      => strlen($licence->signed_blob),
        ]);

        $filename = "utterbar-licence-{$licence->id}.bin";

        return response($licence->signed_blob, 200, [
            'Content-Type'              => 'application/octet-stream',
            'Content-Disposition'       => 'attachment; filename="' . $filename . '"',
            'Content-Length'            => (string) strlen($licence->signed_blob),
            'X-Content-Type-Options'    => 'nosniff',
            // Don't let intermediaries cache a binary that the admin
            // expects to download fresh each time. Same posture as
            // the licence-resend mail flow, just enforced at the HTTP
            // layer rather than the mail headers.
            'Cache-Control'             => 'private, no-store, no-cache, must-revalidate',
        ]);
    }

    /**
     * Air-gapped activation: register a device against this licence
     * from a JSON request file the user exported on an offline machine.
     *
     * The desktop's Settings → Licence → "Export activation request"
     * writes a JSON document with format magic
     * `utterbar-licence-request`, format_version 1, the machine
     * fingerprint, device name, platform, app_version and
     * generated_at. The user hand-carries the file to their IT admin,
     * who uploads it here. We validate the format, enforce
     * `max_devices`, and create an Activation row identical in shape
     * to an online activation EXCEPT `offline = true` and
     * `last_heartbeat` stays null.
     *
     * Idempotent on `(licence_id, fingerprint_hash)` — the unique
     * index on the table already enforces this; on a duplicate
     * upload we surface a friendly "already registered" flash
     * rather than a 500. The admin then clicks the existing
     * "Download licence.bin" button to get the file to send back.
     *
     * Hard cap: file size 4 KB. The legitimate document is around
     * 250 bytes; 4 KB leaves headroom for future fields without
     * letting an attacker waste server memory parsing JSON.
     *
     * Audit: every upload is logged with the actor email, licence
     * id, and the first 12 chars of the fingerprint. Full
     * fingerprint stays in the activations row.
     */
    public function uploadActivationRequest(Request $request, int $id): RedirectResponse
    {
        $licence = Licence::findOrFail($id);

        // File size + presence. A missing file or one over 4 KB
        // hits 422 before we touch the JSON parser.
        $request->validate([
            'request_file' => ['required', 'file', 'max:4'], // KB
        ]);

        $file = $request->file('request_file');
        $contents = (string) file_get_contents($file->getRealPath());

        $payload = json_decode($contents, associative: true);
        if (!is_array($payload)) {
            return redirect()
                ->route('admin.licences.show', $licence->id)
                ->with('flash', [
                    'type' => 'error',
                    'text' => 'Activation request: not valid JSON.',
                ]);
        }

        $errors = $this->validateActivationRequest($payload);
        if ($errors !== null) {
            return redirect()
                ->route('admin.licences.show', $licence->id)
                ->with('flash', [
                    'type' => 'error',
                    'text' => 'Activation request rejected: ' . $errors,
                ]);
        }

        $fingerprint = strtolower((string) $payload['fingerprint']);

        // Idempotent re-upload: if a row already exists for this
        // (licence, fingerprint), surface a clear "already
        // registered" flash and bail out without creating a second
        // row. The unique index on the table would 23000-error
        // anyway; this short-circuit gives the admin a friendlier
        // message.
        $existing = Activation::where('licence_id', $licence->id)
            ->where('fingerprint_hash', $fingerprint)
            ->first();
        if ($existing !== null) {
            return redirect()
                ->route('admin.licences.show', $licence->id)
                ->with('flash', [
                    'type' => 'ok',
                    'text' => 'Activation already registered for this device. Re-download licence.bin if needed.',
                ]);
        }

        // Max devices: count current activations against the
        // licence cap. Same enforcement as the online /activate
        // endpoint, just on the admin upload path.
        $current = Activation::where('licence_id', $licence->id)->count();
        if ($current >= (int) $licence->max_devices) {
            return redirect()
                ->route('admin.licences.show', $licence->id)
                ->with('flash', [
                    'type' => 'error',
                    'text' => "Device limit reached ({$current}/{$licence->max_devices}). Deactivate a device or raise max_devices first.",
                ]);
        }

        $activation = Activation::create([
            'licence_id'       => $licence->id,
            'fingerprint_hash' => $fingerprint,
            'device_name'      => substr((string) $payload['device_name'], 0, 100),
            'platform'         => (string) $payload['platform'],
            'app_version'      => substr((string) $payload['app_version'], 0, 32),
            'activation_token' => bin2hex(random_bytes(32)),
            'first_seen'       => now(),
            'last_heartbeat'   => null,
            'offline'          => true,
        ]);

        Log::info('admin minted offline activation', [
            'licence_id'  => $licence->id,
            'fingerprint' => substr($fingerprint, 0, 12) . '...',
            'device_name' => $activation->device_name,
            'app_version' => $activation->app_version,
            'actor'       => optional($request->user())->email,
        ]);

        return redirect()
            ->route('admin.licences.show', $licence->id)
            ->with('flash', [
                'type' => 'ok',
                'text' => "Offline activation registered for {$activation->device_name}. Click \"Download licence.bin\" to send the file to the user.",
            ]);
    }

    /**
     * Validate the activation-request JSON shape. Returns null on
     * success, or a short error string suitable for flashing to
     * the operator. Kept separate from the controller body so the
     * tests can drive the validator directly without juggling
     * Request fakes.
     */
    private function validateActivationRequest(array $payload): ?string
    {
        if (($payload['format'] ?? null) !== 'utterbar-licence-request') {
            return 'wrong format magic';
        }
        if (($payload['format_version'] ?? null) !== 1) {
            return 'unsupported format_version';
        }

        $fingerprint = $payload['fingerprint'] ?? null;
        if (!is_string($fingerprint) || !preg_match('/^[a-f0-9]{64}$/', $fingerprint)) {
            return 'fingerprint must be 64 lowercase hex chars';
        }

        $device = $payload['device_name'] ?? null;
        if (!is_string($device) || $device === '' || mb_strlen($device) > 100) {
            return 'device_name must be 1-100 chars';
        }

        $platform = $payload['platform'] ?? null;
        if (!is_string($platform) || !in_array($platform, ['windows'], true)) {
            return 'platform must be one of: windows';
        }

        $appVersion = $payload['app_version'] ?? null;
        if (!is_string($appVersion) || $appVersion === '' || mb_strlen($appVersion) > 32) {
            return 'app_version must be 1-32 chars';
        }

        // generated_at is informational; if present it must parse
        // as a non-empty string. We don't reject stale timestamps;
        // the operator can read the value for themselves.
        if (array_key_exists('generated_at', $payload)) {
            $g = $payload['generated_at'];
            if (!is_string($g) || $g === '') {
                return 'generated_at, if present, must be a non-empty string';
            }
        }

        return null;
    }
}
