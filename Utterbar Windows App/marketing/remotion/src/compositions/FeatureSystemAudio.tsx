/**
 * feature-system-audio — 6 s. The three capture sources as they actually
 * appear in Audio Settings: Microphone, System audio, Mixed. Selection
 * animates from Microphone -> System audio. A live transcript grows
 * below showing the other-party's words being captured.
 *
 * Regenerated 2026-04-24 — replaced the fake "video-call window with
 * Utterbar floating beside it" with the actual selector UI. The feature
 * is "three sources, pick one" — so we show the three sources.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween, typewriter } from '../util/anim';

const SOURCES = [
  {
    id: 'microphone',
    label: 'Microphone',
    note: 'Your voice, from the selected mic.',
  },
  {
    id: 'system',
    label: 'System audio',
    note: 'Whatever is playing through your speakers — meetings, videos, voice notes.',
  },
  {
    id: 'mixed',
    label: 'Mixed (mic + system)',
    note: 'Both at once, summed. For calls where you also speak.',
  },
];

const TRANSCRIPT = '…and the quarterly numbers came in about 8% above what we forecast, which the board is going to love.';

export function FeatureSystemAudio() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Selection switch: microphone (0) → system (1) at ~1.4 s.
  const selected = frame < f(1.4) ? 0 : 1;
  const typed = typewriter(TRANSCRIPT, frame, f(2.0), f(5.4));

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Label strip */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 70,
          transform: 'translateX(-50%)',
          fontFamily: fonts.mono,
          fontSize: 18,
          letterSpacing: '0.18em',
          textTransform: 'uppercase',
          color: tokens.muted,
        }}
      >
        Audio source
      </div>

      {/* Three source cards, horizontal row. The selected one lights
          accent border + shows the accent LED dot. */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 120,
          transform: 'translateX(-50%)',
          width: 1180,
          maxWidth: '92%',
          display: 'grid',
          gridTemplateColumns: '1fr 1fr 1fr',
          gap: 18,
        }}
      >
        {SOURCES.map((s, i) => {
          const active = selected === i;
          const accentTransition = active
            ? tween(frame, f(1.3), f(1.6), 0, 1)
            : 1 - tween(frame, f(1.3), f(1.6), 0, 1);
          return (
            <div
              key={s.id}
              style={{
                background: tokens.bg1,
                borderRadius: 18,
                padding: '22px 24px',
                border: `1px solid ${
                  active ? `rgba(34, 211, 238, ${0.3 + accentTransition * 0.35})` : tokens.border
                }`,
                boxShadow: active
                  ? `0 0 0 1px rgba(34, 211, 238, ${accentTransition * 0.3})`
                  : 'none',
                minHeight: 170,
                display: 'flex',
                flexDirection: 'column',
                gap: 10,
              }}
            >
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 10,
                }}
              >
                <div
                  style={{
                    width: 10,
                    height: 10,
                    borderRadius: '50%',
                    background: active ? tokens.accent : 'rgba(255,255,255,0.14)',
                  }}
                />
                <div
                  style={{
                    fontSize: 22,
                    fontWeight: 600,
                    color: tokens.fgHi,
                    letterSpacing: '-0.005em',
                  }}
                >
                  {s.label}
                </div>
              </div>
              <div
                style={{
                  fontSize: 18,
                  lineHeight: 1.55,
                  color: tokens.muted,
                }}
              >
                {s.note}
              </div>
            </div>
          );
        })}
      </div>

      {/* Live transcript card — appears under the selector, grows text as
          the user "hears" whatever the system is playing. */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 400,
          transform: 'translateX(-50%)',
          width: 1060,
          maxWidth: '86%',
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 17,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 14,
            display: 'flex',
            alignItems: 'center',
            gap: 10,
          }}
        >
          <div
            style={{
              width: 8,
              height: 8,
              borderRadius: '50%',
              background: tokens.accent,
              opacity: frame >= f(2.0) && frame < f(5.5) ? 1 : 0.3,
              animation: frame >= f(2.0) && frame < f(5.5) ? 'none' : 'none',
            }}
          />
          Live transcript
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 18,
            padding: '28px 32px',
            border: `1px solid ${tokens.border}`,
            fontSize: 24,
            lineHeight: 1.55,
            color: tokens.fg,
            minHeight: 200,
          }}
        >
          {typed}
        </div>
      </div>
    </AbsoluteFill>
  );
}
