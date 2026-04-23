/**
 * HostWindow — the generic "app window" chrome we drop the user into when
 * we want to show "Utterbar works in any app". Windows-style minimise /
 * maximise / close glyphs on the right, a title bar on top, and a
 * customisable inner area.
 *
 * Variants:
 *   - browser  → address bar + tab row (Chrome/Edge-style, Windows-11 rounded corners)
 *   - chat     → left sidebar with channels/threads + main conversation pane
 *   - editor   → code/notes editor with a file list on the left
 *   - email    → mail-list column + reading pane
 *   - plain    → just the title bar + content slot
 *
 * Callers compose the body — we only provide the chrome.
 */
import React from 'react';
import { tokens, fonts } from '../design/tokens';

type Variant = 'browser' | 'chat' | 'editor' | 'email' | 'plain';

type Props = {
  title?: string;
  variant?: Variant;
  width?: number;
  height?: number;
  children?: React.ReactNode;
  accent?: string;
};

export function HostWindow({
  title = 'Window',
  variant = 'plain',
  width = 1200,
  height = 720,
  children,
  accent = '#4b8dff',
}: Props) {
  return (
    <div
      style={{
        width,
        height,
        borderRadius: 12,
        background: '#202123',
        overflow: 'hidden',
        boxShadow:
          '0 24px 60px rgba(0,0,0,0.45), 0 8px 20px rgba(0,0,0,0.25), 0 0 0 1px rgba(255,255,255,0.04)',
        display: 'flex',
        flexDirection: 'column',
        fontFamily: fonts.sans,
        color: '#E8EAED',
      }}
    >
      <TitleBar title={title} variant={variant} accent={accent} />
      <div style={{ flex: 1, position: 'relative', overflow: 'hidden', background: '#1A1B1E' }}>
        {children}
      </div>
    </div>
  );
}

function TitleBar({ title, variant, accent }: { title: string; variant: Variant; accent: string }) {
  if (variant === 'browser') return <BrowserBar title={title} accent={accent} />;
  if (variant === 'chat') return <ChatBar title={title} accent={accent} />;
  if (variant === 'editor') return <EditorBar title={title} />;
  if (variant === 'email') return <EmailBar title={title} accent={accent} />;
  return <PlainBar title={title} />;
}

function WindowButtons() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 0 }}>
      <WinBtn symbol="—" />
      <WinBtn symbol="▢" />
      <WinBtn symbol="×" hover="#E81123" />
    </div>
  );
}

function WinBtn({ symbol, hover }: { symbol: string; hover?: string }) {
  return (
    <div
      style={{
        width: 46,
        height: 32,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: 13,
        color: 'rgba(255,255,255,0.7)',
        background: hover === '#E81123' ? 'rgba(232,17,35,0.08)' : 'transparent',
      }}
    >
      {symbol}
    </div>
  );
}

function PlainBar({ title }: { title: string }) {
  return (
    <div
      style={{
        height: 32,
        background: '#2A2B2E',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        paddingLeft: 14,
        fontSize: 12,
        color: 'rgba(255,255,255,0.7)',
        fontWeight: 500,
      }}
    >
      <span>{title}</span>
      <WindowButtons />
    </div>
  );
}

