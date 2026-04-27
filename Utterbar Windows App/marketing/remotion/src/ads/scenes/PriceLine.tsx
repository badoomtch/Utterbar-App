/**
 * Animated price line — used in Ad10PriceOnce. Displays a price string
 * with the slash-line replaced over time. The composition cycles through:
 *
 *   £39.99/month   ←  what users brace for
 *   £39.99/year    ←  what users hope for
 *   £39.99 once    ←  what they actually get
 *
 * Each variant fades in over the previous, with the trailing word
 * (month / year / once) swapping cleanly. Caller supplies the active
 * index + per-line opacity from the parent composition.
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
        gap: 16,
        opacity,
        fontFamily: fonts.sans,
      }}
    >
      <div
        style={{
          fontSize: 140,
          fontWeight: 600,
          color: tokens.fgHi,
          letterSpacing: '-0.04em',
          textDecoration: variant.strikeThrough ? 'line-through' : 'none',
          textDecorationColor: variant.strikeThrough ? `${tokens.muted}aa` : undefined,
          textDecorationThickness: 4,
        }}
      >
        £39.99
      </div>
      <div
        style={{
          fontSize: 56,
          fontWeight: 500,
          color: variant.trailingColor ?? tokens.muted,
          letterSpacing: '-0.01em',
          marginTop: 18,
        }}
      >
        {variant.trailing}
      </div>
    </div>
  );
}
