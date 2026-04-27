/**
 * ad-09-manifesto — 28s. v3: each line holds slightly longer, end
 * card 4.5s. Pill scale 2.0 (already).
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { EndCard } from '../frame/EndCard';

const LINES: { text: string; at: number; until: number; size?: number; color?: string }[] = [
  { text: 'Press a key.',                                       at: 0.3,  until: 2.6,  size: 180 },
  { text: 'Speak.',                                             at: 2.6,  until: 4.2,  size: 220 },
  { text: 'The words appear in whatever app has your cursor.',  at: 4.2,  until: 8.6,  size: 90 },
  { text: 'Slack. Chrome. VS Code. ChatGPT.',                   at: 8.6,  until: 12.0, size: 100, color: '#22D3EE' },
  { text: 'No cloud.',                                          at: 12.0, until: 14.4, size: 200 },
  { text: 'No account.',                                        at: 14.4, until: 16.8, size: 200 },
  { text: 'No subscription.',                                   at: 16.8, until: 19.6, size: 180 },
  { text: 'Just dictation.',                                    at: 19.6, until: 22.4, size: 180, color: '#22D3EE' },
  { text: 'For Windows.',                                       at: 22.4, until: 24.8, size: 160, color: '#22D3EE' },
];

export function Ad09Manifesto() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const pillElapsed = (frame / fps) * 1000;

  const active = LINES.find((l) => frame >= f(l.at) && frame < f(l.until));
  const opacity = active
    ? Math.min(
        tween(frame, f(active.at), f(active.at) + 6, 0, 1),
        tween(frame, f(active.until) - 8, f(active.until), 1, 0),
      )
    : 0;
  const slide = active ? tween(frame, f(active.at), f(active.at) + 12, 22, 0) : 0;

  return (
    <SocialFrame variant="canvas">
      <div style={{ position: 'absolute', top: 200, left: '50%', transform: 'translateX(-50%) scale(2.0)', transformOrigin: 'top center', opacity: 0.92 }}>
        <BarDriver state="listening" elapsedMs={pillElapsed} />
      </div>

      {active && (
        <div
          style={{
            position: 'absolute',
            top: 720,
            left: 0,
            right: 0,
            textAlign: 'center',
            padding: '0 70px',
            opacity,
            transform: `translateY(${slide}px)`,
            fontFamily: fonts.sans,
            color: active.color ?? tokens.fgHi,
            fontSize: active.size ?? 120,
            fontWeight: 700,
            lineHeight: 1.1,
            letterSpacing: '-0.03em',
          }}
        >
          {active.text}
        </div>
      )}

      <EndCard tagline="Dictation that stays on your PC." startAt={24.8} />
    </SocialFrame>
  );
}
