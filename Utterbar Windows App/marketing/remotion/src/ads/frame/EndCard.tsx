/**
 * End card — final 4-5 seconds of every ad. v3 fixes the v2 bug where
 * the URL was only readable for ~0.7s. The whole card is meant to be
 * a held image you can screenshot, not a flash. Slower fades and a
 * separate `holdSeconds` knob the parent uses to size composition
 * duration around it.
 *
 * Element timing (relative to startAt, in frames @30fps):
 *   0  – 8   backdrop fades in
 *   8  – 22  pill drops in + scales up
 *   18 – 36  tagline rises in
 *   28 – 50  url rises in
 *   50+      held until the end of the composition
 *
 * Caller decides total visibility by setting the composition's
 * durationInFrames to startAt + END_CARD_TOTAL_SECONDS * fps.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';

/** End cards must hold for this long after their startAt. Compositions
 *  use this to size their `durationInFrames`. */
export const END_CARD_HOLD_SECONDS = 4.5;

type Props = {
  tagline: string;
  startAt: number;
};

export function EndCard({ tagline, startAt }: Props) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  if (frame < f(startAt)) return null;

  const startF = f(startAt);
  const backdrop = tween(frame, startF + 0,  startF + 8,  0, 1);
  const pillFade = tween(frame, startF + 8,  startF + 20, 0, 1);
  const pillSlide = tween(frame, startF + 8,  startF + 22, 24, 0);
  const taglineFade = tween(frame, startF + 18, startF + 36, 0, 1);
  const taglineSlide = tween(frame, startF + 18, startF + 36, 18, 0);
  const urlFade = tween(frame, startF + 28, startF + 50, 0, 1);
  const urlSlide = tween(frame, startF + 28, startF + 50, 18, 0);

  return (
    <AbsoluteFill
      style={{
        background: 'rgba(8, 12, 20, 0.96)',
        backdropFilter: 'blur(40px)',
        opacity: backdrop,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 80,
        fontFamily: fonts.sans,
      }}
    >
      <div
        style={{
          opacity: pillFade,
          transform: `translateY(${pillSlide}px) scale(2.4)`,
          transformOrigin: 'center',
        }}
      >
        <BarDriver state="idle" />
      </div>

      <div
        style={{
          fontSize: 92,
          fontWeight: 700,
          color: tokens.fgHi,
          textAlign: 'center',
          maxWidth: 1000,
          lineHeight: 1.1,
          letterSpacing: '-0.025em',
          opacity: taglineFade,
          transform: `translateY(${taglineSlide}px)`,
          padding: '0 60px',
        }}
      >
        {tagline}
      </div>

      <div
        style={{
          fontSize: 108,
          fontFamily: fonts.mono,
          color: tokens.accent,
          textShadow: `0 0 48px ${tokens.accent}aa`,
          letterSpacing: '-0.02em',
          fontWeight: 700,
          opacity: urlFade,
          transform: `translateY(${urlSlide}px)`,
        }}
      >
        utterbar.com
      </div>
    </AbsoluteFill>
  );
}
