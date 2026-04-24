<?php

namespace Tests\Feature;

use App\Models\Release;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

/**
 * Verifies the two API endpoints the Tauri desktop app depends on:
 *
 *   GET /api/pricing               → `pricing.rs` -> ServerPricing
 *   GET /api/updater/check         → `updater.rs` -> ServerAvailable
 *
 * These tests pin the exact JSON field names + shapes that the Rust
 * client's serde structs expect. Renaming or dropping a field here would
 * silently downgrade every installed copy of Utterbar's in-app modal to
 * its offline fallback.
 */
class ApiContractTest extends TestCase
{
    use RefreshDatabase;

    public function test_pricing_returns_expected_shape_and_gbp_price(): void
    {
        config([
            'services.stripe.pro_amount'   => 3999,
            'services.stripe.pro_currency' => 'gbp',
        ]);

        $resp = $this->getJson('/api/pricing');
        $resp->assertOk();
        $resp->assertJsonStructure([
            'amount_usd',   // legacy client field name (amount in `currency`)
            'currency',
            'period',
            'display',
        ]);
        $resp->assertJsonPath('amount_usd', 39.99);
        $resp->assertJsonPath('currency', 'GBP');
        $resp->assertJsonPath('period', 'one_time');
        $resp->assertJsonPath('display', '£39.99');
    }

    public function test_pricing_formats_whole_unit_price_without_decimals(): void
    {
        config([
            'services.stripe.pro_amount'   => 4000,
            'services.stripe.pro_currency' => 'gbp',
        ]);
        $resp = $this->getJson('/api/pricing');
        $resp->assertOk();
        $resp->assertJsonPath('display', '£40');
    }

    public function test_pricing_handles_usd_currency(): void
    {
        config([
            'services.stripe.pro_amount'   => 2900,
            'services.stripe.pro_currency' => 'usd',
        ]);
        $resp = $this->getJson('/api/pricing');
        $resp->assertOk();
        $resp->assertJsonPath('display', '$29');
    }

    public function test_updater_returns_no_update_when_no_releases_published(): void
    {
        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', false);
        $resp->assertJsonPath('current', '0.6.0');
    }

    public function test_updater_returns_newer_stable_release(): void
    {
        Release::create([
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => 'releases/0.6.2/Utterbar-Setup-0.6.2.exe',
            'sha256'         => str_repeat('a', 64),
            'notes_markdown' => "- First change\n- Second change",
            'file_size_bytes'=> 12345678,
            'published_at'   => now(),
        ]);

        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', true);
        $resp->assertJsonPath('version', '0.6.2');
        $resp->assertJsonPath('sha256', str_repeat('a', 64));
        $resp->assertJsonPath('size_bytes', 12345678);
        $resp->assertJsonPath('notes_markdown', "- First change\n- Second change");
        // download_url must be built off the CDN base + r2_key. Source
        // of truth is `filesystems.disks.r2.url` to match Release model.
        $base = rtrim((string) (config('filesystems.disks.r2.url') ?: env('CLOUDFLARE_R2_PUBLIC_URL', 'https://dl.utterbar.com')), '/');
        $resp->assertJsonPath(
            'download_url',
            $base . '/releases/0.6.2/Utterbar-Setup-0.6.2.exe'
        );
    }

    public function test_updater_does_not_downgrade_or_suggest_same_version(): void
    {
        Release::create([
            'version'        => '0.6.0',
            'channel'        => 'stable',
            'r2_key'         => 'releases/0.6.0/Utterbar-Setup-0.6.0.exe',
            'sha256'         => str_repeat('b', 64),
            'notes_markdown' => '',
            'file_size_bytes'=> 10000000,
            'published_at'   => now(),
        ]);

        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', false);

        // Running a newer client than any published release must also
        // report no-update (not downgrade to the old one).
        $resp = $this->getJson('/api/updater/check?current=0.6.5&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', false);
    }

    public function test_updater_ignores_beta_channel_when_stable_requested(): void
    {
        Release::create([
            'version'        => '0.7.0',
            'channel'        => 'beta',
            'r2_key'         => 'releases/0.7.0/Utterbar-Setup-0.7.0.exe',
            'sha256'         => str_repeat('c', 64),
            'notes_markdown' => '',
            'file_size_bytes'=> 11000000,
            'published_at'   => now(),
        ]);
        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', false);

        // Beta channel request finds the beta release.
        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=beta');
        $resp->assertOk();
        $resp->assertJsonPath('available', true);
        $resp->assertJsonPath('version', '0.7.0');
    }

