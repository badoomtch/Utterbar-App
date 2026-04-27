/**
 * ad-12-founders-note — 20s. The "made by one person who got tired of
 * typing" message. No mock app, no host window. A short typewritten
 * note appears in mono, line by line, with the pill listening above.
 * Reads as a personal letter, not as marketing copy.
 *
 * Different angle from anything in the existing pack: founder voice,
 * personal register, the BRIEF.md ethos line ("Local-only, no cloud,
 * no account for the app, no subscription tax on speaking") rendered
 * as the closing.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { EndCard } from '../frame/EndCard';
import { humanType } from '../frame/humanType';

const NOTE_LINES = [
  { text: "I built this because I got tired of typing.", at: 1.0,  seed: 0x10 },
  { text: "Press Ctrl + Space.",                       at: 4.5,  seed: 0x20 },
  { text: "Speak.",                                    at: 6.0,  seed: 0x30 },
  { text: "The words appear in whatever app you’re in.", at: 7.5,  seed: 0x40 },
  { text: "Audio never leaves your machine.",          at: 11.0, seed: 0x50 },
  { text: "No cloud, no account, no subscription.",    at: 13.5, seed: 0x60 },
  { text: "Just dictation. For Windows. £39.99 once.", at: 16.0, seed: 0x70 },
];

export function Ad12FoundersNote() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Pill stays in a soft listening pulse the whole time.
  const pillElapsed = (frame / fps) * 1000;

  return (
    <SocialFrame variant="canvas">
      <div
        style={{
          position: 'absolute',
          top: 90,
          left: '50%',
          transform: 'translateX(-50%)',
          fontFamily: fonts.mono,
          fontSize: 16,
          letterSpacing: '0.18em',
          textTransform: 'uppercase',
          color: tokens.muted,
        }}
      >
        A note from the maker
      </div>

      <div
        style={{
          position: 'absolute',
          top: 200,
          left: '50%',
          transform: 'translateX(-50%) scale(1.1)',
          transformOrigin: 'top center',
          opacity: 0.86,
        }}
      >
        <BarDriver state="listening" elapsedMs={pillElapsed} />
      </div>

      <div
        style={{
          position: 'absolute',
          top: 460,
          left: 0,
          right: 0,
          padding: '0 90px',
          fontFamily: fonts.mono,
          fontSize: 36,
          color: tokens.fg,
          lineHeight: 1.55,
          letterSpacing: '-0.005em',
        }}
      >
        {NOTE_LINES.map((line, i) => {
          const startFrame = f(line.at);
          const op = tween(frame, startFrame, startFrame + 8, 0, 1);
          const text = humanType(line.text, frame, startFrame, fps, line.seed);
          if (frame < startFrame) return null;
          return (
            <div
              key={i}
              style={{
                opacity: op,
                marginBottom: 18,
                color: i === NOTE_LINES.length - 1 ? tokens.fgHi : tokens.fg,
              }}
            >
              {text}
              {text.length < line.text.length && (
                <span
                  style={{
                    display: 'inline-block',
                    width: 12,
                    height: 28,
                    background: tokens.accent,
                    marginLeft: 4,
                    verticalAlign: 'baseline',
                  }}
                />
              )}
            </div>
          );
        })}
      </div>

      <EndCard tagline="Made for people who'd rather speak." startAt={18.0} />
    </SocialFrame>
  );
}
