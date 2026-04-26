<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Models\Release;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;

/**
 * GET /admin/analytics — aggregate dashboard for download events.
 *
 * "Unique" is per-day-per-IP-per-UA, see migration. The figures shown
 * are aggregates over those de-duped rows, not server hits. Geographic
 * breakdown only includes traffic that came through Cloudflare (i.e.
 * had a CF-IPCountry header); direct origin hits are bucketed as
 * "Unknown".
 */
class AnalyticsController extends Controller
{
    public function index(Request $request)
    {
        // All-time unique downloaders (deduped across releases too — a
        // person who downloads v0.7.2 then v0.7.5 counts once).
        $uniqueDownloadersAllTime = (int) DB::table('download_events')
            ->select(DB::raw('COUNT(DISTINCT ip_hash) AS c'))
            ->value('c');

        // All-time download events (per release, not collapsed across).
        $totalEvents = (int) DB::table('download_events')->count();

        // Last 30 days, per day. Used by a small sparkline / list view.
        $since = now()->subDays(29)->startOfDay()->toDateString();
        $byDay = DB::table('download_events')
            ->select('event_date', DB::raw('COUNT(*) AS events'))
            ->where('event_date', '>=', $since)
            ->groupBy('event_date')
            ->orderBy('event_date')
            ->get();

        // Per-release table. Joined with releases for display order.
        $perRelease = DB::table('releases')
            ->leftJoin('download_events', 'releases.id', '=', 'download_events.release_id')
            ->select(
                'releases.id',
                'releases.version',
                'releases.channel',
                'releases.published_at',
                DB::raw('COUNT(download_events.id) AS events'),
                DB::raw('COUNT(DISTINCT download_events.ip_hash) AS unique_downloaders'),
            )
            ->groupBy('releases.id', 'releases.version', 'releases.channel', 'releases.published_at')
            ->orderByDesc('releases.published_at')
            ->get();

        // Country split. NULL country bucketed as "Unknown" (typically
        // direct hits not through Cloudflare).
        $byCountry = DB::table('download_events')
            ->select(
                DB::raw("COALESCE(country, '??') AS country"),
                DB::raw('COUNT(DISTINCT ip_hash) AS uniques'),
            )
            ->groupBy('country')
            ->orderByDesc('uniques')
            ->limit(15)
            ->get();

        return view('pages.admin.analytics', [
            'uniqueDownloadersAllTime' => $uniqueDownloadersAllTime,
            'totalEvents'              => $totalEvents,
            'byDay'                    => $byDay,
            'perRelease'               => $perRelease,
            'byCountry'                => $byCountry,
        ]);
    }
}
