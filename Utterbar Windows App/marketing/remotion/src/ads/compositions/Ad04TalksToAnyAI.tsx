/**
 * ad-04-talks-to-any-ai — 14s. v3: cycle is now 1.5s per assistant
 * (was 1.4s — gives reply text actually time to read), holds last
 * assistant for 1.5s, end card 4.5s. Pill scale 2.4→2.0.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { AssistantChat } from '../scenes/AssistantChat';

const PROMPT = "Summarise this thread for me — three bullets, no fluff.";

const ASSISTANTS = [
  { label: 'ChatGPT',         color: '#10A37F', reply: 'Three bullets:\n• Webhook 422s started after the 14:10 deploy…' },
  { label: 'Claude',          color: '#D97757', reply: 'Sure — three bullets:\n• Staging deploy at 14:10 broke webhook signature…' },
  { label: 'Gemini',          color: '#4285F4', reply: 'Summary:\n• Webhook 422s appeared after 14:10…' },
  { label: 'Copilot',         color: '#0078D4', reply: 'Three bullets:\n• 14:10 deploy correlates with the 422 spike…' },
  { label: 'Your local model', color: '#67E8F9', reply: 'Three:\n• Failures begin 14:10, signature mismatch…' },
];

const PILL_HOLD = 1.0;
const BEAT = 1.5;

export function Ad04TalksToAnyAI() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const pillListening = frame < f(PILL_HOLD);
  const elapsedMs = pillListening ? (frame / fps) * 1000 : 0;

  const tInCycle = Math.max(0, (frame - f(PILL_HOLD)) / fps);
  const idx = Math.min(ASSISTANTS.length - 1, Math.floor(tInCycle / BEAT));
  const tIntoBeat = tInCycle - idx * BEAT;
  const a = ASSISTANTS[idx];

  // Reply snaps in fully at the start of each beat (not typewritten — the
  // ad is about brand-cycling, not about watching tokens stream).
  const replyText = pillListening ? '' : a.reply;
  const swapFade = tween(frame, f(PILL_HOLD + idx * BEAT), f(PILL_HOLD + idx * BEAT + 0.2), 0, 1);

  return (
    <SocialFrame variant="canvas">
      <div style={{ position: 'absolute', top: 100, left: '50%', transform: 'translateX(-50%) scale(2.0)', transformOrigin: 'top center' }}>
        <BarDriver state={pillListening ? 'listening' : 'idle'} elapsedMs={elapsedMs} />
      </div>

      <div style={{ position: 'absolute', top: 380, left: '50%', transform: 'translateX(-50%)', opacity: swapFade }}>
        <AssistantChat
          assistantLabel={a.label}
          assistantColor={a.color}
          draft=""
          sent={pillListening ? undefined : PROMPT}
          reply={pillListening ? undefined : replyText}
          replyStreaming={false}
          showInputCaret={false}
        />
      </div>

      <Captions
        cues={[
          { text: 'One hotkey.',    at: 0.2, until: 2.4 },
          { text: 'Any AI.',        at: 2.6, until: 6.0 },
          { text: 'Even your own.', at: 7.0, until: 9.5 },
        ]}
      />

      <EndCard tagline="Voice into any AI." startAt={9.5} />
    </SocialFrame>
  );
}
