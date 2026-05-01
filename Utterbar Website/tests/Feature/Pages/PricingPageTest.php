<?php

namespace Tests\Feature\Pages;

use Tests\TestCase;

/**
 * Pin the volume-tier copy on the public /pricing page so it can
 * never drift out of sync with the server-side rules in
 * App\Services\Pricing\VolumePricing.
 *
 * The advertised seat bands MUST match what the controller will
 * actually accept and what the webhook will mint:
 *   * Self-serve floor:    5 seats   (VolumePricing::MIN_SEATS)
 *   * Tier boundary:       25 seats  (VolumePricing::BUSINESS_TIER_THRESHOLD)
 *   * Self-serve ceiling:  100 seats (VolumePricing::MAX_SEATS)
 *   * Above ceiling:       contact (more than 100 seats, not "100+")
 *
 * The "100+" wording is specifically discouraged because 100 is
 * itself self-serve; "more than 100" is the correct cut.
 *
 * If any of these strings drift, the customer-facing page lies
 * about what checkout will actually do, which in turn produces
 * support tickets. Pin them here.
 */
class PricingPageTest extends TestCase
{
    public function test_pricing_page_renders(): void
    {
        $this->get(route('pricing'))->assertStatus(200);
    }

    public function test_pricing_page_shows_team_tier_band(): void
    {
        $response = $this->get(route('pricing'));
        $response->assertStatus(200);
        $response->assertSee('5 to 24 seats', false);
    }

    public function test_pricing_page_shows_business_tier_band(): void
    {
        $response = $this->get(route('pricing'));
        $response->assertStatus(200);
        $response->assertSee('25 to 100 seats', false);
    }

    public function test_pricing_page_routes_above_self_serve_to_business_enquiry(): void
    {
        $response = $this->get(route('pricing'));
        $response->assertStatus(200);
        // "Need more than 100" (not "Need 100+ seats?") because 100 is
        // self-serve; the contact path covers the band ABOVE 100.
        $response->assertSee('Need more than 100', false);
    }

    public function test_pricing_page_does_not_claim_three_seat_starting_floor(): void
    {
        $response = $this->get(route('pricing'));
        $response->assertStatus(200);
        // Pre-volume-checkout copy advertised "3 to 24 seats" / "Volume
        // discounts kick in at three seats". Self-serve floor is now 5;
        // any "three seats" wording would lie about the validator.
        $response->assertDontSee('three seats', false);
    }

    public function test_pricing_page_does_not_use_100_plus_shorthand(): void
    {
        $response = $this->get(route('pricing'));
        $response->assertStatus(200);
        // "100+" is ambiguous about whether 100 itself is self-serve
        // (it is) or contact-only (it is not). Reject the shorthand.
        $response->assertDontSee('100+ seats', false);
    }
}
