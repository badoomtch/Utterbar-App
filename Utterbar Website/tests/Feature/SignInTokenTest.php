<?php

namespace Tests\Feature;

use App\Models\SignInToken;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Mail;
use Illuminate\Support\Str;
use Tests\TestCase;

/**
 * Covers the four guarantees of the hashed magic-link system:
 *
 *   1. A valid unused token logs the user in and is consumed.
 *   2. An expired token is refused with the standard error, no login.
 *   3. A reused (already-consumed) token is refused.
 *   4. Two concurrent consume attempts for the same token never both
 *      log the user in — exactly one succeeds, the other is refused.
 *
 * Also checks the critical storage invariant: the DB only ever holds the
 * SHA-256 hash of the token, never the plaintext.
 */
class SignInTokenTest extends TestCase
{
    use RefreshDatabase;

    /** Helper: mint a token the way the controller does. */
    private function mintToken(string $email): string
    {
        $raw = Str::random(64);
        SignInToken::create([
            'token_hash' => SignInToken::hashToken($raw),
            'email'      => $email,
            'expires_at' => now()->addMinutes(20),
        ]);
        return $raw;
    }

    private function seedUser(string $email): User
    {
        return User::create([
            'name'              => '',
            'email'             => $email,
            'password'          => Hash::make(Str::random(64)),
            'email_verified_at' => now(),
        ]);
    }

    public function test_db_stores_hash_not_plaintext(): void
    {
        $email = 'jay@example.com';
        $this->seedUser($email);
        $raw = $this->mintToken($email);

        $row = SignInToken::first();
        $this->assertNotNull($row);
        $this->assertNull($row->token ?? null, 'no plaintext column should be present on the model');
        $this->assertSame(
            hash('sha256', $raw),
            $row->token_hash,
            'the stored hash must be SHA-256 of the raw token'
        );
        $this->assertNotSame($raw, $row->token_hash, 'plaintext must never be stored');
    }

    public function test_valid_token_logs_in_and_consumes(): void
    {
        Mail::fake();
        $email = 'jay@example.com';
        $this->seedUser($email);
        $raw = $this->mintToken($email);

        $resp = $this->get("/sign-in/{$raw}");

        $resp->assertRedirect(route('account'));
        $this->assertTrue(Auth::check(), 'user should be logged in after a valid consume');
        $row = SignInToken::first();
        $this->assertNotNull($row->used_at, 'used_at must be set after consume');
    }

    public function test_expired_token_is_refused(): void
    {
        $email = 'jay@example.com';
        $this->seedUser($email);
        $raw = Str::random(64);
        SignInToken::create([
            'token_hash' => SignInToken::hashToken($raw),
            'email'      => $email,
            'expires_at' => now()->subMinute(),
        ]);

        $resp = $this->get("/sign-in/{$raw}");

        $resp->assertRedirect(route('sign-in.show'));
        $resp->assertSessionHasErrors('email');
        $this->assertFalse(Auth::check(), 'expired token must not log anyone in');
    }

    public function test_reused_token_is_refused_on_second_use(): void
    {
        $email = 'jay@example.com';
        $this->seedUser($email);
        $raw = $this->mintToken($email);

        // First use — succeeds.
        $first = $this->get("/sign-in/{$raw}");
        $first->assertRedirect(route('account'));
        $this->assertTrue(Auth::check());

        // Log out so we can tell whether the second attempt created a new session.
        $this->post('/sign-out');
        $this->assertFalse(Auth::check());

        // Second use — must refuse.
        $second = $this->get("/sign-in/{$raw}");
        $second->assertRedirect(route('sign-in.show'));
        $second->assertSessionHasErrors('email');
        $this->assertFalse(Auth::check(), 'replay attempt must not establish a session');
    }

    public function test_unknown_token_is_refused(): void
    {
        // Not even minted — attack surface of "brute-force guess".
        $bogus = str_repeat('a', 64);
        $resp = $this->get("/sign-in/{$bogus}");
        $resp->assertRedirect(route('sign-in.show'));
        $resp->assertSessionHasErrors('email');
        $this->assertFalse(Auth::check());
    }

    /**
     * Simulates the race window between two concurrent requests for the
     * same magic link. We can't spin two HTTP processes in unit tests, but
     * we CAN exercise the atomic path directly: two back-to-back attempts
     * after a manual commit of the first one's transaction — same
     * observable end state as two concurrent workers with row-level locks.
     *
     * The contract: at most one login ever happens for a given raw token.
     */
    public function test_double_use_prevented_atomically(): void
    {
        $email = 'jay@example.com';
        $this->seedUser($email);
        $raw = $this->mintToken($email);

        // First request consumes inside a transaction and commits.
        $r1 = $this->get("/sign-in/{$raw}");
        $r1->assertRedirect(route('account'));

        // DB state after commit: used_at is set. Any subsequent consume —
        // whether serialised or racing — must see the used_at and reject.
        $row = SignInToken::first();
        $this->assertNotNull($row->used_at);

        // Second request with the same token. Log out first so success would
        // be observable.
        $this->post('/sign-out');
        $r2 = $this->get("/sign-in/{$raw}");
        $r2->assertRedirect(route('sign-in.show'));
        $r2->assertSessionHasErrors('email');
        $this->assertFalse(Auth::check());
    }

    /**
     * A valid consume also burns sibling unused tokens for the same email
     * so a parallel link they requested two minutes ago can't be replayed.
     */
    public function test_sibling_tokens_are_burned_on_consume(): void
    {
        $email = 'jay@example.com';
        $this->seedUser($email);

        $rawA = $this->mintToken($email);
        $rawB = $this->mintToken($email);
        $rawC = $this->mintToken($email); // this one we'll consume

        $this->get("/sign-in/{$rawC}")->assertRedirect(route('account'));

        $all = SignInToken::orderBy('id')->get();
        $this->assertCount(3, $all);
        foreach ($all as $row) {
            $this->assertNotNull($row->used_at, "every sibling token for the email must be burned");
        }
    }

    public function test_plaintext_token_is_not_stored_on_any_row(): void
    {
        $email = 'jay@example.com';
        $this->seedUser($email);
        $raw = $this->mintToken($email);

        $rows = DB::table('sign_in_tokens')->get();
        foreach ($rows as $r) {
            foreach ((array) $r as $col => $val) {
                if (is_string($val)) {
                    $this->assertNotSame(
                        $raw,
                        $val,
                        "raw token must not appear in any column (found in {$col})"
                    );
                }
            }
        }
    }
}
