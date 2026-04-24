<?php

namespace Tests\Feature;

use App\Models\Release;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Storage;
use Illuminate\Support\Str;
use Tests\TestCase;

/**
 * Publish-time validation on both entry points into the release table:
 *
 *   - POST /api/admin/releases   (AdminReleaseApiController, bearer auth)
 *   - POST /admin/releases       (Admin\ReleaseController, session auth)
 *
 * Both must enforce the same rules so the two paths cannot diverge:
 *
 *   1. SHA-256 is REQUIRED (fix #1). Blank / missing / wrong-length /
 *      non-hex values all reject with a 422.
 *   2. Version must be strict X.Y.Z — no prerelease suffixes like
 *      `0.7.0-rc.1` (fix #3). Admin publish path must match what the
 *      updater can actually compare.
 *
 * Validation is duplicated between the two controllers (not shared via a
 * Rule object or FormRequest), so we exercise BOTH entry points: the
 * bearer-token API path via Http JSON and the session-authed browser
 * path via `actingAs()` + form POST. Identical assertion shape under
 * both makes a future divergence visible immediately.
 */
class ReleasePublishValidationTest extends TestCase
{
    use RefreshDatabase;

    private const ADMIN_TOKEN = 'test-admin-token-for-release-validation';
    private const ADMIN_EMAIL = 'admin@example.com';

