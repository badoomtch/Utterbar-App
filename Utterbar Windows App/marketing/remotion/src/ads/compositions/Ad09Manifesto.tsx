/**
 * ad-09-manifesto — 30s. The brand film. Pure kinetic typography over
 * a slow-pulsing pill. No mock app, no host window. Ten short lines,
 * each one held for ~2.4-3.0s, fading through. The pill stays small
 * and centred at the top throughout.
 *
 * Lines lifted from the BRIEF.md anchors and elevator pitch — verified
 * British spellings, no banned words.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { EndCard } from '../frame/EndCard';

const LINES: { text: string; at: number; until: number; size?: number; color?: string }[] = [
  { text: 'Press a key.',                          at: 0.5,  until: 3.0 },
  { text: 'Speak.',                                at: 3.0,  until: 5.0, size: 180 },
  { text: 'The words appear in whatever app has your cursor.', at: 5.0, until: 9.0, size: 72 },
  { text: 'Slack. Chrome. VS Code. ChatGPT.',      at: 9.0,  until: 12.0, size: 78, color: '#22D3EE' },
  { text: 'No cloud.',                             at: 12.0, until: 14.5, size: 140 },
  { text: 'No account.',                           at: 14.5, until: 17.0, size: 140 },
  { text: 'No subscription.',                      at: 17.0, until: 19.5, size: 140 },
  { text: 'Just dictation.',                       at: 19.5, until: 22.5, size: 130, color: '#22D3EE' },
  { text: 'On your machine.',                      at: 22.5, until: 25.5, size: 110 },
  { text: 'For Windows.',                          at: 25.5, until: 27.5, size: 110, color: '#22D3EE' },
];

export function Ad09Manifesto() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Pill is a soft, slow listening pulse throughout.
  const pillElapsed = (frame / fps) * 1000;

  const active = LINES.find((l) => frame >= f(l.at) && frame < f(l.until));
  const opacity = active
    ? Math.min(
        tween(frame, f(active.at), f(active.at) + 8, 0, 1),
        tween(frame, f(active.until) - 10, f(active.until), 1, 0),
      )
    : 0;
  const slide = active ? tween(frame, f(active.at), f(active.at) + 16, 16, 0) : 0;

  return (
    <SocialFrame variant="canvas">
      <div
        style={{
          position: 'absolute',
          top: 220,
          left: '50%',
          transform: 'translateX(-50%) scale(1.4)',
          transformOrigin: 'top center',
          opacity: 0.92,
        }}
      >
        <BarDriver state="listening" elapsedMs={pillElapsed} />
      </div>

      {active && (
        <div
          style={{
            position: 'absolute',
            top: 700,
            left: 0,
            right: 0,
            textAlign: 'center',
            padding: '0 80px',
            opacity,
            transform: `translateY(${slide}px)`,
            fontFamily: fonts.sans,
            color: active.color ?? tokens.fgHi,
            fontSize: active.size ?? 96,
            fontWeight: 600,
            lineHeight: 1.2,
            letterSpacing: '-0.025em',
          }}
        >
          {active.text}
        </div>
      )}

      <EndCard tagline="Dictation that stays on your PC." startAt={27.5} />
    </SocialFrame>
  );
}
