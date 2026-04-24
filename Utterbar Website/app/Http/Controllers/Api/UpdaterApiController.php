<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Models\Release;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

/**
 * GET /api/updater/check?current=X.Y.Z&channel=stable
 *
 * Desktop app contract (see `Utterbar Windows App/src-tauri/src/commands/updater.rs`
 * module docstring). Returns the newest published release in the chosen
 * channel when it is strictly greater than the `current` version, else an
 * `available: false` response.
 *
 *   Response 200 (update available):
 *   {
 *     "available":      true,
 *     "version":        "0.6.1",
 *     "download_url":   "https://dl.utterbar.com/releases/0.6.1/Utterbar-Setup-0.6.1.exe",
 *     "sha256":         "47bd8249...",
 *     "notes_markdown": "- First change\n- Second change",
 *     "size_bytes":     10962662
 *   }
 *
 *   Response 200 (no update):
 *   { "available": false, "current": "0.6.0" }
 *
 * Version comparison is a straight three-segment numeric compare.
 * Non-numeric / non-semver input (`current=foo`) returns no-update rather
 * than 4xx — the client treats every non-200 as no-update anyway, and we
 * prefer not to leak "valid format" oracles on a fully-public endpoint.
 *
 * Unauthenticated, rate-limit applied at the route level.
 */
class UpdaterApiController extends Controller
{
    public function check(Request $request): JsonResponse
    {
        $current = (string) $request->query('current', '');
        $channel = (string) $request->query('channel', 'stable');
        if (!\in_array($channel, ['stable', 'beta'], true)) {
            $channel = 'stable';
        }

        $currentParts = self::parseSemver($current);

        // Updater eligibility filter. Publish-time validation now forbids
        // null sha256 and non-X.Y.Z versions, but pre-existing rows
        // from before that tightening may violate either constraint, so
        // we also filter at read time:
        //
        //   - channel must match
        //   - published_at must be set
        //   - sha256 must be a valid 64-char hex string (client install
        //     path requires it; a release without sha256 cannot be
        //     installed and must not be advertised)
        //   - version must parse as strict X.Y.Z
        //   - r2_key must be present (without it we can't build a
        //     download URL the client can actually fetch)
        //
        // We pick the newest eligible row, not the newest row overall —
        // so a malformed row published most recently won't shadow an
        // older usable one. SHA/version validation is done in PHP
        // (`pickEligibleRelease`) rather than a DB `REGEXP` because
        // SQLite — the test driver — doesn't ship a regexp function by
        // default, and portable behaviour matters more than one fewer
        // round-trip on a request that runs at most a few times per
        // user per day.
        $latest = self::pickEligibleRelease($channel);

        if (!$latest) {
            return response()->json([
                'available' => false,
                'current'   => $current,
            ]);
        }

        $latestParts = self::parseSemver((string) $latest->version);

        // If either side is unparseable, no update. This is deliberately
        // conservative — we'd rather tell a client "no update" than push
        // one on a malformed request.
        if ($currentParts === null || $latestParts === null) {
            return response()->json([
                'available' => false,
                'current'   => $current,
            ]);
        }

        if (self::compare($latestParts, $currentParts) <= 0) {
            return response()->json([
                'available' => false,
                'current'   => $current,
            ]);
        }

        // Build the canonical download URL. Prefer the versioned filename
        // (`Utterbar-Setup-<v>.exe`) because that's what publish-release.sh
        // uploads as the authoritative key; fall back to the legacy
        // unversioned filename that older admin rows may point at.
        //
        // Source of truth for the CDN base is `filesystems.disks.r2.url`
        // (matches `Release::downloadUrl()` so every surface quotes the
        // same origin).
        $r2Key = (string) ($latest->r2_key ?: "releases/{$latest->version}/Utterbar-Setup-{$latest->version}.exe");
        $cdnBase = (string) (config('filesystems.disks.r2.url') ?: env('CLOUDFLARE_R2_PUBLIC_URL', 'https://dl.utterbar.com'));
        $downloadUrl = rtrim($cdnBase, '/') . '/' . ltrim($r2Key, '/');

        return response()->json([
            'available'      => true,
            'version'        => (string) $latest->version,
            'download_url'   => $downloadUrl,
            'sha256'         => (string) ($latest->sha256 ?? ''),
            'notes_markdown' => (string) ($latest->notes_markdown ?? ''),
            // Release model stores the file size as `file_size_bytes`; the
            // client wire field is `size_bytes` to match Rust's struct.
            'size_bytes'     => $latest->file_size_bytes !== null ? (int) $latest->file_size_bytes : null,
        ]);
    }

    /**
     * Parse a three-segment semver (`X.Y.Z`). Returns [major, minor, patch]
     * or null if the input doesn't match. We deliberately accept no pre-
     * release or build metadata — Utterbar has never shipped one and this
     * endpoint is not the place to start. The admin publish path now
     * rejects prerelease suffixes for the same reason (see
     * ReleaseController + AdminReleaseApiController version regex), so
     * publishing rules and this parser stay in lockstep.
     *
     * @return array{0:int,1:int,2:int}|null
     */
    private static function parseSemver(string $v): ?array
    {
        $v = trim($v);
        if ($v === '' || !preg_match('/^(\d+)\.(\d+)\.(\d+)$/', $v, $m)) {
            return null;
        }
        return [(int) $m[1], (int) $m[2], (int) $m[3]];
    }

    /**
     * Validate that a value is a non-empty 64-char hex string — the
     * shape the desktop updater's SHA-256 verification expects.
     */
    private static function isValidSha256(string $s): bool
    {
        return preg_match('/^[A-Fa-f0-9]{64}$/', $s) === 1;
    }

    /**
     * Fallback release picker. Walks published rows in the requested
     * channel, newest-first, and returns the first one that passes all
     * updater-eligibility checks (valid X.Y.Z version, valid SHA-256,
     * non-empty r2_key). Used when the DB-level regex filter isn't
     * portable (SQLite in tests, some MariaDB builds).
     */
    private static function pickEligibleRelease(string $channel): ?Release
    {
        $candidates = Release::query()
            ->where('channel', $channel)
            ->whereNotNull('published_at')
            ->orderByDesc('published_at')
            ->get();

        foreach ($candidates as $row) {
            if (self::parseSemver((string) $row->version) === null) {
                continue;
            }
            if (!self::isValidSha256((string) ($row->sha256 ?? ''))) {
                continue;
            }
            if (empty($row->r2_key)) {
                continue;
            }
            return $row;
        }
        return null;
    }

    /**
     * Lexicographic three-segment compare. Returns -1, 0, or 1.
     * @param array{0:int,1:int,2:int} $a
     * @param array{0:int,1:int,2:int} $b
     */
    private static function compare(array $a, array $b): int
    {
        for ($i = 0; $i < 3; $i++) {
            if ($a[$i] < $b[$i]) return -1;
            if ($a[$i] > $b[$i]) return 1;
        }
        return 0;
    }
}
