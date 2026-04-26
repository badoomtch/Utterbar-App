/**
 * feature-works-anywhere — 8 s. The pill sits at the top of the frame,
 * a generic text field with a blinking cursor sits below.
 *
 * The flow narrates the real app: hold the hotkey, talk while the pill
 * records (field stays empty, that's the speaking part), processing
 * lights up briefly, then the text pastes in fast — like a real paste,
 * not a typewriter. The pill returns to idle and the field's accent
 * glow fades.
 *
 * Timeline:
 *   0.0 – 0.7   idle        empty field, caret blinks
 *   0.7 – 4.5   listening   "Recording" chip, timer ticks 0:00 → 0:03
 *   4.5 – 5.5   processing  "Processing" chip, field still empty
 *   5.5 – 6.0   processing  text pastes in over 0.5 s
 *   6.0 – 8.0   idle        text stays, accent glow fades
 *
 * Skips the `success` phase deliberately — see HeroPillLoop comment.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import { tween, typewriter } from '../util/anim';

const SENTENCE = 'The typed text just shows up — in any Windows app, wherever the cursor is.';

export function FeatureWorksAnywhere() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  let state: 'idle' | 'listening' | 'processing' = 'idle';
  if (frame < f(0.7)) state = 'idle';
  else if (frame < f(4.5)) state = 'listening';
  else if (frame < f(6.0)) state = 'processing';
  else state = 'idle';

  const elapsed = state === 'listening' ? Math.max(0, (frame - f(0.7)) / fps) : 0;

  // Text pastes in fast at the end of processing — the moment the real
  // app would inject the transcript. Half-second window so it reads as
  // "boom, it's there", not as a typewriter.
  const typed = typewriter(SENTENCE, frame, f(5.5), f(6.0));

  // Field accent glow rises with the paste, then fades while the bar
  // returns to idle. Peaks at the moment the text lands.
  const successGlow = tween(frame, f(5.5), f(6.0), 0, 1) * tween(frame, f(6.2), f(7.4), 1, 0);

  // Caret blink — visible until the paste fires.
  const caretBlink = Math.floor(frame / 12) % 2;
  const showCaret = frame < f(5.5);

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Pill at the top — the floating utility is always visible. */}
      <div
        style={{
          position: 'absolute',
          top: 90,
          left: '50%',
          transform: 'translateX(-50%)',
        }}
      >
        <div style={{ transform: 'scale(1.85)', transformOrigin: 'top center' }}>
          <BarDriver state={state} elapsedMs={elapsed * 1000} />
        </div>
      </div>

      {/* Generic text field — sits in the centre-bottom of the frame. */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 480,
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
          }}
        >
          Any Windows app — wherever the cursor is
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 16,
            padding: '32px 36px',
            border: `1px solid ${
              successGlow > 0.05
                ? `rgba(34, 211, 238, ${0.15 + successGlow * 0.55})`
                : tokens.border
            }`,
            minHeight: 220,
            fontSize: 28,
            lineHeight: 1.55,
            color: tokens.fg,
            position: 'relative',
            boxShadow: successGlow > 0.05
              ? `0 0 0 ${6 * successGlow}px rgba(34, 211, 238, ${0.1 * successGlow})`
              : 'none',
          }}
        >
          {typed}
          {showCaret && (
            <span
              style={{
                display: 'inline-block',
                width: 3,
                height: 30,
                background: tokens.accent,
                marginLeft: 2,
                verticalAlign: 'middle',
                transform: 'translateY(-2px)',
                opacity: caretBlink,
              }}
            />
          )}
        </div>
      </div>
    </AbsoluteFill>
  );
}
