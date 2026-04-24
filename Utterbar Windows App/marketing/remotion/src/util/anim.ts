/**
 * Animation helpers shared across compositions. Thin wrappers on
 * `interpolate` so every composition reads at the same level of
 * abstraction — "fade in at frame 20 over 240 ms", not hand-rolled
 * linear tweens.
 */
import { interpolate, type InterpolateOptions } from 'remotion';
import { EASE } from '../design/tokens';

/** Convert milliseconds to frames at the composition's fps. */
export const msToFrames = (ms: number, fps: number) => Math.round((ms / 1000) * fps);

/**
 * Fade / slide / scale a value between two keyframes. Easing defaults to
 * the canonical Utterbar EASE curve. Caller passes the frame plus
 * start/end frame window.
 */
export function tween(
  frame: number,
  startFrame: number,
  endFrame: number,
  from: number,
  to: number,
  opts: Omit<InterpolateOptions, 'easing'> & { easing?: InterpolateOptions['easing'] } = {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  },
) {
  const easing = opts.easing ?? bezierEasing(EASE[0], EASE[1], EASE[2], EASE[3]);
  return interpolate(frame, [startFrame, endFrame], [from, to], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    ...opts,
    easing,
  });
}

/** A minimal cubic-bezier easing matching the canonical EASE. Pure JS so
 *  Remotion's `interpolate` can consume it without any extra deps.
 *  Algorithm: Newton-Raphson iteration to solve for `t` given `x`. */
export function bezierEasing(p1x: number, p1y: number, p2x: number, p2y: number) {
  const NEWTON_ITER = 4;
  const NEWTON_EPS = 0.001;

  function A(a: number, b: number) {
    return 1.0 - 3.0 * b + 3.0 * a;
  }
  function B(a: number, b: number) {
    return 3.0 * b - 6.0 * a;
  }
  function C(a: number) {
    return 3.0 * a;
  }
  function bezier(t: number, a: number, b: number) {
    return ((A(a, b) * t + B(a, b)) * t + C(a)) * t;
  }
  function slope(t: number, a: number, b: number) {
    return 3.0 * A(a, b) * t * t + 2.0 * B(a, b) * t + C(a);
  }

  function solve(x: number) {
    let t = x;
    for (let i = 0; i < NEWTON_ITER; i++) {
      const s = slope(t, p1x, p2x);
      if (s === 0.0) return t;
      const cx = bezier(t, p1x, p2x) - x;
      t -= cx / s;
    }
    return t;
  }

  return (x: number) => {
    if (x <= 0) return 0;
    if (x >= 1) return 1;
    if (p1x === p1y && p2x === p2y) return x;
    return bezier(solve(x), p1y, p2y);
  };
}

/**
 * Typewriter — reveals `text` one character at a time between
 * `startFrame` and `endFrame`. Pure computation; the caller wraps in
 * <span>. The character slice is linear (typing is boring; acceleration
 * reads as "computer cheating").
 */
export function typewriter(
  text: string,
  frame: number,
  startFrame: number,
  endFrame: number,
): string {
  if (frame <= startFrame) return '';
  if (frame >= endFrame) return text;
  const p = (frame - startFrame) / (endFrame - startFrame);
  const n = Math.floor(p * text.length);
  return text.slice(0, n);
}

/** Repeating pulse oscillator, 0..1, sine-based. `period` in frames. */
export function pulse(frame: number, period: number, phase = 0): number {
  return 0.5 + 0.5 * Math.sin(((frame / period) * 2 * Math.PI) + phase);
}
