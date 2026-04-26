<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Models\Tip;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;

/**
 * GET /admin/tips — voluntary tip jar dashboard.
 *
 * Reads the `tips` table (populated by StripeWebhookController on
 * `checkout.session.completed` with sku=utterbar-tip). Aggregates total
 * raised, count, average, last-30-days bar chart, recent tips list.
 */
class TipController extends Controller
{
    public function index(Request $request)
    {
        // Top-line totals. Sum in pence, format on render.
        $totalsByCurrency = DB::table('tips')
            ->select('currency', DB::raw('SUM(amount_pence) AS total'), DB::raw('COUNT(*) AS count'))
            ->groupBy('currency')
            ->orderByDesc('total')
            ->get();

        $totalCount = (int) DB::table('tips')->count();

        // Average tip — only meaningful when at least one currency is
        // dominant. Guarded against zero-row state.
        $primary = $totalsByCurrency->first();
        $averagePence = ($primary && $primary->count > 0)
            ? (int) round($primary->total / $primary->count)
            : 0;
        $primaryCurrency = $primary?->currency ?? 'gbp';

        // Last 30 days, per day. Used for a small bar chart.
        $since = now()->subDays(29)->startOfDay();
        $byDay = DB::table('tips')
            ->select(DB::raw('DATE(paid_at) AS day'), DB::raw('COUNT(*) AS count'), DB::raw('SUM(amount_pence) AS total'))
            ->where('paid_at', '>=', $since)
            ->groupBy('day')
            ->orderBy('day')
            ->get();

        // Recent tips. Most-recent 25.
        $recent = Tip::orderByDesc('paid_at')->limit(25)->get();

        return view('pages.admin.tips', [
            'totalsByCurrency' => $totalsByCurrency,
            'totalCount'       => $totalCount,
            'averagePence'     => $averagePence,
            'primaryCurrency'  => $primaryCurrency,
            'byDay'            => $byDay,
            'recent'           => $recent,
        ]);
    }
}
