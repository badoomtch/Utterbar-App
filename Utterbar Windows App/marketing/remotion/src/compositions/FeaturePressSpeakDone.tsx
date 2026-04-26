/**
 * feature-press-speak-done — 6 s banner loop of the floating bar moving
 * through its active states. Shorter than the hero, but with the same
 * structural pattern: idle → listening → processing → idle.
 *
 * Deliberately skips the `success` phase because the shipping
 * FloatingBar's `expanded` boolean only stays true for active phases
 * and a brief terminal window; in headless render the terminal window
 * is unreliable, so success can collapse the bar around the chip.
 * Sticking to active phases keeps the bar visibly expanded throughout.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import { tween } from '../util/anim';

export function FeaturePressSpeakDone() {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  let state: 'idle' | 'listening' | 'processing' = 'idle';
  if (frame < f(1.0)) state = 'idle';
  else if (frame < f(3.5)) state = 'listening';
  else if (frame < f(5.0)) state = 'processing';
  else state = 'idle';

  const elapsedMs = state === 'listening' ? Math.max(0, frame - f(1.0)) * (1000 / fps) : 0;
  const seamFade = tween(frame, durationInFrames - f(0.2), durationInFrames, 1, 0.94);

  return (
    <AbsoluteFill style={{ background: tokens.bg0 }}>
      <AbsoluteFill
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          opacity: seamFade,
        }}
      >
        <div
          style={{
            transform: 'scale(2.46)',
            transformOrigin: 'center',
          }}
        >
          <BarDriver state={state} elapsedMs={elapsedMs} />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
}
