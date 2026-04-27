/**
 * ad-10-price-once — 11s. v3: each price holds 1.4s, the cyan
 * "once" sits for 2s before end card. End card 4.5s.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tween } from '../../util/anim';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { PriceLine, PRICE_VARIANTS } from '../scenes/PriceLine';

const BEAT = 1.4;

export function Ad10PriceOnce() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const ops = PRICE_VARIANTS.map((_, i) => {
    const start = f(0.2 + i * BEAT);
    if (i === PRICE_VARIANTS.length - 1) {
      return tween(frame, start, start + 8, 0, 1);
    }
    const end = f(0.2 + (i + 1) * BEAT);
    return Math.min(
      tween(frame, start, start + 6, 0, 1),
      tween(frame, end - 8, end, 1, 0),
    );
  });

  return (
    <SocialFrame variant="canvas">
      <div style={{ position: 'absolute', top: 0, bottom: 0, left: 0, right: 0 }}>
        {PRICE_VARIANTS.map((v, i) => (
          <PriceLine key={i} variant={v} opacity={ops[i]} />
        ))}
      </div>

      <Captions
        cues={[
          { text: 'Not monthly.', at: 0.2, until: 1.5 },
          { text: 'Not yearly.',  at: 1.7, until: 2.9 },
          { text: 'Once.',        at: 3.1, until: 6.4 },
        ]}
      />

      <EndCard tagline="One-time licence. £39.99." startAt={6.5} />
    </SocialFrame>
  );
}
