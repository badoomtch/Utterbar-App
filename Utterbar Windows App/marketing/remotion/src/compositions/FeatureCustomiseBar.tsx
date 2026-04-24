/**
 * feature-customise-bar — 8 s. The Settings → Appearance panel. Three
 * theme tiles on the left; a live floating-bar preview on the right.
 * A Windows cursor flies in, clicks each tile in turn, the preview
 * flips theme + accent each click. Ends on the Paper / Violet state
 * with the cursor off-frame — resting on the result.
 *
 * This is the canonical UI-in-motion take: we're literally showing the
 * Appearance panel being operated. No tagline, no eyebrow, no beat
 * label. Background is tokens.bg0 end-to-end so the comp blends into
 * the site section.
 *
 * Timeline (30 fps):
 *   0.0–0.5  idle — Obsidian tile already selected, pill in Obsidian
 *   0.5–1.2  cursor flies in from off-frame
 *   1.4      click on Graphite tile
 *   1.5      pill re-tones → Graphite / Amber
 *   3.0      cursor slides to Paper tile
 *   3.4      click
 *   3.5      pill re-tones → Paper / Violet (the money flip)
 *   5.5      cursor slides back to Obsidian
 *   5.9      click
 *   6.0      pill re-tones → Obsidian / Cyan (loop-safe resting state)
 *   6.3–6.9  cursor exits off-frame
 *   6.9–8.0  hold on Obsidian resting state
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import type { Theme } from '@/generated/Theme';
import type { Accent } from '@/generated/Accent';
import { Cursor, ClickPulse, cursorPath } from '../util/Cursor';

// ---- themes ---------------------------------------------------------------
//
// `theme` / `accent` are the canonical ids BarDriver passes straight to
// `applyThemeClasses` — they toggle `.theme-*` / `.accent-*` on <html> and
// the real FloatingBar's CSS var cascade resolves to the right palette.
// Everything else (pill body stops, accent hex/hex2) exists only for the
// swatch tiles on the left; they're illustrative, not driving the pill.
const THEMES = [
  {
    theme: 'obsidian' as Theme,
    accent: 'cyan' as Accent,
    name: 'Obsidian',
    accentName: 'Cyan',
    pillTop: '#0B0B10',
    pillBot: '#000000',
    accentHex: '#22D3EE',
  },
  {
    theme: 'graphite' as Theme,
    accent: 'amber' as Accent,
    name: 'Graphite',
    accentName: 'Amber',
    pillTop: '#2F3A52',
    pillBot: '#181F2E',
    accentHex: '#F59E0B',
  },
  {
    theme: 'paper' as Theme,
    accent: 'violet' as Accent,
    name: 'Paper',
    accentName: 'Violet',
    pillTop: '#FFFFFF',
    pillBot: '#EDEAE1',
    accentHex: '#8B5CF6',
  },
] as const;

// ---- layout ---------------------------------------------------------------
const CARD_W = 1640;
const CARD_H = 820;
const CARD_X = (1920 - CARD_W) / 2; // 140
const CARD_Y = (1080 - CARD_H) / 2; // 130
const CARD_PAD = 56;

// Tile column left-edge + widths.
const TILE_COL_W = 620;
const TILE_H = 168;
const TILE_GAP = 20;
const TILE_X = CARD_X + CARD_PAD;
const TILES_TOP = CARD_Y + CARD_PAD + 84; // after "Appearance" section header

// Per-tile click targets (centre of tile).
const tileCy = (i: number) => TILES_TOP + i * (TILE_H + TILE_GAP) + TILE_H / 2;
const TILE_CX = TILE_X + TILE_COL_W / 2;

// Off-frame cursor anchor.
const CURSOR_OFF_X = 1820;
const CURSOR_OFF_Y = 1080;

export function FeatureCustomiseBar() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // --- timing --------------------------------------------------------------
  const cursorInStart = f(0.5);
  const cursorInEnd = f(1.2);

  const clickGraphiteAt = f(1.4);
  const slideToPaperStart = f(3.0);
  const slideToPaperEnd = f(3.4);
  const clickPaperAt = f(3.5);
  const slideToObsidianStart = f(5.5);
  const slideToObsidianEnd = f(5.9);
  const clickObsidianAt = f(6.0);
  const cursorOutStart = f(6.3);
  const cursorOutEnd = f(6.9);

  // Selected theme index at any point in time.
  const selectedIdx =
    frame < clickGraphiteAt
      ? 0
      : frame < clickPaperAt
        ? 1
        : frame < clickObsidianAt
          ? 2
          : 0;
  const selected = THEMES[selectedIdx];

  // --- cursor path ---------------------------------------------------------
  const cursor = cursorPath(frame, [
    { x: CURSOR_OFF_X, y: CURSOR_OFF_Y, t: cursorInStart },
    { x: TILE_CX, y: tileCy(1), t: cursorInEnd },
    { x: TILE_CX, y: tileCy(1), t: slideToPaperStart },
    { x: TILE_CX, y: tileCy(2), t: slideToPaperEnd },
    { x: TILE_CX, y: tileCy(2), t: slideToObsidianStart },
    { x: TILE_CX, y: tileCy(0), t: slideToObsidianEnd },
    { x: TILE_CX, y: tileCy(0), t: cursorOutStart },
    { x: CURSOR_OFF_X, y: CURSOR_OFF_Y, t: cursorOutEnd },
  ]);
  const cursorVisible = frame >= cursorInStart && frame <= cursorOutEnd + 1;

  // --- preview wash flips with selected accent -----------------------------
  const washAccent = selected.accentHex;

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Ambient accent wash — lifts the card off bg-0. */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: '50%',
          transform: 'translate(-50%, -50%)',
          width: 1400,
          height: 500,
          background: `radial-gradient(ellipse at center, ${washAccent}22 0%, transparent 65%)`,
          filter: 'blur(90px)',
          transition: 'background 200ms linear',
        }}
      />

      {/* Settings card */}
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
          gap: 48,
          boxShadow: '0 32px 80px rgba(0,0,0,0.5)',
        }}
      >
        {/* Left column — section header + theme tiles */}
        <div style={{ width: TILE_COL_W, display: 'flex', flexDirection: 'column' }}>
          <div
            style={{
              fontSize: 56,
              fontWeight: 700,
              color: tokens.fgHi,
              letterSpacing: '-0.025em',
              lineHeight: 1,
              marginBottom: 12,
            }}
          >
            Appearance
          </div>
          <div
            style={{
              fontSize: 24,
              color: tokens.muted,
              marginBottom: 28,
            }}
          >
            Theme & accent
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: TILE_GAP }}>
            {THEMES.map((t, i) => {
              const isSelected = i === selectedIdx;
              return (
                <div
                  key={t.theme}
                  style={{
                    height: TILE_H,
                    background: tokens.bg2,
                    borderRadius: 18,
                    padding: 20,
                    display: 'flex',
                    alignItems: 'center',
                    gap: 20,
                    border: `2px solid ${isSelected ? t.accentHex : tokens.border}`,
                    boxShadow: isSelected ? `0 0 0 2px ${t.accentHex}33` : 'none',
                    transition: 'border 120ms linear',
                  }}
                >
                  {/* Swatch — mini representation of the theme */}
                  <div
                    style={{
                      width: 120,
                      height: 120,
                      borderRadius: 14,
                      background: `linear-gradient(180deg, ${t.pillTop}, ${t.pillBot})`,
                      border: `1px solid ${tokens.border}`,
                      flexShrink: 0,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                    }}
                  >
                    <div
                      style={{
                        width: 24,
                        height: 24,
                        borderRadius: 999,
                        background: t.accentHex,
                        boxShadow: `0 0 0 3px ${t.accentHex}33`,
                      }}
                    />
                  </div>
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div
                      style={{
                        fontSize: 32,
                        fontWeight: 700,
                        color: tokens.fgHi,
                        letterSpacing: '-0.01em',
                      }}
                    >
                      {t.name}
                    </div>
                    <div
                      style={{
                        fontFamily: fonts.mono,
                        fontSize: 20,
                        color: tokens.muted,
                        marginTop: 6,
                      }}
                    >
                      Accent · {t.accentName}
                    </div>
                  </div>
                  {/* Radio dot */}
                  <div
                    style={{
                      width: 28,
                      height: 28,
                      borderRadius: 999,
                      border: `2px solid ${isSelected ? t.accentHex : tokens.borderStrong}`,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      flexShrink: 0,
                    }}
                  >
                    {isSelected && (
                      <div
                        style={{
                          width: 14,
                          height: 14,
                          borderRadius: 999,
                          background: t.accentHex,
                        }}
                      />
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Right column — live preview */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 20,
              letterSpacing: '0.14em',
              textTransform: 'uppercase',
              color: tokens.muted,
              marginBottom: 28,
            }}
          >
            Preview
          </div>
          {/* Preview stage — switches tone so the Paper pill has its light
              substrate to sit on; Obsidian + Graphite share the dark stage
              so the dark-theme comparison reads against one backdrop. */}
          <div
            style={{
              flex: 1,
              borderRadius: 22,
              background:
                selected.theme === 'paper'
                  ? '#F6F4EE'
                  : selected.theme === 'graphite'
                    ? '#12151C'
                    : '#0B0E15',
              border: `1px solid ${tokens.border}`,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              position: 'relative',
              overflow: 'hidden',
            }}
          >
            {/* Wash inside the preview frame */}
            <div
              style={{
                position: 'absolute',
                left: '50%',
                top: '50%',
                transform: 'translate(-50%, -50%)',
                width: 600,
                height: 260,
                background: `radial-gradient(ellipse at center, ${selected.accentHex}33 0%, transparent 65%)`,
                filter: 'blur(50px)',
              }}
            />
            <ThemedFloatingBar theme={selected} />
          </div>
        </div>
      </div>

      {/* Click pulses — cyan stays the same colour regardless of theme
          because it's a UI click affordance, not a themed element. */}
      <ClickPulse frame={frame} startFrame={clickGraphiteAt} x={TILE_CX} y={tileCy(1)} />
      <ClickPulse frame={frame} startFrame={clickPaperAt} x={TILE_CX} y={tileCy(2)} />
      <ClickPulse frame={frame} startFrame={clickObsidianAt} x={TILE_CX} y={tileCy(0)} />

      {cursorVisible && <Cursor x={cursor.x} y={cursor.y} />}
    </AbsoluteFill>
  );
}

// Preview pill — renders the shipping FloatingBar driven through BarDriver
// with the selected {theme, accent} pair. BarDriver flips the `.theme-*` /
// `.accent-*` classes on <html> so the real pill's CSS var cascade resolves
// to the themed palette. A 1.6× scale matches the surrounding card layout;
// the pill's intrinsic geometry comes from the shipping component.
function ThemedFloatingBar({
  theme,
}: {
  theme: (typeof THEMES)[number];
}) {
  return (
    <div
      style={{
        transform: 'scale(1.6)',
        transformOrigin: 'center',
        position: 'relative',
        zIndex: 1,
      }}
    >
      <BarDriver
        state="listening"
        elapsedMs={3400}
        theme={theme.theme}
        accent={theme.accent}
      />
    </div>
  );
}