    public function test_updater_rejects_malformed_current_as_no_update(): void
    {
        Release::create([
            'version'        => '0.7.0',
            'channel'        => 'stable',
            'r2_key'         => 'releases/0.7.0/Utterbar-Setup-0.7.0.exe',
            'sha256'         => str_repeat('d', 64),
            'notes_markdown' => '',
            'file_size_bytes'=> 12000000,
            'published_at'   => now(),
        ]);
        foreach (['', 'foo', '0.6', 'v0.6.0', '0.6.0-beta'] as $bad) {
            $resp = $this->getJson('/api/updater/check?current=' . urlencode($bad) . '&channel=stable');
            $resp->assertOk();
            $resp->assertJsonPath('available', false);
        }
    }

    /* --------------------------------------------------------------------
     * Fix #1 — updater must not advertise releases that cannot be installed.
     * The desktop client's `install_update` requires `sha256`, `version`,
     * and a usable `download_url`. A row missing any of them is "visible
     * but broken" and must not surface as `available: true`.
     * -------------------------------------------------------------------- */

    public function test_updater_skips_release_with_missing_sha256(): void
    {
        Release::create([
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => 'releases/0.6.2/Utterbar-Setup-0.6.2.exe',
            'sha256'         => null,
            'notes_markdown' => '',
            'file_size_bytes'=> 10000000,
            'published_at'   => now(),
        ]);
        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', false);
    }

    public function test_updater_skips_release_with_short_sha256(): void
    {
        Release::create([
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => 'releases/0.6.2/Utterbar-Setup-0.6.2.exe',
            'sha256'         => 'abcdef', // too short
            'notes_markdown' => '',
            'file_size_bytes'=> 10000000,
            'published_at'   => now(),
        ]);
        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', false);
    }

    public function test_updater_skips_release_with_non_hex_sha256(): void
    {
        Release::create([
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => 'releases/0.6.2/Utterbar-Setup-0.6.2.exe',
            'sha256'         => str_repeat('z', 64), // right length, not hex
            'notes_markdown' => '',
            'file_size_bytes'=> 10000000,
            'published_at'   => now(),
        ]);
        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', false);
    }

    public function test_updater_skips_release_with_missing_r2_key(): void
    {
        Release::create([
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => '',
            'sha256'         => str_repeat('a', 64),
            'notes_markdown' => '',
            'file_size_bytes'=> 10000000,
            'published_at'   => now(),
        ]);
        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', false);
    }

    public function test_updater_falls_back_to_older_valid_when_newest_is_malformed(): void
    {
        // Older but valid:
        Release::create([
            'version'        => '0.6.1',
            'channel'        => 'stable',
            'r2_key'         => 'releases/0.6.1/Utterbar-Setup-0.6.1.exe',
            'sha256'         => str_repeat('a', 64),
            'notes_markdown' => 'older valid',
            'file_size_bytes'=> 9000000,
            'published_at'   => now()->subDay(),
        ]);
        // Newer but missing sha256 — must NOT shadow the older valid row.
        Release::create([
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => 'releases/0.6.2/Utterbar-Setup-0.6.2.exe',
            'sha256'         => null,
            'notes_markdown' => 'newer broken',
            'file_size_bytes'=> 10000000,
            'published_at'   => now(),
        ]);

        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', true);
        $resp->assertJsonPath('version', '0.6.1');
        $resp->assertJsonPath('notes_markdown', 'older valid');
    }

    /* --------------------------------------------------------------------
     * Fix #3 — prerelease versions are rejected at publish time and
     * filtered at read time. The updater and the publish paths must be
     * consistent: if we can't compare a version, we never create it and
     * never serve it.
     * -------------------------------------------------------------------- */

    public function test_updater_skips_prerelease_row_even_if_persisted(): void
    {
        // Simulate a pre-existing DB row from before the publish regex
        // was tightened. Must NOT surface via the updater.
        Release::create([
            'version'        => '0.7.0-rc.1',
            'channel'        => 'stable',
            'r2_key'         => 'releases/0.7.0-rc.1/Utterbar-Setup-0.7.0-rc.1.exe',
            'sha256'         => str_repeat('a', 64),
            'notes_markdown' => '',
            'file_size_bytes'=> 10000000,
            'published_at'   => now(),
        ]);
        $resp = $this->getJson('/api/updater/check?current=0.6.0&channel=stable');
        $resp->assertOk();
        $resp->assertJsonPath('available', false);
    }
}
