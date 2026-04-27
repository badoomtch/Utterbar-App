/**
 * Generic Slack-style DM thread, vertical layout. v2 — bubbles 2× v1
 * size, content 1020 wide instead of 920, dropped the avatar header
 * chrome (it ate vertical for nothing). The thread IS the action;
 * everything else was decoration.
 *
 * No Slack logo. Title bar reads "Direct message". Brand-agnostic.
 */
import React from 'react';
import { useCurrentFrame } from 'remotion';
import { tokens, fonts } from '../../design/tokens';

type IncomingMsg = { from: 'them' | 'you'; text: string; ts: string };

type Props = {
  peer: string;
  history: IncomingMsg[];
  draft: string;
  sent?: string;
  showCaret?: boolean;
};

export function SlackVerticalThread({ peer, history, draft, sent, showCaret = true }: Props) {
  const frame = useCurrentFrame();
  const caretOn = Math.floor(frame / 12) % 2 === 0;

  return (
    <div
      style={{
        width: 1020,
        background: tokens.bg1,
        borderRadius: 26,
        border: `1px solid ${tokens.border}`,
        overflow: 'hidden',
        fontFamily: fonts.sans,
        boxShadow: '0 30px 80px rgba(0,0,0,0.55), 0 2px 0 rgba(255,255,255,0.04) inset',
      }}
    >
      <ThreadHeader peer={peer} />
      <div
        style={{
          minHeight: 980,
          display: 'flex',
          flexDirection: 'column',
          gap: 28,
          padding: '32px 36px 24px 36px',
        }}
      >
        {history.map((m, i) => (
          <Bubble key={i} msg={m} />
        ))}
        {sent && <Bubble msg={{ from: 'you', text: sent, ts: 'now' }} />}
      </div>
      <InputRow draft={draft} caretOn={showCaret && caretOn} />
    </div>
  );
}

function ThreadHeader({ peer }: { peer: string }) {
  return (
    <div
      style={{
        padding: '24px 32px',
        borderBottom: `1px solid ${tokens.border}`,
        display: 'flex',
        alignItems: 'center',
        gap: 18,
      }}
    >
      <div
        style={{
          width: 56,
          height: 56,
          borderRadius: 12,
          background: 'linear-gradient(135deg, #f97316 0%, #ec4899 100%)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#fff',
          fontWeight: 700,
          fontSize: 28,
        }}
      >
        {peer.slice(0, 1).toUpperCase()}
      </div>
      <div style={{ display: 'flex', flexDirection: 'column' }}>
        <div style={{ color: tokens.fgHi, fontSize: 36, fontWeight: 600, lineHeight: 1.05 }}>
          {peer}
        </div>
        <div
          style={{
            color: tokens.muted,
            fontSize: 18,
            fontFamily: fonts.mono,
            letterSpacing: '0.05em',
            textTransform: 'uppercase',
            marginTop: 4,
          }}
        >
          Direct message
        </div>
      </div>
    </div>
  );
}

function Bubble({ msg }: { msg: IncomingMsg }) {
  const mine = msg.from === 'you';
  return (
    <div style={{ display: 'flex', justifyContent: mine ? 'flex-end' : 'flex-start' }}>
      <div style={{ maxWidth: '88%' }}>
        <div
          style={{
            padding: '24px 32px',
            borderRadius: 26,
            borderTopRightRadius: mine ? 8 : 26,
            borderTopLeftRadius: mine ? 26 : 8,
            background: mine ? `${tokens.accent}26` : tokens.bg2,
            border: `1px solid ${mine ? `${tokens.accent}66` : tokens.border}`,
            color: tokens.fgHi,
            fontSize: 38,
            lineHeight: 1.35,
            fontWeight: 500,
          }}
        >
          {msg.text}
        </div>
        <div
          style={{
            marginTop: 8,
            color: tokens.fgDim,
            fontFamily: fonts.mono,
            fontSize: 18,
            textAlign: mine ? 'right' : 'left',
          }}
        >
          {msg.ts}
        </div>
      </div>
    </div>
  );
}

function InputRow({ draft, caretOn }: { draft: string; caretOn: boolean }) {
  return (
    <div
      style={{
        margin: '0 32px 32px 32px',
        padding: '22px 24px',
        borderRadius: 18,
        background: tokens.bg0,
        border: `1px solid ${tokens.border}`,
        display: 'flex',
        alignItems: 'center',
        gap: 16,
      }}
    >
      <div style={{ flex: 1, fontSize: 32, color: tokens.fg, lineHeight: 1.3, minHeight: 40 }}>
        {draft || <span style={{ color: tokens.fgDim }}>Message…</span>}
        {caretOn && (
          <span
            style={{
              display: 'inline-block',
              width: 3,
              height: 32,
              background: tokens.muted,
              marginLeft: draft ? 3 : 0,
              verticalAlign: 'text-bottom',
            }}
          />
        )}
      </div>
      <div
        style={{
          width: 52,
          height: 52,
          borderRadius: 12,
          background: draft ? tokens.accent : 'rgba(255,255,255,0.06)',
          color: draft ? '#06121A' : tokens.muted,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontWeight: 800,
          fontSize: 24,
        }}
      >
        ↑
      </div>
    </div>
  );
}
