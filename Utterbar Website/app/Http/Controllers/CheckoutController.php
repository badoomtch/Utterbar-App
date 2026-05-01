<?php

namespace App\Http\Controllers;

use App\Services\Pricing\VolumePricing;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use Stripe\Checkout\Session as StripeSession;
use Stripe\Stripe;

class CheckoutController extends Controller
{
    /**
     * POST /api/checkout/create-session
     *
     * Creates a Stripe Checkout Session for Utterbar Pro and returns the URL
     * for the browser to redirect to. Also sets `session_id` on the returned
     * payload so the client can pass it back if needed.
     */
    public function createSession(Request $request): JsonResponse
    {
        if (!config('services.stripe.enabled')) {
            return response()->json(['error' => 'checkout_disabled', 'message' => 'Checkout is not available yet.'], 503);
        }

        $secret = config('services.stripe.secret');
        if (!$secret) {
            Log::error('Stripe secret key missing');
            return response()->json(['error' => 'misconfigured', 'message' => 'Checkout is misconfigured.'], 500);
        }

        $validated = $request->validate([
            'email' => 'nullable|email:rfc|max:255',
        ]);

        Stripe::setApiKey($secret);

        $priceId  = config('services.stripe.pro_price_id');
        $amount   = (int) config('services.stripe.pro_amount');
        $currency = (string) config('services.stripe.pro_currency');

        $lineItem = $priceId
            ? ['price' => $priceId, 'quantity' => 1]
            : [
                'price_data' => [
                    'currency' => $currency,
                    'unit_amount' => $amount,
                    'product_data' => [
                        'name' => 'Utterbar Pro',
                        'description' => 'Lifetime licence. Three devices. No subscription.',
                    ],
                ],
                'quantity' => 1,
            ];

        $params = [
            'mode' => 'payment',
            'line_items' => [$lineItem],
            'success_url' => route('thanks') . '?session_id={CHECKOUT_SESSION_ID}',
            'cancel_url'  => route('pricing'),
            'billing_address_collection' => 'required',
            'metadata' => [
                'sku' => config('services.licence.sku_string', 'utterbar-pro-lifetime'),
            ],
            'payment_intent_data' => [
                'metadata' => [
                    'sku' => config('services.licence.sku_string', 'utterbar-pro-lifetime'),
                ],
            ],
        ];

        if ($email = $validated['email'] ?? null) {
            $params['customer_email'] = $email;
        }

        if (config('services.stripe.automatic_tax')) {
            $params['automatic_tax'] = ['enabled' => true];
        }

        try {
            $session = StripeSession::create($params);
        } catch (\Stripe\Exception\ApiErrorException $e) {
            Log::error('Stripe checkout session creation failed', ['error' => $e->getMessage()]);
            return response()->json(['error' => 'stripe_error', 'message' => 'Could not open checkout. Try again?'], 502);
        }

        return response()->json([
            'id'  => $session->id,
            'url' => $session->url,
        ]);
    }

