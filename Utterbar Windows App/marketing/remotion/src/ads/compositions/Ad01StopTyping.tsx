/**
 * ad-01-stop-typing — 13.3s. v3 fixes the v2 timing collapse: outcome
 * holds for 3s now (was 1.6s), end card holds 4.5s (was 1.6s), pill
 * scaled to safe 2.0×.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { humanType } from '../frame/humanType';
import { SlackVerticalThread } from '../scenes/SlackVerticalThread';

const DRAFT = 'hey can you take a quick look at the';
const FINAL =
  "Hey, can you take a quick look at the staging deploy when you get a sec? Webhook 422s started showing up after the 14:10 push.";

const HISTORY = [
  { from: 'them' as const, text: 'Push went out 12 min ago.', ts: '14:22' },
  { from: 'them' as const, text: 'Anything blowing up?', ts: '14:23' },
];

export function Ad01StopTyping() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const draft = frame < f(2.4) ? humanType(DRAFT, frame, 0, fps, 0x1234) : DRAFT;

  let pillState: 'idle' | 'listening' | 'processing' = 'idle';
  if (frame >= f(2.4) && frame < f(5.4)) pillState = 'listening';
  else if (frame >= f(5.4) && frame < f(5.8)) pillState = 'processing';
  const elapsedMs = pillState === 'listening' ? ((frame - f(2.4)) / fps) * 1000 : 0;

  // Result snaps in fully formed at 5.8 — that's how dictation actually
  // works. Holds visible until end card at 8.8 = 3 seconds of payoff.
  const sentText = frame >= f(5.8) ? FINAL : undefined;
  const draftAfterSent = frame >= f(5.8) ? '' : draft;

  return (
    <SocialFrame variant="desktop">
      <div style={{ position: 'absolute', top: 100, left: '50%', transform: 'translateX(-50%) scale(2.0)', transformOrigin: 'top center' }}>
        <BarDriver state={pillState} elapsedMs={elapsedMs} />
      </div>

      <div style={{ position: 'absolute', top: 360, left: '50%', transform: 'translateX(-50%)' }}>
        <SlackVerticalThread
          peer="Marcus"
          history={HISTORY}
          draft={draftAfterSent}
          sent={sentText}
          showCaret={frame < f(2.4)}
        />
      </div>

      <Captions
        cues={[
          { text: 'Stop typing.',         at: 0.2, until: 2.3 },
          { text: 'Press · speak · done.', at: 2.6, until: 5.7 },
          { text: 'It just appears.',     at: 5.9, until: 8.7 },
        ]}
      />

      <EndCard tagline="Speak into anything." startAt={8.8} />
    </SocialFrame>
  );
}
