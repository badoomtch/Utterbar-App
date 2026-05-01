<?php

namespace Tests\Feature\Checkout;

use App\Http\Controllers\StripeWebhookController;
use App\Mail\LicenceDelivery;
use App\Models\Licence;
use App\Services\Pricing\VolumePricing;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Mail;
use ReflectionMethod;
use Tests\TestCase;

/**
 * Webhook minting for both single-user and volume SKUs.
 *
 * The Stripe-Signature verification path is exercised by the SDK at
 * the route layer; these tests bypass the route and call the
 * handler's protected `handleCheckoutCompleted` directly via
 * reflection with a synthetic session object. This lets us assert
 * the mint logic without re-implementing Stripe's HMAC.
 *
 * What's pinned:
 *   * single-user mint creates a licence with max_devices = 3
 *     (or whatever services.licence.max_devices says) and the
 *     single-user SKU.
 *   * volume mint creates a licence with max_devices = quantity and
 *     the volume SKU.
 *   * volume mint refuses out-of-range quantities even if a tampered
 *     metadata field gets past the controller (defence in depth).
 *   * idempotency: a duplicate session does not mint a second row.
 *   * licence-delivery email is queued for both branches; the
 *     mailable carries the maxDevices view-data.
 */
class StripeWebhookMintTest extends TestCase
{
    use RefreshDatabase;

    /**
     * Generate a working licence keypair for the test run so
     * LicenceSigner::sign succeeds. Without this, the signer throws
     * "LICENCE_PRIVATE_KEY_B64 is not configured" and the mint
     * transaction rolls back.
     */
    protected function setUp(): void
    {
        parent::setUp();

        $kp     = sodium_crypto_sign_keypair();
        $sk     = sodium_crypto_sign_secretkey($kp);
        $pk     = sodium_crypto_sign_publickey($kp);

        config([
            'services.licence.private_key_b64'      => base64_encode($sk),
            'services.licence.public_key_b64'       => base64_encode($pk),
            'services.licence.payload_version'      => 1,
            'services.licence.sku_id'               => 1,
            'services.licence.sku_string'           => 'utterbar-pro-lifetime',
            'services.licence.sku_string_volume'    => 'utterbar-pro-volume',
            'services.licence.max_devices'          => 3,
            'services.stripe.volume_per_seat_pence'           => 2900,
            'services.stripe.volume_per_seat_business_pence'  => 1900,
            'services.stripe.pro_currency'          => 'gbp',
            'services.stripe.pro_amount'            => 3999,
            // Suppress AdminNotify mail in these tests; it's covered
            // separately. Setting notify_email to null makes
            // AdminNotify a silent no-op.
            'services.admin.notify_email'           => null,
        ]);
    }

    /**
     * Build a synthetic Stripe Session-shaped object. Real Stripe
     * Session objects subclass StripeObject, but the webhook
     * handler only reads scalar properties via the magic getter, so
     * a plain stdClass with the right shape is indistinguishable.
     */
    private function makeSession(array $overrides = []): object
    {
        $defaults = [
            'id'              => 'cs_test_' . bin2hex(random_bytes(8)),
            'payment_status'  => 'paid',
            'customer'        => 'cus_test_' . bin2hex(random_bytes(6)),
            'payment_intent'  => 'pi_test_' . bin2hex(random_bytes(8)),
            'customer_email'  => 'buyer@example.com',
            'metadata'        => (object) ['sku' => 'utterbar-pro-lifetime'],
            'customer_details' => (object) ['email' => 'buyer@example.com'],
        ];
        $merged = array_replace($defaults, $overrides);
        return (object) $merged;
    }

    /** Reflection helper to call the protected handler. */
    private function callHandler(object $session): void
    {
        $controller = new StripeWebhookController();
        $m = new ReflectionMethod($controller, 'handleCheckoutCompleted');
        $m->setAccessible(true);
        $m->invoke($controller, $session);
    }

    public function test_single_user_mint_creates_3_device_licence(): void
    {
        Mail::fake();

        // No amount_total / currency on the synthetic session, so
        // the helper falls back to services.stripe.pro_amount /
        // pro_currency. This test pins the *fallback* path; the
        // session-amount-wins behaviour is in
        // test_single_user_mint_persists_session_amount_total below.
        $session = $this->makeSession([
            'metadata' => (object) ['sku' => 'utterbar-pro-lifetime'],
        ]);

        $this->callHandler($session);

        $row = Licence::where('stripe_checkout_session_id', $session->id)->first();
        $this->assertNotNull($row, 'single-user licence row was not minted');
        $this->assertSame('utterbar-pro-lifetime', $row->sku);
        $this->assertSame(3, $row->max_devices);
        $this->assertSame('buyer@example.com', $row->email);
        $this->assertSame('active', $row->status);
        $this->assertNotEmpty($row->signed_blob, 'signed_blob must be populated by the signer');

        // Fallback: services.stripe.pro_amount = 3999 (per setUp).
        $this->assertSame(3999, $row->amount_pence);
        $this->assertSame('gbp', $row->currency);

        Mail::assertSent(LicenceDelivery::class, function ($mail) use ($row) {
            return $mail->licence->id === $row->id;
        });
    }

