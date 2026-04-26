/**
 * feature-modes — 7 s. One raw transcript at the top types itself out,
 * then the three shipping modes (Voice-to-text / Message / Meeting)
 * populate underneath in sequence, each reshaping the same raw speech.
 *
 * Regenerated 2026-04-24 for the close-framing pass. Dropped the outer
 * card — the composition now lives directly on --bg-0 so it reads as
 * "the mode cards themselves, on the page", not "a screenshot of a card
 * sitting inside a bigger card". Frame shrunk to 1440x900 so the mode
 * names and bodies are legible on a phone.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween, typewriter } from '../util/anim';

const SAMPLE = {
  raw: 'so basically we need to push the launch to Tuesday because QA flagged two issues with the auth flow and we want them fixed before marketing pings the list',
  modes: [
    {
      id: 'voice-to-text',
      name: 'Voice-to-text',
      description: 'Raw, verbatim',
      output:
        'so basically we need to push the launch to Tuesday because QA flagged two issues with the auth flow and we want them fixed before marketing pings the list',
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
      description: 'Verbatim, line breaks',
      output:
        '• Launch pushed to Tuesday\n• QA flagged 2 issues in auth flow\n• Fix before marketing announcement',
    },
  ],
};

export function FeatureModes() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Raw types 0.3 -> 1.8, modes enter at 2.3, 3.5, 4.7, body typewriter
  // 0.7 s each, ends at 5.4 s then holds on the resting state for 1.6 s.
  const rawTyped = typewriter(SAMPLE.raw, frame, f(0.3), f(1.8));
  const modeEnter = [f(2.3), f(3.5), f(4.7)];

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Raw transcript — 1040 px max so it breaks into two lines at the
          widest, reads cleanly at mobile widths, and the quoted italic
          feels like dictation rather than a static quote. */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 90,
          transform: 'translateX(-50%)',
          width: 1160,
          maxWidth: '90%',
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 16,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 14,
          }}
        >
          You said
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 18,
            padding: '22px 28px',
            fontSize: 24,
            lineHeight: 1.5,
            color: tokens.fg,
            fontStyle: 'italic',
            border: `1px solid ${tokens.border}`,
            minHeight: 104,
          }}
        >
          &ldquo;{rawTyped}&rdquo;
        </div>
      </div>

      {/* Three mode cards — tight grid, roomy padding, accent rail marks
          the selected mode (all three "selected" visually because each
          is showing its own output of the same speech). */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 360,
          transform: 'translateX(-50%)',
          width: 1260,
          maxWidth: '94%',
          display: 'grid',
          gridTemplateColumns: '1fr 1fr 1fr',
          gap: 20,
        }}
      >
        {SAMPLE.modes.map((m, i) => {
          const opacity = tween(frame, modeEnter[i] - 6, modeEnter[i] + 6, 0, 1);
          const y = tween(frame, modeEnter[i] - 6, modeEnter[i] + 12, 12, 0);
          const bodyStart = modeEnter[i] + f(0.1);
          const bodyEnd = modeEnter[i] + f(0.8);
          const body = typewriter(m.output, frame, bodyStart, bodyEnd);
          return (
            <div
              key={m.id}
              style={{
                background: tokens.bg1,
                borderRadius: 20,
                padding: '26px 28px',
                opacity,
                transform: `translateY(${y}px)`,
                position: 'relative',
                border: `1px solid ${tokens.border}`,
                minHeight: 360,
              }}
            >
              <span
                style={{
                  position: 'absolute',
                  left: 0,
                  top: 26,
                  bottom: 26,
                  width: 3,
                  borderRadius: 999,
                  background: tokens.accent,
                }}
              />
              <div
                style={{
                  fontSize: 26,
                  fontWeight: 700,
                  color: tokens.fgHi,
                  marginBottom: 4,
                  paddingLeft: 14,
                  letterSpacing: '-0.01em',
                }}
              >
                {m.name}
              </div>
              <div
                style={{
                  fontFamily: fonts.mono,
                  fontSize: 18,
                  letterSpacing: '0.14em',
                  textTransform: 'uppercase',
                  color: tokens.muted,
                  marginBottom: 18,
                  paddingLeft: 14,
                }}
              >
                {m.description}
              </div>
              <div
                style={{
                  fontSize: 18,
                  lineHeight: 1.55,
                  color: tokens.fg,
                  whiteSpace: 'pre-line',
                  paddingLeft: 14,
                }}
              >
                {body}
              </div>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
}
