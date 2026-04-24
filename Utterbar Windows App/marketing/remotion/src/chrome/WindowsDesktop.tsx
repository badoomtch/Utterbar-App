/**
 * Windows-11-style desktop backdrop. Windows-first — NEVER a macOS frame.
 * Used by the "where Utterbar lives" shots, the works-anywhere demo,
 * the onboarding walkthrough, and anywhere we need to sell "on your PC".
 *
 * Contents:
 *   - soft cyan-navy gradient wallpaper (matches Windows 11 default vibe
 *     without infringing on the Microsoft bloom) plus a subtle radial
 *     highlight in the top-right
 *   - centred taskbar along the bottom (Start · Search · Task View · 6 app icons)
 *   - optional wallpaper accent tone override (for accent variants)
 *
 * Children render ABOVE the wallpaper but BELOW the taskbar.
 */
import React from 'react';
import { tokens } from '../design/tokens';

type Props = {
  children?: React.ReactNode;
  accent?: string;
};

export function WindowsDesktop({ children, accent = tokens.accent }: Props) {
  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        overflow: 'hidden',
        background: `radial-gradient(120% 80% at 78% 15%, ${accent}22 0%, transparent 55%),
                     radial-gradient(100% 70% at 20% 100%, #1c2547 0%, transparent 60%),
                     linear-gradient(180deg, #0B1024 0%, #151C36 55%, #0A0E1E 100%)`,
      }}
    >
      {/* Soft atmospheric highlight top-right — sells "morning light on a
          modern desktop wallpaper" without a photograph we'd have to
          licence. */}
      <div
        style={{
          position: 'absolute',
          top: '-10%',
          right: '-10%',
          width: 900,
          height: 900,
          borderRadius: 999,
          background: `radial-gradient(circle, ${accent}22 0%, transparent 70%)`,
          filter: 'blur(60px)',
        }}
      />

      {children}

      {/* Taskbar — Windows 11-style centred pill cluster at the bottom. */}
      <Taskbar accent={accent} />
    </div>
  );
}

function Taskbar({ accent }: { accent: string }) {
  return (
    <div
      style={{
        position: 'absolute',
        left: 0,
        right: 0,
        bottom: 0,
        height: 48,
        background: 'rgba(10, 14, 30, 0.72)',
        backdropFilter: 'blur(20px)',
        WebkitBackdropFilter: 'blur(20px)',
        borderTop: '1px solid rgba(255, 255, 255, 0.06)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 6,
      }}
    >
      {/* Start flag — stylised four-squares glyph. */}
      <TaskbarIcon>
        <svg width={18} height={18} viewBox="0 0 20 20">
          <rect x="1" y="1" width="8" height="8" fill={accent} rx="1" />
          <rect x="11" y="1" width="8" height="8" fill="#fff" rx="1" opacity="0.88" />
          <rect x="1" y="11" width="8" height="8" fill="#fff" rx="1" opacity="0.88" />
          <rect x="11" y="11" width="8" height="8" fill="#fff" rx="1" opacity="0.88" />
        </svg>
      </TaskbarIcon>

      {/* Search */}
      <TaskbarIcon>
        <svg width={18} height={18} viewBox="0 0 24 24" fill="none" stroke="#fff" strokeOpacity="0.9" strokeWidth="2">
          <circle cx="11" cy="11" r="7" />
          <path d="M16 16l5 5" strokeLinecap="round" />
        </svg>
      </TaskbarIcon>

      {/* Task view */}
      <TaskbarIcon>
        <svg width={18} height={18} viewBox="0 0 24 24" fill="none" stroke="#fff" strokeOpacity="0.9" strokeWidth="2">
          <rect x="3" y="4" width="7" height="7" rx="1" />
          <rect x="14" y="4" width="7" height="7" rx="1" />
          <rect x="3" y="14" width="7" height="7" rx="1" />
          <rect x="14" y="14" width="7" height="7" rx="1" />
        </svg>
      </TaskbarIcon>

      {/* App icons — coloured tiles representing Edge, File Explorer,
          Slack-ish, Notion-ish, Spotify-ish, Terminal. Intentionally
          abstract so we're not passing off third-party marks as our own. */}
      <TaskbarTile color="#1876d2" />
      <TaskbarTile color="#ffc107" />
      <TaskbarTile color="#7c3aed" />
      <TaskbarTile color="#0ea371" />
      <TaskbarTile color="#1db954" />
      <TaskbarTile color="#22d3ee" />
    </div>
  );
}

function TaskbarIcon({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        width: 36,
        height: 36,
        borderRadius: 6,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'transparent',
      }}
    >
      {children}
    </div>
  );
}

function TaskbarTile({ color }: { color: string }) {
  return (
    <div
      style={{
        width: 30,
        height: 30,
        borderRadius: 5,
        background: `linear-gradient(135deg, ${color}, ${color}AA)`,
        boxShadow: `inset 0 0 0 1px rgba(255,255,255,0.1)`,
      }}
    />
  );
}
