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
 * Admin dashboard revenue accounting (the Gross / Net of refunds
 * tiles on /admin).
 *
 * Pre-fix the controller computed revenue as
 *   licences_total * services.stripe.pro_amount
 * which silently miscounted volume rows (priced per seat at £29 or
 * £19, not £39.99). The fix stores the actual purchase amount on
 * each new Stripe-minted row and the dashboard SUMs the column.
 *
 * What's pinned here:
 *   * Gross is the SUM(amount_pence) over rows where amount_pence
 *     is NOT NULL.
 *   * Net subtracts refunded rows but NOT revoked rows (existing
 *     semantic — revoke can be an admin action with no money back).
 *   * Mixed seed (single-user + volume + a refunded volume row +
 *     a legacy null-amount row) produces the right gross/net and
 *     surfaces the legacy row in the new "Untracked rows" tile.
 *   * Empty DB renders zero gross / zero net / zero untracked.
 */
class DashboardRevenueTest extends TestCase
{
    use RefreshDatabase;

    protected function setUp(): void
    {
        parent::setUp();
        // Pin currency + per-seat rates so the asserted pence values
        // don't depend on whatever env the test happens to inherit.
        config([
            'services.stripe.pro_amount'                      => 3999, // £39.99
            'services.stripe.pro_currency'                    => 'gbp',
            'services.stripe.volume_per_seat_pence'           => 2900, // £29
            'services.stripe.volume_per_seat_business_pence'  => 1900, // £19
            'services.licence.sku_string'                     => 'utterbar-pro-lifetime',
            'services.licence.sku_string_volume'              => 'utterbar-pro-volume',
        ]);
    }

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

    /**
     * Lightweight licence factory. `amount_pence` and `currency`
     * default to null (legacy / admin-manual shape) so each test
     * only fills them when it's testing a Stripe-minted row.
     */
    private function seedLicence(array $overrides = []): Licence
    {
        return Licence::create(array_merge([
            'key_lookup'   => strtoupper(Str::random(20)),
            'email'        => 'buyer-' . Str::random(6) . '@example.com',
            'sku'          => 'utterbar-pro-lifetime',
            'max_devices'  => 3,
            'status'       => 'active',
            'issued_at'    => now(),
            'amount_pence' => null,
            'currency'     => null,
        ], $overrides));
    }

    public function test_empty_dashboard_reports_zero_revenue_and_zero_untracked(): void
    {
        $admin = $this->seedAdmin();
        $response = $this->actingAs($admin)->get(route('admin.index'));

        $response->assertStatus(200);
        $html = $response->getContent();

        $this->assertMatchesRegularExpression('/Gross.*?>\s*GBP 0\.00\s*</s', $html);
        $this->assertMatchesRegularExpression('/Net of refunds.*?>\s*GBP 0\.00\s*</s', $html);
        $this->assertMatchesRegularExpression('/Untracked rows.*?>\s*0\s*</s', $html);
    }

    public function test_gross_sums_amount_pence_across_single_user_and_volume(): void
    {
        $admin = $this->seedAdmin();

        // Single-user £39.99
        $this->seedLicence([
            'sku'          => 'utterbar-pro-lifetime',
            'max_devices'  => 3,
            'amount_pence' => 3999,
            'currency'     => 'gbp',
        ]);
        // Volume team-tier 10 seats × £29 = £290.00
        $this->seedLicence([
            'sku'          => VolumePricing::sku(),
            'max_devices'  => 10,
            'amount_pence' => 10 * 2900,
            'currency'     => 'gbp',
        ]);
        // Volume business-tier 50 seats × £19 = £950.00
        $this->seedLicence([
            'sku'          => VolumePricing::sku(),
            'max_devices'  => 50,
            'amount_pence' => 50 * 1900,
            'currency'     => 'gbp',
        ]);

        // Gross expectation: 3999 + 29000 + 95000 = 127999 pence =
        // £1279.99. Per-tile copy is "GBP 1,279.99".
        $response = $this->actingAs($admin)->get(route('admin.index'));
        $response->assertStatus(200);
        $html = $response->getContent();

        $this->assertMatchesRegularExpression('/Gross.*?>\s*GBP 1,279\.99\s*</s', $html);
        // No refunds → Net == Gross.
        $this->assertMatchesRegularExpression('/Net of refunds.*?>\s*GBP 1,279\.99\s*</s', $html);
        $this->assertMatchesRegularExpression('/Untracked rows.*?>\s*0\s*</s', $html);
    }

