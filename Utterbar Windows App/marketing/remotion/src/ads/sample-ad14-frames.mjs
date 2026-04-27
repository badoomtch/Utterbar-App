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

const SAMPLES = [
  { name: 'shot1-hook',         frame: 60 },
  { name: 'shot2-dictation',    frame: 190 },
  { name: 'shot3-modes',        frame: 270 },
  { name: 'shot4-vocabulary',   frame: 410 },
  { name: 'shot5-files',        frame: 540 },
  { name: 'shot6-history',      frame: 660 },
  { name: 'shot7-privacy',      frame: 780 },
  { name: 'shot8-close',        frame: 870 },
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
