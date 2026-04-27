/**
 * ad-07-teach-vocabulary — 15s. v3: take 1 result snaps in (was
 * typewritten, missed the punchline), holds long enough to read
 * the wrong text, vocabulary chip lands cleanly, take 2 result
 * snaps in and holds for comparison. Pill scale 2.0 (already).
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';

const WRONG = "Ship the an throw pick integration before launch.";
const RIGHT = "Ship the Anthropic integration before launch.";

export function Ad07TeachVocabulary() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const takeOnePill = frame >= f(0.2) && frame < f(2.0);
  // Take 1 result snaps in at 2.0 — wrong on purpose, want it readable.
  const takeOneText = frame >= f(2.0) ? WRONG : '';
  const showTeach = frame >= f(5.0);
  const showVocabChip = frame >= f(5.7);

  const takeTwoPill = frame >= f(7.0) && frame < f(8.4);
  const takeTwoText = frame >= f(8.4) ? RIGHT : '';

  const elapsed1 = takeOnePill ? ((frame - f(0.2)) / fps) * 1000 : 0;
  const elapsed2 = takeTwoPill ? ((frame - f(7.0)) / fps) * 1000 : 0;

  return (
    <SocialFrame variant="canvas">
      <Panel top={140} label="Take 1" highlight={frame >= f(3.5) && frame < f(7.0)}>
        <PillRow listening={takeOnePill} elapsedMs={elapsed1} />
        <Output text={takeOneText} highlight={frame >= f(3.5) ? "an throw pick" : undefined} />
      </Panel>

      {showTeach && <VocabularyTeach showChip={showVocabChip} />}

      <Panel top={1080} label="After teaching" accent={tokens.accent}>
        <PillRow listening={takeTwoPill} elapsedMs={elapsed2} />
        <Output text={takeTwoText} highlight={takeTwoText.includes('Anthropic') ? 'Anthropic' : undefined} highlightColor={tokens.accent} />
      </Panel>

      <Captions
        cues={[
          { text: 'It misheard.',          at: 3.5, until: 5.0 },
          { text: 'Teach it once.',        at: 5.1, until: 6.9 },
          { text: 'Now it gets it right.', at: 8.5, until: 10.4 },
        ]}
      />

      <EndCard tagline="Teach it your jargon." startAt={10.5} />
    </SocialFrame>
  );
}

function Panel({
  top,
  label,
  accent = tokens.muted,
  highlight,
  children,
}: {
  top: number;
  label: string;
  accent?: string;
  highlight?: boolean;
  children?: React.ReactNode;
}) {
  return (
    <div
      style={{
        position: 'absolute',
        top,
        left: '50%',
        transform: 'translateX(-50%)',
        width: 1020,
        background: tokens.bg1,
        border: `2px solid ${highlight ? tokens.rose : tokens.border}`,
        borderRadius: 26,
        padding: '24px 32px 32px 32px',
        display: 'flex',
        flexDirection: 'column',
        gap: 22,
        boxShadow: highlight ? `0 0 50px ${tokens.rose}55` : 'none',
      }}
    >
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
      {children}
    </div>
  );
}

function PillRow({ listening, elapsedMs }: { listening: boolean; elapsedMs: number }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'center', padding: '12px 0' }}>
      <div style={{ transform: 'scale(2.0)' }}>
        <BarDriver state={listening ? 'listening' : 'idle'} elapsedMs={elapsedMs} />
      </div>
    </div>
  );
}

function Output({
  text,
  highlight,
  highlightColor = tokens.rose,
}: {
  text: string;
  highlight?: string;
  highlightColor?: string;
}) {
  if (!text) {
    return (
      <div style={{ minHeight: 130, fontFamily: fonts.sans, fontSize: 40, color: tokens.fgDim, lineHeight: 1.4, fontStyle: 'italic' }}>
        Listening…
      </div>
    );
  }
  if (!highlight || !text.includes(highlight)) {
    return (
      <div style={{ minHeight: 130, fontFamily: fonts.sans, fontSize: 40, color: tokens.fgHi, lineHeight: 1.4, fontWeight: 500 }}>
        {text}
      </div>
    );
  }
  const i = text.indexOf(highlight);
  return (
    <div style={{ minHeight: 130, fontFamily: fonts.sans, fontSize: 40, color: tokens.fgHi, lineHeight: 1.4, fontWeight: 500 }}>
      {text.slice(0, i)}
      <span
        style={{
          background: `${highlightColor}33`,
          color: highlightColor,
          padding: '4px 14px',
          borderRadius: 8,
          fontWeight: 700,
        }}
      >
        {highlight}
      </span>
      {text.slice(i + highlight.length)}
    </div>
  );
}

function VocabularyTeach({ showChip }: { showChip: boolean }) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);
  const slide = tween(frame, f(5.0), f(5.7), 80, 0);
  const fade = tween(frame, f(5.0), f(5.7), 0, 1);
  return (
    <div
      style={{
        position: 'absolute',
        top: 920,
        left: '50%',
        transform: `translate(-50%, ${slide}px)`,
        opacity: fade,
        background: tokens.bg2,
        border: `1px solid ${tokens.accent}66`,
        borderRadius: 18,
        padding: '20px 32px',
        display: 'flex',
        alignItems: 'center',
        gap: 18,
        fontFamily: fonts.sans,
      }}
    >
      <div style={{ color: tokens.muted, fontSize: 24, fontFamily: fonts.mono, letterSpacing: '0.12em', textTransform: 'uppercase', fontWeight: 600 }}>
        Vocabulary
      </div>
      <div style={{ width: 1, height: 30, background: tokens.border }} />
      <div
        style={{
          color: tokens.accent,
          fontSize: 36,
          fontWeight: 700,
          padding: '8px 22px',
          borderRadius: 999,
          background: showChip ? `${tokens.accent}26` : 'transparent',
          border: showChip ? `1px solid ${tokens.accent}77` : 'none',
        }}
      >
        + Anthropic
      </div>
    </div>
  );
}
