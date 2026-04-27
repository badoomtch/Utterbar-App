/**
 * ad-05-thirty-languages — 13s. v3: each language holds for 1.2s
 * (was 1.0s, too fast to read non-English samples), chip strip
 * holds 2s before end card. Pill scale 2.4→2.0.
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

const BEAT = 1.2;

export function Ad05ThirtyLanguages() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const elapsedMs = (frame / fps) * 1000;

  const langOps = LANGS.map((_, i) => {
    const start = f(i * BEAT);
    const end = f((i + 1) * BEAT);
    return Math.min(
      tween(frame, start, start + 4, 0, 1),
      tween(frame, end - 5, end, 1, 0),
    );
  });

  const cycleEnd = LANGS.length * BEAT; // 7.2s

  return (
    <SocialFrame variant="canvas">
      <div style={{ position: 'absolute', top: 100, left: '50%', transform: 'translateX(-50%) scale(2.0)', transformOrigin: 'top center' }}>
        <BarDriver state="listening" elapsedMs={elapsedMs} />
      </div>

      <div style={{ position: 'absolute', top: 540, left: 0, right: 0, height: 540 }}>
        {LANGS.map((lang, i) => (
          <LanguageBadge key={lang.code} lang={lang} opacity={langOps[i]} />
        ))}
      </div>

      {frame >= f(cycleEnd) && <AndMoreStrip startFrame={f(cycleEnd)} />}

      <Captions
        cues={[
          { text: '30+ languages.',  at: 0.2, until: 5.6 },
          { text: 'All on your PC.', at: 5.8, until: 8.5 },
        ]}
      />

      <EndCard tagline="Speak any language." startAt={8.5} />
    </SocialFrame>
  );
}

function AndMoreStrip({ startFrame }: { startFrame: number }) {
  const frame = useCurrentFrame();
  const op = tween(frame, startFrame, startFrame + 8, 0, 1);
  const chips = ['IT', 'PT', 'NL', 'PL', 'TR', 'RU', 'HI', 'KO', 'ZH', 'SV', '+24'];
  return (
    <div
      style={{
        position: 'absolute',
        top: 1240,
        left: 0,
        right: 0,
        display: 'flex',
        flexWrap: 'wrap',
        justifyContent: 'center',
        gap: 16,
        padding: '0 60px',
        opacity: op,
      }}
    >
      {chips.map((c, i) => (
        <div
          key={i}
          style={{
            padding: '14px 26px',
            fontFamily: fonts.mono,
            fontSize: 36,
            color: c.startsWith('+') ? tokens.accent : tokens.muted,
            background: tokens.bg2,
            border: `1px solid ${tokens.border}`,
            borderRadius: 999,
            letterSpacing: '0.08em',
            fontWeight: 600,
          }}
        >
          {c}
        </div>
      ))}
    </div>
  );
}
