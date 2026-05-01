<?php

namespace App\Http\Controllers;

use App\Models\Licence;
use App\Services\Pricing\VolumePricing;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;

class AdminController extends Controller
{
    /**
     * GET /admin — operator dashboard. Read-only overview of the licence
     * system's state; destructive actions live behind artisan commands
     * (`licence:revoke`, `licence:unrevoke`) to avoid footguns in the browser.
     */
    public function index(Request $request)
    {
        $volumeSku = VolumePricing::sku();

        $counts = [
            'licences_total'    => Licence::count(),
            'licences_active'   => Licence::where('status', 'active')->count(),
            'licences_refunded' => Licence::where('status', 'refunded')->count(),
            'licences_revoked'  => Licence::where('status', 'revoked')->count(),
            'activations_total' => DB::table('activations')->count(),
            'webhook_total'     => DB::table('stripe_webhook_events')->count(),
            'webhook_errors'    => DB::table('stripe_webhook_events')->whereNotNull('error')->count(),
            'contact_total'     => DB::table('contact_messages')->count(),

            // B2B counts. Cheap aggregates over the licences table —
            // one COUNT and one SUM, both indexed on `sku` is fine for
            // current cardinality. If volume rows ever 100x in
            // volume, revisit.
            'volume_licences_total'  => Licence::where('sku', $volumeSku)->count(),
            'volume_licences_active' => Licence::where('sku', $volumeSku)->where('status', 'active')->count(),
            // Total seats sold across volume rows. Refunded / revoked
            // rows are NOT excluded — the operator wants to see the
            // gross seat count that came in through self-serve, not
            // the currently-active count, because seat reconciliation
            // happens at the row level (the "Active volume" stat
            // above is the live-Pro figure).
            'volume_seats_sold'      => (int) Licence::where('sku', $volumeSku)->sum('max_devices'),

            // Rows where we DO have a per-row amount captured at mint
            // time vs rows where we don't. Stripe-webhook mints
            // (single-user and volume) populate amount_pence from
            // 2026-05-02 onward; admin manual mints (no Stripe
            // payment behind them) and pre-2026-05-02 rows stay
            // null and surface in the "Untracked rows" tile so the
            // operator knows the gross figure isn't all-in.
            'licences_with_amount'   => Licence::whereNotNull('amount_pence')->count(),
            'licences_without_amount' => Licence::whereNull('amount_pence')->count(),
        ];

        // Revenue: SUM the per-row amount_pence column. This honours
        // both single-user (£39.99) and volume (£29 / £19 per seat ×
        // quantity) without the controller having to reverse the
        // pricing model. Rows without amount_pence are excluded
        // entirely — see the `licences_without_amount` count above
        // for the operator-visible flag. Net subtracts refunded rows
        // (refunded keeps the existing semantic; revoked is NOT
        // subtracted because revoke can be a no-money-back admin
        // action — chargebacks set status='revoked' too, but the
        // money-back signal there is the dispute, not the row).
        $grossPence = (int) Licence::whereNotNull('amount_pence')->sum('amount_pence');
        $netPence   = (int) Licence::whereNotNull('amount_pence')
            ->where('status', '!=', 'refunded')
            ->sum('amount_pence');

        // Display currency: pick the most common non-null currency on
        // the licences table, falling back to the configured pro
        // currency when the table is empty (typically only happens in
        // dev / test). The dashboard renders a single GBP / USD label
        // alongside the formatted amount; if a future deployment ever
        // takes payments in two currencies, the figures stop being
        // directly summable and we'd need a per-currency breakdown,
        // but that's a problem for the day a non-GBP price actually
        // ships.
        $currency = strtoupper(
            (string) (Licence::whereNotNull('currency')
                ->select('currency')
                ->groupBy('currency')
                ->orderByRaw('COUNT(*) desc')
                ->limit(1)
                ->value('currency')
                ?? config('services.stripe.pro_currency', 'gbp'))
        );

        $recentLicences = Licence::withCount('activations')
            ->orderByDesc('id')
            ->limit(20)
            ->get();

        $recentWebhooks = DB::table('stripe_webhook_events')
            ->orderByDesc('id')
            ->limit(15)
            ->get();

        $recentSignIns = DB::table('sign_in_tokens')
            ->orderByDesc('id')
            ->limit(10)
            ->get();

        return view('pages.admin.index', [
            'counts'         => $counts,
            'currency'       => $currency,
            'grossPence'     => $grossPence,
            'netPence'       => $netPence,
            'recentLicences' => $recentLicences,
            'recentWebhooks' => $recentWebhooks,
            'recentSignIns'  => $recentSignIns,
        ]);
    }
}
