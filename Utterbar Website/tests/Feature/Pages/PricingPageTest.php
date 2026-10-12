<?php

namespace Tests\Feature\Pages;

use Tests\TestCase;

/**
 * Pin the /pricing page's core message now that Utterbar is free and
 * open source. The page must render, say it is free, and carry no
 * residual paid-tier language (a price, a "Buy" CTA, volume seats).
 */
class PricingPageTest extends TestCase
{
    public function test_pricing_page_renders(): void
    {
        $this->get(route('pricing'))->assertStatus(200);
    }

    public function test_pricing_page_says_it_is_free_and_open_source(): void
    {
        $response = $this->get(route('pricing'));
        $response->assertStatus(200);
        $response->assertSee('Free and open source', false);
        $response->assertSee(config('site.github_repository_url'), false);
    }

    public function test_pricing_page_has_no_price_or_buy_cta(): void
    {
        $response = $this->get(route('pricing'));
        $response->assertStatus(200);
        // No paid-tier residue: no GBP price, no Pro upsell, no seats.
        $response->assertDontSee('£39.99', false);
        $response->assertDontSee('per seat', false);
        $response->assertDontSee('Buy Pro', false);
        $response->assertDontSee('our servers do are host the installer', false);
    }

    public function test_pricing_page_offers_a_download(): void
    {
        $response = $this->get(route('pricing'));
        $response->assertStatus(200);
        $response->assertSee(route('download'), false);
    }
}
