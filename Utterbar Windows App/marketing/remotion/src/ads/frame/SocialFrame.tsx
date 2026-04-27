/**
 * Vertical 1080×1920 frame wrapper. Pure background, no chrome — the
 * v1 had a faux Win11 taskbar pinned at the bottom which (1) was tiny
 * at thumb-distance phone viewing, (2) added zero meaning to the ad,
 * (3) ate the lower-third caption safe area. Cut it.
 *
 * Two variants:
 *   - 'desktop' → cyan-tinged radial wallpaper (used when the ad
 *                 evokes "on a PC")
 *   - 'canvas'  → near-black with a soft top cyan wash (brand films,
 *                 typography pieces, anything that wants to feel
 *                 weightless)
 */
import React from 'react';
import { AbsoluteFill } from 'remotion';
import { tokens } from '../../design/tokens';

type Props = {
  variant?: 'desktop' | 'canvas';
  children?: React.ReactNode;
};

export function SocialFrame({ variant = 'desktop', children }: Props) {
  const wallpaper =
    variant === 'desktop'
      ? `radial-gradient(120% 50% at 78% 8%, ${tokens.accent}26 0%, transparent 55%),
         radial-gradient(80% 35% at 18% 96%, ${tokens.accent3}30 0%, transparent 60%),
         linear-gradient(180deg, #08101a 0%, ${tokens.bg0} 38%, ${tokens.bg0} 100%)`
      : `radial-gradient(70% 40% at 50% 6%, ${tokens.accent}1a 0%, transparent 65%), ${tokens.bg0}`;

  return (
    <AbsoluteFill style={{ background: wallpaper, overflow: 'hidden' }}>
      {children}
    </AbsoluteFill>
  );
}
