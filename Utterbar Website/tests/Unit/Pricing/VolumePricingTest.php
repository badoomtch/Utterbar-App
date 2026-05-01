<?php

namespace Tests\Unit\Pricing;

use App\Services\Pricing\VolumePricing;
use Tests\TestCase;

/**
 * Pure logic tests for the volume pricing helper.
 *
 * The helper is the single source of truth for the boundary
 * (BUSINESS_TIER_THRESHOLD), the seat range (MIN_SEATS / MAX_SEATS),
 * the per-seat price lookup, and the total computation. The
 * controller and the webhook BOTH read these; if any of these
 * tests change, both production callsites need a re-read.
 */
class VolumePricingTest extends TestCase
{
    public function test_seat_range_constants_match_brief(): void
    {
        $this->assertSame(5,   VolumePricing::MIN_SEATS);
        $this->assertSame(100, VolumePricing::MAX_SEATS);
        $this->assertSame(25,  VolumePricing::BUSINESS_TIER_THRESHOLD);
    }

    public function test_unit_pence_is_team_tier_below_threshold(): void
    {
        config([
            'services.stripe.volume_per_seat_pence'          => 2900,
            'services.stripe.volume_per_seat_business_pence' => 1900,
        ]);
        $this->assertSame(2900, VolumePricing::unitPence(5));
        $this->assertSame(2900, VolumePricing::unitPence(10));
        $this->assertSame(2900, VolumePricing::unitPence(24));
    }

    public function test_unit_pence_is_business_tier_at_and_above_threshold(): void
    {
        config([
            'services.stripe.volume_per_seat_pence'          => 2900,
            'services.stripe.volume_per_seat_business_pence' => 1900,
        ]);
        $this->assertSame(1900, VolumePricing::unitPence(25));
        $this->assertSame(1900, VolumePricing::unitPence(50));
        $this->assertSame(1900, VolumePricing::unitPence(100));
    }

    public function test_total_pence_is_unit_times_quantity(): void
    {
        config([
            'services.stripe.volume_per_seat_pence'          => 2900,
            'services.stripe.volume_per_seat_business_pence' => 1900,
        ]);
        // 5 seats * £29 = £145.
        $this->assertSame(2900 * 5, VolumePricing::totalPence(5));
        // 24 seats * £29 = £696 (just below the boundary).
        $this->assertSame(2900 * 24, VolumePricing::totalPence(24));
        // 25 seats * £19 = £475 (at the boundary, drops to business tier).
        $this->assertSame(1900 * 25, VolumePricing::totalPence(25));
        // 100 seats * £19 = £1900.
        $this->assertSame(1900 * 100, VolumePricing::totalPence(100));
    }

    /**
     * Boundary monotonicity: the total at quantity 25 should NOT
     * exceed the total at quantity 24, despite the per-seat drop.
     * (£29 * 24 = £696, £19 * 25 = £475 — total falls.) The check
     * here is "no perverse incentive to buy 24 instead of 25".
     */
    public function test_business_tier_kicks_in_at_25_with_no_perverse_incentive(): void
    {
        config([
            'services.stripe.volume_per_seat_pence'          => 2900,
            'services.stripe.volume_per_seat_business_pence' => 1900,
        ]);
        $this->assertLessThan(
            VolumePricing::totalPence(24),
            VolumePricing::totalPence(25),
            'business tier total at 25 must be strictly less than team tier total at 24'
        );
    }

    public function test_currency_falls_through_to_pro_currency(): void
    {
        config(['services.stripe.pro_currency' => 'gbp']);
        $this->assertSame('gbp', VolumePricing::currency());
        config(['services.stripe.pro_currency' => 'USD']); // upper-case input, lower-case output
        $this->assertSame('usd', VolumePricing::currency());
    }

    public function test_sku_reads_from_config_with_default(): void
    {
        config(['services.licence.sku_string_volume' => null]);
        $this->assertSame('utterbar-pro-volume', VolumePricing::sku());
        config(['services.licence.sku_string_volume' => 'utterbar-pro-volume-test']);
        $this->assertSame('utterbar-pro-volume-test', VolumePricing::sku());
    }

    public function test_is_valid_quantity_accepts_only_ints_in_range(): void
    {
        $this->assertTrue(VolumePricing::isValidQuantity(5));
        $this->assertTrue(VolumePricing::isValidQuantity(50));
        $this->assertTrue(VolumePricing::isValidQuantity(100));
    }

    public function test_is_valid_quantity_rejects_out_of_range(): void
    {
        $this->assertFalse(VolumePricing::isValidQuantity(0));
        $this->assertFalse(VolumePricing::isValidQuantity(4));
        $this->assertFalse(VolumePricing::isValidQuantity(101));
        $this->assertFalse(VolumePricing::isValidQuantity(-1));
        $this->assertFalse(VolumePricing::isValidQuantity(PHP_INT_MAX));
    }

    /**
     * The helper is strict on type — strings, floats, null, arrays,
     * booleans all return false. Worth pinning explicitly because
     * the request-side Laravel `integer` rule is MORE permissive:
     * it accepts numeric strings like "5" (which parse as ints) and
     * leaves the controller to cast the validated value via
     * `(int) $validated['quantity']` before any pricing math. This
     * helper sits downstream of that cast on the request path, but
     * is also called from non-request paths (the webhook re-checks
     * metadata.quantity here) where no Laravel validator runs first.
     * For those callers, the strict type predicate is the only fence.
     */
    public function test_is_valid_quantity_rejects_non_int_types(): void
    {
        $this->assertFalse(VolumePricing::isValidQuantity('5'));
        $this->assertFalse(VolumePricing::isValidQuantity(5.0));
        $this->assertFalse(VolumePricing::isValidQuantity(5.5));
        $this->assertFalse(VolumePricing::isValidQuantity(null));
        $this->assertFalse(VolumePricing::isValidQuantity(true));
        $this->assertFalse(VolumePricing::isValidQuantity([]));
    }
}
