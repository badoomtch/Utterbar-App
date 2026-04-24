/**
 * feature-system-audio — 7 s. Pure UI-in-motion: a meeting window on the
 * left, Utterbar on the right set to System-audio source pulling
 * transcript lines in real time. No Callout, no tagline — the bar label
 * "Source: System audio" is the only text that isn't part of the meeting
 * or transcript UI, and it's part of the app's real chrome.
 *
 * Ends on the final transcript line landed + the pill still listening,
 * so the loop reads as "the meeting is still going" rather than an ad
 * restart.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import { tween, typewriter } from '../util/anim';

const LINES = [
  { t: 0.6, who: 'Priya', color: '#7c3aed', text: "OK so the release is cut — what's outstanding?" },
  { t: 2.4, who: 'Kiran', color: '#0ea371', text: 'Just the updater smoke test, the rest is green.' },
  { t: 4.4, who: 'Priya', color: '#7c3aed', text: "Great — let's ship at 10 tomorrow." },
];

export function FeatureSystemAudio() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          gap: 56,
          alignItems: 'center',
          justifyContent: 'center',
          padding: '80px 80px',
        }}
      >
        {/* Left — mock meeting window, full-height-ish */}
        <div style={{ width: 820, flexShrink: 0 }}>
          <div
            style={{
              background: '#141519',
              borderRadius: 22,
              overflow: 'hidden',
              boxShadow: '0 28px 70px rgba(0,0,0,0.55)',
            }}
          >
            <div
              style={{
                height: 56,
                background: '#1E1F24',
                display: 'flex',
                alignItems: 'center',
                paddingLeft: 24,
                fontSize: 22,
                color: tokens.muted,
                fontFamily: fonts.sans,
              }}
            >
              Video call · 2 participants
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 0 }}>
              <ParticipantTile who="Priya" color="#7c3aed" />
              <ParticipantTile who="Kiran" color="#0ea371" />
            </div>
          </div>
        </div>

        {/* Right — pill + transcript */}
        <div style={{ width: 860, display: 'flex', flexDirection: 'column', gap: 36 }}>
          <div style={{ position: 'relative', paddingTop: 32 }}>
            <BarDriver state="listening" elapsedMs={(frame / fps) * 1000} />
            {/* "Source: System audio" label floating above the pill —
                part of the real app chrome when a non-mic source is
                selected. Bumped for mobile legibility. */}
            <div
              style={{
                position: 'absolute',
                top: 0,
                left: 28,
                fontFamily: fonts.mono,
                fontSize: 20,
                letterSpacing: '0.14em',
                textTransform: 'uppercase',
                color: tokens.accent2,
                background: tokens.bg1,
                padding: '6px 14px',
                borderRadius: 6,
                border: `1px solid ${tokens.border}`,
              }}
            >
              Source: System audio
            </div>
          </div>

          <div
            style={{
              background: tokens.bg1,
              borderRadius: 22,
              padding: 36,
              border: `1px solid ${tokens.border}`,
              boxShadow: '0 18px 50px rgba(0,0,0,0.4)',
              minHeight: 420,
            }}
          >
            <div
              style={{
                fontFamily: fonts.mono,
                fontSize: 20,
                letterSpacing: '0.14em',
                textTransform: 'uppercase',
                color: tokens.muted,
                marginBottom: 24,
              }}
            >
              Transcript · live
            </div>
            {LINES.map((line) => {
              const start = f(line.t);
              const end = start + f(1.2);
              const opacity = tween(frame, start - 4, start + 4, 0, 1);
              const typed = typewriter(line.text, frame, start, end);
              return (
                <div
                  key={line.who + line.t}
                  style={{ marginBottom: 22, opacity, display: 'flex', gap: 18 }}
                >
                  <div
                    style={{
                      width: 42,
                      height: 42,
                      borderRadius: 999,
                      background: line.color,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      fontSize: 20,
                      fontWeight: 700,
                      color: '#fff',
                      flexShrink: 0,
                    }}
                  >
                    {line.who[0]}
                  </div>
                  <div style={{ fontSize: 28, lineHeight: 1.45, color: tokens.fg }}>
                    <span style={{ fontWeight: 600, color: tokens.fgHi }}>{line.who}:</span>{' '}
                    {typed}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
}

function ParticipantTile({ who, color }: { who: string; color: string }) {
  return (
    <div
      style={{
        height: 420,
        background: `linear-gradient(135deg, ${color}44, ${color}11)`,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        flexDirection: 'column',
        gap: 24,
        position: 'relative',
      }}
    >
      <div
        style={{
          width: 140,
          height: 140,
          borderRadius: 999,
          background: color,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 58,
          fontWeight: 700,
          color: '#fff',
          boxShadow: `0 10px 40px ${color}66`,
        }}
      >
        {who[0]}
      </div>
      <div style={{ fontSize: 26, fontWeight: 500, color: tokens.fg }}>{who}</div>
      <div
        style={{
          position: 'absolute',
          left: 20,
          bottom: 18,
          fontFamily: fonts.mono,
          fontSize: 18,
          color: tokens.muted,
        }}
      >
        unmuted
      </div>
    </div>
  );
}
