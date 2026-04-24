/**
 * feature-press-speak-done — 5 s tight pill-state loop. The site pairs
 * this with the textual "Press. Speak. Done." headline, so the video
 * doesn't repeat any of that copy — it just shows the pill running
 * through its four states at roughly 60 % frame width.
 *
 * States cycle: idle → listening → processing → success → idle so the
 * MP4 loops cleanly. Ends on idle, starts on idle — the hand-off is
 * invisible on playback.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import { pulse, tween } from '../util/anim';

export function FeaturePressSpeakDone() {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // State windows (5 s total):
  //   0.0–0.5  idle        (loop seam, hand-off zone)
  //   0.5–3.2  listening   (timer ticks 0 → 2.7 s)
  //   3.2–3.7  processing
  //   3.7–4.3  success
  //   4.3–5.0  idle        (back to start)
  let state: 'idle' | 'listening' | 'processing' | 'success' = 'idle';
  if (frame < f(0.5)) state = 'idle';
  else if (frame < f(3.2)) state = 'listening';
  else if (frame < f(3.7)) state = 'processing';
  else if (frame < f(4.3)) state = 'success';
  else state = 'idle';

  const elapsed = state === 'listening' ? (frame - f(0.5)) / fps : 0;

  // Wash intensity breathes with state — subtle, so the wash never
  // out-shouts the pill itself.
  const washIntensity =
    state === 'listening'
      ? 0.9 + 0.1 * pulse(frame, 8)
      : state === 'success'
        ? 1.2
        : 0.55;

  // Loop-seam: ensure the last few frames hand the first frames a matching
  // opacity so the MP4 loops without a visible cut.
  const seamFade = tween(frame, durationInFrames - f(0.2), durationInFrames, 1, 0.96);

  return (
    <AbsoluteFill style={{ background: tokens.bg0 }}>
      {/* Cyan wash halo behind the pill */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: '50%',
          transform: 'translate(-50%, -50%)',
          width: 1100,
          height: 460,
          background: `radial-gradient(ellipse at center, ${tokens.accent}${state === 'success' ? '55' : '33'} 0%, transparent 65%)`,
          filter: 'blur(70px)',
          opacity: washIntensity,
        }}
      />

      {/* Centred pill — scaled up for 400 px mobile render */}
      <AbsoluteFill
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          opacity: seamFade,
        }}
      >
        {/* Scaled up ~2.3× → ~1470 px wide = ~77 % of the 1920 frame,
            meeting the recipe's 70–90 % hero-element target. Success
            bumps slightly more for the "inserted" hit. */}
        <div
          style={{
            transform: `scale(${state === 'success' ? 2.32 : 2.28})`,
            transition: 'transform 100ms',
            transformOrigin: 'center',
          }}
        >
          <BarDriver state={state} elapsedMs={elapsed * 1000} />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
}
