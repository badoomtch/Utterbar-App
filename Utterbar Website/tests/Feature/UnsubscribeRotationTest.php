<?php

namespace Tests\Feature;

use App\Models\Release;
use App\Models\UpdateSubscriber;
use App\Models\User;
use App\Mail\UpdateAnnouncement;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Mail;
use Illuminate\Support\Facades\Storage;
use Illuminate\Support\Str;
use Tests\TestCase;

/**
 * Fix #2 — a failed email send must NOT invalidate the subscriber's
 * existing (still-valid) unsubscribe token. The old blast code called
 * `rotateTokenAndSave()` before `Mail::send()`, so a mid-blast SMTP
 * failure would leave the subscriber with a DB hash matching a plaintext
 * URL that was never delivered — their last working unsubscribe link
 * (from the previous successful email) was already dead.
 *
 * New contract:
 *   - `prepareRotation()` stages a new token in memory only.
 *   - `commitRotation()` persists — called ONLY after a successful send.
 *   - `discardRotation()` restores the previous `token_hash` in memory.
 *   - A failed send never calls commit; DB state stays on the previous
 *     token, so the last-successful-email unsubscribe link keeps working.
 *
 * Coverage:
 *   - Model-level: prepareRotation, commitRotation, discardRotation.
 *   - End-to-end failure AND success paths through each of the three
 *     blast sites that were changed, so a future regression in any one
 *     of them is caught in isolation:
 *       • POST /api/admin/releases         (AdminReleaseApiController)
 *       • POST /admin/releases             (Admin\ReleaseController, session auth)
 *       • artisan updates:announce         (UpdatesAnnounce console command)
 *   - The console command's --dry-run path (no send → no rotation).
 */
class UnsubscribeRotationTest extends TestCase
{
    use RefreshDatabase;

    private const ADMIN_TOKEN = 'test-admin-token-for-unsub-rotation';
    private const ADMIN_EMAIL = 'admin@example.com';

    protected function setUp(): void
    {
        parent::setUp();
        putenv('ADMIN_API_TOKEN=' . self::ADMIN_TOKEN);
        $_ENV['ADMIN_API_TOKEN'] = self::ADMIN_TOKEN;
        putenv('ADMIN_EMAILS=' . self::ADMIN_EMAIL);
        $_ENV['ADMIN_EMAILS'] = self::ADMIN_EMAIL;
        Storage::fake('r2');
    }

    /** Seeds and returns the admin user for actingAs() on browser-form tests. */
    private function adminUser(): User
    {
        return User::create([
            'name'              => 'Admin',
            'email'             => self::ADMIN_EMAIL,
            'password'          => Hash::make(Str::random(64)),
            'email_verified_at' => now(),
        ]);
    }

    /**
     * Install a Mail transport that throws on every send. Used by tests
     * that need to exercise the blast's catch branch — `Mail::fake()`
     * treats every send as success and has no public "fail next send" hook.
     *
     * Implementation: register a one-off listener on MessageSending that
     * throws. Laravel's MailManager fires this before handing off to the
     * transport, so the throw propagates up through `Mail::to()->send()`
     * into the blast's try/catch. Cleaner than swapping the mailer
     * binding (which the MailManager bypasses in Laravel 12).
     */
    private function installFailingMailer(): void
    {
        $this->app['events']->listen(
            \Illuminate\Mail\Events\MessageSending::class,
            function () {
                throw new \RuntimeException('simulated SMTP failure');
            },
        );
    }

    private function seedActive(string $email = 'jay@example.com'): array
    {
        $raw = Str::random(64);
        $sub = UpdateSubscriber::create([
            'email'         => $email,
            'source'        => 'app',
            'subscribed_at' => now()->subDay(),
            'confirmed_at'  => now()->subDay(),
            'token_hash'    => UpdateSubscriber::hashToken($raw),
        ]);
        return [$sub, $raw];
    }

    /* ----- Model-level primitives ----- */

    public function test_prepare_rotation_does_not_persist(): void
    {
        [$sub, $oldRaw] = $this->seedActive();
        $oldHash = $sub->token_hash;

        $newRaw = $sub->prepareRotation();

        $this->assertNotSame($oldRaw, $newRaw);
        $this->assertSame($newRaw, $sub->rawToken);
        $this->assertSame(UpdateSubscriber::hashToken($newRaw), $sub->token_hash);

        // DB row is unchanged — this is the whole point.
        $row = UpdateSubscriber::find($sub->id);
        $this->assertSame($oldHash, $row->token_hash, 'prepareRotation must not touch the DB');
    }

