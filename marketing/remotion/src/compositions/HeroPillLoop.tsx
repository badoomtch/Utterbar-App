/**
 * hero-pill-loop — the headline video at the top of the marketing home
 * page. 6 s, loops cleanly. No copy; the pill is the copy.
 *
 * Narrative (6 s / 180 frames @ 30 fps):
 *   0.00–0.80  idle pill, subtle ambient pulse on the mini-mark LED dot
 *   0.80–1.40  state-pill crossfades Ready → Recording, border lights
 *              cyan, mic icon swells to rose "stop" square
 *   1.40–4.40  listening — waveform animates from low to full energy;
 *              timer ticks 00:00 → 00:03
 *   4.40–5.00  crossfade listening → processing, waveform collapses
 *   5.00–5.60  processing → success flash ("Inserted" on moss)
 *   5.60–6.00  settle back to idle (loop seam)
 *
 * The composition is BG-0 (#0A0E17 Obsidian navy — matches the site's
 * video section) with a soft cyan accent wash behind the pill so the
 * bar reads as levitating rather than floating on nothing.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import { pulse, tween } from '../util/anim';

export function HeroPillLoop() {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();

  // Phase boundaries in frames.
  const f = (sec: number) => Math.round(sec * fps);
  const idleEnd = f(0.8);
  const listenStart = f(1.4);
  const listenEnd = f(4.4);
  const procEnd = f(5.0);
  const successEnd = f(5.6);

  let state: 'idle' | 'listening' | 'processing' | 'success' = 'idle';
  if (frame < idleEnd) state = 'idle';
  else if (frame < listenStart) state = 'idle'; // crossfade window handled below
  else if (frame < listenEnd) state = 'listening';
  else if (frame < procEnd) state = 'processing';
  else if (frame < successEnd) state = 'success';
  else state = 'idle';

  // Only the elapsed timer is still a local driver — waveform energy,
  // LED breathe and recording dot now come from the real pill's CSS
  // animations (`.ub-bar-wave__bar`, `.ub-bar-ripple`, `pulseDot`). We
  // no longer synthesise a `waveProgress` / `ambient` value here: those
  // were inputs to the port's hand-drawn waveform, which is gone.
  const listenPulse = (frame - listenStart) / fps; // real seconds, for timer
  const elapsed = state === 'listening' ? Math.min(3, Math.max(0, listenPulse)) : 0;

  // Loop-seam: ensure the last few frames hand the first frames a matching
  // opacity so the MP4 loops without a visible cut.
  const seamFade = tween(frame, durationInFrames - f(0.2), durationInFrames, 1, 0.9);

  // Behind-pill atmospheric wash — scales subtly with state so the whole
  // frame "inhales" with the pill, rather than staying flat.
  const washIntensity =
    state === 'listening' ? 0.9 + 0.1 * pulse(frame, 8) : state === 'success' ? 1.2 : 0.5;

  return (
    <AbsoluteFill style={{ background: tokens.bg0 }}>
      {/* Cyan wash halo — sits behind the pill, scales with listening
          pulse. Blur is heavy so it reads as atmosphere, not a ring. */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: '50%',
          transform: 'translate(-50%, -50%)',
          width: 900,
          height: 400,
          background: `radial-gradient(ellipse at center, ${tokens.accent}${state === 'success' ? '55' : '33'} 0%, transparent 65%)`,
          filter: 'blur(60px)',
          opacity: washIntensity,
        }}
      />

      {/* Central stage — pill dead-centre, pinned width so the content
          width doesn't jitter between states. */}
      <AbsoluteFill
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          opacity: seamFade,
        }}
      >
        {/* Scaled up ~2.2× so the hero pill lands at ~1410 px wide
            — 73 % of the 1920 frame, meeting the hero recipe. At 1× the
            bar shrinks to ~125 px on a 400 px mobile render and the
            word-mark becomes unreadable. Success bumps slightly more
            for the "inserted" hit. */}
        <div
          style={{
            transform: `scale(${state === 'success' ? 2.24 : 2.2})`,
            transition: 'transform 100ms',
            transformOrigin: 'center',
          }}
        >
          {/* Drive the real FloatingBar via BarDriver — seeds
              `useDictation`/`useSettings` with a deterministic phase
              on every frame. `elapsedMs` is what the pill's mono
              timer reads; multiply from the seconds-based local
              driver. BarDriver picks its own width/shape from the
              shipping pill; we deliberately don't pin `width`. */}
          <BarDriver state={state} elapsedMs={elapsed * 1000} />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
}
