/**
 * Hotkey badge — v2 keys are huge. The chord IS the hero in Ad06; at
 * v1 size (70 px tall) it read as a tiny chip. Now Space caps at
 * 140 px tall and ~280 px wide, full hand-feel.
 */
import React from 'react';
import { tokens, fonts } from '../../design/tokens';

type Props = {
  pressed?: boolean;
};

export function HotkeyBadge({ pressed = false }: Props) {
  return (
    <div
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: 28,
        padding: '32px 44px',
        background: tokens.bg2,
        border: `1px solid ${tokens.border}`,
        borderRadius: 28,
        fontFamily: fonts.mono,
      }}
    >
      <KeyCap label="Ctrl" pressed={pressed} />
      <div style={{ color: tokens.fgDim, fontSize: 44, fontWeight: 300 }}>+</div>
      <KeyCap label="Space" wide pressed={pressed} />
    </div>
  );
}

function KeyCap({ label, wide, pressed }: { label: string; wide?: boolean; pressed?: boolean }) {
  return (
    <div
      style={{
        minWidth: wide ? 280 : 140,
        height: 140,
        padding: '0 32px',
        background: pressed ? tokens.accent3 : tokens.bg1,
        border: `1px solid ${pressed ? tokens.accent : tokens.border}`,
        borderBottomWidth: pressed ? 1 : 8,
        borderRadius: 22,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        color: pressed ? '#06121A' : tokens.fgHi,
        fontSize: 50,
        fontWeight: pressed ? 700 : 500,
        transform: `translateY(${pressed ? 6 : 0}px)`,
        boxShadow: pressed ? `0 0 40px ${tokens.accent}88` : 'none',
      }}
    >
      {label}
    </div>
  );
}
