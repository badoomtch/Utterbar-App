/**
 * feature-local-privacy — 6 s. Close shot of the real Privacy settings
 * card from the shipping app. Three context toggles (active-app,
 * selection, clipboard), all OFF — the buyer's default. Below them, a
 * paths card shows the local on-disk locations where data lives.
 *
 * Regenerated 2026-04-24 — killed the "mic → local → app with cloud
 * crossed out" diagram. The real story isn't an abstract arrow; it's
 * that every switch is a local switch and there's no cloud toggle
 * anywhere on the page.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween } from '../util/anim';

const TOGGLES = [
  { label: 'Allow active-app context', hint: 'App executable and window title.' },
  { label: 'Allow selection context', hint: 'Currently selected text in the foreground app.' },
  { label: 'Allow clipboard context', hint: 'Most recently copied text.' },
];

const PATHS = [
  { label: 'Recordings', path: '%APPDATA%\\Utterbar\\recordings' },
  { label: 'History', path: '%APPDATA%\\Utterbar\\history.db' },
  { label: 'Logs', path: '%APPDATA%\\Utterbar\\logs' },
];

function ToggleSwitch({ checked }: { checked: boolean }) {
  return (
    <div
      style={{
        width: 40,
        height: 22,
        borderRadius: 999,
        background: checked ? tokens.accent : 'rgba(255,255,255,0.14)',
        position: 'relative',
      }}
    >
      <div
        style={{
          position: 'absolute',
          top: 3,
          left: checked ? 19 : 3,
          width: 16,
          height: 16,
          borderRadius: '50%',
          background: '#fff',
        }}
      />
    </div>
  );
}

export function FeatureLocalPrivacy() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Reveal: card fades in 0.2 -> 0.8; toggle rows stagger 0.9, 1.2, 1.5;
  // paths card 2.3; hold 2.8 -> 6.0.
  const cardIn = tween(frame, f(0.2), f(0.8), 0, 1);
  const togglesEnter = [f(0.9), f(1.2), f(1.5)];
  const pathsCardIn = tween(frame, f(2.3), f(2.9), 0, 1);

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Header strip */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 80,
          transform: 'translateX(-50%)',
          width: 920,
          maxWidth: '90%',
          opacity: cardIn,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 8,
          }}
        >
          Settings · Privacy
        </div>
        <div
          style={{
            fontSize: 36,
            fontWeight: 600,
            color: tokens.fgHi,
            letterSpacing: '-0.02em',
            marginBottom: 4,
          }}
        >
          Context capture
        </div>
        <div
          style={{
            fontSize: 16,
            color: tokens.muted,
            marginBottom: 22,
          }}
        >
          Which signals may a mode use to tailor its output?
        </div>
      </div>

      {/* Toggles card */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 230,
          transform: 'translateX(-50%)',
          width: 920,
          maxWidth: '90%',
          background: tokens.bg1,
          borderRadius: 20,
          border: `1px solid ${tokens.border}`,
          overflow: 'hidden',
          opacity: cardIn,
        }}
      >
        {TOGGLES.map((t, i) => {
          const enter = togglesEnter[i];
          const opacity = tween(frame, enter - 5, enter + 8, 0, 1);
          return (
            <div
              key={t.label}
              style={{
                padding: '22px 28px',
                borderTop: i === 0 ? 'none' : `1px solid ${tokens.border}`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: 32,
                opacity,
              }}
            >
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 20, color: tokens.fgHi, marginBottom: 4 }}>
                  {t.label}
                </div>
                <div style={{ fontSize: 18, color: tokens.muted, lineHeight: 1.5 }}>
                  {t.hint}
                </div>
              </div>
              <ToggleSwitch checked={false} />
            </div>
          );
        })}
      </div>

      {/* Paths card — "Everything lives here on your machine." */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 560,
          transform: 'translateX(-50%)',
          width: 920,
          maxWidth: '90%',
          opacity: pathsCardIn,
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
          Data lives on your machine
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 18,
            border: `1px solid ${tokens.border}`,
            padding: '6px 0',
          }}
        >
          {PATHS.map((p, i) => (
            <div
              key={p.label}
              style={{
                padding: '12px 24px',
                borderTop: i === 0 ? 'none' : `1px solid ${tokens.border}`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: 24,
              }}
            >
              <span style={{ fontSize: 16, color: tokens.fg }}>{p.label}</span>
              <span
                style={{
                  fontFamily: fonts.mono,
                  fontSize: 18,
                  color: tokens.muted,
                }}
              >
                {p.path}
              </span>
            </div>
          ))}
        </div>
      </div>
    </AbsoluteFill>
  );
}
