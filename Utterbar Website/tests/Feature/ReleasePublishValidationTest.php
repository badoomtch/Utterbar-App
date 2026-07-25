<?php

namespace Tests\Feature;

use App\Models\Release;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Storage;
use Tests\TestCase;

/**
 * Publish-time validation on the release table's entry point:
 *
 *   - POST /api/admin/releases   (AdminReleaseApiController, bearer auth)
 *
 * The rules enforced:
 *
 *   1. SHA-256 is REQUIRED. Blank / missing / wrong-length / non-hex
 *      values all reject with a 422.
 *   2. Version must be strict X.Y.Z — no prerelease suffixes like
 *      `0.7.0-rc.1`. Admin publish path must match what the updater can
 *      actually compare.
 *
 * The headless bearer-token API is the only publish surface: the app is
 * free and open source, but only releases signed with the operator's
 * updater key are served. (The old session-authed admin web form was
 * removed with the rest of the accounts/admin panel.)
 */
class ReleasePublishValidationTest extends TestCase
{
    use RefreshDatabase;

    private const ADMIN_TOKEN = 'test-admin-token-for-release-validation';

    protected function setUp(): void
    {
        parent::setUp();
        // Ensure the admin.api middleware accepts our Bearer header, and
        // the R2 disk under test doesn't require real credentials.
        putenv('ADMIN_API_TOKEN=' . self::ADMIN_TOKEN);
        $_ENV['ADMIN_API_TOKEN'] = self::ADMIN_TOKEN;
        Storage::fake('r2');
    }

    /**
     * Common body fields for a well-formed publish request.
     *
     * The validator recomputes the SHA-256 from the R2 object and rejects
     * pasted SHAs that don't match. Tests that want a successful publish
     * must therefore pass the SHA of the bytes they're seeding. We seed
     * deterministic bytes ("utterbar-test-installer-<r2_key>") and
     * pre-compute their hash so callers can spread `body()` and get a
     * passing case.
     *
     * Tests that want to assert SHA mismatch can override `sha256` with
     * any other 64-hex string and the validator will reject.
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

    /* ----- sha256 is required ----- */

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

    /* ----- version must be X.Y.Z (no prerelease) ----- */

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
        // bytes we seeded. The ReleaseValidator must catch it.
        $body['sha256'] = str_repeat('b', 64);
        $resp = $this->postApi($body);
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'sha_mismatch');
        $this->assertSame(0, Release::count(), 'mismatch must not persist');
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
     * r2_key shape: only `releases/<version>/<filename>.exe` is allowed
     * and the version segment must match the submitted version. Without
     * this, a leaked admin token could pin a release row at any
     * pre-existing R2 object — even one outside the releases/ prefix.
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
