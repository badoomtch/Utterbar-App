/**
 * Generic AI-chat surface, vertical. Brand-agnostic chrome — caller
 * passes the assistant *label* (e.g. "ChatGPT", "Claude", "Gemini"),
 * which renders as a neutral dot + label so the same component
 * services Ad04TalksToAnyAI's brand-cycle.
 *
 * Caller controls draft / sent / reply and the caret-blink phase via
 * useCurrentFrame in their own composition.
 */
import React from 'react';
import { useCurrentFrame } from 'remotion';
import { tokens, fonts } from '../../design/tokens';

type Props = {
  assistantLabel: string;
  /** Optional dot colour — defaults to brand cyan. Use to give the
   *  illusion of switching assistants in Ad04. */
  assistantColor?: string;
  draft: string;
  sent?: string;
  reply?: string;
  showInputCaret?: boolean;
  /** When `reply` is partial (still streaming), show the trailing caret. */
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
        width: 920,
        background: tokens.bg1,
        borderRadius: 22,
        border: `1px solid ${tokens.border}`,
        overflow: 'hidden',
        fontFamily: fonts.sans,
        boxShadow: '0 30px 80px rgba(0,0,0,0.55)',
      }}
    >
      <Header label={assistantLabel} color={assistantColor} />

      <div
        style={{
          minHeight: 720,
          padding: '24px 28px 8px 28px',
          display: 'flex',
          flexDirection: 'column',
          gap: 18,
        }}
      >
        {sent && <UserBubble text={sent} />}
        {reply && <AssistantBubble text={reply} color={assistantColor} streaming={replyStreaming} caretOn={caretOn} />}
      </div>

      <div
        style={{
          margin: '0 24px 24px 24px',
          padding: '16px 18px',
          borderRadius: 14,
          background: tokens.bg0,
          border: `1px solid ${tokens.border}`,
          display: 'flex',
          alignItems: 'center',
          gap: 12,
        }}
      >
        <div style={{ flex: 1, fontSize: 22, color: tokens.fg, minHeight: 28 }}>
          {draft || <span style={{ color: tokens.fgDim }}>Ask anything…</span>}
          {showInputCaret && caretOn && (
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
            background: draft ? assistantColor : 'rgba(255,255,255,0.06)',
            color: draft ? '#06121A' : tokens.muted,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 18,
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
        padding: '20px 24px',
        borderBottom: `1px solid ${tokens.border}`,
        display: 'flex',
        alignItems: 'center',
        gap: 12,
      }}
    >
      <div
        style={{
          width: 28,
          height: 28,
          borderRadius: 999,
          background: color,
          boxShadow: `0 0 16px ${color}88`,
        }}
      />
      <div style={{ color: tokens.fgHi, fontSize: 22, fontWeight: 600, letterSpacing: '-0.01em' }}>
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
          maxWidth: '82%',
          padding: '14px 18px',
          borderRadius: 18,
          borderTopRightRadius: 6,
          background: `${tokens.accent}1f`,
          border: `1px solid ${tokens.accent}55`,
          color: tokens.fgHi,
          fontSize: 22,
          lineHeight: 1.4,
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
    <div style={{ display: 'flex', justifyContent: 'flex-start', gap: 12 }}>
      <div
        style={{
          width: 28,
          height: 28,
          borderRadius: 999,
          background: color,
          marginTop: 6,
          flexShrink: 0,
          boxShadow: `0 0 12px ${color}66`,
        }}
      />
      <div
        style={{
          maxWidth: '84%',
          padding: '14px 18px',
          borderRadius: 18,
          borderTopLeftRadius: 6,
          background: tokens.bg0,
          border: `1px solid ${tokens.border}`,
          color: tokens.fg,
          fontSize: 22,
          lineHeight: 1.5,
          whiteSpace: 'pre-wrap',
        }}
      >
        {text}
        {streaming && (
          <span
            style={{
              display: 'inline-block',
              width: 2,
              height: 22,
              background: color,
              marginLeft: 2,
              verticalAlign: 'text-bottom',
              opacity: caretOn ? 1 : 0,
            }}
          />
        )}
      </div>
    </div>
  );
}
