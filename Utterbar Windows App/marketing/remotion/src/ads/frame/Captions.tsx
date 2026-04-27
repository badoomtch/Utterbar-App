/**
 * Burned-in caption strip — required because 80%+ of social plays muted.
 * Sits in the lower third (above the taskbar safe-area). One short line
 * at a time; bold weight, plenty of contrast.
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
  /** Distance from the bottom of the frame. Default leaves room for the
   *  faux taskbar from SocialFrame. */
  bottom?: number;
};

export function Captions({ cues, bottom = 160 }: Props) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const active = cues.find((c) => frame >= f(c.at) && frame < f(c.until));
  if (!active) return null;

  const opacity = Math.min(
    tween(frame, f(active.at), f(active.at) + 6, 0, 1),
    tween(frame, f(active.until) - 6, f(active.until), 1, 0),
  );

  return (
    <div
      style={{
        position: 'absolute',
        left: '50%',
        bottom,
        transform: 'translateX(-50%)',
        width: 920,
        textAlign: 'center',
        fontFamily: fonts.sans,
        fontSize: 56,
        lineHeight: 1.2,
        fontWeight: 600,
        color: tokens.fgHi,
        textShadow: '0 4px 24px rgba(0,0,0,0.6), 0 1px 0 rgba(0,0,0,0.4)',
        letterSpacing: '-0.01em',
        opacity,
      }}
    >
      {active.text}
    </div>
  );
}
