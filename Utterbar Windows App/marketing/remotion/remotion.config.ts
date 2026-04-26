// Remotion config — scoped to the marketing workspace. Nothing here touches
// the shipping Tauri/Vite build; everything the renderer needs lives under
// `marketing/remotion/src/` plus the shared app source tree at `../../src/`.
import { Config } from '@remotion/cli/config';
// Single source of truth for the webpack alias table — `render.mjs` imports
// the same module, so the CLI-path (studio / `remotion still`) and the
// programmatic-path (batch renderer) can't drift.
// @ts-expect-error — sibling .mjs, no types needed; Remotion config isn't
// part of the typecheck.
import { webpackOverride } from './webpack-override.mjs';

// H.264 MP4 for the website. Still-poster exports are PNG via `remotion still`.
Config.setVideoImageFormat('jpeg');
Config.setCodec('h264');
// A CRF of 18 keeps the pill's thin strokes and wordmark gradient crisp under
// YouTube/Vercel re-encodes. Anything north of 22 softens the 2 px border.
Config.setCrf(18);
// 1920x1080 is the universal web master. Individual compositions can still
// override via `<Composition width={…} height={…} />` (press stills are 2560x1600).
Config.setPixelFormat('yuv420p');
// (Config.setLogLevel was removed in Remotion 4.x; pass `--log=info` on the
// CLI / `logLevel` option to renderMedia if quieter output is needed.)

// --------------------------------------------------------------------------
// Webpack override — point `@/*` at the real app `src/` and stub Tauri APIs.
// The actual alias table lives in `./webpack-override.mjs` so render.mjs
// can import the same function. See that file's header for the rationale.
// --------------------------------------------------------------------------
Config.overrideWebpackConfig(webpackOverride);
