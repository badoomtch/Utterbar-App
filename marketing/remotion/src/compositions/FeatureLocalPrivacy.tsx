/**
 * feature-local-privacy — 8 s. Zoomed onto the Settings → Privacy
 * panel. Three cloud-flavoured toggles are all off (that's the
 * feature). As the card settles, each row gets a small moss-green
 * "✓ Local only" badge that fades in one after another, and a status
 * strip at the bottom shows "0 B sent to the cloud" ticking nothing
 * while the pill idles. The loop ends on the fully-confirmed state.
 *
 * No cursor — the interaction the viewer needs to believe in is that
 * nothing is _happening_. A cursor clicking toggles would actively
 * muddy that. Instead the motion is a calm verification sweep.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween } from '../util/anim';

// ---- layout ---------------------------------------------------------------
const CARD_W = 1500;
const CARD_H = 880;
const CARD_X = (1920 - CARD_W) / 2; // 210
const CARD_Y = (1080 - CARD_H) / 2; // 100
const CARD_PAD = 56;

// Privacy toggles we're showing. Matches the real Settings → Privacy
// panel's copy (short enough for the 400 px mobile render to still parse).
const ROWS = [
  {
    icon: 'cloud',
    title: 'Cloud transcription',
    sub: 'Send audio to a cloud ASR as a fallback.',
  },
  {
    icon: 'sparkles',
    title: 'Cloud post-processing',
    sub: 'Route transcripts through a cloud LLM for polish.',
  },
  {
    icon: 'activity',
    title: 'Usage telemetry',
    sub: 'Share anonymous usage metrics with the developer.',
  },
] as const;

export function FeatureLocalPrivacy() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Per-row confirmation badge timings — staggered 0.7 s apart so the
  // sweep is obvious without being impatient.
  const badgeAt = [f(1.4), f(2.1), f(2.8)];

  // Status strip reveal — lands at 4.0 s, after the three rows have all
  // confirmed. Sits on the rest of the composition as the final state.
  const statusIn = tween(frame, f(4.0), f(4.6), 0, 1);

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
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
          boxShadow: '0 32px 80px rgba(0,0,0,0.5)',
        }}
      >
        {/* Header */}
        <div>
          <div
            style={{
              fontSize: 72,
              fontWeight: 700,
              color: tokens.fgHi,
              letterSpacing: '-0.025em',
              lineHeight: 1,
            }}
          >
            Privacy
          </div>
          <div
            style={{
              fontSize: 28,
              color: tokens.muted,
              marginTop: 18,
              lineHeight: 1.35,
            }}
          >
            Everything stays on this PC by default.
          </div>
        </div>

        {/* Toggle rows */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 20, flex: 1 }}>
          {ROWS.map((row, i) => {
            const badgeT = tween(frame, badgeAt[i], badgeAt[i] + 10, 0, 1);
            return (
              <div
                key={row.title}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 24,
                  background: tokens.bg2,
                  borderRadius: 18,
                  padding: '22px 28px',
                  border: `1px solid ${tokens.border}`,
                }}
              >
                {/* Icon — neutral bg, muted glyph so nothing cyan competes
                    with the moss confirmation badge. */}
                <div
                  style={{
                    width: 62,
                    height: 62,
                    borderRadius: 14,
                    background: tokens.bg1,
                    border: `1px solid ${tokens.border}`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                  }}
                >
                  <Icon name={row.icon} />
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      fontSize: 30,
                      fontWeight: 600,
                      color: tokens.fgHi,
                      letterSpacing: '-0.01em',
                    }}
                  >
                    {row.title}
                  </div>
                  <div
                    style={{
                      fontSize: 22,
                      color: tokens.muted,
                      marginTop: 4,
                    }}
                  >
                    {row.sub}
                  </div>
                </div>
                {/* "Local only" confirmation badge — fades in per-row,
                    moss ✓ on the card's bg-1 so it reads against the
                    bg-2 row. */}
                <div
                  style={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: 8,
                    padding: '8px 14px',
                    background: 'rgba(52, 211, 153, 0.12)',
                    border: `1px solid rgba(52, 211, 153, ${0.4 * badgeT})`,
                    borderRadius: 999,
                    opacity: badgeT,
                    transform: `translateY(${(1 - badgeT) * 6}px)`,
                    flexShrink: 0,
                  }}
                >
                  <svg
                    width={20}
                    height={20}
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke={tokens.moss}
                    strokeWidth="3"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <path d="M5 12l4 4L19 6" />
                  </svg>
                  <span
                    style={{
                      fontSize: 20,
                      fontWeight: 500,
                      color: tokens.moss,
                      fontFamily: fonts.mono,
                      letterSpacing: '0.04em',
                    }}
                  >
                    Local only
                  </span>
                </div>
                {/* Switch — off. Paired to the right of the badge to
                    anchor the row's right edge. */}
                <Switch on={false} />
              </div>
            );
          })}
        </div>

        {/* Status strip — bottom of the card. "0 B sent to the cloud"
            as the final sit-on state. The pill-like row style mirrors
            the rest of the app's status strips. */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '20px 28px',
            background: tokens.bg2,
            borderRadius: 16,
            border: `1px solid ${tokens.border}`,
            opacity: statusIn,
            transform: `translateY(${(1 - statusIn) * 8}px)`,
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
            <div
              style={{
                width: 12,
                height: 12,
                borderRadius: 999,
                background: tokens.moss,
                boxShadow: `0 0 0 4px rgba(52, 211, 153, 0.22)`,
              }}
            />
            <div style={{ fontSize: 26, color: tokens.fgHi, fontWeight: 500 }}>
              On-device · Parakeet ASR
            </div>
          </div>
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 24,
              color: tokens.muted,
              fontVariantNumeric: 'tabular-nums',
            }}
          >
            0 B sent to the cloud
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
}

function Switch({ on }: { on: boolean }) {
  return (
    <div
      style={{
        width: 72,
        height: 40,
        borderRadius: 999,
        background: on ? tokens.accent : tokens.bg1,
        border: `2px solid ${on ? tokens.accent : tokens.borderStrong}`,
        position: 'relative',
        flexShrink: 0,
        transition: 'background 120ms linear',
      }}
    >
      <div
        style={{
          position: 'absolute',
          top: 3,
          left: on ? 34 : 3,
          width: 30,
          height: 30,
          borderRadius: 999,
          background: on ? tokens.bg0 : tokens.fgDim,
          transition: 'left 160ms ease',
        }}
      />
    </div>
  );
}

function Icon({ name }: { name: 'cloud' | 'sparkles' | 'activity' }) {
  const common = {
    width: 28,
    height: 28,
    viewBox: '0 0 24 24',
    fill: 'none',
    stroke: tokens.muted,
    strokeWidth: 2,
    strokeLinecap: 'round' as const,
    strokeLinejoin: 'round' as const,
  };
  if (name === 'cloud') {
    return (
      <svg {...common}>
        <path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z" />
      </svg>
    );
  }
  if (name === 'sparkles') {
    return (
      <svg {...common}>
        <path d="M12 3l2.5 5.5L20 11l-5.5 2.5L12 19l-2.5-5.5L4 11l5.5-2.5L12 3z" />
      </svg>
    );
  }
  return (
    <svg {...common}>
      <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
    </svg>
  );
}
