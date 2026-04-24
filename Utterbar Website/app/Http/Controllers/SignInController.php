<?php

namespace App\Http\Controllers;

use App\Mail\SignInLink;
use App\Models\Licence;
use App\Models\SignInToken;
use App\Models\User;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Mail;
use Illuminate\Support\Str;

class SignInController extends Controller
{
    /** Minutes a sign-in token is valid for. */
    private const TOKEN_TTL_MIN = 20;

    public function show(Request $request)
    {
        if (Auth::check()) {
            return redirect()->route('account');
        }
        return view('pages.sign-in');
    }

    /**
     * POST /sign-in — accept email, email a magic link if there's a matching
     * account or licence. Always return the same confirmation page to block
     * email enumeration.
     */
    public function sendLink(Request $request)
    {
        $data = $request->validate([
            'email' => 'required|email:rfc|max:255',
        ]);
        $email = strtolower(trim($data['email']));

        // Always render the confirmation view, regardless of whether we found
        // a matching account. The attacker can't tell "found" from "not found".
        $sentView = view('pages.sign-in-sent', ['email' => $email]);

        $user = User::where('email', $email)->first();
        $hasLicence = Licence::where('email', $email)->exists();

        if (!$user && !$hasLicence) {
            // Nothing to grant access to. Silently do nothing.
            return $sentView;
        }

        // Create the user lazily if they have licences but no user record yet
        // (can happen for purchases made before accounts were introduced).
        if (!$user) {
            $user = User::create([
                'name'     => '',
                'email'    => $email,
                'password' => Hash::make(Str::random(64)),  // unusable password
                'email_verified_at' => now(),
            ]);
        }

        $token = $this->issueToken($email, $request);

        try {
            Mail::to($email)->send(new SignInLink($token, $email));
        } catch (\Throwable $e) {
            Log::error('Sign-in link delivery failed', ['email' => $email, 'err' => $e->getMessage()]);
            // Still show the confirmation page — don't leak delivery failure.
        }

        return $sentView;
    }

    /**
     * GET /sign-in/{token} — consume a magic link, create session, redirect.
     *
     * Atomic + single-use. The guarantee is: at most one HTTP request for a
     * given token ever returns a logged-in session, even under concurrent
     * requests. Implementation:
     *
     *   1. Hash the raw token (the DB only stores the hash — see SignInToken).
     *   2. Inside a DB transaction, SELECT ... FOR UPDATE on the matching row.
     *      Locks that single row against any other concurrent SELECT FOR
     *      UPDATE. The second concurrent request blocks until the first
     *      commits / rolls back.
     *   3. Re-check validity INSIDE the lock (expired, already-used).
     *   4. Mark used, burn sibling tokens for the same email, commit.
     *   5. Only AFTER commit do we log the user in.
     *
     * SQLite's "FOR UPDATE" is a no-op, but SQLite serialises writes anyway
     * via the journal, so the `used_at` check inside the transaction still
     * rejects the second writer. MySQL InnoDB honours FOR UPDATE properly.
     */
    public function consume(Request $request, string $token): RedirectResponse
    {
        $hash = SignInToken::hashToken($token);

        $row = DB::transaction(function () use ($hash) {
            // lockForUpdate: SELECT ... FOR UPDATE. Concurrent consume()
            // calls on the same token serialise here.
            $r = SignInToken::where('token_hash', $hash)->lockForUpdate()->first();
            if (!$r || !$r->isValid()) {
                return null;
            }

            // Mark used inside the same transaction. Anything reading this
            // row after commit sees used_at != null and rejects.
            $r->used_at = now();
            $r->save();

            // Burn any OTHER unused tokens for this email in the same
            // transaction so the replay window doesn't leak through a
            // parallel link that was also in flight.
            SignInToken::where('email', $r->email)
                ->whereNull('used_at')
                ->where('id', '!=', $r->id)
                ->update(['used_at' => now()]);

            return $r;
        });

        if (!$row) {
            return redirect()->route('sign-in.show')->withErrors([
                'email' => 'That sign-in link is invalid or has expired. Request a new one below.',
            ]);
        }

        $user = User::where('email', $row->email)->first();
        if (!$user) {
            // Should only happen if the user was deleted between link send and click.
            $user = User::create([
                'name'     => '',
                'email'    => $row->email,
                'password' => Hash::make(Str::random(64)),
                'email_verified_at' => now(),
            ]);
        } elseif (!$user->email_verified_at) {
            $user->email_verified_at = now();
            $user->save();
        }

        Auth::login($user, remember: true);
        $request->session()->regenerate();

        return redirect()->route('account');
    }

    public function signOut(Request $request): RedirectResponse
    {
        Auth::logout();
        $request->session()->invalidate();
        $request->session()->regenerateToken();

        return redirect()->route('home');
    }

    private function issueToken(string $email, Request $request): string
    {
        // 64-char URL-safe token, ~380 bits of entropy. We store only its
        // SHA-256 hash (see SignInToken::hashToken) so a DB dump cannot be
        // used to log in as the user. The raw token is returned for the
        // magic-link URL and never persisted anywhere server-side.
        $token = Str::random(64);

        SignInToken::create([
            'token_hash' => SignInToken::hashToken($token),
            'email'      => $email,
            'expires_at' => now()->addMinutes(self::TOKEN_TTL_MIN),
            'ip'         => $request->ip(),
            'user_agent' => substr((string) $request->userAgent(), 0, 255),
        ]);

        return $token;
    }
}
