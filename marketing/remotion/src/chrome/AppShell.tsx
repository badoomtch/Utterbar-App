/**
 * AppShell — the main Utterbar window chrome (not the floating bar). Port
 * of `src/routes/MainShell.tsx` simplified for marketing renders.
 *
 * 248 px sidebar (bg-1) with nav primary + nav settings groups, content
 * area on bg-0. Active nav items use the §1.3 active-state vocabulary:
 * bg-bg-2 lift + 2 px cyan left rail. A tiny brand block sits above the
 * nav with the pill mark + wordmark.
 *
 * The caller passes the active route label + the content to render. We
 * deliberately skip the real settings secondary-nav — marketing shots
 * live at the route level, not nested drill-downs.
 */
import React from 'react';
import { tokens, fonts } from '../design/tokens';
import { UtterbarLogoMark } from '@/components/bar/UtterbarLogoMark';
import { UtterbarWordmark } from '@/components/bar/UtterbarWordmark';

const NAV_PRIMARY = [
  { id: 'dashboard', label: 'Dashboard' },
  { id: 'history', label: 'History' },
  { id: 'modes', label: 'Modes' },
  { id: 'vocabulary', label: 'Vocabulary' },
  { id: 'files', label: 'Files' },
];

const NAV_SETTINGS = [
  { id: 'general', label: 'General' },
  { id: 'utterbar', label: 'Utterbar' },
  { id: 'audio', label: 'Audio' },
  { id: 'models', label: 'Models' },
  { id: 'shortcuts', label: 'Shortcuts' },
  { id: 'text', label: 'Text output' },
  { id: 'accessibility', label: 'Accessibility' },
  { id: 'privacy', label: 'Privacy' },
  { id: 'advanced', label: 'Advanced' },
  { id: 'licence', label: 'Licence' },
  { id: 'about', label: 'About' },
];

type Props = {
  active?: string;
  width?: number;
  height?: number;
  children?: React.ReactNode;
  tier?: 'free' | 'pro';
};

export function AppShell({
  active = 'dashboard',
  width = 1400,
  height = 880,
  children,
  tier = 'pro',
}: Props) {
  return (
    <div
      style={{
        width,
        height,
        display: 'flex',
        background: tokens.bg0,
        borderRadius: 10,
        overflow: 'hidden',
        fontFamily: fonts.sans,
        color: tokens.fg,
        boxShadow: '0 24px 60px rgba(0,0,0,0.55), 0 0 0 1px rgba(255,255,255,0.04)',
      }}
    >
      {/* Sidebar */}
      <aside
        style={{
          width: 248,
          background: tokens.bg1,
          display: 'flex',
          flexDirection: 'column',
          padding: '20px 12px 16px',
          flexShrink: 0,
        }}
      >
        {/* Brand block */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '0 8px 16px' }}>
          <UtterbarLogoMark size={22} />
          {/* Real Wordmark inherits its ink from CSS `currentColor`. */}
          <span style={{ color: tokens.fgHi, display: 'inline-flex' }}>
            <UtterbarWordmark size={16} />
          </span>
        </div>

        <NavGroup title={null} items={NAV_PRIMARY} active={active} />
        <div style={{ height: 20 }} />
        <NavGroup title="Settings" items={NAV_SETTINGS} active={active} />

        <div style={{ flex: 1 }} />
        <TierLine tier={tier} />
      </aside>

      {/* Content */}
      <main
        style={{
          flex: 1,
          background: tokens.bg0,
          overflow: 'hidden',
          position: 'relative',
        }}
      >
        {children}
      </main>
    </div>
  );
}

function NavGroup({
  title,
  items,
  active,
}: {
  title: string | null;
  items: { id: string; label: string }[];
  active: string;
}) {
  return (
    <div>
      {title && (
        <div
          style={{
            fontSize: 11,
            fontFamily: fonts.sans,
            fontWeight: 500,
            letterSpacing: '0.12em',
            textTransform: 'uppercase',
            color: tokens.fgDim,
            padding: '0 10px 8px',
          }}
        >
          {title}
        </div>
      )}
      {items.map((it) => (
        <NavItem key={it.id} label={it.label} active={it.id === active} />
      ))}
    </div>
  );
}

function NavItem({ label, active }: { label: string; active: boolean }) {
  return (
    <div
      style={{
        position: 'relative',
        display: 'flex',
        alignItems: 'center',
        height: 32,
        paddingLeft: 14,
        borderRadius: 8,
        marginBottom: 2,
        background: active ? tokens.bg2 : 'transparent',
        color: active ? tokens.fgHi : tokens.fg,
        fontSize: 13,
        fontWeight: active ? 500 : 400,
      }}
    >
      {active && (
        <span
          style={{
            position: 'absolute',
            left: 0,
            top: 6,
            bottom: 6,
            width: 2,
            borderRadius: 999,
            background: tokens.accent,
          }}
        />
      )}
      {label}
    </div>
  );
}

function TierLine({ tier }: { tier: 'free' | 'pro' }) {
  if (tier === 'pro') {
    return (
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          padding: '10px 10px',
          fontSize: 11,
          color: tokens.muted,
          fontFamily: fonts.mono,
        }}
      >
        <span
          style={{
            width: 6,
            height: 6,
            borderRadius: 999,
            background: tokens.moss,
            boxShadow: `0 0 6px ${tokens.moss}`,
          }}
        />
        Pro
      </div>
    );
  }
  return (
    <div
      style={{
        padding: '10px 10px',
        fontSize: 12,
        color: tokens.accent,
        fontFamily: fonts.sans,
        fontWeight: 500,
      }}
    >
      Upgrade to Pro →
    </div>
  );
}
