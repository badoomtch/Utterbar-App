/**
 * Design tokens — direct port of the Obsidian theme out of
 * `src/styles/tokens.css` in the shipping app. This file is the SINGLE
 * source of truth for every colour the Remotion pack renders with;
 * surfaces are elevated by tone (bg0 → bg1 → bg2), never by borders.
 *
 * Cyan budget rule: at most three distinct cyan moments per frame (the
 * wordmark "bar" half, the pill mark's waveform, and the active-rail /
 * CTA — everything else stays neutral).
 */

// Named tokens (prefer these over raw hex in components).
export const tokens = {
  // Surfaces — Obsidian navy ladder. These match the marketing site's
  // resources/css/app.css custom properties exactly, so a composition
  // rendered on bg-0 vanishes seamlessly into any page section.
  //   bg0  page canvas / video section background
  //   bg1  card surfaces (feature cards, route cards)
  //   bg2  elevated surfaces (inputs, chips, modal cards)
  // Historical note: the first port used a true-black ladder (#000).
  // The real site is #0A0E17 — subtly blue-tinted navy. Any video
  // rendered with bg-0 = #000 prints a visible cutout on the real
  // site, which is what Jay flagged on 23 Apr. These are the source
  // of truth.
  bg0: '#0A0E17',
  bg1: '#10151F',
  bg2: '#171C29',

  // Borders — hairlines only, always rgba so they lift on any surface.
  border: 'rgba(255, 255, 255, 0.08)',
  borderStrong: 'rgba(255, 255, 255, 0.14)',

  // Text ladder.
  fgHi: '#FFFFFF',
  fg: '#E8E8EB',
  muted: '#8A8A94',
  fgDim: '#5A5A64',

  // Brand cyan family. `--accent-2 → --accent` drives the wordmark
  // gradient in the canonical direction (left=light, right=brand).
  accent: '#22D3EE',
  accent2: '#67E8F9',
  accent3: '#0891B2',
  gradientCyanStart: '#00e5ff',
  gradientCyanEnd: '#00a3ff',

  // Pill tokens — the Obsidian floating-bar body.
  pillTop: '#0B0B10',
  pillBot: '#000000',
  pillTopLit: '#14141B',
  pillBotLit: '#07070B',
  pillFg: 'rgba(255, 255, 255, 0.92)',
  pillFgMuted: 'rgba(255, 255, 255, 0.78)',
  pillStroke: 'rgba(255, 255, 255, 0.72)',
  pillStrokeSoft: 'rgba(255, 255, 255, 0.14)',
  pillDim: 'rgba(255, 255, 255, 0.14)',
  pillSurface: 'rgba(255, 255, 255, 0.06)',
  pillSurfaceHi: 'rgba(255, 255, 255, 0.12)',

  // Mark-specific ring + hole-punch (white on dark themes).
  markRing: '#FFFFFF',
  markHole: '#FFFFFF',

  // Semantic (kept identical to the app).
  amber: '#F59E0B',
  rose: '#F43F5E',
  moss: '#34D399',

  // Radii.
  radiusCard: 18,
  radiusBlock: 24,
  radiusInput: 10,
  radiusKbd: 6,
  radiusPill: 999,
} as const;

// One-stop shorthand for the canonical wordmark gradient string.
export const gradientWordmark = `linear-gradient(90deg, ${tokens.gradientCyanStart} 0%, ${tokens.gradientCyanEnd} 100%)`;

// Fonts — self-hosted via @fontsource-variable in the Remotion pack.
// Instrument Sans drives everything except literal code/data (versions,
// paths, timestamps). Geist Mono handles the mono family.
export const fonts = {
  sans: "'Instrument Sans Variable', 'Instrument Sans', ui-sans-serif, system-ui, -apple-system, 'Segoe UI Variable', 'Segoe UI', Roboto, Arial, sans-serif",
  mono: "'Geist Mono Variable', 'Geist Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
} as const;

// Motion tokens — ported from `src/design/motion.ts`. The EASE curve is
// the canonical "settle" — fast out, calm in, no overshoot.
export const EASE = [0.32, 0.72, 0, 1] as const;
// Duration buckets (ms). Remotion works in frames so callers convert via
// `Math.round(durations.glide / (1000 / fps))` as needed.
export const durations = {
  pop: 180,
  snap: 200,
  glide: 240,
  settle: 320,
} as const;
