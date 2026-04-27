/**
 * ad-07-teach-vocabulary — 12s. Two takes of the same sentence side-
 * by-side. First take: the proper noun "Anthropic" gets transcribed
 * as "an throw pick" (the natural mistake). Cursor adds the word to
 * a tiny vocabulary chip stack. Second take of the same sentence:
 * the proper noun lands correctly.
 *
 * Different angle from FeatureVocabulary (which is a settings panel
 * walkthrough). This is the moment-of-fix demo — the BEFORE and
 * AFTER, side by side.
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

const WRONG = "We need to ship the an throw pick integration before launch.";
const RIGHT = "We need to ship the Anthropic integration before launch.";

export function Ad07TeachVocabulary() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Take 1: 0.5s pill listening, 2.0s output, 1.0s "wrong" highlight, 1.5s teach.
  const takeOnePill = frame >= f(0.5) && frame < f(2.5);
  const takeOneText = frame >= f(2.5) ? humanType(WRONG, frame, f(2.5), fps, 0x111) : '';
  const showTeach = frame >= f(5.5);
  const showVocabChip = frame >= f(6.5);

  // Take 2: starts at 8.0s. 0.5s pill, output appears in full at 9.5s.
  const takeTwoPill = frame >= f(8.0) && frame < f(9.5);
  const takeTwoText = frame >= f(9.5) ? humanType(RIGHT, frame, f(9.5), fps, 0x222) : '';

  const elapsed1 = takeOnePill ? ((frame - f(0.5)) / fps) * 1000 : 0;
  const elapsed2 = takeTwoPill ? ((frame - f(8.0)) / fps) * 1000 : 0;

  return (
    <SocialFrame variant="canvas">
      <Eyebrow text="Teach it your jargon" />

      {/* TAKE 1 panel */}
      <Panel top={200} label="Take 1" showHighlight={frame >= f(5.0) && frame < f(7.5)}>
        <PillRow listening={takeOnePill} elapsedMs={elapsed1} />
        <Output text={takeOneText} highlight={frame >= f(5.0) ? "an throw pick" : undefined} />
      </Panel>

      {/* Vocabulary teach chip flies up between panels */}
      {showTeach && <VocabularyTeach showChip={showVocabChip} />}

      {/* TAKE 2 panel */}
      <Panel top={1100} label="Take 2 — after teaching" accent={tokens.accent}>
        <PillRow listening={takeTwoPill} elapsedMs={elapsed2} />
        <Output text={takeTwoText} highlight={takeTwoText.includes('Anthropic') ? 'Anthropic' : undefined} highlightColor={tokens.accent} />
      </Panel>

      <Captions
        cues={[
          { text: 'It misheard "Anthropic".', at: 4.5, until: 7.0 },
          { text: 'Teach it once.', at: 7.0, until: 8.5 },
          { text: 'Now it gets it right.', at: 9.6, until: 11.5 },
        ]}
        bottom={120}
      />

      <EndCard tagline="Teach it your jargon. Once." startAt={11.5} />
    </SocialFrame>
  );
}

function Eyebrow({ text }: { text: string }) {
  return (
    <div
      style={{
        position: 'absolute',
        top: 100,
        left: '50%',
        transform: 'translateX(-50%)',
        fontFamily: fonts.mono,
        fontSize: 18,
        letterSpacing: '0.18em',
        textTransform: 'uppercase',
        color: tokens.muted,
      }}
    >
      {text}
    </div>
  );
}

function Panel({
  top,
  label,
  accent = tokens.muted,
  showHighlight,
  children,
}: {
  top: number;
  label: string;
  accent?: string;
  showHighlight?: boolean;
  children?: React.ReactNode;
}) {
  return (
    <div
      style={{
        position: 'absolute',
        top,
        left: '50%',
        transform: 'translateX(-50%)',
        width: 940,
        background: tokens.bg1,
        border: `1px solid ${showHighlight ? tokens.rose : tokens.border}`,
        borderRadius: 22,
        padding: '20px 26px 26px 26px',
        display: 'flex',
        flexDirection: 'column',
        gap: 18,
        boxShadow: showHighlight ? `0 0 40px ${tokens.rose}33` : 'none',
      }}
    >
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
      {children}
    </div>
  );
}

function PillRow({ listening, elapsedMs }: { listening: boolean; elapsedMs: number }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'center', padding: '8px 0' }}>
      <div style={{ transform: 'scale(1.4)' }}>
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
      <div
        style={{
          minHeight: 110,
          fontFamily: fonts.sans,
          fontSize: 26,
          color: tokens.fgDim,
          lineHeight: 1.45,
        }}
      >
        Listening…
      </div>
    );
  }
  if (!highlight) {
    return (
      <div style={{ minHeight: 110, fontFamily: fonts.sans, fontSize: 26, color: tokens.fgHi, lineHeight: 1.45 }}>
        {text}
      </div>
    );
  }
  const i = text.indexOf(highlight);
  if (i < 0) {
    return (
      <div style={{ minHeight: 110, fontFamily: fonts.sans, fontSize: 26, color: tokens.fgHi, lineHeight: 1.45 }}>
        {text}
      </div>
    );
  }
  return (
    <div style={{ minHeight: 110, fontFamily: fonts.sans, fontSize: 26, color: tokens.fgHi, lineHeight: 1.45 }}>
      {text.slice(0, i)}
      <span
        style={{
          background: `${highlightColor}22`,
          color: highlightColor,
          padding: '2px 8px',
          borderRadius: 6,
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
  const slide = tween(frame, f(5.5), f(6.5), 80, 0);
  const fade = tween(frame, f(5.5), f(6.5), 0, 1);
  return (
    <div
      style={{
        position: 'absolute',
        top: 920,
        left: '50%',
        transform: `translate(-50%, ${slide}px)`,
        opacity: fade,
        background: tokens.bg2,
        border: `1px solid ${tokens.accent}55`,
        borderRadius: 14,
        padding: '14px 22px',
        display: 'flex',
        alignItems: 'center',
        gap: 14,
        fontFamily: fonts.sans,
      }}
    >
      <div style={{ color: tokens.muted, fontSize: 18, fontFamily: fonts.mono, letterSpacing: '0.1em', textTransform: 'uppercase' }}>
        Vocabulary
      </div>
      <div style={{ width: 1, height: 22, background: tokens.border }} />
      <div
        style={{
          color: tokens.accent,
          fontSize: 22,
          fontWeight: 600,
          padding: '4px 14px',
          borderRadius: 999,
          background: showChip ? `${tokens.accent}1f` : 'transparent',
          border: showChip ? `1px solid ${tokens.accent}66` : 'none',
        }}
      >
        + Anthropic
      </div>
    </div>
  );
}
