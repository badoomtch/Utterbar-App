/**
 * ad-02-voice-is-faster — 12s. v3: voice surface result holds for
 * 4.5s (was ~3s) so the comparison reads — typing keeps grinding
 * on while the voice panel sits done. Pill scale 2.6→2.0.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { humanType } from '../frame/humanType';

const SENTENCE = "Push the staging fix when CI clears and ping me on Slack.";

export function Ad02VoiceIsFaster() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const typingText = humanType(SENTENCE, frame, 0, fps, 0xbeef);

  let pillState: 'idle' | 'listening' | 'processing' = 'idle';
  if (frame < f(0.4)) pillState = 'idle';
  else if (frame < f(2.6)) pillState = 'listening';
  else if (frame < f(3.0)) pillState = 'processing';

  const voiceDone = frame >= f(3.0);
  const elapsedMs = pillState === 'listening' ? ((frame - f(0.4)) / fps) * 1000 : 0;
  const raceTime = Math.max(0, frame / fps);

  return (
    <SocialFrame variant="canvas">
      <RaceClock seconds={raceTime} />

      <Panel
        top={220}
        label="Typing"
        accent={tokens.muted}
        done={typingText.length === SENTENCE.length}
        doneAtSec={raceTime}
      >
        <TypingSurface text={typingText} active={typingText.length < SENTENCE.length} />
      </Panel>

      <Panel
        top={1080}
        label="Utterbar"
        accent={tokens.accent}
        done={voiceDone}
        doneAtSec={3.0}
      >
        <VoiceSurface state={pillState} elapsedMs={elapsedMs} showResult={voiceDone} result={SENTENCE} />
      </Panel>

      <Captions
        cues={[
          { text: 'Race the keyboard.', at: 0.1, until: 2.9 },
          { text: 'Voice wins.',        at: 3.1, until: 7.4 },
        ]}
      />

      <EndCard tagline="Faster than typing." startAt={7.5} />
    </SocialFrame>
  );
}

function RaceClock({ seconds }: { seconds: number }) {
  return (
    <div
      style={{
        position: 'absolute',
        top: 60,
        left: '50%',
        transform: 'translateX(-50%)',
        fontFamily: fonts.mono,
        fontSize: 96,
        color: tokens.fgHi,
        fontVariantNumeric: 'tabular-nums',
        letterSpacing: '-0.02em',
        fontWeight: 600,
      }}
    >
      {seconds.toFixed(1)}s
    </div>
  );
}

type PanelProps = { top: number; label: string; accent: string; done: boolean; doneAtSec: number; children?: React.ReactNode };
function Panel({ top, label, accent, done, doneAtSec, children }: PanelProps) {
  return (
    <div
      style={{
        position: 'absolute',
        top,
        left: '50%',
        transform: 'translateX(-50%)',
        width: 1020,
        height: 760,
        background: tokens.bg1,
        border: `2px solid ${done ? `${accent}aa` : tokens.border}`,
        borderRadius: 28,
        padding: '28px 32px',
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
        boxShadow: done ? `0 0 80px ${accent}44` : 'none',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 28,
            letterSpacing: '0.16em',
            textTransform: 'uppercase',
            color: accent,
            fontWeight: 600,
          }}
        >
          {label}
        </div>
        {done && (
          <div style={{ fontFamily: fonts.mono, fontSize: 32, color: accent, fontWeight: 700 }}>
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
        borderRadius: 18,
        border: `1px solid ${tokens.border}`,
        padding: '32px 36px',
        fontFamily: 'Instrument Sans Variable, sans-serif',
        fontSize: 44,
        color: tokens.fgHi,
        lineHeight: 1.4,
        fontWeight: 500,
      }}
    >
      {text}
      {active && blink && (
        <span style={{ display: 'inline-block', width: 4, height: 44, background: tokens.muted, marginLeft: text ? 3 : 0, verticalAlign: 'text-bottom' }} />
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
        borderRadius: 18,
        border: `1px solid ${tokens.border}`,
        padding: '32px 36px',
        display: 'flex',
        flexDirection: 'column',
        gap: 24,
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      {!showResult && (
        <div style={{ transform: 'scale(2.0)', transformOrigin: 'center' }}>
          <BarDriver state={state} elapsedMs={elapsedMs} />
        </div>
      )}
      {showResult && (
        <div
          style={{
            fontFamily: 'Instrument Sans Variable, sans-serif',
            fontSize: 44,
            color: tokens.fgHi,
            lineHeight: 1.4,
            fontWeight: 500,
            width: '100%',
          }}
        >
          {result}
        </div>
      )}
    </div>
  );
}
