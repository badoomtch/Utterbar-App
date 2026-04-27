/**
 * ad-02-voice-is-faster — 11s. Split-vertical race against the clock.
 *
 * Two stacked panels share one rolling wall-clock counter at the top:
 *   - top  panel: typing the same sentence into a chat input,
 *                 reaches end of sentence around 8.5s
 *   - bot  panel: pill listens for ~2.5s, message lands fully formed
 *                 around 5.5s, panel sits "done" while top still types
 *
 * The visual punchline is the bottom panel finishing first and sitting
 * idle while the top panel grinds away.
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

const SENTENCE =
  "Push the staging fix when CI clears and ping me on Slack so I can verify before five.";

export function Ad02VoiceIsFaster() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const typingPanelText = humanType(SENTENCE, frame, f(0.5), fps, 0xbeef);

  // Voice panel: pill flickers idle → listening → processing → success,
  // then text snaps in.
  let pillState: 'idle' | 'listening' | 'processing' = 'idle';
  if (frame >= f(0.5) && frame < f(1.0)) pillState = 'idle';
  else if (frame >= f(1.0) && frame < f(4.5)) pillState = 'listening';
  else if (frame >= f(4.5) && frame < f(5.2)) pillState = 'processing';

  const voicePanelDone = frame >= f(5.2);
  const elapsedMs = pillState === 'listening' ? ((frame - f(1.0)) / fps) * 1000 : 0;

  // Race clock — counts up from 0 while either panel is still working.
  const raceTime = Math.max(0, (frame - f(0.5)) / fps);

  return (
    <SocialFrame variant="canvas">
      <RaceClock seconds={raceTime} />

      {/* Top panel — TYPING */}
      <Panel
        top={260}
        label="Typing"
        accent={tokens.muted}
        done={typingPanelText.length === SENTENCE.length}
        doneAtSec={raceTime}
      >
        <TypingSurface text={typingPanelText} active={typingPanelText.length < SENTENCE.length} />
      </Panel>

      {/* Bottom panel — DICTATION */}
      <Panel
        top={1080}
        label="Utterbar"
        accent={tokens.accent}
        done={voicePanelDone}
        doneAtSec={5.2}
      >
        <VoiceSurface
          state={pillState}
          elapsedMs={elapsedMs}
          showResult={voicePanelDone}
          result={SENTENCE}
        />
      </Panel>

      <Captions
        cues={[
          { text: 'Race the keyboard.', at: 0.3, until: 4.5 },
          { text: 'Voice wins.', at: 5.4, until: 8.0 },
        ]}
        bottom={120}
      />

      <EndCard tagline="Faster than typing. Every time." startAt={8.5} />
    </SocialFrame>
  );
}

function RaceClock({ seconds }: { seconds: number }) {
  return (
    <div
      style={{
        position: 'absolute',
        top: 90,
        left: '50%',
        transform: 'translateX(-50%)',
        fontFamily: fonts.mono,
        fontSize: 64,
        color: tokens.fgHi,
        fontVariantNumeric: 'tabular-nums',
        letterSpacing: '-0.02em',
      }}
    >
      {seconds.toFixed(1)}s
    </div>
  );
}

type PanelProps = {
  top: number;
  label: string;
  accent: string;
  done: boolean;
  doneAtSec: number;
  children?: React.ReactNode;
};
function Panel({ top, label, accent, done, doneAtSec, children }: PanelProps) {
  return (
    <div
      style={{
        position: 'absolute',
        top,
        left: '50%',
        transform: 'translateX(-50%)',
        width: 940,
        height: 720,
        background: tokens.bg1,
        border: `1px solid ${done ? `${accent}88` : tokens.border}`,
        borderRadius: 24,
        padding: '24px 28px',
        display: 'flex',
        flexDirection: 'column',
        gap: 16,
        boxShadow: done ? `0 0 60px ${accent}33` : 'none',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 16,
            letterSpacing: '0.16em',
            textTransform: 'uppercase',
            color: accent,
          }}
        >
          {label}
        </div>
        {done && (
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 18,
              color: accent,
            }}
          >
            ✓ {doneAtSec.toFixed(1)}s
          </div>
        )}
      </div>
      <div style={{ flex: 1 }}>{children}</div>
    </div>
  );
}

function TypingSurface({ text, active }: { text: string; active: boolean }) {
  const frame = useCurrentFrame();
  const blink = Math.floor(frame / 12) % 2 === 0;
  return (
    <div
      style={{
        height: '100%',
        background: tokens.bg0,
        borderRadius: 16,
        border: `1px solid ${tokens.border}`,
        padding: '28px 30px',
        fontFamily: 'Instrument Sans Variable, sans-serif',
        fontSize: 28,
        color: tokens.fgHi,
        lineHeight: 1.5,
      }}
    >
      {text}
      {active && blink && (
        <span
          style={{
            display: 'inline-block',
            width: 3,
            height: 28,
            background: tokens.muted,
            marginLeft: text ? 2 : 0,
            verticalAlign: 'text-bottom',
          }}
        />
      )}
    </div>
  );
}

function VoiceSurface({
  state,
  elapsedMs,
  showResult,
  result,
}: {
  state: 'idle' | 'listening' | 'processing';
  elapsedMs: number;
  showResult: boolean;
  result: string;
}) {
  return (
    <div
      style={{
        height: '100%',
        background: tokens.bg0,
        borderRadius: 16,
        border: `1px solid ${tokens.border}`,
        padding: '28px 30px',
        display: 'flex',
        flexDirection: 'column',
        gap: 24,
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      {!showResult && (
        <div style={{ transform: 'scale(2.4)', transformOrigin: 'center' }}>
          <BarDriver state={state} elapsedMs={elapsedMs} />
        </div>
      )}
      {showResult && (
        <div
          style={{
            fontFamily: 'Instrument Sans Variable, sans-serif',
            fontSize: 28,
            color: tokens.fgHi,
            lineHeight: 1.5,
            textAlign: 'left',
            width: '100%',
          }}
        >
          {result}
        </div>
      )}
    </div>
  );
}