    public function test_commit_rotation_persists_new_hash(): void
    {
        [$sub] = $this->seedActive();
        $newRaw = $sub->prepareRotation();
        $sub->commitRotation();

        $row = UpdateSubscriber::find($sub->id);
        $this->assertSame(UpdateSubscriber::hashToken($newRaw), $row->token_hash);
    }

    public function test_discard_rotation_reverts_in_memory_hash(): void
    {
        [$sub, $oldRaw] = $this->seedActive();
        $oldHash = $sub->token_hash;

        $sub->prepareRotation();
        $sub->discardRotation();

        // In-memory model is restored.
        $this->assertSame($oldHash, $sub->token_hash);
        $this->assertNull($sub->rawToken);

        // DB was never touched.
        $row = UpdateSubscriber::find($sub->id);
        $this->assertSame($oldHash, $row->token_hash);

        // Old URL still resolves.
        $this->get('/updates/unsubscribe?t=' . $oldRaw)
            ->assertOk()
            ->assertViewHas('status', 'ok');
    }

    /* ----- End-to-end: failed send must leave DB untouched ----- */

    public function test_admin_api_blast_preserves_old_token_when_mail_fails(): void
    {
        [, $oldRaw] = $this->seedActive();
        $oldRow = UpdateSubscriber::first();
        $oldHash = $oldRow->token_hash;

        $this->installFailingMailer();

        $r2Key = 'releases/0.6.2/Utterbar-Setup-0.6.2.exe';
        Storage::disk('r2')->put($r2Key, 'x');

        $resp = $this->postJson('/api/admin/releases', [
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => $r2Key,
            'sha256'         => str_repeat('a', 64),
            'notes_markdown' => 'notes',
            'announce'       => true,
        ], ['Authorization' => 'Bearer ' . self::ADMIN_TOKEN]);

        $resp->assertStatus(201);
        $resp->assertJsonPath('announced_to', 0);

        // The blast's catch path must have called discardRotation(),
        // leaving the DB token_hash untouched.
        $after = UpdateSubscriber::first();
        $this->assertSame(
            $oldHash,
            $after->token_hash,
            'a failed send must NOT advance the DB token_hash'
        );

        // The pre-existing unsubscribe URL still works — the user who
        // got a delivered email last month can still click their
        // unsubscribe link.
        $this->get('/updates/unsubscribe?t=' . $oldRaw)
            ->assertOk()
            ->assertViewHas('status', 'ok');
    }

    public function test_admin_api_blast_rotates_token_when_mail_succeeds(): void
    {
        [, $oldRaw] = $this->seedActive();
        $oldRow = UpdateSubscriber::first();
        $oldHash = $oldRow->token_hash;

        Mail::fake();

        $r2Key = 'releases/0.6.2/Utterbar-Setup-0.6.2.exe';
        Storage::disk('r2')->put($r2Key, 'x');

        $resp = $this->postJson('/api/admin/releases', [
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => $r2Key,
            'sha256'         => str_repeat('a', 64),
            'notes_markdown' => 'notes',
            'announce'       => true,
        ], ['Authorization' => 'Bearer ' . self::ADMIN_TOKEN]);

        $resp->assertStatus(201);
        $resp->assertJsonPath('announced_to', 1);

        // Mail::fake treats send() as success — so rotation must commit.
        $after = UpdateSubscriber::first();
        $this->assertNotSame(
            $oldHash,
            $after->token_hash,
            'a successful send must advance the DB token_hash'
        );

        // The old URL is now dead; the Mailable was assembled with the
        // new raw token (held on the in-memory model), not the old one.
        $this->get('/updates/unsubscribe?t=' . $oldRaw)
            ->assertOk()
            ->assertViewHas('status', 'error');

        // And UpdateAnnouncement was queued to our active subscriber.
        Mail::assertSent(UpdateAnnouncement::class, 1);
    }

    /* ----- End-to-end: admin BROWSER path (POST /admin/releases) ----- */

    public function test_admin_browser_blast_preserves_old_token_when_mail_fails(): void
    {
        [, $oldRaw] = $this->seedActive();
        $oldHash = UpdateSubscriber::first()->token_hash;

        $this->installFailingMailer();

        $r2Key = 'releases/0.6.2/Utterbar-Setup-0.6.2.exe';
        Storage::disk('r2')->put($r2Key, 'x');

        $resp = $this->actingAs($this->adminUser())->post('/admin/releases', [
            'version'  => '0.6.2',
            'channel'  => 'stable',
            'r2_key'   => $r2Key,
            'sha256'   => str_repeat('a', 64),
            'notes'    => 'notes',
            'announce' => '1',
        ]);

        // Browser path redirects back to the index after publish; the
        // catch branch in blastAnnouncement() swallows the send failure
        // and records announced_to=0, so the redirect still succeeds.
        $resp->assertRedirect(route('admin.releases.index'));

        // DB untouched: old hash remains, old URL still resolves.
        $after = UpdateSubscriber::first();
        $this->assertSame(
            $oldHash,
            $after->token_hash,
            'browser blast with failed send must NOT advance the DB token_hash'
        );
        $this->get('/updates/unsubscribe?t=' . $oldRaw)
            ->assertOk()
            ->assertViewHas('status', 'ok');
    }

