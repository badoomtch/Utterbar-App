/**
 * Vocabulary chip — used by Ad13TheProduct beat 4.
 *
 * Compact chip card showing one entry being added to vocabulary. The
 * shipping app's Vocabulary route is dense (multiple cards, hint columns,
 * weight sliders, replacements section) — too much to read in a 4-second
 * vertical beat. This is the "what the user sees in their head" version:
 * a single chip animating from `wrongWord` (struck) to `targetWord` with
 * a green check.
 *
 * Animation timeline (relative to this component being mounted, not
 * relative to the global frame — caller controls visibility window):
 *   0 –  6   chip fades up
 *   6 – 16   wrongWord struck through with a thin line
 *  16 – 30   targetWord and check slide in from the right
 *  30+      held
 *
 * Caller passes `localFrame` (frame since the chip became visible). This
 * keeps the component itself frame-agnostic which makes it easier to
 * compose into different ads.
 */
import React from 'react';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';

type Props = {
  /** Frame relative to when the chip first appears. Caller computes
   *  this as `useCurrentFrame() - chipStartFrame`. */
  localFrame: number;
  /** What the recogniser heard (and got wrong). */
  wrongWord: string;
  /** What the user wants (the vocabulary entry). */
  targetWord: string;
};

export function VocabularyChip({ localFrame, wrongWord, targetWord }: Props) {
  if (localFrame < 0) return null;

  const chipFade = tween(localFrame, 0, 6, 0, 1);
  const chipSlide = tween(localFrame, 0, 8, 12, 0);

  // Strike-through line on wrongWord grows from left to right between
  // frames 6 and 16. Implemented as a sibling div whose width tweens —
  // easier to control than text-decoration animation timing.
  const strikeWidth = tween(localFrame, 6, 16, 0, 100); // %

  // Target word + check slide in from the right between 16 and 30.
  const targetFade = tween(localFrame, 16, 28, 0, 1);
  const targetSlide = tween(localFrame, 16, 30, 24, 0);

  return (
    <div
      style={{
        opacity: chipFade,
        transform: `translateY(${chipSlide}px)`,
        background: tokens.bg1,
        border: `1px solid ${tokens.border}`,
        borderLeft: `3px solid ${tokens.accent}`,
        borderRadius: 22,
        padding: '32px 44px',
        fontFamily: fonts.sans,
        display: 'flex',
        alignItems: 'center',
        gap: 36,
        boxShadow: '0 24px 80px rgba(0,0,0,0.45)',
      }}
    >
      <div
        style={{
          color: tokens.muted,
          fontSize: 22,
          letterSpacing: '0.16em',
          textTransform: 'uppercase',
          fontWeight: 500,
          minWidth: 220,
        }}
      >
        Vocabulary
      </div>

      {/* Wrong word with growing strike-through */}
      <div style={{ position: 'relative', display: 'inline-block' }}>
        <div
          style={{
            color: tokens.muted,
            fontSize: 56,
            fontWeight: 500,
            letterSpacing: '-0.01em',
            lineHeight: 1,
            whiteSpace: 'nowrap',
          }}
        >
          {wrongWord}
        </div>
        <div
          style={{
            position: 'absolute',
            left: 0,
            top: '52%',
            height: 4,
            width: `${strikeWidth}%`,
            background: tokens.rose,
            borderRadius: 2,
            opacity: localFrame >= 6 ? 1 : 0,
          }}
        />
      </div>

      {/* Arrow + target word + check */}
      <div
        style={{
          opacity: targetFade,
          transform: `translateX(${-targetSlide}px)`,
          display: 'flex',
          alignItems: 'center',
          gap: 28,
        }}
      >
        <div
          style={{
            color: tokens.fgDim,
            fontSize: 38,
            fontFamily: fonts.mono,
            fontWeight: 500,
          }}
        >
          →
        </div>
        <div
          style={{
            color: tokens.fgHi,
            fontSize: 56,
            fontWeight: 600,
            letterSpacing: '-0.015em',
            lineHeight: 1,
            whiteSpace: 'nowrap',
          }}
        >
          {targetWord}
        </div>
        {/* Green check tick */}
        <svg width={42} height={42} viewBox="0 0 42 42" fill="none">
          <circle cx={21} cy={21} r={20} stroke={tokens.moss} strokeWidth={2.5} fill="none" opacity={0.6} />
          <path
            d="M 12 22 L 19 28 L 30 16"
            stroke={tokens.moss}
            strokeWidth={3.5}
            strokeLinecap="round"
            strokeLinejoin="round"
            fill="none"
            style={{
              filter: `drop-shadow(0 0 8px ${tokens.moss}88)`,
            }}
          />
        </svg>
      </div>
    </div>
  );
}
