/**
 * ad-05-thirty-languages — 12s. Speed-cycle through 6 languages, each
 * with the SAME spoken thought rendered in the native script. The pill
 * sits centred and steady; the language and the transcribed sentence
 * crossfade beneath it every ~1.6 seconds. End frame holds on a stack
 * of "and 24 more" small chips.
 *
 * Different angle from FeatureMultilingual (which is one slow tour).
 * This is a snappy demo: same pill, same hotkey, six languages in 10s.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { LANGS, LanguageBadge } from '../scenes/LanguageBadge';

const BEAT = 1.4; // seconds per language

export function Ad05ThirtyLanguages() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Pill always listening throughout the cycle.
  const elapsedMs = (frame / fps) * 1000;

  // Index of the active language by frame.
  const idx = Math.min(LANGS.length - 1, Math.floor(frame / fps / BEAT));

  // Per-language opacity: each badge fades in at its window start and out
  // at its window end (with a small overlap for crossfade).
  const langOps = LANGS.map((_, i) => {
    const start = f(i * BEAT);
    const end = f((i + 1) * BEAT);
    return Math.min(
      tween(frame, start, start + 6, 0, 1),
      tween(frame, end - 8, end, 1, 0),
    );
  });

  const cycleEnd = LANGS.length * BEAT;

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
        Same hotkey. Any language.
      </div>

      <div
        style={{
          position: 'absolute',
          top: 170,
          left: '50%',
          transform: 'translateX(-50%) scale(2.0)',
          transformOrigin: 'top center',
        }}
      >
        <BarDriver state="listening" elapsedMs={elapsedMs} />
      </div>

      {/* Stack of language badges, one is visible at a time via opacity */}
      <div
        style={{
          position: 'absolute',
          top: 540,
          left: 0,
          right: 0,
          height: 360,
        }}
      >
        {LANGS.map((lang, i) => (
          <LanguageBadge key={lang.code} lang={lang} opacity={langOps[i]} />
        ))}
      </div>

      {/* "And 24 more" chip strip appears at end of cycle */}
      {frame >= f(cycleEnd) && (
        <AndMoreStrip startFrame={f(cycleEnd)} />
      )}

      <Captions
        cues={[
          { text: '30+ languages.', at: 0.4, until: 5.0 },
          { text: 'None of them leave your PC.', at: 5.2, until: 9.0 },
        ]}
        bottom={120}
      />

      <EndCard tagline="Speak any language. Local." startAt={9.5} />
    </SocialFrame>
  );
}

function AndMoreStrip({ startFrame }: { startFrame: number }) {
  const frame = useCurrentFrame();
  const op = tween(frame, startFrame, startFrame + 12, 0, 1);
  const chips = ['IT', 'PT', 'NL', 'PL', 'TR', 'RU', 'HI', 'KO', 'ZH', 'SV', '+24'];
  return (
    <div
      style={{
        position: 'absolute',
        top: 700,
        left: 0,
        right: 0,
        display: 'flex',
        flexWrap: 'wrap',
        justifyContent: 'center',
        gap: 12,
        padding: '0 60px',
        opacity: op,
      }}
    >
      {chips.map((c, i) => (
        <div
          key={i}
          style={{
            padding: '10px 18px',
            fontFamily: fonts.mono,
            fontSize: 22,
            color: c.startsWith('+') ? tokens.accent : tokens.muted,
            background: tokens.bg2,
            border: `1px solid ${tokens.border}`,
            borderRadius: 999,
            letterSpacing: '0.08em',
          }}
        >
          {c}
        </div>
      ))}
    </div>
  );
}
