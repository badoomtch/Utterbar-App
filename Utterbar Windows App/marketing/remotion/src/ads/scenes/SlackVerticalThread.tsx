/**
 * Generic Slack-style DM thread, vertical layout. No Slack logo — title
 * bar reads "Direct message". The point of every ad here is "any chat
 * app", not "we partner with Slack".
 *
 * Caller supplies:
 *   - peer: name shown in the title bar + their two existing messages
 *   - draft: what's currently being typed in the input (caller controls)
 *   - sent:  what to render as a sent bubble at the bottom of the thread
 *   - showCaret: whether the input field shows a blinking caret
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
        width: 920,
        background: tokens.bg1,
        borderRadius: 22,
        border: `1px solid ${tokens.border}`,
        overflow: 'hidden',
        fontFamily: fonts.sans,
        boxShadow: '0 30px 80px rgba(0,0,0,0.55), 0 2px 0 rgba(255,255,255,0.04) inset',
      }}
    >
      <ThreadHeader peer={peer} />
      <div
        style={{
          minHeight: 760,
          display: 'flex',
          flexDirection: 'column',
          gap: 18,
          padding: '24px 28px 16px 28px',
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
        padding: '20px 24px',
        borderBottom: `1px solid ${tokens.border}`,
        display: 'flex',
        alignItems: 'center',
        gap: 14,
      }}
    >
      <div
        style={{
          width: 36,
          height: 36,
          borderRadius: 8,
          background: 'linear-gradient(135deg, #f97316 0%, #ec4899 100%)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#fff',
          fontWeight: 700,
          fontSize: 18,
        }}
      >
        {peer.slice(0, 1).toUpperCase()}
      </div>
      <div style={{ display: 'flex', flexDirection: 'column' }}>
        <div style={{ color: tokens.fgHi, fontSize: 22, fontWeight: 600 }}>{peer}</div>
        <div
          style={{
            color: tokens.muted,
            fontSize: 14,
            fontFamily: fonts.mono,
            letterSpacing: '0.05em',
            textTransform: 'uppercase',
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
      <div style={{ maxWidth: '78%' }}>
        <div
          style={{
            padding: '14px 18px',
            borderRadius: 18,
            borderTopRightRadius: mine ? 6 : 18,
            borderTopLeftRadius: mine ? 18 : 6,
            background: mine ? `${tokens.accent}1f` : tokens.bg2,
            border: `1px solid ${mine ? `${tokens.accent}55` : tokens.border}`,
            color: tokens.fgHi,
            fontSize: 22,
            lineHeight: 1.4,
          }}
        >
          {msg.text}
        </div>
        <div
          style={{
            marginTop: 6,
            color: tokens.fgDim,
            fontFamily: fonts.mono,
            fontSize: 12,
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
        margin: '0 24px 24px 24px',
        padding: '14px 16px',
        borderRadius: 14,
        background: tokens.bg0,
        border: `1px solid ${tokens.border}`,
        display: 'flex',
        alignItems: 'center',
        gap: 12,
      }}
    >
      <div style={{ flex: 1, fontSize: 22, color: tokens.fg, lineHeight: 1.35, minHeight: 28 }}>
        {draft || <span style={{ color: tokens.fgDim }}>Message…</span>}
        {caretOn && (
          <span
            style={{
              display: 'inline-block',
              width: 2,
              height: 22,
              background: tokens.muted,
              marginLeft: draft ? 2 : 0,
              verticalAlign: 'text-bottom',
            }}
          />
        )}
      </div>
      <div
        style={{
          width: 40,
          height: 40,
          borderRadius: 10,
          background: draft ? tokens.accent : 'rgba(255,255,255,0.06)',
          color: draft ? '#06121A' : tokens.muted,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontWeight: 800,
          fontSize: 18,
        }}
      >
        ↑
      </div>
    </div>
  );
}
