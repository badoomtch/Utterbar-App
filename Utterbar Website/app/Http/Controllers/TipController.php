<?php

namespace App\Http\Controllers;

use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use Stripe\Checkout\Session as StripeSession;
use Stripe\Stripe;

/**
 * Tip jar.
 *
 * Standalone "Customer chooses price" Stripe Price configured in the
 * dashboard (Optional Gratuity category, GBP, min £1 / max £100,
 * suggested £5). When the user clicks one of the preset pill buttons
 * (£3 / £5 / £10) we POST here, redirect them to a one-shot Stripe
 * Checkout Session that lands them directly on the card form with the
 * amount preselected. "Custom" sends amount = null and Stripe's own
 * amount-input UI takes over.
 *
 * Tips are NOT licences. The webhook handler (StripeWebhookController)
 * looks at the SKU metadata to decide whether to mint a Pro key — tip
 * sessions carry sku=tip and skip licence minting. Stripe still emails
 * a receipt automatically; we don't need our own thanks email.
 */
class TipController extends Controller
{
    public function checkout(Request $request): RedirectResponse
    {
        if (!config('services.stripe.enabled')) {
            return redirect()->route('home')
                ->with('error', 'Tipping is not available right now — try again soon.');
        }
        if (!config('services.stripe.tip_price_id')) {
            // Stripe Price not configured; the buttons should already
            // hide themselves but defensively redirect home.
            return redirect()->route('home');
        }

        $secret = config('services.stripe.secret');
        if (!$secret) {
            Log::error('Stripe secret key missing for tip checkout');
            return redirect()->route('home')
                ->with('error', 'Tipping is misconfigured. Sorry about that.');
        }

        // Validate the preset amount (in pence). Null = use the dashboard
        // Price's amount picker. Any value is clamped to £1-£100 so a
        // tampered form can't push a £10,000 charge through.
        $validated = $request->validate([
            'amount' => 'nullable|integer|min:100|max:10000',
        ]);
        $amountPence = $validated['amount'] ?? null;

        Stripe::setApiKey($secret);

        $priceId = config('services.stripe.tip_price_id');

        // When an amount is preselected, we use price_data with the same
        // *product* (resolved by the API on Stripe's side from the Price
        // ID), so the customer sees the locked-in amount on Checkout. When
        // null, we send the Price ID directly and Stripe renders its
        // built-in amount picker. Both paths converge on the same Stripe
        // product and thus the same tax category / receipt branding.
        if ($amountPence !== null) {
            // Resolve the product ID from the configured Price so price_data
            // can reference it. Cheap one-time lookup; cached in case we
            // hit Stripe rate limits (highly unlikely on a tip flow but
            // belt-and-braces).
            $productId = $this->resolveProductIdFromPrice($priceId);
            if (!$productId) {
                // Fall back to the dashboard Price (user picks amount on
                // Stripe). Worse UX but doesn't break the workflow.
                $lineItem = ['price' => $priceId, 'quantity' => 1];
            } else {
                $lineItem = [
                    'price_data' => [
                        'currency'    => 'gbp',
                        'unit_amount' => $amountPence,
                        'product'     => $productId,
                    ],
                    'quantity' => 1,
                ];
            }
        } else {
            // User chose "Custom" — send them to Stripe's amount picker.
            $lineItem = ['price' => $priceId, 'quantity' => 1];
        }

        $params = [
            'mode'        => 'payment',
            'line_items'  => [$lineItem],
            'success_url' => route('tip.thanks') . '?session_id={CHECKOUT_SESSION_ID}',
            'cancel_url'  => route('home'),
            // Donations don't need a billing address — keep the form
            // small. Stripe will still capture what their tax rules
            // require (typically nothing for an "Optional Gratuity"
            // category product).
            'billing_address_collection' => 'auto',
            'metadata' => [
                'sku' => 'utterbar-tip',
            ],
            'payment_intent_data' => [
                'metadata' => [
                    'sku' => 'utterbar-tip',
                ],
            ],
        ];

        try {
            $session = StripeSession::create($params);
        } catch (\Stripe\Exception\ApiErrorException $e) {
            Log::error('Stripe tip session creation failed', ['error' => $e->getMessage()]);
            return redirect()->route('home')
                ->with('error', 'Could not open the tip checkout. Try again in a moment.');
        }

        return redirect()->away($session->url, 303);
    }

    /**
     * GET variant for the desktop app's "Donate" surface. The app opens
     * `https://utterbar.com/tip/start?amount=500` in the user's default
     * browser; this method just delegates to `checkout()` after promoting
     * the query parameter into the request body. Same throttle, same
     * validation, same Stripe session.
     */
    public function start(Request $request): RedirectResponse
    {
        $request->merge(['amount' => $request->query('amount')]);
        return $this->checkout($request);
    }

    public function thanks()
    {
        return view('pages.tip.thanks');
    }

    /**
     * Lookup the Stripe Product ID associated with a given Price. Cached
     * for an hour so repeat tip clicks don't re-hit the API. Returns null
     * on any failure — the caller falls back to the dashboard Price.
     */
    private function resolveProductIdFromPrice(string $priceId): ?string
    {
        return cache()->remember(
            "stripe.price.product.{$priceId}",
            3600,
            function () use ($priceId) {
                try {
                    $price = \Stripe\Price::retrieve($priceId);
                    return is_string($price->product) ? $price->product : null;
                } catch (\Throwable $e) {
                    Log::warning('Could not resolve Stripe product from price', [
                        'price' => $priceId,
                        'error' => $e->getMessage(),
                    ]);
                    return null;
                }
            },
        );
    }
}
