/**
 * Render 8 representative stills from ad-14, one per shot, into
 * out/ads/_samples/. Used during composition development to eyeball each
 * shot without watching the full video. Not part of the shipping
 * pipeline; safe to delete after the ad is signed off.
 */
import { bundle } from '@remotion/bundler';
import { renderStill, selectComposition, openBrowser } from '@remotion/renderer';
import path from 'node:path';
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { webpackOverride } from '../../webpack-override.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..', '..');
const ENTRY = path.join(ROOT, 'src', 'ads', 'index.ts');
const OUT_DIR = path.join(ROOT, 'out', 'ads', '_samples');

// 45s timeline @ 30fps. Each frame is mid-shot in the readable hold.
//   1. Hook       0.0-3.5    →  60 ( 2.0s, "Speak." holding)
//   2. Dictation  3.5-11.5   → 240 ( 8.0s, full sentence + glow at peak)
//   3. Modes      11.5-18.5  → 480 (16.0s, both card bodies typed in, holding)
//   4. Vocabulary 18.5-24.5  → 690 (23.0s, after sentence visible)
//   5. Files      24.5-30.5  → 870 (29.0s, transcript modal up)
//   6. History    30.5-36.0  → 1020 (34.0s, detail panel readable)
//   7. Privacy    36.0-41.0  → 1170 (39.0s, all 3 models + monitor at 0)
//   8. Close      41.0-45.0  → 1320 (44.0s, full close card)
const SAMPLES = [
  { name: 'shot1-hook',         frame: 60 },
  { name: 'shot2-dictation',    frame: 240 },
  { name: 'shot3-modes',        frame: 480 },
  { name: 'shot4-vocabulary',   frame: 690 },
  { name: 'shot5-files',        frame: 870 },
  { name: 'shot6-history',      frame: 1020 },
  { name: 'shot7-privacy',      frame: 1170 },
  { name: 'shot8-close',        frame: 1320 },
];

await mkdir(OUT_DIR, { recursive: true });
console.log('Bundling…');
const serveUrl = await bundle({ entryPoint: ENTRY, webpackOverride });
const browser = await openBrowser('chrome', { chromiumOptions: { disableWebSecurity: true, headless: true } });
const composition = await selectComposition({ serveUrl, id: 'ad-14-local-dictation', inputProps: {} });

for (const s of SAMPLES) {
  const out = path.join(OUT_DIR, `${s.name}.png`);
  console.log(`▶ ${s.name} @ frame ${s.frame}`);
  await renderStill({ composition, serveUrl, output: out, frame: s.frame, imageFormat: 'png', puppeteerInstance: browser });
}
await browser.close({ silent: true });
console.log('done');
