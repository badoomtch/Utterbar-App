/**
 * ad-12-founders-note — 22s. v3: each line still typewrites (this
 * is the founder note format — the typewriter IS the medium), but
 * the last line holds 1.5s before end card.
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
  { text: "I built this because I got tired of typing.",        at: 0.6,  seed: 0x10 },
  { text: "Ctrl + Space.",                                       at: 4.6,  seed: 0x20 },
  { text: "Speak.",                                              at: 6.0,  seed: 0x30 },
  { text: "The words appear in any app.",                        at: 7.4,  seed: 0x40 },
  { text: "No cloud. No account. No subscription.",              at: 10.6, seed: 0x60 },
  { text: "£39.99. Once. For Windows.",                          at: 14.4, seed: 0x70 },
];

export function Ad12FoundersNote() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const pillElapsed = (frame / fps) * 1000;

  return (
    <SocialFrame variant="canvas">
      <div style={{ position: 'absolute', top: 160, left: '50%', transform: 'translateX(-50%) scale(1.6)', transformOrigin: 'top center', opacity: 0.92 }}>
        <BarDriver state="listening" elapsedMs={pillElapsed} />
      </div>

      <div
        style={{
          position: 'absolute',
          top: 540,
          left: 0,
          right: 0,
          padding: '0 80px',
          fontFamily: fonts.mono,
          fontSize: 54,
          color: tokens.fg,
          lineHeight: 1.5,
          letterSpacing: '-0.01em',
          fontWeight: 500,
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
                marginBottom: 26,
                color: i === NOTE_LINES.length - 1 ? tokens.fgHi : tokens.fg,
              }}
            >
              {text}
              {text.length < line.text.length && (
                <span
                  style={{
                    display: 'inline-block',
                    width: 14,
                    height: 42,
                    background: tokens.accent,
                    marginLeft: 6,
                    verticalAlign: 'baseline',
                  }}
                />
              )}
            </div>
          );
        })}
      </div>

      <EndCard tagline="Made for people who'd rather speak." startAt={17.5} />
    </SocialFrame>
  );
}