    public function test_single_user_mint_persists_session_amount_total(): void
    {
        Mail::fake();

        // The buyer was charged 4500 pence (£45.00) at checkout.
        // The pro_amount config is 3999. The webhook MUST persist
        // what the buyer paid, NOT what the current config would
        // recompute. This is the load-bearing property: a price
        // change between Checkout-Session creation and webhook
        // delivery cannot rewrite history.
        $session = $this->makeSession([
            'metadata'     => (object) ['sku' => 'utterbar-pro-lifetime'],
            'amount_total' => 4500,
            'currency'     => 'gbp',
        ]);

        $this->callHandler($session);

        $row = Licence::where('stripe_checkout_session_id', $session->id)->first();
        $this->assertNotNull($row);
        $this->assertSame(4500, $row->amount_pence, 'session amount_total must win over pro_amount config');
        $this->assertSame('gbp', $row->currency);
    }

    public function test_single_user_mint_lowercases_session_currency(): void
    {
        Mail::fake();

        // Stripe sends currency lowercase in practice, but defensive:
        // if a session ever carries an uppercase code (Stripe libs
        // returning ISO-style "USD", a test fixture, a custom
        // checkout integration), the persisted column must still
        // be lowercase so AdminController's most-common-currency
        // grouping stays consistent.
        $session = $this->makeSession([
            'metadata'     => (object) ['sku' => 'utterbar-pro-lifetime'],
            'amount_total' => 5000,
            'currency'     => 'USD',
        ]);

        $this->callHandler($session);

        $row = Licence::where('stripe_checkout_session_id', $session->id)->first();
        $this->assertSame(5000, $row->amount_pence);
        $this->assertSame('usd', $row->currency);
    }

    public function test_volume_mint_creates_licence_with_quantity_max_devices(): void
    {
        Mail::fake();

        $quantity = 50; // business tier
        // No amount_total / currency on the synthetic session, so
        // the helper falls back to VolumePricing::totalPence /
        // currency. This test pins the *fallback* path; session-
        // amount-wins is in
        // test_volume_mint_persists_session_amount_total below.
        $session = $this->makeSession([
            'metadata' => (object) [
                'sku'         => VolumePricing::sku(),
                'quantity'    => (string) $quantity,
                'max_devices' => (string) $quantity,
                'unit_pence'  => '1900',
                'currency'    => 'gbp',
            ],
        ]);

        $this->callHandler($session);

        $row = Licence::where('stripe_checkout_session_id', $session->id)->first();
        $this->assertNotNull($row, 'volume licence row was not minted');
        $this->assertSame(VolumePricing::sku(), $row->sku);
        $this->assertSame($quantity, $row->max_devices);
        $this->assertSame('buyer@example.com', $row->email);
        $this->assertSame('active', $row->status);
        $this->assertNotEmpty($row->signed_blob);

        // Fallback: 50 seats × £19/seat = 95000 pence. Reached
        // because amount_total is absent on the synthetic session
        // — the helper computes from VolumePricing as the second
        // arm.
        $this->assertSame(50 * 1900, $row->amount_pence);
        $this->assertSame('gbp', $row->currency);

        Mail::assertSent(LicenceDelivery::class, function ($mail) use ($row) {
            return $mail->licence->id === $row->id;
        });
    }

    public function test_volume_mint_works_at_lower_tier_boundary(): void
    {
        Mail::fake();

        $session = $this->makeSession([
            'metadata' => (object) [
                'sku'         => VolumePricing::sku(),
                'quantity'    => '5', // MIN_SEATS, team tier
                'max_devices' => '5',
            ],
        ]);

        $this->callHandler($session);

        $row = Licence::where('stripe_checkout_session_id', $session->id)->first();
        $this->assertNotNull($row);
        $this->assertSame(5, $row->max_devices);

        // Fallback (no amount_total on the session): team tier
        // computes 5 × £29 = 14500 pence. Pinned so a future
        // VolumePricing boundary change (e.g. 25 → 20) is caught
        // by this test rather than silently shifting the
        // fallback figure.
        $this->assertSame(5 * 2900, $row->amount_pence);
        $this->assertSame('gbp', $row->currency);
    }

