/**
 * Batch-render the social ads from the AdsRoot to MP4 in marketing/
 * remotion/out/ads/. Mirrors the existing render.mjs (which renders the
 * landscape pack from Root.tsx) but points at src/ads/index.ts and
 * outputs vertical 1080×1920 H.264.
 *
 * Run from the marketing/remotion/ directory:
 *   node src/ads/render-ads.mjs                     # all ads, MP4 only
 *   node src/ads/render-ads.mjs ad-01-stop-typing   # one ad
 *   node src/ads/render-ads.mjs ad-13-the-product   # the hero ad
 *
 * Posters: the hero ad gets a poster PNG dropped next to the MP4
 * (out/ads/<id>-poster.png) at the frame defined by POSTER_FRAMES below.
 * The 12 social ads don't get posters — they're scrolled past on TikTok
 * /Reels, not embedded as <video poster=...>.
 */
import { bundle } from '@remotion/bundler';
import {
  renderMedia,
  renderStill,
  selectComposition,
  openBrowser,
} from '@remotion/renderer';
import path from 'node:path';
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { webpackOverride } from '../../webpack-override.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..', '..');
const ENTRY = path.join(ROOT, 'src', 'ads', 'index.ts');
const OUT_DIR = path.join(ROOT, 'out', 'ads');

const ALL_IDS = [
  'ad-01-stop-typing',
  'ad-02-voice-is-faster',
  'ad-03-local-only',
  'ad-04-talks-to-any-ai',
  'ad-05-thirty-languages',
  'ad-06-hold-hotkey',
  'ad-07-teach-vocabulary',
  'ad-08-windows-first',
  'ad-09-manifesto',
  'ad-10-price-once',
  'ad-11-no-account',
  'ad-12-founders-note',
  'ad-13-the-product',
  'ad-14-local-dictation',
];

// Frames where a still poster reads cleanly. For ad-13 we want the
// "shaped output" moment — beat 3, around f(B2_END=8.5) + 36 + 14 = 305:
// the pill has settled back to idle after success, the shaped transcript
// panel is fully laid out with its full sentence visible. Reads as the
// product-promise frame: speak, get clean text. Frame 320 gives a
// little extra hold for the still to feel composed.
const POSTER_FRAMES = {
  'ad-13-the-product': 320,
  // Shot 2 of ad-14 (v2 pacing). Shot 2 spans global frames 105-345.
  // Internal beat: sentence finishes paste at internal f(3.7) = global
  // frame 216. Glow holds at peak from internal 3.5-7.0 = global 210-315.
  // Frame 240 (=8.0s) sits squarely in the held-readable centre — full
  // sentence in the field, accent glow at peak, pill back to idle.
  'ad-14-local-dictation': 240,
};

const target = process.argv[2];
const ids = target ? [target] : ALL_IDS;

if (target && !ALL_IDS.includes(target)) {
  console.error(`Unknown id: ${target}`);
  console.error(`Known ids:\n  ${ALL_IDS.join('\n  ')}`);
  process.exit(1);
}

await mkdir(OUT_DIR, { recursive: true });

console.log(`Bundling ads entrypoint at ${ENTRY}…`);
const t0 = Date.now();
const serveUrl = await bundle({ entryPoint: ENTRY, webpackOverride });
console.log(`  bundle done in ${((Date.now() - t0) / 1000).toFixed(1)}s`);

// Reuse one Chromium across all renders — saves ~2s per output.
const browser = await openBrowser('chrome', {
  chromiumOptions: { disableWebSecurity: true, headless: true },
});

const failures = [];
for (const id of ids) {
  const outPath = path.join(OUT_DIR, `${id}.mp4`);
  console.log(`▶ ${id}`);
  const t = Date.now();
  try {
    const composition = await selectComposition({ serveUrl, id, inputProps: {} });
    await renderMedia({
      composition,
      serveUrl,
      codec: 'h264',
      outputLocation: outPath,
      // Same encode flags as render.mjs — typewriter-heavy scenes need the
      // headroom CRF 17 + slow gives to avoid frame stutter.
      imageFormat: 'png',
      pixelFormat: 'yuv420p',
      colorSpace: 'bt709',
      crf: 17,
      x264Preset: 'slow',
      puppeteerInstance: browser,
      concurrency: null,
    });
    console.log(`    done in ${((Date.now() - t) / 1000).toFixed(1)}s → ${outPath}`);

    // Poster — only for ids registered in POSTER_FRAMES. Reuse the
    // composition handle from the video render. The website uses this
    // as <video poster> so the player surface looks like the product
    // before play.
    if (POSTER_FRAMES[id] !== undefined) {
      const posterPath = path.join(OUT_DIR, `${id}-poster.png`);
      const tp = Date.now();
      await renderStill({
        composition,
        serveUrl,
        output: posterPath,
        frame: POSTER_FRAMES[id],
        imageFormat: 'png',
        puppeteerInstance: browser,
      });
      console.log(`    poster done in ${((Date.now() - tp) / 1000).toFixed(1)}s → ${posterPath}`);
    }
  } catch (err) {
    console.error(`    FAILED: ${err?.message ?? err}`);
    failures.push({ id, error: err });
  }
}

await browser.close({ silent: true });

const total = ids.length;
const ok = total - failures.length;
console.log(`\n${ok}/${total} rendered successfully → ${OUT_DIR}`);
if (failures.length) {
  for (const f of failures) console.log(`  × ${f.id}: ${f.error?.message ?? f.error}`);
  process.exit(1);
}
