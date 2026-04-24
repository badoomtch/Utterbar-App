/**
 * feature-vocabulary — 8 s. A single UI moment: the user adds a word
 * to the Custom Words card. No payoff beat, no tagline, no brand reveal
 * — the video is just a piece of the app moving. When it ends the
 * frame sits on the resting state of the card with the new chip in
 * place, so looping reads as "and here's the app with one more word
 * in it" rather than restarting an ad.
 *
 * Background is tokens.bg0 (the site's page canvas); the card surface
 * uses bg1 so the comp's outer edges vanish into the site's video
 * section and only the card reads as "the app".
 *
 * Timeline (30 fps):
 *   0.0–0.5  idle — card at rest with 3 existing chips
 *   0.5–1.1  Windows cursor flies in from off-frame to the compose input
 *   1.1      click — cyan pulse ring expands and fades over 400 ms
 *   1.3–2.8  typewriter reveals "Parakeet" in the input with a caret
 *   2.8–3.2  cursor slides to the Add button
 *   3.2      click — cyan pulse
 *   3.4–4.2  "Parakeet" chip pops in with a strong cyan glow
 *   4.2–5.2  glow decays
 *   5.2–5.8  cursor exits off-frame
 *   5.8–8.0  hold on the resulting state (card with 4 chips, no cursor)
 *
 * Legibility floor for the site's 400 px mobile render: title 72 pt,
 * sub 28 pt, chips 30 pt, input text 36 pt. All readable at 4.8× shrink.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween, typewriter, pulse } from '../util/anim';
import { Cursor, ClickPulse, cursorPath } from '../util/Cursor';

// ---- layout ---------------------------------------------------------------
const CARD_W = 1500;
const CARD_H = 720;
const CARD_X = (1920 - CARD_W) / 2; // 210
const CARD_Y = (1080 - CARD_H) / 2; // 180
const CARD_PAD = 56;

// Compose-row vertical centre — empirically derived from the typography
// above it (72 pt heading + 32 gap + 28 pt sub + 32 gap + half of the
// 104 px tall compose row). If any of those sizes change, update this.
const COMPOSE_CY = CARD_Y + CARD_PAD + 90 + 32 + 40 + 32 + 52;

// Horizontal cursor targets inside the compose row.
const INPUT_CX = 900;
const ADD_BTN_CX = CARD_X + CARD_W - CARD_PAD - 88;

// Cursor entry/exit — just past the bottom-right corner so the arrow
// never clips visibly at the frame edge.
const CURSOR_OFF_X = 1800;
const CURSOR_OFF_Y = 1080;

// Existing vocabulary before the user adds "Parakeet".
const EXISTING_WORDS = ['sherpa-onnx', 'Tauri', 'WASAPI'];

// ---- component ------------------------------------------------------------
export function FeatureVocabulary() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // --- beat timing --------------------------------------------------------
  const cursorInStart = f(0.5);
  const cursorInEnd = f(1.1);
  const inputClickAt = f(1.15);
  const typeStart = f(1.3);
  const typeEnd = f(2.8);
  const cursorToAddStart = f(2.8);
  const cursorToAddEnd = f(3.2);
  const addClickAt = f(3.25);
  const chipLandAt = f(3.4);
  const chipGlowEnd = f(5.2);
  const cursorOutStart = f(5.2);
  const cursorOutEnd = f(5.8);

  // --- cursor path --------------------------------------------------------
  const cursor = cursorPath(frame, [
    { x: CURSOR_OFF_X, y: CURSOR_OFF_Y, t: cursorInStart },
    { x: INPUT_CX, y: COMPOSE_CY, t: cursorInEnd },
    { x: INPUT_CX, y: COMPOSE_CY, t: cursorToAddStart },
    { x: ADD_BTN_CX, y: COMPOSE_CY, t: cursorToAddEnd },
    { x: ADD_BTN_CX, y: COMPOSE_CY, t: cursorOutStart },
    { x: CURSOR_OFF_X, y: CURSOR_OFF_Y, t: cursorOutEnd },
  ]);

  // Cursor visibility — hidden before it enters and after it exits so the
  // resting state frames look like a clean screenshot of the app.
  const cursorVisible = frame >= cursorInStart && frame <= cursorOutEnd + 1;

  // --- typed text + caret -------------------------------------------------
  const typedText = typewriter('Parakeet', frame, typeStart, typeEnd);
  const caret = frame >= typeStart && frame < typeEnd && pulse(frame, 16) > 0.5;
  const inputFocused = frame >= inputClickAt && frame < chipLandAt;

  // --- chip land + glow ---------------------------------------------------
  const chipOpacity = tween(frame, chipLandAt, chipLandAt + 6, 0, 1);
  const chipScale = tween(frame, chipLandAt, chipLandAt + f(0.3), 0.7, 1);
  const chipGlow = tween(frame, chipLandAt, chipGlowEnd, 1, 0);

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* The Custom Words card */}
      <div
        style={{
          position: 'absolute',
          left: CARD_X,
          top: CARD_Y,
          width: CARD_W,
          height: CARD_H,
          background: tokens.bg1,
          borderRadius: 28,
          padding: CARD_PAD,
          border: `1px solid ${tokens.border}`,
          boxSizing: 'border-box',
          display: 'flex',
          flexDirection: 'column',
          gap: 32,
          // Soft drop-shadow lifts the card off bg-0 without painting a
          // visible frame — matches the app's card-on-canvas feel.
          boxShadow: '0 32px 80px rgba(0,0,0,0.5)',
        }}
      >
        {/* Header: title + counter */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline' }}>
          <div
            style={{
              fontSize: 72,
              fontWeight: 700,
              color: tokens.fgHi,
              letterSpacing: '-0.025em',
              lineHeight: 1,
            }}
          >
            Custom words
          </div>
          <div
            style={{
              fontSize: 30,
              fontFamily: fonts.mono,
              color: tokens.muted,
            }}
          >
            {frame >= chipLandAt ? EXISTING_WORDS.length + 1 : EXISTING_WORDS.length} / 50
          </div>
        </div>

        {/* Sub-copy */}
        <div style={{ fontSize: 28, color: tokens.muted, lineHeight: 1.4 }}>
          Brand names, jargon, rare proper nouns — the ASR biases toward these.
        </div>

        {/* Compose row: + icon, input, Add button */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 20,
            padding: '22px 28px',
            background: tokens.bg2,
            borderRadius: 16,
            border: `2px solid ${inputFocused ? 'rgba(255,255,255,0.35)' : tokens.borderStrong}`,
            transition: 'border 120ms linear',
          }}
        >
          <span
            style={{
              fontSize: 36,
              fontFamily: fonts.mono,
              color: tokens.muted,
              lineHeight: 1,
            }}
          >
            +
          </span>
          <span
            style={{
              fontSize: 36,
              fontFamily: fonts.mono,
              color: tokens.fgHi,
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              gap: 2,
            }}
          >
            {!inputFocused && typedText.length === 0 && (
              <span style={{ color: tokens.fgDim }}>Add a word…</span>
            )}
            {(inputFocused || typedText.length > 0) && typedText}
            {caret && <span style={{ color: tokens.fgHi, fontWeight: 700 }}>▍</span>}
          </span>
          <div
            style={{
              padding: '14px 34px',
              background: tokens.bg1,
              color: tokens.fgHi,
              border: `1px solid ${tokens.borderStrong}`,
              borderRadius: 12,
              fontSize: 26,
              fontWeight: 500,
              fontFamily: fonts.sans,
              // Faint cyan outline while the cursor is targeting Add —
              // tells the viewer "this is about to be clicked" without a
              // full highlight. Gone by the time the chip lands.
              boxShadow:
                frame >= cursorToAddEnd - 4 && frame < addClickAt + 6
                  ? `0 0 0 2px rgba(34, 211, 238, 0.55)`
                  : 'none',
            }}
          >
            Add
          </div>
        </div>

        {/* Chips grid */}
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 14 }}>
          {EXISTING_WORDS.map((w) => (
            <div
              key={w}
              style={{
                padding: '14px 28px',
                background: tokens.bg2,
                borderRadius: 999,
                fontSize: 30,
                fontFamily: fonts.mono,
                color: tokens.fgHi,
              }}
            >
              {w}
            </div>
          ))}
          {frame >= chipLandAt && (
            <div
              style={{
                padding: '14px 28px',
                background: tokens.bg2,
                borderRadius: 999,
                fontSize: 30,
                fontFamily: fonts.mono,
                color: tokens.fgHi,
                opacity: chipOpacity,
                transform: `scale(${chipScale})`,
                transformOrigin: 'left center',
                boxShadow:
                  chipGlow > 0
                    ? `0 0 ${70 * chipGlow}px rgba(34, 211, 238, ${0.95 * chipGlow}), ` +
                      `0 0 ${28 * chipGlow}px rgba(34, 211, 238, ${0.6 * chipGlow})`
                    : 'none',
                border: `2px solid rgba(34, 211, 238, ${chipGlow})`,
              }}
            >
              Parakeet
            </div>
          )}
        </div>
      </div>

      {/* Click pulses — sit under the cursor so the ring expands from
          beneath the arrow tip. */}
      <ClickPulse frame={frame} startFrame={inputClickAt} x={INPUT_CX} y={COMPOSE_CY} />
      <ClickPulse frame={frame} startFrame={addClickAt} x={ADD_BTN_CX} y={COMPOSE_CY} />

      {/* Cursor — only visible while it's on-screen. */}
      {cursorVisible && <Cursor x={cursor.x} y={cursor.y} />}
    </AbsoluteFill>
  );
}
