/**
 * Batch-render all 12 social ads from the AdsRoot to MP4 in marketing/
 * remotion/out/ads/. Mirrors the existing render.mjs (which renders the
 * landscape pack from Root.tsx) but points at src/ads/index.ts and
 * outputs vertical 1080×1920 H.264.
 *
 * Run from the marketing/remotion/ directory:
 *   node src/ads/render-ads.mjs            # all 12
 *   node src/ads/render-ads.mjs ad-01-stop-typing  # one
 */
import { bundle } from '@remotion/bundler';
import { renderMedia, selectComposition, openBrowser } from '@remotion/renderer';
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
];

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
