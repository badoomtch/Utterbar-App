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
                $flash['text'] .= ' BUT email delivery failed — use Resend on the licence page.';
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
                'type' => 'error', 'text' => 'Not revoked — nothing to undo.',
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
}
