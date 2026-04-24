<?php

namespace App\Http\Controllers;

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
}
