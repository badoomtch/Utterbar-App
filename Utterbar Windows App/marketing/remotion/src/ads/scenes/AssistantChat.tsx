/**
 * Generic AI-chat surface, vertical, v2. Same scale-up as the Slack
 * thread: 1020 wide, bigger bubbles (38 px font), bigger header dot,
 * bigger input field. The whole point is that this surface is the
 * action — it should DOMINATE the lower 70% of the frame.
 */
import React from 'react';
import { useCurrentFrame } from 'remotion';
import { tokens, fonts } from '../../design/tokens';

type Props = {
  assistantLabel: string;
  assistantColor?: string;
  draft: string;
  sent?: string;
  reply?: string;
  showInputCaret?: boolean;
  replyStreaming?: boolean;
};

export function AssistantChat({
  assistantLabel,
  assistantColor = tokens.accent,
  draft,
  sent,
  reply,
  showInputCaret = true,
  replyStreaming = false,
}: Props) {
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
        boxShadow: '0 30px 80px rgba(0,0,0,0.55)',
      }}
    >
      <Header label={assistantLabel} color={assistantColor} />

      <div
        style={{
          minHeight: 880,
          padding: '32px 36px 12px 36px',
          display: 'flex',
          flexDirection: 'column',
          gap: 28,
        }}
      >
        {sent && <UserBubble text={sent} />}
        {reply && (
          <AssistantBubble text={reply} color={assistantColor} streaming={replyStreaming} caretOn={caretOn} />
        )}
      </div>

      <div
        style={{
          margin: '0 32px 32px 32px',
          padding: '24px 28px',
          borderRadius: 18,
          background: tokens.bg0,
          border: `1px solid ${tokens.border}`,
          display: 'flex',
          alignItems: 'center',
          gap: 16,
        }}
      >
        <div style={{ flex: 1, fontSize: 32, color: tokens.fg, minHeight: 40 }}>
          {draft || <span style={{ color: tokens.fgDim }}>Ask anything…</span>}
          {showInputCaret && caretOn && (
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
            background: draft ? assistantColor : 'rgba(255,255,255,0.06)',
            color: draft ? '#06121A' : tokens.muted,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 24,
            fontWeight: 800,
          }}
        >
          ↑
        </div>
      </div>
    </div>
  );
}

function Header({ label, color }: { label: string; color: string }) {
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
          width: 44,
          height: 44,
          borderRadius: 999,
          background: color,
          boxShadow: `0 0 22px ${color}99`,
        }}
      />
      <div
        style={{
          color: tokens.fgHi,
          fontSize: 36,
          fontWeight: 600,
          letterSpacing: '-0.01em',
        }}
      >
        {label}
      </div>
    </div>
  );
}

function UserBubble({ text }: { text: string }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
      <div
        style={{
          maxWidth: '88%',
          padding: '24px 32px',
          borderRadius: 26,
          borderTopRightRadius: 8,
          background: `${tokens.accent}26`,
          border: `1px solid ${tokens.accent}66`,
          color: tokens.fgHi,
          fontSize: 38,
          lineHeight: 1.35,
          fontWeight: 500,
        }}
      >
        {text}
      </div>
    </div>
  );
}

function AssistantBubble({
  text,
  color,
  streaming,
  caretOn,
}: {
  text: string;
  color: string;
  streaming: boolean;
  caretOn: boolean;
}) {
  return (
    <div style={{ display: 'flex', justifyContent: 'flex-start', gap: 16 }}>
      <div
        style={{
          width: 44,
          height: 44,
          borderRadius: 999,
          background: color,
          marginTop: 8,
          flexShrink: 0,
          boxShadow: `0 0 18px ${color}77`,
        }}
      />
      <div
        style={{
          maxWidth: '86%',
          padding: '24px 32px',
          borderRadius: 26,
          borderTopLeftRadius: 8,
          background: tokens.bg0,
          border: `1px solid ${tokens.border}`,
          color: tokens.fg,
          fontSize: 36,
          lineHeight: 1.4,
          whiteSpace: 'pre-wrap',
          fontWeight: 500,
        }}
      >
        {text}
        {streaming && (
          <span
            style={{
              display: 'inline-block',
              width: 3,
              height: 32,
              background: color,
              marginLeft: 3,
              verticalAlign: 'text-bottom',
              opacity: caretOn ? 1 : 0,
            }}
          />
        )}
      </div>
    </div>
  );
}