    public function test_volume_mint_persists_session_amount_total(): void
    {
        Mail::fake();

        // The buyer was charged 30000 pence (£300.00) at checkout
        // for a 10-seat order. VolumePricing today computes
        // 10 × £29 = 29000 pence. The £10 discrepancy stands in
        // for any drift between Checkout-Session creation and
        // webhook delivery (a rate change, a one-off discount, a
        // promo we ran for a customer, anything). The persisted
        // amount MUST be 30000 — what the card was charged — not
        // 29000 — what the helper would compute now.
        $quantity = 10;
        $session = $this->makeSession([
            'metadata' => (object) [
                'sku'         => VolumePricing::sku(),
                'quantity'    => (string) $quantity,
                'max_devices' => (string) $quantity,
                'unit_pence'  => '2900',  // matches VolumePricing today
            ],
            'amount_total' => 30000,       // what Stripe says was paid
            'currency'     => 'gbp',
        ]);

        $this->callHandler($session);

        $row = Licence::where('stripe_checkout_session_id', $session->id)->first();
        $this->assertNotNull($row);
        $this->assertSame(30000, $row->amount_pence, 'session amount_total must win over VolumePricing recomputation');
        $this->assertSame('gbp', $row->currency);
    }

    public function test_volume_mint_ignores_metadata_unit_pence_for_persistence(): void
    {
        Mail::fake();

        // Metadata is informational, not authoritative. A buyer
        // (or an attacker, or a buggy controller) cannot influence
        // the persisted amount by smuggling a different unit_pence
        // into the metadata. With amount_total present, only
        // amount_total is read.
        $session = $this->makeSession([
            'metadata' => (object) [
                'sku'         => VolumePricing::sku(),
                'quantity'    => '10',
                'max_devices' => '10',
                'unit_pence'  => '1', // 1 pence per seat — completely fake
            ],
            'amount_total' => 50000,
            'currency'     => 'gbp',
        ]);

        $this->callHandler($session);

        $row = Licence::where('stripe_checkout_session_id', $session->id)->first();
        $this->assertNotNull($row);
        // Stored amount comes from amount_total, NOT metadata.unit_pence.
        $this->assertSame(50000, $row->amount_pence);
        $this->assertNotSame(10, $row->amount_pence, 'metadata unit_pence × quantity must not be used for persistence');
    }

    public function test_volume_mint_refuses_out_of_range_quantity(): void
    {
        Mail::fake();

        $session = $this->makeSession([
            'metadata' => (object) [
                'sku'         => VolumePricing::sku(),
                'quantity'    => '500',  // tampered, way over MAX_SEATS
                'max_devices' => '500',
            ],
        ]);

        $this->expectException(\RuntimeException::class);
        $this->expectExceptionMessageMatches('/out-of-range metadata.quantity/');

        try {
            $this->callHandler($session);
        } finally {
            // Even on the throw path, no licence row should exist.
            $this->assertSame(
                0,
                Licence::where('stripe_checkout_session_id', $session->id)->count(),
                'no row should be persisted when quantity is rejected'
            );
        }
    }

    public function test_volume_mint_refuses_non_numeric_quantity(): void
    {
        Mail::fake();

        $session = $this->makeSession([
            'metadata' => (object) [
                'sku'      => VolumePricing::sku(),
                'quantity' => 'unlimited',
            ],
        ]);

        $this->expectException(\RuntimeException::class);
        $this->expectExceptionMessageMatches('/missing or non-numeric metadata.quantity/');

        $this->callHandler($session);
    }

    public function test_volume_mint_is_idempotent(): void
    {
        Mail::fake();

        $session = $this->makeSession([
            'metadata' => (object) [
                'sku'         => VolumePricing::sku(),
                'quantity'    => '10',
                'max_devices' => '10',
            ],
        ]);

        $this->callHandler($session);
        $this->callHandler($session); // replayed delivery

        $rows = Licence::where('stripe_checkout_session_id', $session->id)->get();
        $this->assertCount(1, $rows, 'replayed webhook must not double-mint');
    }

    public function test_unrecognised_sku_is_ignored(): void
    {
        Mail::fake();

        $session = $this->makeSession([
            'metadata' => (object) ['sku' => 'something-else'],
        ]);

        $this->callHandler($session);

        $this->assertSame(
            0,
            Licence::where('stripe_checkout_session_id', $session->id)->count(),
            'unrecognised SKU must not mint a licence'
        );
        Mail::assertNothingSent();
    }

    public function test_unpaid_session_is_ignored(): void
    {
        Mail::fake();

        $session = $this->makeSession([
            'payment_status' => 'unpaid',
            'metadata' => (object) ['sku' => 'utterbar-pro-lifetime'],
        ]);

        $this->callHandler($session);

        $this->assertSame(0, Licence::query()->count());
        Mail::assertNothingSent();
    }
}
