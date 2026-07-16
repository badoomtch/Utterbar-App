<?php

namespace App\Http\Controllers;

use App\Mail\UpdateSubscribeConfirm;
use App\Models\UpdateSubscriber;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Mail;
use Illuminate\Support\Str;

class UpdatesController extends Controller
{
    /**
     * POST /api/updates/subscribe
     *
     * Called from the Tauri app Settings → "Email me when there's a new release".
     * Idempotent — re-subscribing a known email resends the confirmation email.
     * Deliberately ambiguous on state ("check your inbox") to avoid enumeration.
     */
    public function subscribe(Request $request): JsonResponse
    {
        $data = $request->validate([
            'email'  => ['required', 'email:rfc', 'max:255'],
            'source' => ['nullable', 'string', 'max:32'],
        ]);

        $email  = strtolower(trim($data['email']));
        $source = $data['source'] ?? 'app';

        $sub = UpdateSubscriber::where('email', $email)->first();

        if ($sub && $sub->isActive()) {
            // Already on the list and confirmed — just say "ok".
            return response()->json([
                'ok' => true,
                'message' => "You're already on the list. Nothing to do.",
            ]);
        }

        if (!$sub) {
            $raw = Str::random(64);
            $sub = UpdateSubscriber::create([
                'email'         => $email,
                'source'        => $source,
                'subscribed_at' => now(),
                // DB only ever sees the hash; plaintext stays in memory
                // on $sub->rawToken for the Mailable to build the URL.
                'token_hash'    => UpdateSubscriber::hashToken($raw),
                'ip'            => $request->ip(),
                'user_agent'    => substr((string) $request->userAgent(), 0, 255),
            ]);
            $sub->rawToken = $raw;
        } elseif ($sub->unsubscribed_at) {
            // Previously unsubscribed — re-opt-in. Fresh token.
            $raw = Str::random(64);
            $sub->token_hash = UpdateSubscriber::hashToken($raw);
            $sub->subscribed_at = now();
            $sub->unsubscribed_at = null;
            $sub->confirmed_at = null;
            $sub->source = $source;
            $sub->ip = $request->ip();
            $sub->user_agent = substr((string) $request->userAgent(), 0, 255);
            $sub->save();
            $sub->rawToken = $raw;
        } else {
            // Existing unconfirmed subscriber: re-send confirmation without
            // rotating the token (the mailable uses whatever URL we mint
            // below, but for unconfirmed rows the token they received first
            // time is still the valid one). We don't have plaintext anymore
            // — re-mint + store new hash. Mirrors the "previously
            // unsubscribed" path deliberately so a single code path is
            // responsible for refreshing the token.
            $raw = Str::random(64);
            $sub->token_hash = UpdateSubscriber::hashToken($raw);
            $sub->save();
            $sub->rawToken = $raw;
        }

        try {
            Mail::to($sub->email)->send(new UpdateSubscribeConfirm($sub));
        } catch (\Throwable $e) {
            Log::warning('Update subscribe confirmation email failed', [
                'email' => $sub->email,
                'err'   => $e->getMessage(),
            ]);
        }

        return response()->json([
            'ok' => true,
            'message' => "Check your inbox to confirm.",
        ]);
    }

    /**
     * GET /updates/confirm?t=<token>
     * One-click confirmation (no form submission required — avoids email-client
     * prefetch issues that would otherwise trigger a POST).
     */
    /**
     * GET /updates/confirm?t=<token>
     *
     * Renders a one-button confirmation page rather than confirming
     * straight from the GET. Email clients (Outlook safe-link, some
     * corporate scanners) prefetch URL targets in incoming messages —
     * a GET that mutated state would burn the token before the user
     * ever clicked. The button POSTs to the same route to actually
     * commit the confirmation.
     *
     * The lookup-only check on GET is cheap and lets us tell the user
     * up-front when their link has already expired or been used,
     * before they even click.
     */
    public function confirmPage(Request $request)
    {
        $token = (string) $request->query('t', '');
        if ($token === '') {
            return $this->invalidConfirm();
        }
        $hash = UpdateSubscriber::hashToken($token);
        $sub = UpdateSubscriber::where('token_hash', $hash)->first();
        if (!$sub || $sub->unsubscribed_at) {
            return $this->invalidConfirm();
        }
        // Already confirmed? Render the success state directly so a
        // second click on the same link doesn't show "invalid".
        if ($sub->confirmed_at) {
            return view('pages.updates-status', [
                'status' => 'ok',
                'title'  => "You're already on the list.",
                'body'   => "Nothing to do. We'll email you from hello@ddraig.dev when there's a new release worth installing.",
            ]);
        }
        return view('pages.updates-confirm', [
            'token' => $token,
        ]);
    }

    /**
     * POST /updates/confirm
     *
     * The real state-mutating endpoint. Reached by clicking the button
     * on the confirmation page. CSRF is enforced by Laravel's default
     * `web` middleware so a third-party page can't drive this. The
     * token in the form body is the one from the original sign-up
     * email; we hash + look it up the same way GET does.
     */
    public function confirmPost(Request $request)
    {
        $token = (string) $request->input('t', '');
        if ($token === '') {
            return $this->invalidConfirm();
        }
        $hash = UpdateSubscriber::hashToken($token);

        // Atomic confirm: transaction + row lock so the confirmation
        // can't race with a re-subscribe that rotates the token. The
        // lookup is by token_hash — plaintext lives only in email URLs.
        $ok = DB::transaction(function () use ($hash) {
            $sub = UpdateSubscriber::where('token_hash', $hash)->lockForUpdate()->first();
            if (!$sub || $sub->unsubscribed_at) {
                return false;
            }
            if (!$sub->confirmed_at) {
                $sub->confirmed_at = now();
                $sub->save();
            }
            return true;
        });

        if (!$ok) {
            return $this->invalidConfirm();
        }

        return view('pages.updates-status', [
            'status' => 'ok',
            'title'  => "You're on the list.",
            'body'   => "We'll email you from hello@ddraig.dev when there's a new release worth installing.",
        ]);
    }

    private function invalidConfirm()
    {
        return view('pages.updates-status', [
            'status' => 'error',
            'title'  => 'That confirmation link is no longer valid.',
            'body'   => 'Subscribe again from the app if you want to be on the list.',
        ]);
    }

    /**
     * GET /updates/unsubscribe?t=<token>
     * Single-click unsubscribe per RFC 8058 norms — no confirmation page,
     * no "are you sure", just done.
     */
    public function unsubscribe(Request $request)
    {
        $token = (string) $request->query('t', '');
        if ($token === '') {
            return $this->invalidUnsubscribe();
        }
        $hash = UpdateSubscriber::hashToken($token);

        $ok = DB::transaction(function () use ($hash) {
            $sub = UpdateSubscriber::where('token_hash', $hash)->lockForUpdate()->first();
            if (!$sub) {
                return false;
            }
            if (!$sub->unsubscribed_at) {
                $sub->unsubscribed_at = now();
                $sub->save();
            }
            return true;
        });

        if (!$ok) {
            return $this->invalidUnsubscribe();
        }

        return view('pages.updates-status', [
            'status' => 'ok',
            'title'  => "Unsubscribed.",
            'body'   => "You won't receive any more release emails. Sorry to see you go.",
        ]);
    }

    private function invalidUnsubscribe()
    {
        return view('pages.updates-status', [
            'status' => 'error',
            'title'  => 'That unsubscribe link is no longer valid.',
            'body'   => 'If you keep receiving emails, reply to any of them and we\'ll take you off manually.',
        ]);
    }
}
