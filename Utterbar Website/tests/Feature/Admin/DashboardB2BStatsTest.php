<?php

namespace Tests\Feature\Admin;

use App\Models\Licence;
use App\Models\User;
use App\Services\Pricing\VolumePricing;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Str;
use Tests\TestCase;

/**
 * Admin dashboard (GET /admin) B2B rollup tiles.
 *
 * Three new stat cards: total volume licences (any status), active
 * volume licences, total seats sold across volume rows. Single-user
 * rows must NOT contribute. Auth gate still applies.
 */
class DashboardB2BStatsTest extends TestCase
{
    use RefreshDatabase;

    private function seedAdmin(string $email = 'admin@example.com'): User
    {
        putenv('ADMIN_EMAILS=' . $email);
        return User::create([
            'name'              => 'Admin',
            'email'             => $email,
            'password'          => Hash::make(Str::random(64)),
            'email_verified_at' => now(),
        ]);
    }

    private function seedLicence(string $sku, int $maxDevices, string $status = 'active'): Licence
    {
        return Licence::create([
            'key_lookup'  => strtoupper(Str::random(20)),
            'email'       => 'buyer-' . Str::random(6) . '@example.com',
            'sku'         => $sku,
            'max_devices' => $maxDevices,
            'status'      => $status,
            'issued_at'   => now(),
        ]);
    }

    public function test_unauthenticated_dashboard_request_is_refused(): void
    {
        $response = $this->get(route('admin.index'));
        $this->assertNotEquals(200, $response->status());
    }

    public function test_dashboard_shows_b2b_stat_labels(): void
    {
        $admin = $this->seedAdmin();
        $response = $this->actingAs($admin)->get(route('admin.index'));

        $response->assertStatus(200);
        // Labels render even on an empty DB. Counts are pinned by
        // the next test below.
        $response->assertSee('Volume licences', false);
        $response->assertSee('Active volume', false);
        $response->assertSee('Seats sold (volume)', false);
    }

    public function test_dashboard_counts_volume_licences_correctly(): void
    {
        $admin = $this->seedAdmin();

        // Seed: 3 volume rows totalling 5+25+50 = 80 seats. One of
        // them is refunded so the active count is 2 of 3.
        $this->seedLicence(VolumePricing::sku(),  5, 'active');
        $this->seedLicence(VolumePricing::sku(), 25, 'active');
        $this->seedLicence(VolumePricing::sku(), 50, 'refunded');

        // Two single-user rows. These must NOT contribute to any of
        // the three new stats — that's the load-bearing property.
        $this->seedLicence('utterbar-pro-lifetime', 3, 'active');
        $this->seedLicence('utterbar-pro-lifetime', 3, 'active');

        $response = $this->actingAs($admin)->get(route('admin.index'));
        $response->assertStatus(200);

        $html = $response->getContent();

        // The admin-stat component renders label-first (in a <p>),
        // then value (in a separate <p>). Match each label and its
        // following numeric value with a non-greedy gap, so a future
        // wrapper restyle (different element, additional spacing,
        // extra inline styles) still resolves correctly.
        $this->assertMatchesRegularExpression(
            '/Volume licences.*?>\s*3\s*</s',
            $html,
            'Volume licences total should be 3 (5+25+50 rows)'
        );
        $this->assertMatchesRegularExpression(
            '/Active volume.*?>\s*2\s*</s',
            $html,
            'Active volume should be 2 (5-seat + 25-seat active rows; 50-seat row is refunded)'
        );
        $this->assertMatchesRegularExpression(
            '/Seats sold \(volume\).*?>\s*80\s*</s',
            $html,
            'Seats sold should be 80 (5+25+50, sum across all volume rows)'
        );
    }

    public function test_dashboard_b2b_stats_are_zero_when_no_volume_rows(): void
    {
        $admin = $this->seedAdmin();
        // Single-user rows only.
        $this->seedLicence('utterbar-pro-lifetime', 3, 'active');
        $this->seedLicence('utterbar-pro-lifetime', 3, 'refunded');

        $response = $this->actingAs($admin)->get(route('admin.index'));
        $response->assertStatus(200);

        $html = $response->getContent();
        $this->assertMatchesRegularExpression('/Volume licences.*?>\s*0\s*</s', $html);
        $this->assertMatchesRegularExpression('/Active volume.*?>\s*0\s*</s', $html);
        $this->assertMatchesRegularExpression('/Seats sold \(volume\).*?>\s*0\s*</s', $html);
    }
}
