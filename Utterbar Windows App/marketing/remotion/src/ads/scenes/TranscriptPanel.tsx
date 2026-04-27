/**
 * Transcript panel — used by Ad13TheProduct beats 2 and 3.
 *
 * Two modes:
 *   - mode="raw"     : shows the live transcript building character-by-character
 *                      via humanType. A blinking caret sits at the cursor.
 *   - mode="shaped"  : shows the same content reformatted by Message mode —
 *                      cleaned punctuation, line breaks, the kind of output
 *                      the post-processing layer produces in the shipping app.
 *
 * Visually it's a clean card: bg-1 plate, border hairline, mono caption
 * row up top ("Voice to Text" / "Message mode"), body text in the sans
 * stack at a generous 44px so it reads at thumb-distance. The caret
 * uses --accent (cyan) and blinks on a 30-frame cycle.
 *
 * Honest illustration, not the literal History route — same convention as
 * SlackVerticalThread. The genuine pill above carries the "this is the
 * real product" weight; this panel carries "this is what real output
 * looks like".
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { humanType } from '../frame/humanType';

type Props = {
  /** Visual treatment of the panel content. */
  mode: 'raw' | 'shaped';
  /** Full target text. In 'raw' mode the panel reveals this character-by-character.
   *  In 'shaped' mode the full text appears at once. */
  text: string;
  /** Frame at which typing begins (raw mode) or the panel materialises (shaped). */
  startFrame: number;
  /** Optional override seed for humanType so two transcripts in the same
   *  composition don't share an identical jitter pattern. */
  seed?: number;
  /** Width override; defaults to 980 (matches NetworkMonitor's stage width). */
  width?: number;
};

export function TranscriptPanel({ mode, text, startFrame, seed = 0xc0de, width = 980 }: Props) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const visible = mode === 'raw' ? humanType(text, frame, startFrame, fps, seed) : text;
  const showCaret = mode === 'raw' && frame >= startFrame;
  // Caret blinks on a 28-frame cycle (~0.93s). Slow enough to read as a
  // text caret rather than a flicker.
  const caretOn = Math.floor((frame - startFrame) / 14) % 2 === 0;

  const eyebrow = mode === 'raw' ? 'Voice to Text' : 'Message mode';

  return (
    <div
      style={{
        width,
        background: tokens.bg1,
        border: `1px solid ${tokens.border}`,
        borderRadius: 24,
        padding: '36px 44px 40px',
        boxShadow: '0 24px 80px rgba(0,0,0,0.45)',
        fontFamily: fonts.sans,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: 28,
        }}
      >
        <div
          style={{
            color: tokens.muted,
            fontSize: 22,
            letterSpacing: '0.16em',
            textTransform: 'uppercase',
            fontWeight: 500,
          }}
        >
          {eyebrow}
        </div>
        {mode === 'shaped' && (
          <div
            style={{
              color: tokens.accent,
              fontSize: 22,
              letterSpacing: '0.04em',
              fontFamily: fonts.mono,
              fontWeight: 500,
            }}
          >
            shaped ·
          </div>
        )}
      </div>

      <div
        style={{
          color: tokens.fg,
          fontSize: 44,
          lineHeight: 1.4,
          letterSpacing: '-0.01em',
          fontWeight: 400,
          minHeight: 220,
          whiteSpace: 'pre-wrap',
        }}
      >
        {visible}
        {showCaret && (
          <span
            style={{
              display: 'inline-block',
              width: 4,
              height: 44,
              marginLeft: 6,
              verticalAlign: '-7px',
              background: tokens.accent,
              boxShadow: `0 0 14px ${tokens.accent}`,
              opacity: caretOn ? 1 : 0,
            }}
          />
        )}
      </div>
    </div>
  );
}
