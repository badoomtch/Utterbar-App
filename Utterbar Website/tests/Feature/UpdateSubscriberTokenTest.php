<?php

namespace Tests\Feature;

use App\Models\UpdateSubscriber;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Mail;
use Illuminate\Support\Str;
use Tests\TestCase;

/**
 * Covers the storage + consume contract for update-subscriber confirmation
 * and unsubscribe tokens after the plaintext → hash migration:
 *
 *   1. DB holds only the hash, never the plaintext.
 *   2. Confirm + unsubscribe URLs resolve via hash lookup.
 *   3. Replayed confirm / unsubscribe URLs are idempotent — no error, no
 *      state change past the first successful action.
 *   4. Unknown / malformed token parameters render the error template.
 *   5. Rotating via `rotateTokenAndSave()` invalidates the previous URL.
 */
class UpdateSubscriberTokenTest extends TestCase
{
    use RefreshDatabase;

    private function seedSubscriber(string $email = 'jay@example.com'): array
    {
        $raw = Str::random(64);
        $sub = UpdateSubscriber::create([
            'email'         => $email,
            'source'        => 'app',
            'subscribed_at' => now(),
            'token_hash'    => UpdateSubscriber::hashToken($raw),
        ]);
        return [$sub, $raw];
    }

    public function test_db_stores_hash_not_plaintext(): void
    {
        [$sub, $raw] = $this->seedSubscriber();
        $row = DB::table('update_subscribers')->first();
        $this->assertNotNull($row);
        $this->assertSame(hash('sha256', $raw), $row->token_hash);
        $this->assertNotSame($raw, $row->token_hash);
        // No plaintext `token` column should remain on the row.
        foreach ((array) $row as $col => $val) {
            if (is_string($val)) {
                $this->assertNotSame($raw, $val, "raw token must not appear in {$col}");
            }
        }
    }

    public function test_confirm_with_valid_token_marks_confirmed(): void
    {
        [, $raw] = $this->seedSubscriber();
        $resp = $this->get('/updates/confirm?t=' . $raw);
        $resp->assertOk();
        $resp->assertViewIs('pages.updates-status');
        $resp->assertViewHas('status', 'ok');
        $this->assertNotNull(UpdateSubscriber::first()->confirmed_at);
    }

    public function test_confirm_replay_is_idempotent(): void
    {
        [, $raw] = $this->seedSubscriber();
        $this->get('/updates/confirm?t=' . $raw)->assertOk();
        $first = UpdateSubscriber::first()->confirmed_at;

        // Replay — must not error and must not advance confirmed_at past
        // the first one (the second call re-reads the row, sees already
        // confirmed, and returns the success view without another save).
        $resp = $this->get('/updates/confirm?t=' . $raw);
        $resp->assertOk();
        $resp->assertViewHas('status', 'ok');
        $this->assertEquals(
            $first->timestamp,
            UpdateSubscriber::first()->confirmed_at->timestamp,
            'replaying a confirmation must not change confirmed_at'
        );
    }

    public function test_unsubscribe_with_valid_token(): void
    {
        [, $raw] = $this->seedSubscriber();
        // Confirm first so the state transition is visible.
        $this->get('/updates/confirm?t=' . $raw)->assertOk();

        $resp = $this->get('/updates/unsubscribe?t=' . $raw);
        $resp->assertOk();
        $resp->assertViewHas('status', 'ok');
        $this->assertNotNull(UpdateSubscriber::first()->unsubscribed_at);
    }

    public function test_unsubscribe_replay_is_idempotent(): void
    {
        [, $raw] = $this->seedSubscriber();
        $this->get('/updates/unsubscribe?t=' . $raw)->assertOk();
        $first = UpdateSubscriber::first()->unsubscribed_at;

        $resp = $this->get('/updates/unsubscribe?t=' . $raw);
        $resp->assertOk();
        $resp->assertViewHas('status', 'ok');
        $this->assertEquals(
            $first->timestamp,
            UpdateSubscriber::first()->unsubscribed_at->timestamp,
            'replaying an unsubscribe must not change unsubscribed_at'
        );
    }

    public function test_unknown_confirm_token_shows_error(): void
    {
        $bogus = str_repeat('z', 64);
        $resp = $this->get('/updates/confirm?t=' . $bogus);
        $resp->assertOk();
        $resp->assertViewHas('status', 'error');
    }

    public function test_unknown_unsubscribe_token_shows_error(): void
    {
        $bogus = str_repeat('z', 64);
        $resp = $this->get('/updates/unsubscribe?t=' . $bogus);
        $resp->assertOk();
        $resp->assertViewHas('status', 'error');
    }

    public function test_empty_token_does_not_leak_success(): void
    {
        $resp = $this->get('/updates/confirm?t=');
        $resp->assertOk();
        $resp->assertViewHas('status', 'error');
    }

    public function test_rotate_invalidates_old_token(): void
    {
        [$sub, $rawOld] = $this->seedSubscriber();

        // Rotate to a fresh token.
        $rawNew = $sub->rotateTokenAndSave();
        $this->assertNotSame($rawOld, $rawNew);
        $this->assertSame($rawNew, $sub->rawToken);

        // Old token no longer resolves.
        $old = $this->get('/updates/confirm?t=' . $rawOld);
        $old->assertOk();
        $old->assertViewHas('status', 'error');

        // New token resolves.
        $new = $this->get('/updates/confirm?t=' . $rawNew);
        $new->assertOk();
        $new->assertViewHas('status', 'ok');
    }

    public function test_subscribe_endpoint_populates_raw_token_and_persists_hash(): void
    {
        Mail::fake();
        $resp = $this->postJson('/api/updates/subscribe', [
            'email'  => 'jay@example.com',
            'source' => 'app',
        ]);
        $resp->assertOk();
        $resp->assertJsonPath('ok', true);

        $sub = UpdateSubscriber::first();
        $this->assertNotNull($sub);
        $this->assertNotNull($sub->token_hash);
        $this->assertSame(64, strlen($sub->token_hash));
        // No plaintext in the DB row.
        $row = DB::table('update_subscribers')->first();
        $this->assertNotSame($sub->token_hash, $sub->rawToken ?? '');
    }
}
