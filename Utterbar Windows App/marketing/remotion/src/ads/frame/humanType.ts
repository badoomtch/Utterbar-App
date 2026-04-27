/**
 * Human-cadence typewriter — the existing util/anim.ts `typewriter` is
 * uniform-rate (good for paste-in moments). Real human typing is
 * burst-pause-burst, with thinking gaps. This module returns the
 * substring of `text` that would have been typed by frame `frame`
 * given a per-character delay sequence derived from `seed`.
 *
 * Deterministic — same seed → same pattern, so renders are stable.
 *
 * Tunable via `wordsPerMinute` (default 65) and `pauseAfterPunct` (true).
 */

const CHAR_MS_BASE = 1000 / ((65 * 5) / 60); // ~185 ms per char at 65 wpm
const JITTER = 0.55;                          // ±55% per-char variance
const SPACE_BONUS = 0.65;                     // spaces are faster
const PUNCT_PAUSE_MS = 320;                   // pause after . , ; ? !

function mulberry32(seed: number) {
  let a = seed | 0;
  return () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const cache = new Map<string, number[]>();

function delays(text: string, seed: number) {
  const k = `${seed}|${text}`;
  if (cache.has(k)) return cache.get(k)!;
  const rng = mulberry32(seed);
  const out: number[] = new Array(text.length);
  for (let i = 0; i < text.length; i += 1) {
    const ch = text[i];
    let ms = CHAR_MS_BASE * (1 + (rng() - 0.5) * 2 * JITTER);
    if (ch === ' ') ms *= SPACE_BONUS;
    if (i > 0 && /[.,;?!]/.test(text[i - 1])) ms += PUNCT_PAUSE_MS;
    out[i] = Math.max(40, Math.round(ms));
  }
  cache.set(k, out);
  return out;
}

/**
 * Return the text the user has typed by `frame` given they started at
 * `startFrame`. Inter-keystroke timing is human-like; same `seed` yields
 * the same pattern frame-for-frame.
 */
export function humanType(
  text: string,
  frame: number,
  startFrame: number,
  fps: number,
  seed = 0xa5a5,
) {
  if (frame <= startFrame) return '';
  const dl = delays(text, seed);
  const elapsedMs = ((frame - startFrame) / fps) * 1000;
  let acc = 0;
  for (let i = 0; i < dl.length; i += 1) {
    acc += dl[i];
    if (acc > elapsedMs) return text.slice(0, i);
  }
  return text;
}

/** Total render time in seconds for a given seed — handy for sizing
 *  composition durations to the typing length. */
export function humanTypeDurationSec(text: string, seed = 0xa5a5) {
  const total = delays(text, seed).reduce((a, b) => a + b, 0);
  return total / 1000;
}