    public function test_net_excludes_refunded_rows(): void
    {
        $admin = $this->seedAdmin();

        // Two single-user purchases (one refunded), one volume sale.
        // Single-user active:    £39.99
        // Single-user refunded:  £39.99  ← excluded from Net
        // Volume 25 seats × £19: £475.00
        $this->seedLicence([
            'sku'          => 'utterbar-pro-lifetime',
            'amount_pence' => 3999,
            'currency'     => 'gbp',
            'status'       => 'active',
        ]);
        $this->seedLicence([
            'sku'          => 'utterbar-pro-lifetime',
            'amount_pence' => 3999,
            'currency'     => 'gbp',
            'status'       => 'refunded',
        ]);
        $this->seedLicence([
            'sku'          => VolumePricing::sku(),
            'max_devices'  => 25,
            'amount_pence' => 25 * 1900, // 47500
            'currency'     => 'gbp',
            'status'       => 'active',
        ]);

        // Gross = 3999 + 3999 + 47500 = 55498 pence = £554.98
        // Net   = 3999 + 47500          = 51499 pence = £514.99
        $response = $this->actingAs($admin)->get(route('admin.index'));
        $response->assertStatus(200);
        $html = $response->getContent();

        $this->assertMatchesRegularExpression('/Gross.*?>\s*GBP 554\.98\s*</s', $html);
        $this->assertMatchesRegularExpression('/Net of refunds.*?>\s*GBP 514\.99\s*</s', $html);
    }

    public function test_revoked_rows_remain_in_net_revenue(): void
    {
        // A revoked-without-refund row keeps contributing to Net —
        // revoke is a no-money-back admin action by default. This
        // pins the existing semantic so a future change has to
        // explicitly update this test rather than silently shift
        // the figure.
        $admin = $this->seedAdmin();

        $this->seedLicence([
            'sku'          => 'utterbar-pro-lifetime',
            'amount_pence' => 3999,
            'currency'     => 'gbp',
            'status'       => 'active',
        ]);
        $this->seedLicence([
            'sku'          => 'utterbar-pro-lifetime',
            'amount_pence' => 3999,
            'currency'     => 'gbp',
            'status'       => 'revoked',
        ]);

        // Both rows are tracked; both contribute to Gross AND Net.
        $response = $this->actingAs($admin)->get(route('admin.index'));
        $html = $response->getContent();

        $this->assertMatchesRegularExpression('/Gross.*?>\s*GBP 79\.98\s*</s', $html);
        $this->assertMatchesRegularExpression('/Net of refunds.*?>\s*GBP 79\.98\s*</s', $html);
    }

    public function test_legacy_null_amount_rows_are_excluded_and_surfaced(): void
    {
        $admin = $this->seedAdmin();

        // One tracked row + two legacy / admin-mint rows with no
        // amount captured. Gross/Net read only the tracked row;
        // the Untracked tile reports the count of the rest.
        $this->seedLicence([
            'sku'          => 'utterbar-pro-lifetime',
            'amount_pence' => 3999,
            'currency'     => 'gbp',
        ]);
        // Admin manual mint shape: no Stripe payment, no amount.
        $this->seedLicence();
        // Pre-2026-05-02 row: minted before the column existed,
        // simulated by leaving amount_pence null.
        $this->seedLicence();

        $response = $this->actingAs($admin)->get(route('admin.index'));
        $html = $response->getContent();

        $this->assertMatchesRegularExpression('/Gross.*?>\s*GBP 39\.99\s*</s', $html);
        $this->assertMatchesRegularExpression('/Net of refunds.*?>\s*GBP 39\.99\s*</s', $html);
        // Two untracked rows present.
        $this->assertMatchesRegularExpression('/Untracked rows.*?>\s*2\s*</s', $html);
    }

    public function test_untracked_tile_reads_zero_when_no_legacy_rows(): void
    {
        // Sanity check: the danger-style highlight on the Untracked
        // tile is only meaningful when there ARE both tracked AND
        // untracked rows. With only tracked rows, the count is 0
        // and the tile is in its quiet state.
        $admin = $this->seedAdmin();

        $this->seedLicence([
            'sku'          => 'utterbar-pro-lifetime',
            'amount_pence' => 3999,
            'currency'     => 'gbp',
        ]);

        $response = $this->actingAs($admin)->get(route('admin.index'));
        $html = $response->getContent();

        $this->assertMatchesRegularExpression('/Untracked rows.*?>\s*0\s*</s', $html);
    }
}
