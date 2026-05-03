import { useEffect, useRef, useState } from 'react';

import { useDictation } from '@/state/dictation';

/**
 * MeterBars — the seven vertical bars in the floating bar's recording
 * cluster, driven by the live audio meter instead of CSS keyframes.
 *
 * Why this exists: the previous decorative wave looped a constant
 * animation regardless of whether the user was actually speaking, so
 * the bar visually said "I'm listening" even during dead silence at
 * the start of a recording. That eroded trust — users couldn't tell at
 * a glance whether the mic was actually picking them up.
 *
 * Data source: `useDictation.meter` (`{ rms, peak, ts_ms }`), updated
 * from the `dictation:meter` Tauri event in AppBootstrap. Frames
 * arrive at ~30 Hz during an active capture; outside listening we
 * receive no frames, so the displayed heights settle to `MIN_HEIGHT_PX`
 * via the CSS height transition.
 *
 * Smoothing approach:
 *   • Each bar gets a fixed phase offset (`PHASE_OFFSETS`) and
 *     amplitude weight (`PER_BAR_GAIN`) so the seven bars don't move
 *     in perfect lockstep — that would read as a flat block, not a
 *     waveform. The per-bar gain is centred slightly on the middle
 *     bars so silence still feels visually balanced.
 *   • A simple exponential smoothing on the displayed amplitude (rAF
 *     loop, decay coefficient 0.85) softens the rise/fall so the
 *     bars don't strobe between syllables. Decay is asymmetric:
 *     attack is faster than release so a sharp word edge still
 *     visibly snaps up.
 *   • CSS `transition: height 80ms ease-out` (set on
 *     `.ub-bar-wave--meter .ub-bar-wave__bar` in globals.css) does the
 *     last 80ms of interpolation between rAF frames so the result
 *     is a fluid ribbon rather than a 30 Hz judder.
 *
 * Reduced-motion: the existing
 * `html[data-reduce-motion='true'] .ub-bar-wave__bar` rule freezes
 * height at 12px and disables both the animation and our transition,
 * so users who opt out of motion see seven still bars even while the
 * meter is updating. No JS check required here — CSS wins via
 * `!important`.
 *
 * Privacy: the meter frame contains only RMS and peak floats. No raw
 * PCM, no transcript, no logging.
 */

const BAR_COUNT = 7;
const MIN_HEIGHT_PX = 4;
const MAX_HEIGHT_PX = 22;
const ATTACK_COEF = 0.55;
const RELEASE_COEF = 0.18;

// Per-bar phase offsets and gains. Picked by eye to give the seven bars
// a natural-looking mid-range silhouette: outer bars softer, middle
// bars taller, with a small phase shift so a steady note doesn't
// produce a flat-top wall. These are time offsets in milliseconds; the
// component samples a small sine on top of the smoothed amplitude
// using `(now + offset) / 200` to give each bar its own pseudo-position
// in the wave. The amplitude is the same across all bars (we don't
// have a real spectrum), but the visual offset reads as one.
const PHASE_OFFSETS = [0, 60, 120, 180, 240, 300, 360];
const PER_BAR_GAIN = [0.78, 0.92, 0.98, 1.0, 0.96, 0.86, 0.72];

export function MeterBars() {
  // Smoothed amplitude in [0, 1]. Stored in a ref so the rAF loop
  // doesn't trigger React re-renders on every frame; only the bar
  // heights in state drive renders, and we set those once per rAF
  // tick (~60 Hz, well within React's bandwidth for a small element
  // tree).
  const smoothedRef = useRef(0);
  const [heights, setHeights] = useState<number[]>(() =>
    new Array(BAR_COUNT).fill(MIN_HEIGHT_PX),
  );

  useEffect(() => {
    // Single long-lived rAF loop. Reads the latest meter from the
    // zustand store inside the tick rather than subscribing as a
    // component-level selector — that would re-fire this useEffect
    // on every meter update (~30 Hz), tearing down and respawning
    // the loop. `getState()` is a synchronous read of the current
    // slot and is the right tool for animation-frame consumers.
    let raf = 0;
    let last = performance.now();

    const tick = (now: number) => {
      const dt = Math.min(64, now - last); // clamp dt across tab-switch jumps
      last = now;

      const meter = useDictation.getState().meter;
      // Combine RMS + peak the same way `Waveform.tsx` did originally
      // — peak gives the sharp transients (consonants, plosives), RMS
      // gives the steady-state energy (vowels, sustained speech).
      // Sensitivity tuning (post-v0.17.0): the bars need to swing more
      // visibly on normal speech. Speech RMS rarely clears 0.3
      // normalised, so a 2.8× multiplier left the bars sitting in the
      // bottom half of their range for a typical user voice. Bumping
      // to 3.6× lifts steady-state vowels into the upper half; bumping
      // peak weighting from 0.6 to 0.75 makes consonants drive the
      // bars harder, which reads as "alive" more than steady-state
      // RMS does. `Math.min(1, ...)` still pins the upper rail, so
      // loud speech can't overflow the height ceiling.
      const target = Math.min(1, Math.max(0, Math.max(meter.rms, meter.peak * 0.75) * 3.6));

      const current = smoothedRef.current;
      // Asymmetric one-pole smoother. Attack (target > current) catches
      // word onsets quickly; release decays slowly so a finished
      // syllable doesn't snap to zero between words.
      const coef = target > current ? ATTACK_COEF : RELEASE_COEF;
      const dt_norm = dt / 16.67; // normalise to ~60fps reference
      smoothedRef.current = current + (target - current) * Math.min(1, coef * dt_norm);

      // Per-bar height. Sine on top of smoothed amplitude gives the
      // natural-looking "wave moves through the bars" feel without
      // touching the audio (we have no spectrum). The sine amplitude
      // shrinks toward silence so a quiet user gets a steady low row,
      // not a decorative wave on dead air.
      const amp = smoothedRef.current;
      const wave_intensity = amp * 0.35;
      const next_heights: number[] = [];
      for (let i = 0; i < BAR_COUNT; i++) {
        const t = (now + PHASE_OFFSETS[i]) / 200;
        const wave = Math.sin(t) * wave_intensity;
        const bar_amp = Math.max(0, Math.min(1, amp * PER_BAR_GAIN[i] + wave));
        next_heights.push(MIN_HEIGHT_PX + bar_amp * (MAX_HEIGHT_PX - MIN_HEIGHT_PX));
      }
      setHeights(next_heights);

      raf = requestAnimationFrame(tick);
    };

    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, []);

  return (
    <div className="ub-bar-wave ub-bar-wave--meter ml-1 mr-1 shrink-0" aria-hidden>
      {heights.map((h, i) => (
        <span
          key={i}
          className="ub-bar-wave__bar"
          style={{ height: `${h.toFixed(1)}px` }}
        />
      ))}
    </div>
  );
}
