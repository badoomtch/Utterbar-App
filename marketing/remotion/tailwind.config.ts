/**
 * Tailwind config for the Remotion bundle.
 *
 * Extends the shipping app's root `tailwind.config.ts` so every custom
 * colour token, radius, font family, keyframe, and animation we rely on
 * in real app components (`src/**`) is defined identically here. The only
 * delta is the `content` scan list — Tailwind builds its utility set
 * from the files it scans, so we point it at BOTH trees:
 *
 *   • `../../src/**` — so every real component's classNames emit their
 *     utilities (text-accent, bg-bg-1, h-[52px], animate-pulseDot, etc.)
 *   • `./src/**`     — so Remotion-local compositions' classNames emit
 *     their utilities too (the few comp-local bits that still use cn()).
 *
 * Keeping this config as a thin re-export prevents the app ↔ Remotion
 * palettes from drifting. If someone adds an accent or a new radius in
 * the root config, it lands here automatically.
 */
import base from '../../tailwind.config';
import type { Config } from 'tailwindcss';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// Resolve content paths to ABSOLUTE file-system paths. Tailwind's own
// content scanner resolves globs relative to the config file's location,
// but the bundler invoking Tailwind (Remotion's webpack + `@remotion/
// tailwind`) may pass the config in from a different cwd, at which point
// relative globs silently miss the repo's `src/` tree — and every
// `flex` / `rounded-full` / `bg-accent` in the real FloatingBar renders
// unstyled. Absolute paths dodge that class of bug entirely.
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..', '..');
const appSrc = path.join(repoRoot, 'src');
const remotionSrc = path.join(__dirname, 'src');

export default {
  ...base,
  content: [
    `${appSrc.replace(/\\/g, '/')}/**/*.{ts,tsx,html}`,
    `${remotionSrc.replace(/\\/g, '/')}/**/*.{ts,tsx}`,
  ],
} satisfies Config;
