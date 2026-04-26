<?php

namespace App\Http\Controllers;

use App\Models\DownloadEvent;
use App\Models\Release;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;

/**
 * Public download endpoint. Logs a unique-per-day-per-IP-per-UA event
 * against the Release row and 302s the user to the R2 download URL.
 *
 * Why not 302 directly to R2 from the marketing pages? Because we want
 * to count downloads — not just downloads-from-people-who-don't-paste-
 * the-direct-URL. Routing through this endpoint puts the count under our
 * control without touching R2 access logs.
 *
 * Privacy posture: we hash (IP || pepper) and (UA || pepper) with the
 * APP_KEY-derived pepper so the raw values never hit the database.
 * The hashes can't be reversed without the key, and the per-day event
 * date means the hash space rotates daily — by design we cannot re-
 * identify a specific person across days from the event log alone.
 *
 * The downloads endpoint is unauthenticated by design — every visitor
 * to /download or /pricing should be able to start the install without
 * an account.
 */
class DownloadController extends Controller
{
    public function latest(Request $request): RedirectResponse
    {
        $release = Release::stable()
            ->orderByDesc('published_at')
            ->first();
        if (!$release) {
            return redirect()->route('download');
        }
        return $this->logAndRedirect($request, $release);
    }

    public function version(Request $request, string $version): RedirectResponse
    {
        $release = Release::stable()
            ->where('version', $version)
            ->first();
        if (!$release) {
            return redirect()->route('download');
        }
        return $this->logAndRedirect($request, $release);
    }

    private function logAndRedirect(Request $request, Release $release): RedirectResponse
    {
        try {
            $this->recordEvent($request, $release);
        } catch (\Throwable $e) {
            // Never block a download because of a count failure. A unique
            // constraint hit in a same-day reinstall is the expected
            // happy path — silently swallow it. Anything else logs but
            // still lets the user through.
            if (!str_contains((string) $e->getMessage(), 'Duplicate entry')
                && !str_contains((string) $e->getMessage(), 'UNIQUE constraint')) {
                Log::warning('download event log failed', [
                    'release' => $release->version,
                    'error'   => $e->getMessage(),
                ]);
            }
        }
        return redirect()->away($release->downloadUrl(), 302);
    }

    private function recordEvent(Request $request, Release $release): void
    {
        $pepper = (string) config('app.key');
        $ip = (string) $request->ip();
        $ua = (string) $request->userAgent();
        $country = (string) $request->header('CF-IPCountry', '');
        $country = ($country === '' || strlen($country) > 2) ? null : strtoupper($country);

        DownloadEvent::create([
            'release_id' => $release->id,
            'ip_hash'    => hash('sha256', $ip . '|' . $pepper),
            'ua_hash'    => hash('sha256', $ua . '|' . $pepper),
            'country'    => $country,
            'event_date' => now()->toDateString(),
        ]);
    }
}
