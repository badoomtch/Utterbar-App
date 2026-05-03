#!/usr/bin/env node
/**
 * Batch renderer for the Utterbar marketing pack.
 *
 * Reads `asset-manifest.json`, bundles `src/index.ts`, then iterates every
 * video and still, rendering:
 *
 *   videos/<id>.mp4              — H.264 / CRF 18 / yuv420p (see remotion.config.ts)
 *   posters/<id>-poster.png      — frame picked by POSTER_FRAMES, default 0
 *   stills/<id>.png              — 2560 × 1600 single-frame PNG
 *
 * Skips any asset whose output already exists unless `--force` is passed.
 * Accepts `--only=<id>` to render one composition by id.
 * Accepts `--kind=videos|posters|stills` — `videos` is MP4s only (no
 *   poster), `posters` is just the PNG posters for video compositions,
 *   `stills` is the press stills. Default is all three.
 *
 * Outputs go under `out/` at the Remotion root. They're copied from there
 * into the Laravel repo's `public/{videos,posters,stills}/` tree by hand.
 */
import { bundle } from '@remotion/bundler';
import {
  renderMedia,
  renderStill,
  selectComposition,
  openBrowser,
} from '@remotion/renderer';
import { readFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { webpackOverride } from './webpack-override.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const here = (p) => resolve(__dirname, p);

// ---------- args ----------------------------------------------------------
const args = new Map(
  process.argv.slice(2).map((a) => {
    const [k, v] = a.replace(/^--/, '').split('=');
    return [k, v ?? true];
  }),
);
const FORCE = !!args.get('force');
const ONLY = args.get('only'); // single composition id
const KIND = args.get('kind'); // 'videos' | 'posters' | 'stills'
const PARITY = !!args.get('parity'); // parity-sheet batch mode

// Per-composition poster-frame overrides. Default is 0. If a composition's
// frame 0 is behind an entrance tween (empty card, fade-in, etc) the poster
// will read blank — pick a later frame where the viewer sees the real
// resting state of the UI.
//
// feature-modes at frame 0 is just the "Raw transcript" label with an empty
// pair of quotes; the typewriter hasn't started. Panels enter at f(2.8)=84,
// f(4.2)=126, f(5.6)=168; each panel's body typewriter ends ~f(panel)+27.
// Frame 200 lands just after the third panel's body finishes typing — all
// three modes are fully populated and the poster reads as the real feature.
//
// feature-local-privacy at frame 0 lands before the "Your PC" panel has
// faded in (panelOpacity tween 0 → 0.8s) and the cyan local loop hasn't
// started drawing yet (loopProgress 1.5s → 4.5s). The result was a
// mostly empty dark poster (~22 KB vs ~70 KB for every other composition).
// Frame 180 = 6.0s, after the loop has fully drawn (4.5s) and before
// the caption tween starts (6.95s): both panels visible, local loop
// fully traced, no outbound activity yet — a clean steady-state poster
// that actually reads as the privacy diagram.
const POSTER_FRAMES = {
  'feature-modes': 200,
  'feature-local-privacy': 180,
};

// ---------- manifest ------------------------------------------------------
const manifest = JSON.parse(readFileSync(here('asset-manifest.json'), 'utf8'));

// Directory creation follows the manifest — whatever `master` / `poster` /
// `file` say relative to the remotion root, we ensure the parent exists.
// Keeping this driven by the manifest means moving outputs elsewhere (e.g.
// the website's public/ tree) is a manifest edit + nothing else.
const ensureDir = (p) => {
  const d = dirname(resolve(__dirname, p));
  if (!existsSync(d)) mkdirSync(d, { recursive: true });
};
for (const v of manifest.videos) {
  ensureDir(v.master);
  ensureDir(v.poster);
}
for (const s of manifest.stills) ensureDir(s.file);

const wanted = (items) => items.filter((it) => !ONLY || it.id === ONLY);
// `videos`, `posters`, and `stills` all iterate the same manifest lists —
// the kind flag controls which outputs get written inside each loop.
const videos = KIND === 'stills' ? [] : wanted(manifest.videos);
const stills = KIND === 'videos' || KIND === 'posters' ? [] : wanted(manifest.stills);

// Per-loop output toggles so `--kind=posters` can reuse the video iteration
// without also re-encoding the MP4s (~10 s each).
const RENDER_MP4 = KIND !== 'posters' && KIND !== 'stills';
const RENDER_POSTER = KIND !== 'videos' && KIND !== 'stills';

if (ONLY && !videos.length && !stills.length) {
  console.error(`No asset found for --only=${ONLY}`);
  process.exit(1);
}

// ---------- parity mode ---------------------------------------------------
// `--parity` renders the `state-parity-sheet` composition six times, once
// per dictation state. The output PNGs live under `out/parity/<state>.png`
// and get diffed by hand against app-side screenshots of the same six
// states. Gate for the batch-3 refactor sign-off — not a shipping asset,
// so skipped by default.
const PARITY_STATES = [
  { id: 'idle', label: 'IDLE', elapsedMs: 0 },
  { id: 'warming', label: 'WARMING', elapsedMs: 0 },
  { id: 'listening', label: 'LISTENING · 00:03', elapsedMs: 3000 },
  { id: 'processing', label: 'PROCESSING', elapsedMs: 0 },
  { id: 'success', label: 'SUCCESS', elapsedMs: 0 },
  { id: 'error', label: 'ERROR', elapsedMs: 0 },
];

// ---------- bundle --------------------------------------------------------
console.log('Bundling Remotion project…');
const t0 = Date.now();
const serveUrl = await bundle({
  entryPoint: here('src/index.ts'),
  // `@remotion/bundler` does NOT consult `remotion.config.ts` — that file
  // is read by the Remotion CLI only. Pass the same webpack override
  // directly so `@/...` and the Tauri stubs resolve in batch renders.
  webpackOverride,
});
console.log(`  bundle done in ${((Date.now() - t0) / 1000).toFixed(1)}s`);

// One browser process is reused across every render in the batch — spinning
// up a fresh Chromium per asset adds ~2 seconds per output for no gain.
const browser = await openBrowser('chrome', {
  // On Windows CI/sandboxes this can hang without these flags. They're
  // harmless on a dev box, so always on.
  chromiumOptions: {
    disableWebSecurity: true,
    headless: true,
  },
});

// ---------- render loop ---------------------------------------------------
const failures = [];

if (PARITY) {
  const parityDir = resolve(__dirname, 'out', 'parity');
  if (!existsSync(parityDir)) mkdirSync(parityDir, { recursive: true });
  console.log(`Parity mode · 6 states → ${parityDir}`);
  for (const s of PARITY_STATES) {
    const out = resolve(parityDir, `${s.id}.png`);
    console.log(`▶ state-parity-sheet/${s.id.padEnd(11)} [1920×1080 still]`);
    const t = Date.now();
    try {
      // Each state is a fresh selectComposition + renderStill pass. The
      // real pill reads from zustand module singletons, so one bundle +
      // one browser can't stack states in parallel — serial is the only
      // correct way to run this.
      const composition = await selectComposition({
        serveUrl,
        id: 'state-parity-sheet',
        inputProps: { state: s.id, elapsedMs: s.elapsedMs, label: s.label },
      });
      await renderStill({
        composition,
        serveUrl,
        output: out,
        frame: 0,
        imageFormat: 'png',
        puppeteerInstance: browser,
        inputProps: { state: s.id, elapsedMs: s.elapsedMs, label: s.label },
      });
      console.log(`  done in ${((Date.now() - t) / 1000).toFixed(1)}s`);
    } catch (err) {
      console.error(`  FAILED: ${err?.message ?? err}`);
      failures.push({ id: `parity/${s.id}`, error: err });
    }
  }
  await browser.close({ silent: true });
  const total = PARITY_STATES.length;
  const ok = total - failures.length;
  console.log(`\n${ok}/${total} parity stills rendered.`);
  if (failures.length) {
    for (const f of failures) console.log(`  × ${f.id}: ${f.error?.message ?? f.error}`);
    process.exit(1);
  }
  process.exit(0);
}

for (const v of videos) {
  const mp4 = resolve(__dirname, v.master);
  const poster = resolve(__dirname, v.poster);
  // Existence check is per-kind so `--kind=posters` doesn't look for
  // the MP4 and vice-versa. Without this, a missing MP4 would force a
  // poster re-render that the user didn't ask for.
  const wantMp4 = RENDER_MP4;
  const wantPoster = RENDER_POSTER;
  const haveWhatWeWant =
    (!wantMp4 || existsSync(mp4)) && (!wantPoster || existsSync(poster));
  if (!FORCE && haveWhatWeWant) {
    console.log(`· ${v.id.padEnd(32)} [skip — already rendered]`);
    continue;
  }
  const kindLabel =
    wantMp4 && wantPoster
      ? `${v.duration_seconds}s @ ${v.width}×${v.height}`
      : wantMp4
        ? `${v.duration_seconds}s @ ${v.width}×${v.height} · mp4 only`
        : `poster @ frame ${POSTER_FRAMES[v.id] ?? 0}`;
  console.log(`▶ ${v.id.padEnd(32)} [${kindLabel}]`);
  const t = Date.now();
  try {
    const composition = await selectComposition({
      serveUrl,
      id: v.id,
      inputProps: {},
    });
    if (wantMp4) {
      await renderMedia({
        composition,
        serveUrl,
        codec: 'h264',
        outputLocation: mp4,
        // Image format: PNG, not JPEG. JPEG intermediates are emitted as
        // full-range ("yuvj420p") by Remotion's encoder, which some browsers
        // decode with a shifted colour range — near-black navy crushes to
        // pure black, producing the "black background" artefact we saw on
        // the first render pass. PNG keeps us in proper `yuv420p` and costs
        // ~30 s extra across the full batch.
        imageFormat: 'png',
        pixelFormat: 'yuv420p',
        // Tag the output as Rec.709 / limited range so browser video
        // decoders can't misinterpret it. Without this the encoded stream
        // can be tagged as "unspecified", which Chrome on Windows and
        // Safari on iOS sometimes guess wrong — crushing --bg-0 to black.
        colorSpace: 'bt709',
        // x264 quality: CRF 17 + slow preset gives the encoder room to
        // handle the typewriter-heavy scenes (feature-modes, feature-
        // vocabulary, feature-history-retranscribe) where per-frame text
        // deltas starved the previous CRF 18 / medium-preset output of
        // bits and read as stutter.
        crf: 17,
        x264Preset: 'slow',
        puppeteerInstance: browser,
        concurrency: null, // let Remotion pick — scales with CPU
      });
    }
    if (wantPoster) {
      await renderStill({
        composition,
        serveUrl,
        output: poster,
        frame: POSTER_FRAMES[v.id] ?? 0,
        imageFormat: 'png',
        puppeteerInstance: browser,
      });
    }
    console.log(`  done in ${((Date.now() - t) / 1000).toFixed(1)}s`);
  } catch (err) {
    console.error(`  FAILED: ${err?.message ?? err}`);
    failures.push({ id: v.id, error: err });
  }
}

for (const s of stills) {
  const out = resolve(__dirname, s.file);
  if (!FORCE && existsSync(out)) {
    console.log(`· ${s.id.padEnd(32)} [skip — already rendered]`);
    continue;
  }
  console.log(`▶ ${s.id.padEnd(32)} [${s.width}×${s.height} still]`);
  const t = Date.now();
  try {
    const composition = await selectComposition({
      serveUrl,
      id: s.id,
      inputProps: {},
    });
    await renderStill({
      composition,
      serveUrl,
      output: out,
      frame: 0,
      imageFormat: 'png',
      puppeteerInstance: browser,
    });
    console.log(`  done in ${((Date.now() - t) / 1000).toFixed(1)}s`);
  } catch (err) {
    console.error(`  FAILED: ${err?.message ?? err}`);
    failures.push({ id: s.id, error: err });
  }
}

// Newer @remotion/renderer requires an options object (destructures `silent`).
await browser.close({ silent: true });

// ---------- summary -------------------------------------------------------
const total = videos.length + stills.length;
const ok = total - failures.length;
console.log(`\n${ok}/${total} rendered successfully.`);
if (failures.length) {
  for (const f of failures) {
    console.log(`  × ${f.id}: ${f.error?.message ?? f.error}`);
  }
  process.exit(1);
}
