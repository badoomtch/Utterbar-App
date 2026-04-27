/**
 * Faux network-monitor strip — sits at the top of the privacy ad.
 * Shows "0 KB sent" while the user dictates a long message, with a
 * fake throughput sparkline that stays flat. This is the visual
 * proof of the local-only claim.
 *
 * Counter is driven by a `bytes` prop; the sparkline is a flat line
 * of the same value. Caller decides whether to ramp it (don't, that's
 * the whole point) or hold at zero (yes, do this).
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
        width: 920,
        background: tokens.bg1,
        borderRadius: 18,
        border: `1px solid ${tokens.border}`,
        padding: '20px 26px',
        fontFamily: fonts.mono,
        display: 'flex',
        alignItems: 'center',
        gap: 24,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <div
          style={{
            width: 10,
            height: 10,
            borderRadius: 999,
            background: tokens.moss,
            boxShadow: `0 0 10px ${tokens.moss}`,
            opacity: blink ? 1 : 0.55,
          }}
        />
        <div
          style={{
            color: tokens.muted,
            fontSize: 14,
            letterSpacing: '0.12em',
            textTransform: 'uppercase',
          }}
        >
          Network out
        </div>
      </div>
      <div style={{ flex: 1, height: 32, position: 'relative' }}>
        <Sparkline />
      </div>
      <div
        style={{
          color: tokens.fgHi,
          fontSize: 28,
          fontVariantNumeric: 'tabular-nums',
          letterSpacing: '0.02em',
        }}
      >
        {bytes.toLocaleString()} <span style={{ color: tokens.muted, fontSize: 18 }}>KB</span>
      </div>
    </div>
  );
}

function Sparkline() {
  // 64 data points, all zero — the visual punchline. A flat line
  // running through the centre of the plot area.
  const w = 580;
  const h = 32;
  return (
    <svg width={w} height={h} viewBox={`0 0 ${w} ${h}`} style={{ overflow: 'visible' }}>
      <line
        x1={0}
        x2={w}
        y1={h / 2}
        y2={h / 2}
        stroke={tokens.moss}
        strokeWidth={2}
        strokeLinecap="round"
        opacity={0.85}
      />
      <line
        x1={0}
        x2={w}
        y1={h / 2}
        y2={h / 2}
        stroke={tokens.moss}
        strokeWidth={10}
        strokeLinecap="round"
        opacity={0.12}
      />
    </svg>
  );
}
