<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Illuminate\Support\Facades\Cache;

/**
 * Per-page Open Graph image generator.
 *
 * Renders a 1200×630 PNG with the page title + tagline drawn over the
 * brand background. Used by every page's <meta property="og:image">
 * via `og.image` route helper. Cached on disk by hash of the inputs so
 * Facebook / Twitter / Slack scrapers hit a static file after the
 * first render — no GD work on the hot path.
 *
 * Inputs come from the query string so the generator is the same code
 * regardless of caller; pages just hit /og/image?title=…&kicker=… and
 * pass whatever's appropriate. Keeps the URL stable + cache-friendly.
 *
 * Falls back to the static og-default.png on any GD failure so a
 * misbehaving font or ext mismatch never breaks social previews.
 */
class OgImageController extends Controller
{
    private const W = 1200;
    private const H = 630;

    public function show(Request $request): Response
    {
        $title  = trim((string) $request->query('title', 'Utterbar'));
        $kicker = trim((string) $request->query('kicker', 'Local dictation for Windows.'));

        // Length-clamp so a runaway query doesn't render a 60-line title.
        $title  = mb_substr($title, 0, 80);
        $kicker = mb_substr($kicker, 0, 100);

        $cacheKey = 'og.' . sha1($title . '|' . $kicker);
        $cachePath = storage_path('app/og-cache/' . $cacheKey . '.png');

        if (!is_dir(dirname($cachePath))) {
            @mkdir(dirname($cachePath), 0775, true);
        }

        if (!file_exists($cachePath)) {
            $bin = $this->render($title, $kicker);
            if ($bin === null) {
                return $this->fallback();
            }
            @file_put_contents($cachePath, $bin);

            // LRU eviction. A scripted attacker can iterate query
            // strings to force unique cache entries; without a cap the
            // droplet disk fills. After every fresh write, sweep the
            // cache directory and delete the oldest files when count
            // exceeds the cap. 5,000 cards is plenty for any
            // realistic site (every page + every dynamic blog post,
            // many times over) at ~250MB total disk.
            $this->evictIfOverCap(dirname($cachePath), 5000);
        }

        $bin = @file_get_contents($cachePath);
        if ($bin === false) {
            return $this->fallback();
        }

        return response($bin, 200, [
            'Content-Type'  => 'image/png',
            // Long cache — content is keyed by hash of the inputs so a
            // copy change yields a new URL anyway.
            'Cache-Control' => 'public, max-age=31536000, immutable',
        ]);
    }

    private function fallback(): Response
    {
        $path = public_path('og/og-default.png');
        $bin = @file_get_contents($path);
        if ($bin === false) {
            return response('', 404);
        }
        return response($bin, 200, [
            'Content-Type'  => 'image/png',
            'Cache-Control' => 'public, max-age=86400',
        ]);
    }

    /**
     * Bounded-size LRU eviction. When the cache directory has more
     * than $maxFiles entries, delete the oldest by mtime until the
     * count is back under cap. Cheap enough to run on every fresh
     * write — the dir is local SSD, the cap is in the thousands, and
     * real production traffic never hits the cap. The sweep is just
     * insurance against the unbounded-write attack vector flagged
     * in the v0.8.x security audit.
     *
     * Failures are silently swallowed: the eviction never blocks the
     * caller. If the disk is already full and unlink fails, the
     * fresh write also fails — same outcome, fewer log spammers.
     */
    private function evictIfOverCap(string $dir, int $maxFiles): void
    {
        $files = @glob($dir . '/*.png');
        if ($files === false || count($files) <= $maxFiles) {
            return;
        }
        // Build [path => mtime] then sort ascending. PHP's sort is
        // stable so ties don't matter here.
        $byTime = [];
        foreach ($files as $f) {
            $m = @filemtime($f);
            if ($m !== false) {
                $byTime[$f] = $m;
            }
        }
        asort($byTime);
        $excess = count($byTime) - $maxFiles;
        $i = 0;
        foreach (array_keys($byTime) as $oldest) {
            if ($i++ >= $excess) {
                break;
            }
            @unlink($oldest);
        }
    }

