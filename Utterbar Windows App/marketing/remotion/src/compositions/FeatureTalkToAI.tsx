/**
 * feature-talk-to-ai — 10 s. The fastest way to talk to an AI assistant.
 *
 * Frame layout: pill at the top, a generic AI-chat surface below it
 * (assistant message at top, user input box at the bottom). The user
 * holds the hotkey, dictates a question via Utterbar, the question
 * lands in the chat input as one fast paste, then the assistant's
 * reply types back in. Loop ends with the reply mid-stream.
 *
 * Deliberately generic — no logos, no brand colours. The point is
 * "this works with whichever assistant you already use", not "we
 * partner with Vendor X". Avatar is a neutral cyan dot.
 *
 * Timeline:
 *   0.0 – 1.0   pill idle, empty input, blinking caret
 *   1.0 – 4.5   pill listening (waveform animates), input still empty
 *               (user is speaking — that's the listening phase)
 *   4.5 – 5.0   pill processing
 *   5.0 – 5.6   user message pastes into the input, then snaps up into
 *               the conversation as a sent message bubble
 *   5.6 – 9.5   assistant reply streams in token-by-token below it
 *   9.5 – 10.0  hold on the populated conversation
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import { tween, typewriter } from '../util/anim';

const USER_MESSAGE =
  'Rewrite this paragraph in plain English without changing the meaning.';

// Long enough that the assistant reply is visibly streaming through
// the end of the composition, not finishing early and going stale.
const ASSISTANT_REPLY =
  'Here it is in plain English: the customer wants the team to keep them informed at every step, with concrete dates rather than vague timelines, and to flag risks early instead of waiting for them to surface in a release meeting.';

export function FeatureTalkToAI() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Pill state arc.
  let pillState: 'idle' | 'listening' | 'processing' = 'idle';
  if (frame < f(1.0)) pillState = 'idle';
  else if (frame < f(4.5)) pillState = 'listening';
  else if (frame < f(5.0)) pillState = 'processing';
  else pillState = 'idle';

  const elapsed =
    pillState === 'listening' ? Math.max(0, (frame - f(1.0)) / fps) : 0;

  // User message: pastes into input over 0.3s, then "sends" — input
  // empties and the message rises into the conversation as a bubble.
  const inputTextProgress = tween(frame, f(5.0), f(5.3), 0, 1);
  const inputText = inputTextProgress < 1 ? typewriter(USER_MESSAGE, frame, f(5.0), f(5.3)) : '';
  const sentBubbleAppear = tween(frame, f(5.3), f(5.6), 0, 1);
  const inputCleared = frame >= f(5.3);

  // Assistant reply streams in starting at 5.6s, finishing-ish at 9.0s.
  const replyTyped = typewriter(ASSISTANT_REPLY, frame, f(5.6), f(9.0));

  // Caret in the input field.
  const caretBlink = Math.floor(frame / 12) % 2;
  const showInputCaret = frame < f(5.0) || (frame >= f(5.3) && frame < f(5.6));

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Eyebrow */}
      <div
        style={{
          position: 'absolute',
          top: 60,
          left: '50%',
          transform: 'translateX(-50%)',
          fontFamily: fonts.mono,
          fontSize: 15,
          letterSpacing: '0.18em',
          textTransform: 'uppercase',
          color: tokens.muted,
        }}
      >
        Talk to your AI assistant — at the speed of speech
      </div>

      {/* Pill at the top */}
      <div
        style={{
          position: 'absolute',
          top: 110,
          left: '50%',
          transform: 'translateX(-50%)',
        }}
      >
        <div style={{ transform: 'scale(1.6)', transformOrigin: 'top center' }}>
          <BarDriver state={pillState} elapsedMs={elapsed * 1000} />
        </div>
      </div>

      {/* Chat surface — sits centred-bottom. Generic, no logos. */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 290,
          transform: 'translateX(-50%)',
          width: 1080,
          maxWidth: '88%',
          background: tokens.bg1,
          borderRadius: 20,
          border: `1px solid ${tokens.border}`,
          padding: '24px 28px 22px 28px',
          minHeight: 480,
          display: 'flex',
          flexDirection: 'column',
          gap: 16,
        }}
      >
        {/* Window-chrome strip — generic mac-style three dots so it
            reads as "an app" without naming one. */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            paddingBottom: 14,
            borderBottom: `1px solid ${tokens.border}`,
            marginBottom: 4,
          }}
        >
          <div style={{ width: 10, height: 10, borderRadius: 999, background: 'rgba(255,255,255,0.16)' }} />
          <div style={{ width: 10, height: 10, borderRadius: 999, background: 'rgba(255,255,255,0.16)' }} />
          <div style={{ width: 10, height: 10, borderRadius: 999, background: 'rgba(255,255,255,0.16)' }} />
          <div
            style={{
              marginLeft: 14,
              fontFamily: fonts.mono,
              fontSize: 12,
              letterSpacing: '0.08em',
              color: tokens.muted,
              textTransform: 'uppercase',
            }}
          >
            Assistant
          </div>
        </div>

        {/* Conversation */}
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: 16,
            minHeight: 320,
          }}
        >
          {/* User message bubble — appears once the input is "sent". */}
          {sentBubbleAppear > 0.01 && (
            <div
              style={{
                display: 'flex',
                justifyContent: 'flex-end',
                opacity: sentBubbleAppear,
                transform: `translateY(${(1 - sentBubbleAppear) * 12}px)`,
              }}
            >
              <div
                style={{
                  maxWidth: '78%',
                  padding: '14px 18px',
                  borderRadius: 18,
                  borderTopRightRadius: 6,
                  background: 'rgba(34, 211, 238, 0.14)',
                  border: `1px solid rgba(34, 211, 238, 0.3)`,
                  color: tokens.fgHi,
                  fontSize: 18,
                  lineHeight: 1.5,
                }}
              >
                {USER_MESSAGE}
              </div>
            </div>
          )}

          {/* Assistant reply — appears when the user message has
              landed. Avatar dot on the left. */}
          {replyTyped.length > 0 && (
            <div style={{ display: 'flex', alignItems: 'flex-start', gap: 12 }}>
              <div
                style={{
                  width: 28,
                  height: 28,
                  borderRadius: 999,
                  background: tokens.accent,
                  flexShrink: 0,
                  marginTop: 4,
                  boxShadow: `0 0 12px rgba(34, 211, 238, 0.4)`,
                }}
              />
              <div
                style={{
                  maxWidth: '82%',
                  padding: '14px 18px',
                  borderRadius: 18,
                  borderTopLeftRadius: 6,
                  background: tokens.bg0,
                  border: `1px solid ${tokens.border}`,
                  color: tokens.fg,
                  fontSize: 18,
                  lineHeight: 1.55,
                  whiteSpace: 'pre-wrap',
                }}
              >
                {replyTyped}
                {/* Streaming caret — only visible while the reply is
                    still being typed. */}
                {replyTyped.length < ASSISTANT_REPLY.length && (
                  <span
                    style={{
                      display: 'inline-block',
                      width: 2,
                      height: 18,
                      background: tokens.accent,
                      marginLeft: 2,
                      verticalAlign: 'text-bottom',
                      opacity: caretBlink,
                    }}
                  />
                )}
              </div>
            </div>
          )}
        </div>

        {/* Input field at the bottom of the surface */}
        <div
          style={{
            marginTop: 'auto',
            display: 'flex',
            alignItems: 'center',
            gap: 12,
            padding: '14px 16px',
            borderRadius: 14,
            background: tokens.bg0,
            border: `1px solid ${tokens.border}`,
          }}
        >
          <div
            style={{
              flex: 1,
              minHeight: 24,
              fontSize: 17,
              color: tokens.fg,
              lineHeight: 1.4,
            }}
          >
            {inputCleared ? '' : inputText}
            {showInputCaret && (
              <span
                style={{
                  display: 'inline-block',
                  width: 2,
                  height: 18,
                  background: tokens.muted,
                  marginLeft: inputText.length > 0 ? 2 : 0,
                  verticalAlign: 'text-bottom',
                  opacity: caretBlink,
                }}
              />
            )}
            {!inputText && !inputCleared && frame < f(1.0) && (
              <span style={{ color: tokens.muted, fontStyle: 'italic' }}>
                Ask anything…
              </span>
            )}
          </div>
          <div
            style={{
              width: 36,
              height: 36,
              borderRadius: 10,
              background: inputText.length > 0 ? tokens.accent : 'rgba(255,255,255,0.06)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: 17,
              color: inputText.length > 0 ? '#06121A' : tokens.muted,
              fontWeight: 700,
            }}
          >
            ↑
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
}