    public function test_admin_browser_blast_rotates_token_when_mail_succeeds(): void
    {
        [, $oldRaw] = $this->seedActive();
        $oldHash = UpdateSubscriber::first()->token_hash;

        Mail::fake();

        $r2Key = 'releases/0.6.2/Utterbar-Setup-0.6.2.exe';
        Storage::disk('r2')->put($r2Key, 'x');

        $resp = $this->actingAs($this->adminUser())->post('/admin/releases', [
            'version'  => '0.6.2',
            'channel'  => 'stable',
            'r2_key'   => $r2Key,
            'sha256'   => str_repeat('a', 64),
            'notes'    => 'notes',
            'announce' => '1',
        ]);
        $resp->assertRedirect(route('admin.releases.index'));

        $after = UpdateSubscriber::first();
        $this->assertNotSame(
            $oldHash,
            $after->token_hash,
            'browser blast with successful send must advance the DB token_hash'
        );
        $this->get('/updates/unsubscribe?t=' . $oldRaw)
            ->assertOk()
            ->assertViewHas('status', 'error');
        Mail::assertSent(UpdateAnnouncement::class, 1);
    }

    /* ----- End-to-end: console command (artisan updates:announce) ----- */

    public function test_console_announce_preserves_old_token_when_mail_fails(): void
    {
        [, $oldRaw] = $this->seedActive();
        $oldHash = UpdateSubscriber::first()->token_hash;

        $this->installFailingMailer();

        $this->artisan('updates:announce', [
            '--release'    => '0.6.2',
            '--subject'    => 'Utterbar 0.6.2 is out',
            '--notes-text' => 'Some release notes.',
        ])
            ->expectsConfirmation('Send to 1 subscribers?', 'yes')
            ->assertExitCode(0);

        // The per-recipient catch branch swallows the throw, logs a
        // `failed:` warning, moves on. Most importantly: the staged
        // rotation was discarded, so the DB hash is untouched.
        $after = UpdateSubscriber::first();
        $this->assertSame(
            $oldHash,
            $after->token_hash,
            'console blast with failed send must NOT advance the DB token_hash'
        );
        $this->get('/updates/unsubscribe?t=' . $oldRaw)
            ->assertOk()
            ->assertViewHas('status', 'ok');
    }

    public function test_console_announce_rotates_token_when_mail_succeeds(): void
    {
        [, $oldRaw] = $this->seedActive();
        $oldHash = UpdateSubscriber::first()->token_hash;

        Mail::fake();

        $this->artisan('updates:announce', [
            '--release'    => '0.6.2',
            '--subject'    => 'Utterbar 0.6.2 is out',
            '--notes-text' => 'Some release notes.',
        ])
            ->expectsConfirmation('Send to 1 subscribers?', 'yes')
            ->assertExitCode(0);

        $after = UpdateSubscriber::first();
        $this->assertNotSame(
            $oldHash,
            $after->token_hash,
            'console blast with successful send must advance the DB token_hash'
        );
        $this->get('/updates/unsubscribe?t=' . $oldRaw)
            ->assertOk()
            ->assertViewHas('status', 'error');
        Mail::assertSent(UpdateAnnouncement::class, 1);
    }

    public function test_console_announce_dry_run_never_rotates(): void
    {
        [, $oldRaw] = $this->seedActive();
        $oldHash = UpdateSubscriber::first()->token_hash;

        // --dry-run skips the send loop entirely, so neither a success
        // nor a failure path can mutate the subscriber. Verify the DB
        // and the old URL both survive untouched.
        $this->artisan('updates:announce', [
            '--release'    => '0.6.2',
            '--subject'    => 'Utterbar 0.6.2 is out',
            '--notes-text' => 'Some release notes.',
            '--dry-run'    => true,
        ])->assertExitCode(0);

        $after = UpdateSubscriber::first();
        $this->assertSame($oldHash, $after->token_hash);
        $this->get('/updates/unsubscribe?t=' . $oldRaw)
            ->assertOk()
            ->assertViewHas('status', 'ok');
    }
}
