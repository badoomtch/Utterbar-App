import type { Transition } from 'framer-motion';

/**
 * Motion tokens — DESIGN_SYSTEM.md §11 is explicit: no bouncy springs in the
 * app chrome. Every transition is a one-curve ease between 180–360ms.
 *
 * The canonical curve is [0.32, 0.72, 0, 1] — Material's "standard" ease,
 * already used by the floating bar's layout tween. It's the same curve the
 * marketing site uses for nav underlines, view transitions, and the hero
 * reveal, which keeps the two products feeling like one product.
 *
 * The names (`pop`, `glide`, `settle`, `snap`) are retained for callsite
 * compatibility — they previously resolved to spring configs — but the
 * underlying primitive is now a plain duration+ease pair:
 *
 *   pop    — 180ms. Quickest. State-pill label swap, toast dismiss.
 *   snap   — 200ms. Tight interaction response (button scales, chips).
 *   glide  — 240ms. Row/view transitions. The default.
 *   settle — 320ms. Slower, more deliberate — modal mount, step change.
 *
 * Framer Motion still respects reduced-motion when MotionConfig.reducedMotion
 * is set to 'always' (see routes/AppBootstrap.tsx), collapsing all four to 0ms.
 */
const EASE = [0.32, 0.72, 0, 1] as const;

export const springs = {
  pop:    { duration: 0.18, ease: EASE } satisfies Transition,
  snap:   { duration: 0.20, ease: EASE } satisfies Transition,
  glide:  { duration: 0.24, ease: EASE } satisfies Transition,
  settle: { duration: 0.32, ease: EASE } satisfies Transition,
} as const;

/**
 * Legacy export kept so existing code that imports individual motion helpers
 * keeps working. New code should pull `springs.*` above directly.
 */
export const fade = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit:    { opacity: 0 },
  transition: springs.pop,
};

export const rise = {
  initial: { opacity: 0, y: 6 },
  animate: { opacity: 1, y: 0 },
  exit:    { opacity: 0, y: -4 },
  transition: springs.glide,
};
