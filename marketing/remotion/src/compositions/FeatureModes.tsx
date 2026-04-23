/**
 * feature-modes — 8 s. Single card showing the Modes component: one raw
 * transcript at the top types out, then three mode panels populate
 * underneath, staggered. Each panel shows how that mode reshapes the
 * same raw speech. No Callout — this is just the Modes UI doing its
 * thing.
 *
 * Ends on all three panels visible with their outputs rendered, so the
 * loop reads as "here's the finished state of the Modes page".
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween, typewriter } from '../util/anim';

const SAMPLE = {
  raw: 'so basically we need to push the launch to Tuesday because the QA team flagged two issues with the auth flow and we want them fixed before marketing pings the list',
  modes: [
    {
      id: 'voice-to-text',
      name: 'Voice-to-text',
      description: 'Raw, verbatim',
      output:
        'so basically we need to push the launch to Tuesday because the QA team flagged two issues with the auth flow and we want them fixed before marketing pings the list',
    },
    {
      id: 'message',
      name: 'Message',
      description: 'Cleaned for chat',
      output:
        'Quick heads up — pushing launch to Tuesday. QA flagged two auth-flow issues we want fixed before marketing sends the announcement.',
    },
    {
      id: 'meeting',
      name: 'Meeting',
      description: 'Bullet-style note',
      output:
        '• Launch pushed to Tuesday\n• QA flagged 2 issues in auth flow\n• Fix before marketing announcement',
    },
  ],
};

export function FeatureModes() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Reveal: raw types 0.4 → 2.2, then each mode panel at 2.8, 4.2, 5.6
  // so we end at 6.6 s and sit on the resting state for 1.4 s.
  const rawTyped = typewriter(SAMPLE.raw, frame, f(0.4), f(2.2));
  const modeEnter = [f(2.8), f(4.2), f(5.6)];

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Single Modes card */}
      <div
        style={{
          position: 'absolute',
          left: 110,
          top: 80,
          width: 1700,
          height: 920,
          background: tokens.bg1,
          borderRadius: 28,
          padding: 48,
          border: `1px solid ${tokens.border}`,
          boxShadow: '0 32px 80px rgba(0,0,0,0.5)',
          boxSizing: 'border-box',
          display: 'flex',
          flexDirection: 'column',
          gap: 32,
        }}
      >
        {/* Raw transcript row — full width, big italic text */}
        <div>
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 22,
              letterSpacing: '0.14em',
              textTransform: 'uppercase',
              color: tokens.muted,
              marginBottom: 16,
            }}
          >
            Raw transcript
          </div>
          <div
            style={{
              background: tokens.bg2,
              borderRadius: 18,
              padding: '28px 32px',
              fontSize: 30,
              lineHeight: 1.45,
              color: tokens.fg,
              fontStyle: 'italic',
              minHeight: 140,
              border: `1px solid ${tokens.border}`,
            }}
          >
            "{rawTyped}"
          </div>
        </div>

        {/* Three mode panels — equal columns, big type */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: '1fr 1fr 1fr',
            gap: 24,
            flex: 1,
            minHeight: 0,
          }}
        >
          {SAMPLE.modes.map((m, i) => {
            const opacity = tween(frame, modeEnter[i] - 6, modeEnter[i] + 6, 0, 1);
            const y = tween(frame, modeEnter[i] - 6, modeEnter[i] + 12, 16, 0);
            // Body typewriters in over 0.5 s after the panel lands.
            const bodyStart = modeEnter[i] + f(0.1);
            const bodyEnd = modeEnter[i] + f(0.9);
            const body = typewriter(m.output, frame, bodyStart, bodyEnd);
            return (
              <div
                key={m.id}
                style={{
                  background: tokens.bg2,
                  borderRadius: 20,
                  padding: 28,
                  opacity,
                  transform: `translateY(${y}px)`,
                  position: 'relative',
                  border: `1px solid ${tokens.border}`,
                  display: 'flex',
                  flexDirection: 'column',
                }}
              >
                <span
                  style={{
                    position: 'absolute',
                    left: 0,
                    top: 28,
                    bottom: 28,
                    width: 3,
                    borderRadius: 999,
                    background: tokens.accent,
                  }}
                />
                <div
                  style={{
                    fontSize: 34,
                    fontWeight: 700,
                    color: tokens.fgHi,
                    marginBottom: 6,
                    paddingLeft: 12,
                    letterSpacing: '-0.01em',
                  }}
                >
                  {m.name}
                </div>
                <div
                  style={{
                    fontFamily: fonts.mono,
                    fontSize: 18,
                    letterSpacing: '0.12em',
                    textTransform: 'uppercase',
                    color: tokens.muted,
                    marginBottom: 22,
                    paddingLeft: 12,
                  }}
                >
                  {m.description}
                </div>
                <div
                  style={{
                    fontSize: 24,
                    lineHeight: 1.5,
                    color: tokens.fg,
                    whiteSpace: 'pre-line',
                    paddingLeft: 12,
                  }}
                >
                  {body}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
}
