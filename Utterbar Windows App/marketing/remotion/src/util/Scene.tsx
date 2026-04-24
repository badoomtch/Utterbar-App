/**
 * Scene — the common wrapper every composition uses so the black bg,
 * font rendering, and centre-alignment don't get re-stated 12 times.
 * Wraps AbsoluteFill, sets font-family to sans, and provides a "stage"
 * slot that centres children with padding.
 */
import React from 'react';
import { AbsoluteFill } from 'remotion';
import { tokens, fonts } from '../design/tokens';

export function Scene({
  children,
  background = tokens.bg0,
  centered = false,
  padding = 0,
}: {
  children: React.ReactNode;
  background?: string;
  centered?: boolean;
  padding?: number;
}) {
  return (
    <AbsoluteFill
      style={{
        background,
        color: tokens.fg,
        fontFamily: fonts.sans,
        padding,
        display: centered ? 'flex' : 'block',
        alignItems: centered ? 'center' : undefined,
        justifyContent: centered ? 'center' : undefined,
      }}
    >
      {children}
    </AbsoluteFill>
  );
}

/** Callout copy block — eyebrow + headline + sub, used on feature shots. */
export function Callout({
  eyebrow,
  title,
  sub,
  align = 'left',
  accentEyebrow = true,
}: {
  eyebrow?: string;
  title: string;
  sub?: string;
  align?: 'left' | 'center';
  accentEyebrow?: boolean;
}) {
  return (
    <div
      style={{
        textAlign: align,
        color: tokens.fg,
        maxWidth: 640,
      }}
    >
      {eyebrow && (
        <div
          style={{
            fontSize: 11,
            fontWeight: 500,
            letterSpacing: '0.14em',
            textTransform: 'uppercase',
            color: accentEyebrow ? tokens.accent : tokens.muted,
            marginBottom: 14,
          }}
        >
          {eyebrow}
        </div>
      )}
      <div
        style={{
          fontSize: 56,
          fontWeight: 700,
          lineHeight: 1.05,
          letterSpacing: '-0.02em',
          color: tokens.fgHi,
        }}
      >
        {title}
      </div>
      {sub && (
        <div
          style={{
            marginTop: 18,
            fontSize: 20,
            lineHeight: 1.45,
            color: tokens.muted,
            maxWidth: 560,
          }}
        >
          {sub}
        </div>
      )}
    </div>
  );
}