    protected function setUp(): void
    {
        parent::setUp();
        // Ensure the admin.api middleware accepts our Bearer header, the
        // admin.email middleware lets our seeded User through, and the
        // R2 disk under test doesn't require real credentials.
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

    /** Common body fields for a well-formed publish request. */
    private function body(array $override = []): array
    {
        // Seed the fake R2 disk so the `file_not_in_bucket` check passes
        // and we reach the real validation / persist path.
        $r2Key = $override['r2_key'] ?? 'releases/0.6.2/Utterbar-Setup-0.6.2.exe';
        Storage::disk('r2')->put($r2Key, 'fake-installer-bytes');

        return array_merge([
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => $r2Key,
            'sha256'         => str_repeat('a', 64),
            'notes_markdown' => 'First change',
        ], $override);
    }

    private function post(array $body)
    {
        return $this->postJson('/api/admin/releases', $body, [
            'Authorization' => 'Bearer ' . self::ADMIN_TOKEN,
        ]);
    }

    /**
     * Browser-form equivalent of `post()` — seeds an admin user, acts as
     * them, and submits the standard HTML form body. The browser form
     * uses `notes` as the field name (the API uses `notes_markdown`), so
     * we translate here and strip any API-only keys. Returns the
     * TestResponse for the same assert* API.
     */
    private function postBrowser(array $body)
    {
        $form = $body;
        if (array_key_exists('notes_markdown', $form)) {
            $form['notes'] = $form['notes_markdown'];
            unset($form['notes_markdown']);
        }
        // Seed the R2 key if the helper hasn't already (the API path
        // pre-seeds in body()).
        if (!empty($form['r2_key']) && !Storage::disk('r2')->exists($form['r2_key'])) {
            Storage::disk('r2')->put($form['r2_key'], 'fake-installer-bytes');
        }
        return $this->actingAs($this->adminUser())->post('/admin/releases', $form);
    }

    /* ----- Fix #1: sha256 is required ----- */

    public function test_publish_rejects_missing_sha256(): void
    {
        $body = $this->body();
        unset($body['sha256']);
        $this->post($body)->assertStatus(422)->assertJsonValidationErrors(['sha256']);
        $this->assertSame(0, Release::count(), 'no release row must be created');
    }

    public function test_publish_rejects_blank_sha256(): void
    {
        $this->post($this->body(['sha256' => '']))
            ->assertStatus(422)
            ->assertJsonValidationErrors(['sha256']);
        $this->assertSame(0, Release::count());
    }

    public function test_publish_rejects_short_sha256(): void
    {
        $this->post($this->body(['sha256' => 'abc123']))
            ->assertStatus(422)
            ->assertJsonValidationErrors(['sha256']);
        $this->assertSame(0, Release::count());
    }

    public function test_publish_rejects_non_hex_sha256(): void
    {
        $this->post($this->body(['sha256' => str_repeat('z', 64)]))
            ->assertStatus(422)
            ->assertJsonValidationErrors(['sha256']);
        $this->assertSame(0, Release::count());
    }

    public function test_publish_accepts_valid_sha256_and_persists_lowercased(): void
    {
        // Upper-case input should round-trip as lower-case in the DB so
        // downstream equality checks (client verification) have a single
        // canonical form.
        $resp = $this->post($this->body(['sha256' => str_repeat('A', 64)]));
        $resp->assertStatus(201);
        $resp->assertJsonPath('ok', true);

        $row = Release::first();
        $this->assertNotNull($row);
        $this->assertSame(str_repeat('a', 64), $row->sha256);
    }

    /* ----- Fix #3: version must be X.Y.Z (no prerelease) ----- */

    public function test_publish_rejects_prerelease_version(): void
    {
        foreach (['0.7.0-rc.1', '0.7.0-beta', '0.7.0-alpha.2', '1.0.0-rc'] as $bad) {
            $resp = $this->post($this->body([
                'version' => $bad,
                'r2_key'  => "releases/{$bad}/Utterbar-Setup-{$bad}.exe",
            ]));
            $resp->assertStatus(422)->assertJsonValidationErrors(['version']);
        }
        $this->assertSame(0, Release::count(), 'no prerelease should persist');
    }

    public function test_publish_rejects_non_semver_version(): void
    {
        foreach (['v0.7.0', '0.7', '0.7.0.1', '1.0', 'abc'] as $bad) {
            $resp = $this->post($this->body([
                'version' => $bad,
                'r2_key'  => "releases/{$bad}/Utterbar-Setup.exe",
            ]));
            $resp->assertStatus(422)->assertJsonValidationErrors(['version']);
        }
        $this->assertSame(0, Release::count());
    }

    public function test_publish_accepts_strict_xyz_version(): void
    {
        $resp = $this->post($this->body(['version' => '0.6.2']));
        $resp->assertStatus(201);
        $this->assertSame(1, Release::where('version', '0.6.2')->count());
    }

    public function test_publish_accepts_beta_channel_with_strict_xyz(): void
    {
        // Beta channel gets its own uniqueness namespace — reuse the
        // stable version string is fine.
        $resp = $this->post($this->body([
            'version' => '0.7.0',
            'channel' => 'beta',
            'r2_key'  => 'releases/0.7.0/Utterbar-Setup-0.7.0.exe',
        ]));
        $resp->assertStatus(201);
        $row = Release::first();
        $this->assertSame('0.7.0', $row->version);
        $this->assertSame('beta', $row->channel);
    }

    /* --------------------------------------------------------------------
     * Browser admin form — POST /admin/releases. Validation is duplicated
     * between the two controllers so these mirror the API tests above.
     * If the two paths drift in future, these are the tests that catch it.
     * -------------------------------------------------------------------- */

    public function test_browser_publish_rejects_missing_sha256(): void
    {
        $body = $this->body();
        unset($body['sha256']);
        $resp = $this->postBrowser($body);
        $resp->assertSessionHasErrors(['sha256']);
        $this->assertSame(0, Release::count(), 'no release row must be created via browser form');
    }

    public function test_browser_publish_rejects_blank_sha256(): void
    {
        $resp = $this->postBrowser($this->body(['sha256' => '']));
        $resp->assertSessionHasErrors(['sha256']);
        $this->assertSame(0, Release::count());
    }

    public function test_browser_publish_rejects_non_hex_sha256(): void
    {
        $resp = $this->postBrowser($this->body(['sha256' => str_repeat('z', 64)]));
        $resp->assertSessionHasErrors(['sha256']);
        $this->assertSame(0, Release::count());
    }

    public function test_browser_publish_rejects_prerelease_version(): void
    {
        foreach (['0.7.0-rc.1', '0.7.0-beta', '0.7.0-alpha.2'] as $bad) {
            $resp = $this->postBrowser($this->body([
                'version' => $bad,
                'r2_key'  => "releases/{$bad}/Utterbar-Setup-{$bad}.exe",
            ]));
            $resp->assertSessionHasErrors(['version']);
        }
        $this->assertSame(0, Release::count());
    }

    public function test_browser_publish_rejects_non_semver_version(): void
    {
        foreach (['v0.7.0', '0.7', '0.7.0.1'] as $bad) {
            $resp = $this->postBrowser($this->body([
                'version' => $bad,
                'r2_key'  => "releases/{$bad}/Utterbar-Setup.exe",
            ]));
            $resp->assertSessionHasErrors(['version']);
        }
        $this->assertSame(0, Release::count());
    }

    public function test_browser_publish_accepts_valid_release_and_persists_lowercased(): void
    {
        $resp = $this->postBrowser($this->body(['sha256' => str_repeat('A', 64)]));
        $resp->assertRedirect(route('admin.releases.index'));
        $row = Release::first();
        $this->assertNotNull($row);
        $this->assertSame('0.6.2', $row->version);
        $this->assertSame(str_repeat('a', 64), $row->sha256, 'sha256 must be persisted lower-case');
    }

    public function test_browser_publish_accepts_beta_channel(): void
    {
        $resp = $this->postBrowser($this->body([
            'version' => '0.7.0',
            'channel' => 'beta',
            'r2_key'  => 'releases/0.7.0/Utterbar-Setup-0.7.0.exe',
        ]));
        $resp->assertRedirect(route('admin.releases.index'));
        $row = Release::first();
        $this->assertSame('beta', $row->channel);
    }
}
