/**
 * hero-pill-loop — the headline video on the marketing home page.
 *
 * 10 s loop, 300 frames at 30 fps. Long, calm pacing on each state so the
 * eye sits with one moment before moving to the next:
 *
 *   0.0-1.5  idle           "Ready" — resting state
 *   1.5-5.0  listening      "Recording", timer 0:00 → 0:03
 *   5.0-6.5  processing     "Processing"
 *   6.5-8.5  success        "Inserted" — the payoff beat
 *   8.5-10.0 idle           seam-safe return to start
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import { tween } from '../util/anim';

export function HeroPillLoop() {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  let state: 'idle' | 'listening' | 'processing' | 'success' = 'idle';
  if (frame < f(1.5)) state = 'idle';
  else if (frame < f(5.0)) state = 'listening';
  else if (frame < f(6.5)) state = 'processing';
  else if (frame < f(8.5)) state = 'success';
  else state = 'idle';

  const elapsedMs = state === 'listening' ? Math.max(0, frame - f(1.5)) * (1000 / fps) : 0;
  const seamFade = tween(frame, durationInFrames - f(0.25), durationInFrames, 1, 0.94);

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
            transform: `scale(${state === 'success' ? 2.44 : 2.4})`,
            transformOrigin: 'center',
          }}
        >
          <BarDriver state={state} elapsedMs={elapsedMs} />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
}
