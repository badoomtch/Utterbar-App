/**
 * Shared webpack override for Remotion.
 *
 * `remotion.config.ts` is consulted by the Remotion CLI (`remotion studio`,
 * `remotion still`, `remotion render`) but NOT by the programmatic bundler
 * API (`@remotion/bundler`'s `bundle()`). Our render.mjs uses the
 * programmatic API, so the same alias table has to be passed in by hand.
 *
 * Keeping it here means the CLI + the batch renderer share one source of
 * truth — the config file imports this, and render.mjs imports this, so
 * there's no way for the two paths to drift.
 *
 *   • `@` path alias resolves to the shipping app's `src/` at the repo
 *     root (../../src). Real components import via `@/...`; those lookups
 *     land in the real tree, not a fork.
 *   • `@tauri-apps/api/{core,window,event}` resolve to inert stubs under
 *     `./src/stubs/`. Every invoke/listen/getCurrentWindow call a real
 *     component makes during mount hits a noop that returns quietly.
 *   • The CSS loader chain is rewritten in-place so `@tailwind` / `@import`
 *     directives inside `../../../src/styles/globals.css` expand at build
 *     time. We wire postcss-loader ourselves rather than going through
 *     `@remotion/tailwind` because that package's 4.0.451 release pairs
 *     poorly with Remotion bundler 4.0.232 (the bundler rebuilds
 *     `module.rules` after the override returns, so an appended rule
 *     silently loses to the stock css-loader-only rule). Doing it here
 *     keeps the plugin order under our control:
 *       postcss-import → tailwindcss → autoprefixer
 *     which is the order Tailwind's own docs require (postcss-import MUST
 *     run before tailwindcss so `@import './tokens.css'` resolves before
 *     the utility layer emits).
 */
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

// Derive __dirname / require without `import.meta`.
//
// `@remotion/bundler` loads this file as a real ESM module (so import.meta
// works). The Remotion CLI (used by `remotion studio` / `remotion render`)
// eval-loads it inside its own config-loader, where import.meta is
// undefined. That's the "ERR_INVALID_ARG_TYPE: path must be string ...
// received undefined" Jay hit on 2026-04-25 trying to start the studio.
//
// Try import.meta.url first (works for the bundler path); fall back to
// __filename if eslint sees that, otherwise resolve via the require path
// of this very module. The triple fallback covers every tool that
// might load this file.
function resolveDirname() {
  try {
    if (typeof import.meta !== 'undefined' && import.meta.url) {
      return path.dirname(fileURLToPath(import.meta.url));
    }
  } catch {
    // import.meta itself can throw under some loaders; fall through.
  }
  // The CLI eval-loader hands us a relative-to-cwd path on argv[1] when
  // it loads remotion.config.ts; the config file is always at the
  // Remotion project root, which is also where this override lives.
  return process.cwd();
}

const __dirname = resolveDirname();
const require_ = createRequire(path.join(__dirname, 'package.json'));

const repoRoot = path.resolve(__dirname, '..', '..');
const appSrc = path.join(repoRoot, 'src');
const stubs = path.join(__dirname, 'src', 'stubs');
const tailwindConfigPath = path.join(__dirname, 'tailwind.config.ts');

// Pre-resolve every loader + postcss plugin at module load so the override
// function itself is synchronous and the same object can be handed back to
// both the Remotion CLI and the programmatic bundler without await.
const styleLoader = require_.resolve('style-loader');
const cssLoader = require_.resolve('@remotion/bundler/css-loader');
const postcssLoader = require_.resolve('postcss-loader');
const tailwindcss = require_.resolve('tailwindcss');
const autoprefixer = require_.resolve('autoprefixer');

export const webpackOverride = (current) => {
  // Drop every existing CSS rule so the one we append below is the single
  // matcher. Remotion's stock rule only runs style-loader +
  // @remotion/bundler/css-loader (no postcss), so `@tailwind` and `@import`
  // directives inside globals.css would pass through raw.
  const existingRules = (current.module?.rules ?? []).filter((rule) => {
    return rule && rule !== '...' && !rule.test?.toString?.().includes('.css');
  });

  const cssRule = {
    test: /\.css$/i,
    // Loaders run right-to-left. Source text hits postcss-loader first
    // (which inlines @imports, expands @tailwind, then autoprefixes),
    // then css-loader emits a JS module that style-loader injects at
    // runtime.
    use: [
      styleLoader,
      cssLoader,
      {
        loader: postcssLoader,
        options: {
          // `config: false` disables file-based postcss.config.js lookup —
          // we want the plugin chain inline and deterministic, not merged
          // with whatever file postcss-loader happens to find walking up
          // from the CSS source's directory.
          postcssOptions: {
            config: false,
            // NOTE: no postcss-import here. `@remotion/bundler/css-loader`
            // already handles `@import` via webpack — each imported CSS
            // file is re-emitted as a webpack request and re-runs the
            // full loader chain (including this postcss-loader). Adding
            // postcss-import on top would try to inline node_modules CSS
            // (fontsource woff2s) through postcss-import's own resolver,
            // which has no knowledge of webpack aliases and blows up on
            // the relative woff2 urls inside `@fontsource-variable/*`.
            plugins: [
              [tailwindcss, { config: tailwindConfigPath }],
              [autoprefixer, {}],
            ],
          },
        },
      },
    ],
  };

  return {
    ...current,
    resolve: {
      ...current.resolve,
      alias: {
        ...(current.resolve?.alias ?? {}),
        // Static replacement for the shipping StatePill — identical chip,
        // no AnimatePresence/motion. Without this the phase chip "flies in"
        // because Remotion captures frames during popLayout exit/enter.
        // Listed BEFORE the catch-all `@` alias so this specific path wins.
        '@/components/bar/StatePill$': path.join(
          __dirname,
          'src',
          'bar',
          'StatePillStatic.tsx',
        ),
        '@': appSrc,
        '@tauri-apps/api/core$': path.join(stubs, 'tauri-core.ts'),
        '@tauri-apps/api/window$': path.join(stubs, 'tauri-window.ts'),
        '@tauri-apps/api/event$': path.join(stubs, 'tauri-event.ts'),
      },
    },
    module: {
      ...current.module,
      rules: [...existingRules, cssRule],
    },
  };
};