function BrowserBar({ title, accent }: { title: string; accent: string }) {
  return (
    <div style={{ background: '#2A2B2E' }}>
      {/* Tab row */}
      <div style={{ display: 'flex', alignItems: 'center', height: 36, paddingLeft: 8 }}>
        {/* Active tab */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            height: 30,
            padding: '0 14px',
            background: '#1A1B1E',
            borderTopLeftRadius: 8,
            borderTopRightRadius: 8,
            fontSize: 12,
            fontWeight: 500,
            color: '#fff',
            maxWidth: 220,
          }}
        >
          <span
            style={{ width: 14, height: 14, borderRadius: 3, background: accent, flexShrink: 0 }}
          />
          <span
            style={{
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {title}
          </span>
          <span style={{ marginLeft: 6, opacity: 0.5, fontSize: 14 }}>×</span>
        </div>
        {/* Ghost tabs */}
        <GhostTab>Inbox</GhostTab>
        <GhostTab>Docs · Design System</GhostTab>
        <div style={{ marginLeft: 4, width: 28, height: 28, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'rgba(255,255,255,0.55)', fontSize: 14 }}>
          +
        </div>
        <div style={{ flex: 1 }} />
        <WindowButtons />
      </div>
      {/* Address bar */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          padding: '6px 12px 10px',
          background: '#1A1B1E',
        }}
      >
        <NavGlyph>‹</NavGlyph>
        <NavGlyph>›</NavGlyph>
        <NavGlyph>↻</NavGlyph>
        <div
          style={{
            flex: 1,
            height: 30,
            background: '#2A2B2E',
            borderRadius: 999,
            paddingLeft: 14,
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            fontSize: 12,
            color: 'rgba(255,255,255,0.72)',
            fontFamily: fonts.mono,
          }}
        >
          <svg width={12} height={12} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 2a6 6 0 0 0-6 6v3H5a1 1 0 0 0-1 1v8a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-8a1 1 0 0 0-1-1h-1V8a6 6 0 0 0-6-6Zm-4 9V8a4 4 0 1 1 8 0v3" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
          {title.toLowerCase().replace(/\s·.*$/, '').replace(/\s+/g, '')}.com
        </div>
        <NavGlyph>⋮</NavGlyph>
      </div>
    </div>
  );
}

function GhostTab({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        height: 30,
        padding: '0 12px',
        display: 'flex',
        alignItems: 'center',
        fontSize: 11,
        fontWeight: 500,
        color: 'rgba(255,255,255,0.55)',
        maxWidth: 180,
        overflow: 'hidden',
        whiteSpace: 'nowrap',
        textOverflow: 'ellipsis',
      }}
    >
      {children}
    </div>
  );
}

function NavGlyph({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        width: 26,
        height: 26,
        borderRadius: 6,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        color: 'rgba(255,255,255,0.7)',
        fontSize: 14,
      }}
    >
      {children}
    </div>
  );
}

function ChatBar({ title, accent }: { title: string; accent: string }) {
  return (
    <div
      style={{
        height: 44,
        background: '#1A1B1E',
        borderBottom: '1px solid rgba(255,255,255,0.06)',
        display: 'flex',
        alignItems: 'center',
        paddingLeft: 14,
        justifyContent: 'space-between',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <div
          style={{
            width: 24,
            height: 24,
            borderRadius: 6,
            background: accent,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 11,
            fontWeight: 700,
            color: '#fff',
          }}
        >
          #
        </div>
        <span style={{ fontWeight: 600, fontSize: 14 }}>{title}</span>
      </div>
      <WindowButtons />
    </div>
  );
}

function EditorBar({ title }: { title: string }) {
  return (
    <div
      style={{
        height: 32,
        background: '#1A1B1E',
        borderBottom: '1px solid rgba(255,255,255,0.04)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        paddingLeft: 14,
        fontSize: 11,
        color: 'rgba(255,255,255,0.6)',
        fontFamily: fonts.mono,
      }}
    >
      <span>{title}</span>
      <WindowButtons />
    </div>
  );
}

function EmailBar({ title, accent }: { title: string; accent: string }) {
  return (
    <div
      style={{
        height: 44,
        background: '#1A1B1E',
        borderBottom: '1px solid rgba(255,255,255,0.06)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        paddingLeft: 14,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <div
          style={{
            width: 24,
            height: 24,
            borderRadius: 6,
            background: accent,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <svg width={12} height={12} viewBox="0 0 24 24" fill="none" stroke="#fff" strokeWidth="2.2">
            <rect x="3" y="5" width="18" height="14" rx="2" />
            <path d="M3 7l9 6 9-6" />
          </svg>
        </div>
        <span style={{ fontWeight: 600, fontSize: 14 }}>{title}</span>
      </div>
      <WindowButtons />
    </div>
  );
}

// Generic caret dummy — used by host-children that want a text-caret pulse.
export function Caret({ on }: { on: boolean }) {
  return (
    <span
      style={{
        display: 'inline-block',
        width: 1,
        height: '1em',
        verticalAlign: '-0.15em',
        background: tokens.accent,
        opacity: on ? 1 : 0,
        marginLeft: 1,
      }}
    />
  );
}
