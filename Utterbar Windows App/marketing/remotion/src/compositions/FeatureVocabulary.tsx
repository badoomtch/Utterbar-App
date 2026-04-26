/**
 * feature-vocabulary — 7 s. The vocabulary list populates with four
 * custom words, then a sentence types itself out with the replacements
 * hitting live so the viewer can watch "J parrie" turn into "Jay Parry"
 * as the ASR output gets post-processed.
 *
 * Regenerated 2026-04-24 — dropped the outer card and a fictional
 * "replacements panel", both of which made the composition read as a
 * diagram rather than the real feature. Now it's two things: the list
 * (real UI) and one sentence that shows the list working.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween, typewriter } from '../util/anim';

const WORDS = [
  { word: 'Utterbar', hint: 'product name' },
  { word: 'Parakeet TDT', hint: 'speech model' },
  { word: 'Jay Parry', hint: 'your name' },
  { word: 'Cloudways', hint: 'host' },
];

const REPLACEMENTS = [
  { from: 'J parrie', to: 'Jay Parry' },
  { from: 'utter bar', to: 'Utterbar' },
];

// The before/after sentence — same shape, typed twice; on the second pass
// the replacements fire and the sentence reads correctly.
const BEFORE =
  "Hi J parrie — the utter bar demo is ready on Cloudways.";
const AFTER =
  "Hi Jay Parry — the Utterbar demo is ready on Cloudways.";

export function FeatureVocabulary() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Word cards stagger in 0.5 -> 1.9 s (4 × 0.35 s).
  const wordEnter = WORDS.map((_, i) => f(0.5 + i * 0.35));

  // Before sentence types 2.5 -> 3.8, replacement swap fires at 4.6 (with a
  // brief cross-tween), after sentence types 4.7 -> 5.6, hold on resting
  // state 5.6 -> 7.0.
  const beforeTyped = typewriter(BEFORE, frame, f(2.5), f(3.8));
  const swap = tween(frame, f(4.4), f(4.8), 0, 1);
  const afterTyped = typewriter(AFTER, frame, f(4.7), f(5.6));

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Left column — vocabulary list. Real styling: tone rows, mono chip
          for the hint. */}
      <div
        style={{
          position: 'absolute',
          left: 100,
          top: 80,
          width: 560,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 18,
          }}
        >
          Custom vocabulary
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 18,
            border: `1px solid ${tokens.border}`,
            overflow: 'hidden',
          }}
        >
          {WORDS.map((w, i) => {
            const opacity = tween(frame, wordEnter[i] - 5, wordEnter[i] + 8, 0, 1);
            const x = tween(frame, wordEnter[i] - 5, wordEnter[i] + 12, -12, 0);
            return (
              <div
                key={w.word}
                style={{
                  padding: '18px 22px',
                  borderTop: i === 0 ? 'none' : `1px solid ${tokens.border}`,
                  display: 'flex',
                  alignItems: 'baseline',
                  justifyContent: 'space-between',
                  opacity,
                  transform: `translateX(${x}px)`,
                }}
              >
                <span style={{ fontSize: 22, color: tokens.fgHi, fontWeight: 600 }}>
                  {w.word}
                </span>
                <span
                  style={{
                    fontFamily: fonts.mono,
                    fontSize: 17,
                    color: tokens.muted,
                    textTransform: 'uppercase',
                    letterSpacing: '0.12em',
                  }}
                >
                  {w.hint}
                </span>
              </div>
            );
          })}
        </div>

        {/* Replacements list below. */}
        <div
          style={{
            marginTop: 24,
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 14,
          }}
        >
          Replacements
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 18,
            border: `1px solid ${tokens.border}`,
            padding: '14px 22px',
          }}
        >
          {REPLACEMENTS.map((r, i) => {
            const enter = f(1.9 + i * 0.25);
            const opacity = tween(frame, enter - 5, enter + 8, 0, 1);
            return (
              <div
                key={r.from}
                style={{
                  padding: '10px 0',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 14,
                  opacity,
                }}
              >
                <span
                  style={{
                    fontFamily: fonts.mono,
                    fontSize: 16,
                    color: tokens.fg,
                    background: tokens.bg2,
                    padding: '4px 10px',
                    borderRadius: 6,
                  }}
                >
                  {r.from}
                </span>
                <span style={{ color: tokens.muted, fontSize: 16 }}>→</span>
                <span
                  style={{
                    fontFamily: fonts.mono,
                    fontSize: 16,
                    color: tokens.fgHi,
                    background: tokens.bg2,
                    padding: '4px 10px',
                    borderRadius: 6,
                  }}
                >
                  {r.to}
                </span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Right column — the sentence. Before types, fades, After types in. */}
      <div
        style={{
          position: 'absolute',
          right: 100,
          top: 160,
          width: 640,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 18,
          }}
        >
          What you dictated
        </div>

        <div
          style={{
            background: tokens.bg1,
            borderRadius: 20,
            padding: '32px 32px',
            border: `1px solid ${tokens.border}`,
            fontSize: 28,
            lineHeight: 1.55,
            color: tokens.fg,
            fontStyle: 'italic',
            position: 'relative',
            minHeight: 340,
          }}
        >
          <div style={{ opacity: 1 - swap }}>
            &ldquo;{beforeTyped}&rdquo;
          </div>
          <div
            style={{
              position: 'absolute',
              top: 32,
              left: 32,
              right: 32,
              opacity: swap,
            }}
          >
            &ldquo;{afterTyped}&rdquo;
          </div>
        </div>

        <div
          style={{
            marginTop: 20,
            fontFamily: fonts.mono,
            fontSize: 17,
            letterSpacing: '0.14em',
            color: tokens.muted,
            textTransform: 'uppercase',
          }}
        >
          {frame < f(4.5) ? 'Before your list kicks in' : 'After your list fires'}
        </div>
      </div>
    </AbsoluteFill>
  );
}
