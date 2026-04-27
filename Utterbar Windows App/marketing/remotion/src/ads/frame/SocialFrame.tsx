/**
 * Vertical 1080×1920 frame wrapper. The whole ad pack composes inside
 * one of these. Provides a soft cyan-tinged dark canvas + a centred
 * safe area for content (rule of thirds, top + bottom 18% reserved for
 * eyebrow / captions / end card).
 *
 * Two variants:
 *   - 'desktop'  → cyan-tinged radial wallpaper, mock taskbar at the
 *                  very bottom. Use when the ad lives "inside" the
 *                  user's PC (most product ads).
 *   - 'canvas'   → pure bg0 with a faint cyan glow top-right. Use for
 *                  brand films and typography pieces.
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
      ? `radial-gradient(120% 60% at 78% 12%, ${tokens.accent}1a 0%, transparent 55%),
         radial-gradient(80% 40% at 18% 92%, ${tokens.accent3}26 0%, transparent 60%),
         linear-gradient(180deg, #08101a 0%, ${tokens.bg0} 40%, ${tokens.bg0} 100%)`
      : `radial-gradient(60% 40% at 50% 8%, ${tokens.accent}14 0%, transparent 65%), ${tokens.bg0}`;

  return (
    <AbsoluteFill style={{ background: wallpaper, overflow: 'hidden' }}>
      {children}
      {variant === 'desktop' && <FauxTaskbar />}
    </AbsoluteFill>
  );
}

function FauxTaskbar() {
  return (
    <div
      style={{
        position: 'absolute',
        left: '50%',
        bottom: 28,
        transform: 'translateX(-50%)',
        height: 56,
        padding: '0 14px',
        borderRadius: 14,
        background: 'rgba(10, 14, 23, 0.78)',
        border: `1px solid ${tokens.border}`,
        backdropFilter: 'blur(18px)',
        display: 'flex',
        alignItems: 'center',
        gap: 14,
      }}
    >
      <TaskbarPip cyan />
      <TaskbarPip />
      <TaskbarPip />
      <TaskbarPip />
      <TaskbarPip />
      <div
        style={{
          width: 1,
          height: 22,
          background: tokens.border,
          margin: '0 4px',
        }}
      />
      <div
        style={{
          fontFamily: tokens.bg0 ? "'Geist Mono Variable', monospace" : 'monospace',
          fontSize: 13,
          color: tokens.muted,
          letterSpacing: '0.04em',
        }}
      >
        14:32
      </div>
    </div>
  );
}

function TaskbarPip({ cyan }: { cyan?: boolean }) {
  return (
    <div
      style={{
        width: 28,
        height: 28,
        borderRadius: 8,
        background: cyan ? `${tokens.accent}3a` : 'rgba(255,255,255,0.06)',
        border: cyan ? `1px solid ${tokens.accent}66` : `1px solid ${tokens.border}`,
      }}
    />
  );
}
