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

    /**
     * Seeds and returns the admin user for actingAs() on browser-form
     * tests. Returns an existing row when the test has already seeded
     * one — tests that loop over invalid inputs re-enter postBrowser()
     * and would otherwise unique-violate on users.email.
     */
    private function adminUser(): User
    {
        return User::firstOrCreate(
            ['email' => self::ADMIN_EMAIL],
            [
                'name'              => 'Admin',
                'password'          => Hash::make(Str::random(64)),
                'email_verified_at' => now(),
            ],
        );
    }

    /**
     * Common body fields for a well-formed publish request.
     *
     * The validator now (post-hardening) recomputes the SHA-256 from
     * the R2 object and rejects pasted SHAs that don't match. Tests
     * that want a successful publish must therefore pass the SHA of
     * the bytes they're seeding. We seed deterministic bytes
     * ("utterbar-test-installer-<r2_key>") and pre-compute their
     * hash so callers can spread `body()` and get a passing case.
     *
     * Tests that want to assert SHA mismatch can override `sha256`
     * with any other 64-hex string and the validator will reject.
     */
    private function body(array $override = []): array
    {
        $r2Key = $override['r2_key'] ?? 'releases/0.6.2/Utterbar-Setup-0.6.2.exe';
        // Bytes are derived from the key so each test's seeded
        // installer is unique — avoids cross-test contamination
        // when a single fake bucket is shared across tests.
        $bytes = "utterbar-test-installer-{$r2Key}";
        Storage::disk('r2')->put($r2Key, $bytes);

        return array_merge([
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => $r2Key,
            // Pasted SHA: by default the SHA of the bytes we just
            // seeded so the validator's pasted-vs-computed equality
            // check passes. Override to assert mismatch.
            'sha256'         => hash('sha256', $bytes),
            'notes_markdown' => 'First change',
        ], $override);
    }

    /** JSON POST to the admin release API, bearer-authenticated. */
    private function postApi(array $body)
    {
        return $this->postJson('/api/admin/releases', $body, [
            'Authorization' => 'Bearer ' . self::ADMIN_TOKEN,
        ]);
    }

    /**
     * Browser-form equivalent of `postApi()` — seeds an admin user, acts
     * as them, and submits the standard HTML form body. The browser form
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
        $this->postApi($body)->assertStatus(422)->assertJsonValidationErrors(['sha256']);
        $this->assertSame(0, Release::count(), 'no release row must be created');
    }

    public function test_publish_rejects_blank_sha256(): void
    {
        $this->postApi($this->body(['sha256' => '']))
            ->assertStatus(422)
            ->assertJsonValidationErrors(['sha256']);
        $this->assertSame(0, Release::count());
    }

    public function test_publish_rejects_short_sha256(): void
    {
        $this->postApi($this->body(['sha256' => 'abc123']))
            ->assertStatus(422)
            ->assertJsonValidationErrors(['sha256']);
        $this->assertSame(0, Release::count());
    }

    public function test_publish_rejects_non_hex_sha256(): void
    {
        $this->postApi($this->body(['sha256' => str_repeat('z', 64)]))
            ->assertStatus(422)
            ->assertJsonValidationErrors(['sha256']);
        $this->assertSame(0, Release::count());
    }

    public function test_publish_accepts_valid_sha256_and_persists_lowercased(): void
    {
        // Upper-case pasted SHA must still match (hash_equals is
        // case-sensitive but the validator lower-cases the pasted
        // value before comparing). The persisted SHA is the one we
        // computed from R2 — always lower-case via `hash()`.
        $body = $this->body();
        $body['sha256'] = strtoupper($body['sha256']);
        $resp = $this->postApi($body);
        $resp->assertStatus(201);
        $resp->assertJsonPath('ok', true);

        $row = Release::first();
        $this->assertNotNull($row);
        $this->assertSame(strtolower($body['sha256']), $row->sha256);
    }

    /* ----- Fix #3: version must be X.Y.Z (no prerelease) ----- */

    public function test_publish_rejects_prerelease_version(): void
    {
        foreach (['0.7.0-rc.1', '0.7.0-beta', '0.7.0-alpha.2', '1.0.0-rc'] as $bad) {
            $resp = $this->postApi($this->body([
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
            $resp = $this->postApi($this->body([
                'version' => $bad,
                'r2_key'  => "releases/{$bad}/Utterbar-Setup.exe",
            ]));
            $resp->assertStatus(422)->assertJsonValidationErrors(['version']);
        }
        $this->assertSame(0, Release::count());
    }

    public function test_publish_accepts_strict_xyz_version(): void
    {
        $resp = $this->postApi($this->body(['version' => '0.6.2']));
        $resp->assertStatus(201);
        $this->assertSame(1, Release::where('version', '0.6.2')->count());
    }

    public function test_publish_accepts_beta_channel_with_strict_xyz(): void
    {
        // Beta channel gets its own uniqueness namespace — reuse the
        // stable version string is fine.
        $resp = $this->postApi($this->body([
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
        $body = $this->body();
        $body['sha256'] = strtoupper($body['sha256']);
        $resp = $this->postBrowser($body);
        $resp->assertRedirect(route('admin.releases.index'));
        $row = Release::first();
        $this->assertNotNull($row);
        $this->assertSame('0.6.2', $row->version);
        $this->assertSame(strtolower($body['sha256']), $row->sha256, 'sha256 must be persisted lower-case');
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

    /* --------------------------------------------------------------------
     * SHA-recompute hardening: server hashes the R2 object and stores
     * the computed value as canonical. The pasted SHA becomes a guard
     * against the wrong-file-uploaded class of error rather than the
     * source of truth for the persisted column.
     * -------------------------------------------------------------------- */

    public function test_api_publish_rejects_when_pasted_sha_differs_from_r2(): void
    {
        $body = $this->body();
        // Override pasted SHA with a value that's still hex+64-long
        // (so the regex validator passes) but doesn't match the
        // bytes we seeded. The new ReleaseValidator must catch it.
        $body['sha256'] = str_repeat('b', 64);
        $resp = $this->postApi($body);
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'sha_mismatch');
        $this->assertSame(0, Release::count(), 'mismatch must not persist');
    }

    public function test_browser_publish_rejects_when_pasted_sha_differs_from_r2(): void
    {
        $body = $this->body();
        $body['sha256'] = str_repeat('b', 64);
        $resp = $this->postBrowser($body);
        $resp->assertSessionHasErrors(['sha256']);
        $this->assertSame(0, Release::count());
    }

    public function test_api_publish_persists_computed_sha_even_when_paste_is_uppercase(): void
    {
        $body = $this->body();
        $expected = $body['sha256'];
        // Paste in upper-case form — validator lower-cases pasted
        // before comparing, then persists the computed (already
        // lower-case) value.
        $body['sha256'] = strtoupper($expected);
        $this->postApi($body)->assertStatus(201);

        $row = Release::first();
        $this->assertSame($expected, $row->sha256);
    }

    /* --------------------------------------------------------------------
     * Stable + announce confirmation: a missing confirm_stable_announce
     * flag must refuse the publish before any R2 work happens. Beta
     * channel + announce is unaffected (announce is only honoured for
     * stable anyway, but the confirmation guard fires only on stable).
     * -------------------------------------------------------------------- */

    public function test_api_stable_announce_without_confirmation_is_refused(): void
    {
        $resp = $this->postApi($this->body([
            'channel'  => 'stable',
            'announce' => true,
        ]));
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'stable_announce_unconfirmed');
        $this->assertSame(0, Release::count());
    }

    public function test_api_stable_announce_with_confirmation_proceeds(): void
    {
        $resp = $this->postApi($this->body([
            'channel'                 => 'stable',
            'announce'                => true,
            'confirm_stable_announce' => true,
        ]));
        $resp->assertStatus(201);
    }

    public function test_browser_stable_announce_without_confirmation_is_refused(): void
    {
        $resp = $this->postBrowser($this->body([
            'channel'  => 'stable',
            'announce' => '1',
        ]));
        $resp->assertSessionHasErrors(['confirm_stable_announce']);
        $this->assertSame(0, Release::count());
    }

    public function test_beta_announce_does_not_require_stable_confirmation(): void
    {
        // Stable-confirmation guard is scoped to channel=stable. A
        // beta publish with announce=true (announce is a no-op for
        // beta in the controllers) must NOT be blocked by the
        // confirmation rule.
        $resp = $this->postApi($this->body([
            'channel'  => 'beta',
            'announce' => true,
        ]));
        $resp->assertStatus(201);
    }

    /* --------------------------------------------------------------------
     * r2_key shape: only `releases/<version>/<filename>.exe` is allowed
     * and the version segment must match the submitted version. Without
     * this, an admin (or a leaked admin token) could pin a release row
     * at any pre-existing R2 object — even one outside the releases/
     * prefix. The shape check is in ReleaseValidator so it covers both
     * the API and browser entry points; the tests below exercise both.
     * -------------------------------------------------------------------- */

    public function test_api_publish_accepts_canonical_r2_key_shape(): void
    {
        $resp = $this->postApi($this->body([
            'r2_key' => 'releases/0.6.2/Utterbar-Setup-0.6.2.exe',
        ]));
        $resp->assertStatus(201);
    }

    public function test_api_publish_rejects_wrong_version_folder(): void
    {
        $resp = $this->postApi($this->body([
            'version' => '0.6.2',
            'r2_key'  => 'releases/0.7.0/Utterbar-Setup-0.7.0.exe',
        ]));
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'r2_key_shape');
        $this->assertSame(0, Release::count());
    }

    public function test_api_publish_rejects_traversal_in_r2_key(): void
    {
        $resp = $this->postApi($this->body([
            'r2_key' => 'releases/0.6.2/../../etc/Utterbar-Setup-0.6.2.exe',
        ]));
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'r2_key_shape');
        $this->assertSame(0, Release::count());
    }

    public function test_api_publish_rejects_nested_path_after_filename(): void
    {
        // Three-segment shape exactly: releases/<v>/<filename>. Any
        // nested directory between the version folder and the file
        // is refused so an attacker can't hide an alternate layout.
        $resp = $this->postApi($this->body([
            'r2_key' => 'releases/0.6.2/sub/Utterbar-Setup-0.6.2.exe',
        ]));
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'r2_key_shape');
    }

    public function test_api_publish_rejects_key_outside_releases_prefix(): void
    {
        $resp = $this->postApi($this->body([
            'r2_key' => 'private/0.6.2/Utterbar-Setup-0.6.2.exe',
        ]));
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'r2_key_shape');
    }

    public function test_api_publish_rejects_non_exe_filename(): void
    {
        $resp = $this->postApi($this->body([
            'r2_key' => 'releases/0.6.2/Utterbar-Setup-0.6.2.zip',
        ]));
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'r2_key_shape');
    }

    public function test_api_publish_rejects_filename_with_query_string(): void
    {
        $resp = $this->postApi($this->body([
            'r2_key' => 'releases/0.6.2/Utterbar-Setup-0.6.2.exe?evil=1',
        ]));
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'r2_key_shape');
    }

    /* ----- Same shape rules on the browser path ----- */

    public function test_browser_publish_rejects_wrong_version_folder(): void
    {
        $resp = $this->postBrowser($this->body([
            'version' => '0.6.2',
            'r2_key'  => 'releases/0.7.0/Utterbar-Setup-0.7.0.exe',
        ]));
        $resp->assertSessionHasErrors(['r2_key']);
        $this->assertSame(0, Release::count());
    }

    public function test_browser_publish_rejects_traversal_in_r2_key(): void
    {
        $resp = $this->postBrowser($this->body([
            'r2_key' => 'releases/0.6.2/../../etc/Utterbar-Setup-0.6.2.exe',
        ]));
        $resp->assertSessionHasErrors(['r2_key']);
        $this->assertSame(0, Release::count());
    }

    public function test_browser_publish_rejects_key_outside_releases_prefix(): void
    {
        $resp = $this->postBrowser($this->body([
            'r2_key' => 'private/0.6.2/Utterbar-Setup-0.6.2.exe',
        ]));
        $resp->assertSessionHasErrors(['r2_key']);
        $this->assertSame(0, Release::count());
    }

    public function test_browser_publish_rejects_non_exe_filename(): void
    {
        $resp = $this->postBrowser($this->body([
            'r2_key' => 'releases/0.6.2/Utterbar-Setup-0.6.2.zip',
        ]));
        $resp->assertSessionHasErrors(['r2_key']);
        $this->assertSame(0, Release::count());
    }

    /* ----- Shape helper directly (regression for any future caller) ----- */

    public function test_r2_key_shape_helper_accepts_canonical_form(): void
    {
        $this->assertNull(\App\Services\Releases\ReleaseValidator::r2KeyShapeError(
            'releases/0.14.13/Utterbar-Setup-0.14.13.exe',
            '0.14.13',
        ));
    }

    public function test_r2_key_shape_helper_rejects_known_bad_shapes(): void
    {
        $bad = [
            ['', '0.14.13'],
            ['releases/0.14.13/Utterbar-Setup-0.14.13.exe?x=1', '0.14.13'],
            ['releases\\0.14.13\\setup.exe', '0.14.13'],  // backslashes
            ['releases/0.14.13/sub/setup.exe', '0.14.13'],
            ['notreleases/0.14.13/setup.exe', '0.14.13'],
            ['releases/0.14.12/setup.exe', '0.14.13'],     // wrong version
            ['releases/0.14.13/setup', '0.14.13'],         // missing .exe
            ['releases/0.14.13/setup.zip', '0.14.13'],     // wrong ext
        ];
        foreach ($bad as [$key, $ver]) {
            $this->assertNotNull(
                \App\Services\Releases\ReleaseValidator::r2KeyShapeError($key, $ver),
                "expected '{$key}' (version {$ver}) to be rejected"
            );
        }
    }
}
