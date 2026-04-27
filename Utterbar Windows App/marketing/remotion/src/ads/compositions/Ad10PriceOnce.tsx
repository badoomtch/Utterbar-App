/**
 * ad-10-price-once — 9s. Addresses the unspoken anxiety around SaaS
 * pricing. Cycles through three price options viewers brace for, with
 * the first two crossed out. Lands on "£39.99 once" — cyan, no slash,
 * the only one not struck through.
 *
 * Different angle from anything in the existing pack: this targets the
 * subscription-fatigue objection directly and pre-empts it.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { PriceLine, PRICE_VARIANTS } from '../scenes/PriceLine';

const BEAT = 1.6;

export function Ad10PriceOnce() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const ops = PRICE_VARIANTS.map((_, i) => {
    const start = f(0.5 + i * BEAT);
    const end = i === PRICE_VARIANTS.length - 1 ? f(0.5 + (i + 1.4) * BEAT) : f(0.5 + (i + 1) * BEAT);
    return Math.min(
      tween(frame, start, start + 8, 0, 1),
      tween(frame, end - 10, end, 1, 0),
    );
  });
  // Last variant lingers — override its trailing fade so it stays visible
  // through to the end card.
  ops[ops.length - 1] = tween(frame, f(0.5 + (PRICE_VARIANTS.length - 1) * BEAT), f(0.5 + (PRICE_VARIANTS.length - 1) * BEAT) + 8, 0, 1);

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
        How much does this cost?
      </div>

      <div style={{ position: 'absolute', top: 700, left: 0, right: 0, height: 240 }}>
        {PRICE_VARIANTS.map((v, i) => (
          <PriceLine key={i} variant={v} opacity={ops[i]} />
        ))}
      </div>

      <Captions
        cues={[
          { text: 'Not monthly.', at: 0.6, until: 2.0 },
          { text: 'Not yearly.', at: 2.2, until: 3.6 },
          { text: 'Once. That’s it.', at: 3.8, until: 6.4 },
        ]}
        bottom={120}
      />

      <EndCard tagline="One-time licence. £39.99." startAt={6.5} />
    </SocialFrame>
  );
}
