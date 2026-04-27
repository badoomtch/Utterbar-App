/**
 * ad-04-talks-to-any-ai — 13s. The "brand-agnostic" sweep.
 *
 * Single chat surface, but the assistant identity cycles through five
 * brands (ChatGPT, Claude, Gemini, Copilot, "Your local model"). Each
 * cycle: the user's same dictated prompt is already in the input, the
 * assistant's avatar+label swaps, the assistant streams a few words of
 * the same answer in its accent colour. ~2.2s per assistant.
 *
 * The point the existing FeatureTalkToAI doesn't make: it works with
 * WHICHEVER assistant the viewer already uses, today, no integration.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { humanType } from '../frame/humanType';
import { AssistantChat } from '../scenes/AssistantChat';

const PROMPT =
  "Summarise this thread for me — three bullets, no fluff.";

const ASSISTANTS = [
  { label: 'ChatGPT',         color: '#10A37F', reply: 'On it. Three bullets:\n• Webhook 422s started after the 14:10 deploy…' },
  { label: 'Claude',          color: '#D97757', reply: 'Sure — three bullets:\n• Staging deploy at 14:10 broke webhook signature…' },
  { label: 'Gemini',          color: '#4285F4', reply: "Here's a three-bullet summary:\n• Webhook 422s appeared after 14:10…" },
  { label: 'Copilot',         color: '#0078D4', reply: 'Summary in three bullets:\n• 14:10 deploy correlates with 422 spike…' },
  { label: 'Your local model', color: '#67E8F9', reply: 'Three bullets:\n• Failures begin at 14:10, signature mismatch…' },
];

const BEAT = 2.2;            // seconds per assistant
const PILL_HOLD = 1.6;       // seconds the pill listens up front

export function Ad04TalksToAnyAI() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // First 1.6s: pill listening, no chat history yet.
  const pillListening = frame < f(PILL_HOLD);
  const elapsedMs = pillListening ? (frame / fps) * 1000 : 0;

  // After PILL_HOLD: cycle through assistants. Index from 0..4.
  const tInCycle = Math.max(0, (frame - f(PILL_HOLD)) / fps);
  const idx = Math.min(ASSISTANTS.length - 1, Math.floor(tInCycle / BEAT));
  const tIntoBeat = tInCycle - idx * BEAT;
  const a = ASSISTANTS[idx];

  // Assistant reply streams in over each beat (1.4s of typing in a 2.2s
  // window — leaves the last 0.8s as visual hold before the swap).
  const replyProgress = Math.max(0, Math.min(1, tIntoBeat / 1.4));
  const replyText = a.reply.slice(0, Math.floor(a.reply.length * replyProgress));

  // Swap dot animation: 0.25s crossfade at the boundary.
  const swapFade = tween(
    frame,
    f(PILL_HOLD + idx * BEAT),
    f(PILL_HOLD + idx * BEAT + 0.25),
    0,
    1,
  );

  return (
    <SocialFrame variant="canvas">
      <div
        style={{
          position: 'absolute',
          top: 90,
          left: '50%',
          transform: 'translateX(-50%)',
          fontFamily: fonts.mono,
          fontSize: 18,
          letterSpacing: '0.18em',
          textTransform: 'uppercase',
          color: tokens.muted,
        }}
      >
        Voice into any AI assistant
      </div>

      {/* Pill — sits up top, listening pose, then idle while assistants cycle */}
      <div
        style={{
          position: 'absolute',
          top: 170,
          left: '50%',
          transform: 'translateX(-50%) scale(2.0)',
          transformOrigin: 'top center',
        }}
      >
        <BarDriver state={pillListening ? 'listening' : 'idle'} elapsedMs={elapsedMs} />
      </div>

      {/* Chat surface */}
      <div
        style={{
          position: 'absolute',
          top: 360,
          left: '50%',
          transform: 'translateX(-50%)',
          opacity: swapFade,
        }}
      >
        <AssistantChat
          assistantLabel={a.label}
          assistantColor={a.color}
          draft=""
          sent={pillListening ? undefined : PROMPT}
          reply={pillListening ? undefined : replyText}
          replyStreaming={replyText.length < a.reply.length}
          showInputCaret={false}
        />
      </div>

      <Captions
        cues={[
          { text: 'One hotkey.', at: 0.3, until: 1.6 },
          { text: 'Any assistant.', at: 1.7, until: 3.6 },
          { text: 'No integration.', at: 3.7, until: 5.6 },
          { text: 'No API key.', at: 5.7, until: 7.6 },
          { text: 'Even your local model.', at: 9.5, until: 11.5 },
        ]}
        bottom={120}
      />

      <EndCard tagline="Voice into any AI." startAt={11.5} />
    </SocialFrame>
  );
}
