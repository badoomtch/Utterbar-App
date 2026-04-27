/**
 * End card — last 2-3 seconds of every ad. Wordmark + one-line tagline +
 * URL. Cyan reserved for the URL only (per design system: ≤3 cyan moments
 * per frame, end card spends its budget on CTA).
 *
 * Caller picks the tagline; URL is fixed because there is only one URL.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';

type Props = {
  tagline: string;
  /** Seconds into the parent composition that the end-card starts. */
  startAt: number;
};

export function EndCard({ tagline, startAt }: Props) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  if (frame < f(startAt)) return null;

  const fade = tween(frame, f(startAt), f(startAt) + 8, 0, 1);
  const slide = tween(frame, f(startAt), f(startAt) + 16, 24, 0);
  const taglineFade = tween(frame, f(startAt) + 6, f(startAt) + 18, 0, 1);
  const urlFade = tween(frame, f(startAt) + 14, f(startAt) + 26, 0, 1);

  return (
    <AbsoluteFill
      style={{
        background: 'rgba(10, 14, 23, 0.88)',
        backdropFilter: 'blur(40px)',
        opacity: fade,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 40,
        fontFamily: fonts.sans,
      }}
    >
      <div style={{ transform: `translateY(${slide}px) scale(2.6)` }}>
        <BarDriver state="idle" />
      </div>

      <div style={{ height: 60 }} />

      <div
        style={{
          fontSize: 64,
          fontWeight: 600,
          color: tokens.fgHi,
          textAlign: 'center',
          maxWidth: 880,
          lineHeight: 1.18,
          letterSpacing: '-0.02em',
          opacity: taglineFade,
          transform: `translateY(${(1 - taglineFade) * 12}px)`,
        }}
      >
        {tagline}
      </div>

      <div
        style={{
          fontSize: 56,
          fontFamily: fonts.mono,
          color: tokens.accent,
          textShadow: `0 0 24px ${tokens.accent}66`,
          letterSpacing: '-0.01em',
          opacity: urlFade,
          transform: `translateY(${(1 - urlFade) * 8}px)`,
        }}
      >
        utterbar.com
      </div>
    </AbsoluteFill>
  );
}
