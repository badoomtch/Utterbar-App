/**
 * ad-01-stop-typing — 12s. The "rage-typing → dictation" beat, but the
 * angle the existing landscape comps don't take: we SHOW the user
 * deleting and restarting before they reach for the hotkey. That
 * deletion moment is the unspoken empathy hook — every chat-typer
 * knows it.
 *
 * 0.0–4.0  draft 1: "hey can you" ... pauses ... deletes back
 * 4.0–5.0  pill appears (idle) above the thread, hotkey pressed cue
 * 5.0–7.5  pill listening (waveform), thread still empty
 * 7.5–8.2  pill processing
 * 8.2–8.6  message snaps in fully formed
 * 8.6–9.5  caption hold "press · speak · done"
 * 9.5–12.0 end card
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
import { SlackVerticalThread } from '../scenes/SlackVerticalThread';

const DRAFT_TYPED = 'hey can you take a quick look at the';
const FINAL_MESSAGE =
  "Hey, can you take a quick look at the staging deploy when you get a sec? Webhook 422s started showing up after the 14:10 push.";

const HISTORY = [
  { from: 'them' as const, text: 'Push went out 12 min ago.', ts: '14:22' },
  { from: 'them' as const, text: 'Anything blowing up?', ts: '14:23' },
];

export function Ad01StopTyping() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Draft 1 grows then deletes back to empty before the hotkey.
  let draft = '';
  if (frame < f(2.6)) {
    draft = humanType(DRAFT_TYPED, frame, 0, fps, 0x1234);
  } else if (frame < f(4.0)) {
    // Backspace from full string back to empty in 1.4s.
    const t = tween(frame, f(2.6), f(4.0), 1, 0);
    draft = DRAFT_TYPED.slice(0, Math.floor(DRAFT_TYPED.length * t));
  }

  // Pill state arc.
  let pillState: 'idle' | 'listening' | 'processing' = 'idle';
  if (frame >= f(4.0) && frame < f(5.0)) pillState = 'idle';
  else if (frame >= f(5.0) && frame < f(7.5)) pillState = 'listening';
  else if (frame >= f(7.5) && frame < f(8.2)) pillState = 'processing';

  const showPill = frame >= f(4.0) && frame < f(9.5);
  const elapsedMs = pillState === 'listening' ? ((frame - f(5.0)) / fps) * 1000 : 0;

  const sentText = frame >= f(8.2) ? FINAL_MESSAGE : undefined;
  const draftAfterSent = frame >= f(8.2) ? '' : draft;

  // Pill drops in from above.
  const pillY = tween(frame, f(4.0), f(4.5), -120, 0);
  const pillOpacity = tween(frame, f(4.0), f(4.4), 0, 1);

  return (
    <SocialFrame variant="desktop">
      {/* Eyebrow text top-left */}
      <Eyebrow text="Direct message · Slack" at={0.2} />

      {/* The thread, centred below the eyebrow */}
      <div
        style={{
          position: 'absolute',
          top: 220,
          left: '50%',
          transform: 'translateX(-50%)',
        }}
      >
        <SlackVerticalThread
          peer="Marcus"
          history={HISTORY}
          draft={draftAfterSent}
          sent={sentText}
          showCaret={frame < f(4.0)}
        />
      </div>

      {/* Pill — sits above the thread, drops in when summoned */}
      {showPill && (
        <div
          style={{
            position: 'absolute',
            top: 130 + pillY,
            left: '50%',
            transform: 'translateX(-50%) scale(2.2)',
            transformOrigin: 'top center',
            opacity: pillOpacity,
          }}
        >
          <BarDriver state={pillState} elapsedMs={elapsedMs} />
        </div>
      )}

      <Captions
        cues={[
          { text: 'Stop typing.', at: 0.4, until: 4.0 },
          { text: 'Press, speak, done.', at: 5.0, until: 8.2 },
          { text: 'It just appears.', at: 8.3, until: 9.4 },
        ]}
      />

      <EndCard tagline="Speak into anything." startAt={9.5} />
    </SocialFrame>
  );
}

function Eyebrow({ text, at }: { text: string; at: number }) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const opacity = tween(frame, Math.round(at * fps), Math.round((at + 0.4) * fps), 0, 1);
  return (
    <div
      style={{
        position: 'absolute',
        top: 80,
        left: '50%',
        transform: 'translateX(-50%)',
        fontFamily: fonts.mono,
        fontSize: 18,
        letterSpacing: '0.18em',
        textTransform: 'uppercase',
        color: tokens.muted,
        opacity,
      }}
    >
      {text}
    </div>
  );
}
