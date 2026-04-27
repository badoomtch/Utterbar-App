/**
 * ad-06-hold-hotkey — 11s. v3: transcript SNAPS in (was typewritten),
 * holds for 2.6s before end card. Pill scale 2.4→2.0.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { HotkeyBadge } from '../scenes/HotkeyBadge';

const REPS = [
  { pressAt: 0.4, releaseAt: 0.95 },
  { pressAt: 1.25, releaseAt: 1.8 },
  { pressAt: 2.1, releaseAt: 4.4 },
];

const TRANSCRIPT = "Coffee at half past — bring the laptop.";

export function Ad06HoldHotkey() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const rep = REPS.find((r) => frame >= f(r.pressAt) && frame < f(r.releaseAt));
  const pressed = !!rep;
  const elapsedMs = pressed && rep ? ((frame - f(rep.pressAt)) / fps) * 1000 : 0;

  // Transcript snaps in fully formed at 4.5 (just after the third
  // release at 4.4) and holds until end card at 7.0 = 2.5s of payoff.
  const showTranscript = frame >= f(4.5);

  return (
    <SocialFrame variant="canvas">
      {pressed && (
        <div style={{ position: 'absolute', top: 200, left: '50%', transform: 'translateX(-50%) scale(2.0)', transformOrigin: 'top center' }}>
          <BarDriver state="listening" elapsedMs={elapsedMs} />
        </div>
      )}

      <div style={{ position: 'absolute', top: 720, left: '50%', transform: `translateX(-50%) scale(${pressed ? 1.04 : 1})` }}>
        <HotkeyBadge pressed={pressed} />
      </div>

      {showTranscript && (
        <div
          style={{
            position: 'absolute',
            top: 1080,
            left: '50%',
            transform: 'translateX(-50%)',
            width: 1020,
            padding: '32px 40px',
            background: tokens.bg1,
            border: `1px solid ${tokens.accent}66`,
            borderRadius: 22,
            color: tokens.fgHi,
            fontSize: 50,
            lineHeight: 1.35,
            fontFamily: fonts.sans,
            fontWeight: 600,
            textAlign: 'center',
            opacity: tween(frame, f(4.5), f(4.7), 0, 1),
          }}
        >
          {TRANSCRIPT}
        </div>
      )}

      <Captions
        cues={[
          { text: 'Hold to talk.',    at: 0.1, until: 1.95 },
          { text: 'Release to send.', at: 4.6, until: 6.9 },
        ]}
      />

      <EndCard tagline="One key. Anywhere on Windows." startAt={7.0} />
    </SocialFrame>
  );
}
