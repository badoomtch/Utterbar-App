/**
 * Burned-in caption strip — bigger and lower than v1. v1 used 56 px font
 * which on a phone displayed at ~360 px wide (arm's length) renders as
 * ~19 px — too small. v2 is 88 px (~30 px on phone), bottom 130 px from
 * frame edge so it falls cleanly inside thumb-zone safe area.
 *
 * Black drop-shadow box behind text — captions read regardless of what's
 * underneath. Real social ads (TikTok, Reels) all do this.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';

export type CaptionCue = {
  text: string;
  /** Seconds — start visible at this time. */
  at: number;
  /** Seconds — fade out at this time. */
  until: number;
};

type Props = {
  cues: CaptionCue[];
  /** Distance from the bottom of the frame. */
  bottom?: number;
};

export function Captions({ cues, bottom = 130 }: Props) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const active = cues.find((c) => frame >= f(c.at) && frame < f(c.until));
  if (!active) return null;

  const opacity = Math.min(
    tween(frame, f(active.at), f(active.at) + 4, 0, 1),
    tween(frame, f(active.until) - 4, f(active.until), 1, 0),
  );

  return (
    <div
      style={{
        position: 'absolute',
        left: 0,
        right: 0,
        bottom,
        display: 'flex',
        justifyContent: 'center',
        opacity,
      }}
    >
      <div
        style={{
          maxWidth: 1000,
          textAlign: 'center',
          fontFamily: fonts.sans,
          fontSize: 88,
          lineHeight: 1.1,
          fontWeight: 700,
          color: tokens.fgHi,
          padding: '14px 28px',
          letterSpacing: '-0.02em',
          textShadow: '0 6px 32px rgba(0,0,0,0.85), 0 2px 0 rgba(0,0,0,0.6)',
        }}
      >
        {active.text}
      </div>
    </div>
  );
}