    /**
     * POST /api/checkout/create-volume-session
     *
     * Volume self-serve checkout. Buyer picks a seat count in
     * [VolumePricing::MIN_SEATS, VolumePricing::MAX_SEATS]; the server
     * computes the per-seat amount at its current tier and creates a
     * Stripe Checkout Session with `unit_amount × quantity` line-item
     * pricing. The webhook (StripeWebhookController) recognises the
     * volume SKU via metadata and mints a single licence row with
     * `max_devices = quantity`.
     *
     * Server controls every monetary value. The request carries only
     * `email` (optional) and `quantity`. Anything else is ignored.
     * Quantity is validated by Laravel's `integer` rule, which accepts
     * a real int or a numeric string that parses as an int (e.g. "10")
     * and rejects floats, non-numeric strings, decimal strings, arrays,
     * and out-of-range values. Whatever passes is cast to a real int
     * before any pricing math runs, and re-checked against the helper's
     * range as defence in depth.
     */
    public function createVolumeSession(Request $request): JsonResponse
    {
        if (!config('services.stripe.enabled')) {
            return response()->json(['error' => 'checkout_disabled', 'message' => 'Checkout is not available yet.'], 503);
        }

        $secret = config('services.stripe.secret');
        if (!$secret) {
            Log::error('Stripe secret key missing');
            return response()->json(['error' => 'misconfigured', 'message' => 'Checkout is misconfigured.'], 500);
        }

        // Validation. Laravel's `integer` rule accepts a real int or
        // a numeric string that parses cleanly as an int (e.g. "10"),
        // and rejects floats, non-numeric strings, decimal strings,
        // booleans, and arrays. `min`/`max` lock the bounds to the
        // helper's documented range so the boundary cannot drift
        // between this method and the helper. Whatever passes here is
        // cast to a real int below before any pricing math runs.
        // The Laravel validator returns a clean 422 with field-keyed
        // messages, which the volume UI surfaces inline.
        $validated = $request->validate([
            'email'    => 'nullable|email:rfc|max:255',
            'quantity' => 'required|integer|min:' . VolumePricing::MIN_SEATS . '|max:' . VolumePricing::MAX_SEATS,
        ]);

        $quantity = (int) $validated['quantity'];

        // Defence in depth: the Laravel validator should have already
        // rejected anything outside the range, but the helper carries
        // its own predicate so a future refactor that swaps validators
        // can't silently widen the band.
        if (!VolumePricing::isValidQuantity($quantity)) {
            return response()->json([
                'error'   => 'invalid_quantity',
                'message' => 'Seat count must be between ' . VolumePricing::MIN_SEATS . ' and ' . VolumePricing::MAX_SEATS . '.',
            ], 422);
        }

        Stripe::setApiKey($secret);

        $unitPence = VolumePricing::unitPence($quantity);
        $currency  = VolumePricing::currency();
        $sku       = VolumePricing::sku();

        // Stripe line-item description must read clearly on the buyer's
        // receipt — "Utterbar Pro · Volume licence (N devices)" — so an
        // accountant looking at the line item six months later can tell
        // it apart from a single-user purchase. The product name stays
        // on-brand; the description carries the device-allowance fact.
        $lineItem = [
            'price_data' => [
                'currency'    => $currency,
                'unit_amount' => $unitPence,
                'product_data' => [
                    'name'        => 'Utterbar Pro — Volume licence',
                    'description' => sprintf(
                        'One licence covering %d devices. Lifetime, no subscription.',
                        $quantity
                    ),
                ],
            ],
            'quantity' => $quantity,
        ];

        // Metadata travels on both the Checkout Session and the
        // Payment Intent so the webhook handler can resolve from
        // either side. `quantity` and `max_devices` are duplicated
        // intentionally — the value is identical (one device per
        // purchased seat), but naming both makes the contract
        // explicit and survives a future refactor that decouples the
        // two concepts.
        $metadata = [
            'sku'         => $sku,
            'quantity'    => (string) $quantity,
            'max_devices' => (string) $quantity,
            'unit_pence'  => (string) $unitPence,
            'currency'    => $currency,
        ];

        $params = [
            'mode' => 'payment',
            'line_items' => [$lineItem],
            'success_url' => route('thanks') . '?session_id={CHECKOUT_SESSION_ID}',
            'cancel_url'  => route('pricing'),
            'billing_address_collection' => 'required',
            'metadata' => $metadata,
            'payment_intent_data' => [
                'metadata' => $metadata,
            ],
        ];

        if ($email = $validated['email'] ?? null) {
            $params['customer_email'] = $email;
        }

        if (config('services.stripe.automatic_tax')) {
            $params['automatic_tax'] = ['enabled' => true];
        }

        try {
            $session = StripeSession::create($params);
        } catch (\Stripe\Exception\ApiErrorException $e) {
            Log::error('Stripe volume checkout session creation failed', [
                'error'    => $e->getMessage(),
                'quantity' => $quantity,
            ]);
            return response()->json(['error' => 'stripe_error', 'message' => 'Could not open checkout. Try again?'], 502);
        }

        return response()->json([
            'id'  => $session->id,
            'url' => $session->url,
        ]);
    }
}
