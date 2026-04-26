/**
 * feature-customise-bar — 6 s, banner aspect. The pill preview sits on
 * the left, six accent swatches sit on the right with the "active"
 * ring stepping through them; the pill's LED + waveform + tail repaint
 * in the selected colour.
 *
 * Regenerated 2026-04-24, then reflowed 2026-04-24 for the banner
 * (1440x520) frame — old vertical stack left dead space top and bottom.
 * Horizontal layout uses the whole frame.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';

const ACCENTS = [
  { id: 'cyan', primary: '#22D3EE', secondary: '#67E8F9' },
  { id: 'teal', primary: '#14B8A6', secondary: '#5EEAD4' },
  { id: 'violet', primary: '#8B5CF6', secondary: '#C4B5FD' },
  { id: 'rose', primary: '#F43F5E', secondary: '#FDA4AF' },
  { id: 'amber', primary: '#F59E0B', secondary: '#FCD34D' },
  { id: 'lime', primary: '#84CC16', secondary: '#BEF264' },
];

function selectedIndex(frame: number, fps: number): number {
  const step = Math.floor((frame / fps) / 0.8) % ACCENTS.length;
  return Math.max(0, Math.min(ACCENTS.length - 1, step));
}

/** A mini-pill that repaints against the given accent. */
function AccentPillPreview({ accent }: { accent: (typeof ACCENTS)[number] }) {
  return (
    <svg viewBox="-2 -2 124 64" width={720} height={360} style={{ overflow: 'visible' }}>
      <rect x={0} y={0} width={120} height={60} rx={30} fill="#06070A" stroke="rgba(255,255,255,0.18)" strokeWidth={0.6} />
      <rect x={0} y={0} width={120} height={60} rx={30} fill="none" stroke="rgba(255,255,255,0.65)" strokeWidth={0.35} />
      <circle cx={20} cy={30} r={6} fill={accent.primary} />
      <circle cx={20} cy={30} r={8.5} fill="none" stroke={accent.primary} strokeOpacity={0.35} strokeWidth={1} />
      <g fill={accent.primary}>
        <rect x={44} y={25} width={4} height={10} rx={2} />
        <rect x={54} y={21} width={4} height={18} rx={2} />
        <rect x={64} y={16} width={4} height={28} rx={2} />
        <rect x={84} y={16} width={4} height={28} rx={2} />
        <rect x={94} y={21} width={4} height={18} rx={2} />
        <rect x={104} y={25} width={4} height={10} rx={2} />
      </g>
      <path d="M 74 10 A 2 2 0 0 1 78 10 V 50 L 74 44 Z" fill={accent.secondary} />
    </svg>
  );
}

export function FeatureCustomiseBar() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const i = selectedIndex(frame, fps);
  const accent = ACCENTS[i];

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Pill — left half, vertically centred. Frame is 1600 wide now. */}
      <div
        style={{
          position: 'absolute',
          left: 0,
          top: '50%',
          transform: 'translateY(-50%)',
          width: 1000,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          filter: 'drop-shadow(0 18px 40px rgba(0,0,0,0.6))',
        }}
      >
        <AccentPillPreview accent={accent} />
      </div>

      {/* Right panel — label + swatch column, vertically centred. */}
      <div
        style={{
          position: 'absolute',
          right: 100,
          top: '50%',
          transform: 'translateY(-50%)',
          width: 460,
          display: 'flex',
          flexDirection: 'column',
          gap: 28,
        }}
      >
        <div>
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 17,
              letterSpacing: '0.18em',
              textTransform: 'uppercase',
              color: tokens.muted,
              marginBottom: 4,
            }}
          >
            Accent
          </div>
          <div
            style={{
              fontSize: 40,
              fontWeight: 600,
              color: accent.secondary,
              letterSpacing: '-0.02em',
              textTransform: 'capitalize',
            }}
          >
            {accent.id}
          </div>
        </div>

        <div style={{ display: 'flex', gap: 18, flexWrap: 'wrap' }}>
          {ACCENTS.map((a, idx) => {
            const active = idx === i;
            return (
              <div
                key={a.id}
                style={{
                  width: 52,
                  height: 52,
                  borderRadius: '50%',
                  background: a.primary,
                  boxShadow: active
                    ? `0 0 0 3px ${tokens.bg0}, 0 0 0 6px ${a.primary}`
                    : 'none',
                  transform: `scale(${active ? 1.08 : 1})`,
                }}
              />
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
}
