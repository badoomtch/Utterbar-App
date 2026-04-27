/**
 * Animated price line — v2 hero scale (220 px main, was 140). The
 * trailing /month, /year, "once" sits inline at 80 px so the slash
 * struck-through reads as a single typographic gesture.
 */
import React from 'react';
import { tokens, fonts } from '../../design/tokens';

export type PriceVariant = {
  trailing: string;
  trailingColor?: string;
  strikeThrough?: boolean;
};

export const PRICE_VARIANTS: PriceVariant[] = [
  { trailing: '/ month', strikeThrough: true },
  { trailing: '/ year',  strikeThrough: true },
  { trailing: 'once',    trailingColor: '#22D3EE' },
];

type Props = {
  variant: PriceVariant;
  opacity: number;
};

export function PriceLine({ variant, opacity }: Props) {
  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 24,
        opacity,
        fontFamily: fonts.sans,
      }}
    >
      <div
        style={{
          fontSize: 220,
          fontWeight: 700,
          color: tokens.fgHi,
          letterSpacing: '-0.04em',
          textDecoration: variant.strikeThrough ? 'line-through' : 'none',
          textDecorationColor: variant.strikeThrough ? `${tokens.muted}aa` : undefined,
          textDecorationThickness: 6,
          lineHeight: 1,
        }}
      >
        £39.99
      </div>
      <div
        style={{
          fontSize: 80,
          fontWeight: 600,
          color: variant.trailingColor ?? tokens.muted,
          letterSpacing: '-0.01em',
          marginTop: 30,
        }}
      >
        {variant.trailing}
      </div>
    </div>
  );
}
