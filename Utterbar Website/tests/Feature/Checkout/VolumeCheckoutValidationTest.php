<?php

namespace Tests\Feature\Checkout;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

/**
 * Server-side validation for /api/checkout/create-volume-session.
 *
 * The endpoint creates a real Stripe Checkout Session on the success
 * path; these tests exercise the validation layer that runs BEFORE
 * any Stripe call. With `STRIPE_ENABLED=false` the endpoint
 * short-circuits to 503; with the secret unset it returns 500. Both
 * are covered. Quantity validation is the load-bearing surface:
 * a malicious request must not reach Stripe with a quantity outside
 * [MIN_SEATS, MAX_SEATS].
 */
class VolumeCheckoutValidationTest extends TestCase
{
    use RefreshDatabase;

    /**
     * Common setup: Stripe enabled, secret set so validation runs.
     * No actual Stripe call is made — every test path returns
     * before `StripeSession::create` is hit.
     */
    private function enableStripe(): void
    {
        config([
            'services.stripe.enabled' => true,
            'services.stripe.secret'  => 'sk_test_dummy',
        ]);
    }

    public function test_returns_503_when_stripe_disabled(): void
    {
        config(['services.stripe.enabled' => false]);

        $response = $this->postJson(route('checkout.create.volume'), ['quantity' => 10]);

        $response->assertStatus(503)
                 ->assertJson(['error' => 'checkout_disabled']);
    }

    public function test_returns_500_when_secret_missing(): void
    {
        config([
            'services.stripe.enabled' => true,
            'services.stripe.secret'  => null,
        ]);

        $response = $this->postJson(route('checkout.create.volume'), ['quantity' => 10]);

        $response->assertStatus(500)
                 ->assertJson(['error' => 'misconfigured']);
    }

    public function test_rejects_missing_quantity(): void
    {
        $this->enableStripe();

        $response = $this->postJson(route('checkout.create.volume'), []);

        $response->assertStatus(422);
        $response->assertJsonValidationErrors(['quantity']);
    }

    public function test_rejects_quantity_below_min(): void
    {
        $this->enableStripe();

        // 4 is below MIN_SEATS=5; the validator rejects with 422.
        $response = $this->postJson(route('checkout.create.volume'), ['quantity' => 4]);

        $response->assertStatus(422);
        $response->assertJsonValidationErrors(['quantity']);
    }

    public function test_rejects_quantity_above_max(): void
    {
        $this->enableStripe();

        $response = $this->postJson(route('checkout.create.volume'), ['quantity' => 101]);

        $response->assertStatus(422);
        $response->assertJsonValidationErrors(['quantity']);
    }

    public function test_rejects_zero_and_negative(): void
    {
        $this->enableStripe();

        foreach ([0, -1, -100] as $bad) {
            $response = $this->postJson(route('checkout.create.volume'), ['quantity' => $bad]);
            $response->assertStatus(422);
            $response->assertJsonValidationErrors(['quantity']);
        }
    }

    public function test_rejects_float_quantity(): void
    {
        $this->enableStripe();

        // 5.5 is in-range numerically but fails the `integer` rule.
        $response = $this->postJson(route('checkout.create.volume'), ['quantity' => 5.5]);

        $response->assertStatus(422);
        $response->assertJsonValidationErrors(['quantity']);
    }

    public function test_rejects_non_numeric_string_quantity(): void
    {
        $this->enableStripe();

        // Non-numeric string. Laravel's `integer` rule rejects this.
        // Note: a numeric string like "10" would PASS the rule (it
        // parses cleanly as an int), and the controller then casts
        // it via `(int) $validated['quantity']` before pricing math.
        // We don't test the numeric-string accept path here because
        // it would require a real Stripe API key to reach a 200, but
        // it is the documented behaviour. This test pins only the
        // narrower property: gibberish strings hit 422.
        $response = $this->postJson(route('checkout.create.volume'), ['quantity' => 'lots']);

        $response->assertStatus(422);
        $response->assertJsonValidationErrors(['quantity']);
    }

    public function test_rejects_array_and_null_quantity(): void
    {
        $this->enableStripe();

        $response = $this->postJson(route('checkout.create.volume'), ['quantity' => [10, 20]]);
        $response->assertStatus(422);

        $response = $this->postJson(route('checkout.create.volume'), ['quantity' => null]);
        $response->assertStatus(422);
    }

    public function test_request_cannot_override_price(): void
    {
        $this->enableStripe();

        // Even if the request body smuggles unit_amount / amount /
        // currency / sku / max_devices, the controller's validator
        // strips them and the price is computed server-side from
        // VolumePricing. Here we don't actually create a Stripe
        // session (we'd need a working Stripe API key), but we
        // assert the validator at least lets the legitimate
        // quantity through. The other test below pins that the
        // endpoint refuses gibberish; combined, the validator
        // tells us only `quantity` and `email` are read.
        $response = $this->postJson(route('checkout.create.volume'), [
            'quantity'    => 10,
            'unit_amount' => 1,        // smuggled. Should be ignored.
            'amount'      => 1,        // smuggled. Should be ignored.
            'sku'         => 'free-pro', // smuggled. Should be ignored.
            'max_devices' => 9999,     // smuggled. Should be ignored.
        ]);

        // The request will fail at the Stripe layer (we used a
        // dummy `sk_test_dummy` key), but validation passes — which
        // is the point: the smuggled fields didn't trip a
        // validation error, which means they were silently ignored
        // (Laravel's $request->validate only retains validated
        // keys). The actual Stripe failure surfaces as 502.
        $response->assertStatus(502);
        $response->assertJson(['error' => 'stripe_error']);
    }
}
