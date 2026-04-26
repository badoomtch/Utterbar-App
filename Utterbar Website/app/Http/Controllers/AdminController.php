<?php

namespace App\Http\Controllers;

use App\Models\Licence;
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
        $counts = [
            'licences_total'    => Licence::count(),
            'licences_active'   => Licence::where('status', 'active')->count(),
            'licences_refunded' => Licence::where('status', 'refunded')->count(),
            'licences_revoked'  => Licence::where('status', 'revoked')->count(),
            'activations_total' => DB::table('activations')->count(),
            'webhook_total'     => DB::table('stripe_webhook_events')->count(),
            'webhook_errors'    => DB::table('stripe_webhook_events')->whereNotNull('error')->count(),
            'contact_total'     => DB::table('contact_messages')->count(),
        ];

        // Revenue: count active + refunded separately for quick eyeball.
        // Amounts are pulled from config since we don't store them per-licence yet.
        $amountPence = (int) config('services.stripe.pro_amount', 4900);
        $currency    = strtoupper((string) config('services.stripe.pro_currency', 'gbp'));
        $grossPence  = $counts['licences_total'] * $amountPence;
        $netPence    = ($counts['licences_total'] - $counts['licences_refunded']) * $amountPence;

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
            'amountPence'    => $amountPence,
            'currency'       => $currency,
            'grossPence'     => $grossPence,
            'netPence'       => $netPence,
            'recentLicences' => $recentLicences,
            'recentWebhooks' => $recentWebhooks,
            'recentSignIns'  => $recentSignIns,
        ]);
    }
}