    /**
     * Compose the OG image. Returns PNG bytes or null on failure.
     *
     * Visual: dark background (bg-0), large white title, smaller cyan
     * kicker line above it, "utterbar.com" footer in mono. Two-column
     * safe-zone padding (60px sides) so platforms that crop edges
     * don't eat the text.
     */
    private function render(string $title, string $kicker): ?string
    {
        if (!\extension_loaded('gd')) {
            return null;
        }

        $img = imagecreatetruecolor(self::W, self::H);
        if ($img === false) {
            return null;
        }

        // Brand colours, hex-decoded once.
        $bg     = imagecolorallocate($img, 0x06, 0x12, 0x1A); // --bg-0
        $bgTop  = imagecolorallocate($img, 0x0B, 0x1A, 0x24); // subtle band
        $accent = imagecolorallocate($img, 0x22, 0xD3, 0xEE); // --accent
        $fg     = imagecolorallocate($img, 0xF1, 0xF5, 0xF9); // --fg-hi
        $muted  = imagecolorallocate($img, 0x94, 0xA3, 0xB8); // --muted

        // Background — flat fill. A radial gradient via GD is heavyweight
        // and not worth the cost for a social card; flat reads cleaner.
        imagefilledrectangle($img, 0, 0, self::W, self::H, $bg);

        // Top band — a cyan accent strip 6px tall. Cheap brand cue.
        imagefilledrectangle($img, 0, 0, self::W, 6, $accent);

        // Subtle banding overlay at the top fifth so the title area
        // doesn't read as a solid void.
        imagefilledrectangle($img, 0, 6, self::W, 130, $bgTop);

        $fontPath = $this->resolveFontPath();

        if ($fontPath === null) {
            // No TTF available — fall back to GD's built-in bitmap font.
            // Less pretty but still readable. (We expect Cloudways to
            // have DejaVu installed; this is a belt-and-braces.)
            imagestring($img, 5, 60, 60, $kicker, $accent);
            imagestring($img, 5, 60, 100, $title, $fg);
            imagestring($img, 4, 60, self::H - 60, 'utterbar.com', $muted);
        } else {
            // Kicker line — small cyan text above the title.
            $this->drawText($img, $kicker, $fontPath, 28, 60, 200, $accent);

            // Title — large white, wrapped to fit width.
            $this->drawWrappedText(
                $img,
                $title,
                $fontPath,
                72,
                60,
                280,
                self::W - 120,
                $fg,
                90,
            );

            // Footer brand line.
            $this->drawText($img, 'utterbar.com', $fontPath, 24, 60, self::H - 50, $muted);
        }

        ob_start();
        $ok = imagepng($img, null, 6);
        $bin = ob_get_clean();
        imagedestroy($img);

        return ($ok && $bin !== false) ? (string) $bin : null;
    }

    /**
     * Best-effort lookup of a usable TTF font. Cloudways images ship
     * DejaVu by default; we also accept the bundled Geist/Instrument
     * Sans fonts if someone's added them under public/fonts/. Returns
     * null if nothing usable is found.
     */
    private function resolveFontPath(): ?string
    {
        $candidates = [
            // Bundled (preferred — matches the site)
            public_path('fonts/InstrumentSans-Bold.ttf'),
            public_path('fonts/InstrumentSans-SemiBold.ttf'),
            public_path('fonts/InstrumentSans-Regular.ttf'),
            // System DejaVu (Cloudways default)
            '/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf',
            '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf',
            // Fallbacks
            '/usr/share/fonts/TTF/DejaVuSans.ttf',
        ];
        foreach ($candidates as $p) {
            if (is_file($p) && is_readable($p)) {
                return $p;
            }
        }
        return null;
    }

    private function drawText($img, string $text, string $font, int $size, int $x, int $y, int $colour): void
    {
        @imagettftext($img, $size, 0, $x, $y, $colour, $font, $text);
    }

    /**
     * Word-wrap word-by-word to fit a max width in pixels, drawing each
     * line at $lineHeight increments. We don't truncate — if the title
     * is genuinely too long for two lines, we let it spill rather than
     * silently dropping content.
     */
    private function drawWrappedText(
        $img,
        string $text,
        string $font,
        int $size,
        int $x,
        int $y,
        int $maxWidth,
        int $colour,
        int $lineHeight,
    ): void {
        $words = preg_split('/\s+/', $text) ?: [];
        $line = '';
        $cursorY = $y;
        foreach ($words as $word) {
            $candidate = $line === '' ? $word : ($line . ' ' . $word);
            $box = @imagettfbbox($size, 0, $font, $candidate);
            $width = $box ? abs($box[2] - $box[0]) : 0;
            if ($width > $maxWidth && $line !== '') {
                @imagettftext($img, $size, 0, $x, $cursorY, $colour, $font, $line);
                $cursorY += $lineHeight;
                $line = $word;
            } else {
                $line = $candidate;
            }
        }
        if ($line !== '') {
            @imagettftext($img, $size, 0, $x, $cursorY, $colour, $font, $line);
        }
    }
}
