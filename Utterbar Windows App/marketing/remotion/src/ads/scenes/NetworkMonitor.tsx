/**
 * Faux network-monitor strip — v2 scaled up. Counter is the visual
 * hero now (96 px, was 28). Whole strip is wider (1020 vs 920) and
 * taller. Sparkline lengthens to fill.
 */
import React from 'react';
import { useCurrentFrame } from 'remotion';
import { tokens, fonts } from '../../design/tokens';

type Props = {
  bytes: number;
};

export function NetworkMonitor({ bytes }: Props) {
  const frame = useCurrentFrame();
  const blink = Math.floor(frame / 18) % 2 === 0;

  return (
    <div
      style={{
        width: 1020,
        background: tokens.bg1,
        borderRadius: 22,
        border: `1px solid ${tokens.border}`,
        padding: '32px 38px',
        fontFamily: fonts.mono,
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
        <div
          style={{
            width: 16,
            height: 16,
            borderRadius: 999,
            background: tokens.moss,
            boxShadow: `0 0 14px ${tokens.moss}`,
            opacity: blink ? 1 : 0.55,
          }}
        />
        <div
          style={{
            color: tokens.muted,
            fontSize: 24,
            letterSpacing: '0.14em',
            textTransform: 'uppercase',
          }}
        >
          Network out · this device
        </div>
      </div>
      <div style={{ display: 'flex', alignItems: 'flex-end', gap: 28 }}>
        <div
          style={{
            color: tokens.fgHi,
            fontSize: 96,
            fontVariantNumeric: 'tabular-nums',
            letterSpacing: '-0.02em',
            fontWeight: 600,
            lineHeight: 1,
          }}
        >
          {bytes.toLocaleString()}
        </div>
        <div style={{ color: tokens.muted, fontSize: 36, marginBottom: 10 }}>KB sent</div>
      </div>
      <div style={{ height: 48, position: 'relative' }}>
        <Sparkline />
      </div>
    </div>
  );
}

function Sparkline() {
  const w = 940;
  const h = 48;
  return (
    <svg width={w} height={h} viewBox={`0 0 ${w} ${h}`} style={{ overflow: 'visible' }}>
      <line x1={0} x2={w} y1={h / 2} y2={h / 2} stroke={tokens.moss} strokeWidth={3} strokeLinecap="round" opacity={0.85} />
      <line x1={0} x2={w} y1={h / 2} y2={h / 2} stroke={tokens.moss} strokeWidth={14} strokeLinecap="round" opacity={0.14} />
    </svg>
  );
}
