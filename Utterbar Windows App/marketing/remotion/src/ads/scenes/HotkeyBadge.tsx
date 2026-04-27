/**
 * Hotkey badge — visual rep of the Ctrl+Space chord. Used in any ad
 * where we want to communicate "press the hotkey" without showing a
 * hand. Two key caps with a "+" between, with a subtle pulse cue
 * driven by a `pressed` prop (caller decides timing).
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
        gap: 16,
        padding: '18px 26px',
        background: tokens.bg2,
        border: `1px solid ${tokens.border}`,
        borderRadius: 16,
        fontFamily: fonts.mono,
      }}
    >
      <KeyCap label="Ctrl" pressed={pressed} />
      <div style={{ color: tokens.fgDim, fontSize: 22 }}>+</div>
      <KeyCap label="Space" wide pressed={pressed} />
    </div>
  );
}

function KeyCap({ label, wide, pressed }: { label: string; wide?: boolean; pressed?: boolean }) {
  return (
    <div
      style={{
        minWidth: wide ? 140 : 70,
        height: 70,
        padding: '0 18px',
        background: pressed ? tokens.accent3 : tokens.bg1,
        border: `1px solid ${pressed ? tokens.accent : tokens.border}`,
        borderBottomWidth: pressed ? 1 : 4,
        borderRadius: 12,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        color: pressed ? '#06121A' : tokens.fgHi,
        fontSize: 24,
        fontWeight: pressed ? 700 : 500,
        transform: `translateY(${pressed ? 3 : 0}px)`,
        boxShadow: pressed ? `0 0 24px ${tokens.accent}66` : 'none',
      }}
    >
      {label}
    </div>
  );
}
