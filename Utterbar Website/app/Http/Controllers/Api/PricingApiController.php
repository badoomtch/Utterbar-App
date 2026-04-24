<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use Illuminate\Http\JsonResponse;

/**
 * GET /api/pricing — current Pro-tier pricing for the in-app upgrade modal.
 *
 * The desktop app's `src-tauri/src/commands/pricing.rs` fetches this on
 * demand when the UpgradeModal opens. Its `ServerPricing` struct expects:
 *
 *   {
 *     "amount_usd":  <f64>,          // actually amount-in-`currency`,
 *                                    //   kept named _usd for legacy reasons
 *                                    //   in the client wire format
 *     "currency":    "GBP" | "USD" | "EUR",
 *     "period":      "one_time" | "monthly" | "yearly",
 *     "display":     "£39.99"         // optional; client reconstructs if
 *                                    //   absent. Strongly prefer to send
 *                                    //   it — picks up currency-symbol and
 *                                    //   decimal handling from the server
 *                                    //   without the client guessing.
 *   }
 *
 * Values come from Stripe-aligned env vars so there's ONE source of truth
 * for the price: `STRIPE_PRO_AMOUNT` (pence/cents) + `STRIPE_PRO_CURRENCY`
 * + a hard-coded period ("one_time", since Utterbar Pro is a lifetime
 * licence). The in-app fallback was historically $29; this endpoint
 * removes the drift so Pro always reads as the live Stripe price.
 *
 * The endpoint is public and unauthenticated — an unreleased build
 * could fetch this before the user has a licence. It exposes nothing
 * sensitive.
 */
class PricingApiController extends Controller
{
    public function show(): JsonResponse
    {
        // `STRIPE_PRO_AMOUNT` is in the smallest currency unit (pence for
        // GBP, cents for USD). Normalise to a whole-unit float with two
        // decimals for the client + display string.
        $minor = (int) config('services.stripe.pro_amount', env('STRIPE_PRO_AMOUNT', 3999));
        $currency = strtoupper((string) config('services.stripe.pro_currency', env('STRIPE_PRO_CURRENCY', 'GBP')));
        $amount = $minor / 100.0;

        $symbol = match ($currency) {
            'GBP' => '£',
            'USD' => '$',
            'EUR' => '€',
            default => '',
        };

        // "£39.99" if there are sub-unit pence, "£40" if zero. Matches the
        // Rust client's format_price() fallback so the UI reads the same
        // whether the display string came from the server or was
        // reconstructed locally.
        $display = $symbol . (fmod($amount, 1.0) === 0.0
            ? number_format($amount, 0, '.', '')
            : number_format($amount, 2, '.', ''));

        return response()->json([
            'amount_usd' => $amount,
            'currency'   => $currency,
            'period'     => 'one_time',
            'display'    => $display,
        ]);
    }
}
