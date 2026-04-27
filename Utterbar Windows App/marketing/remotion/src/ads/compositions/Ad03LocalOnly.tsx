/**
 * ad-03-local-only — 10s. The privacy ad. Network monitor sits at the
 * top reading "0 KB" the whole time the user dictates a long, sensitive-
 * sounding message. The visual proof IS the ad.
 *
 * Different angle from FeatureLocalPrivacy (which is a settings panel
 * focused on the toggle). This is "watch the network counter while I
 * say something private — nothing moves."
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
import { NetworkMonitor } from '../scenes/NetworkMonitor';

const PRIVATE_MESSAGE =
  "Q2 numbers are off by about £180k against forecast — Marcus thinks it's the new ad spend, but we should not flag it before the board call on Friday.";

export function Ad03LocalOnly() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Pill arc — nice long listening window so the network counter
  // gets to sit at zero for the entire act.
  let pillState: 'idle' | 'listening' | 'processing' = 'idle';
  if (frame >= f(0.5) && frame < f(1.0)) pillState = 'idle';
  else if (frame >= f(1.0) && frame < f(6.0)) pillState = 'listening';
  else if (frame >= f(6.0) && frame < f(6.6)) pillState = 'processing';

  const elapsedMs = pillState === 'listening' ? ((frame - f(1.0)) / fps) * 1000 : 0;

  // Message lands all at once at 6.6s.
  const showMessage = frame >= f(6.6);
  const transcript = showMessage
    ? humanType(PRIVATE_MESSAGE, frame, f(6.6), fps, 0xc0de)
    : '';

  // The counter stays at 0. That's the whole point.
  const bytes = 0;

  // Eyebrow flickers in.
  const eyebrowOp = tween(frame, f(0.0), f(0.4), 0, 1);

  return (
    <SocialFrame variant="canvas">
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
          opacity: eyebrowOp,
        }}
      >
        Live network monitor · this device
      </div>

      {/* Network monitor up top */}
      <div
        style={{
          position: 'absolute',
          top: 160,
          left: '50%',
          transform: 'translateX(-50%)',
        }}
      >
        <NetworkMonitor bytes={bytes} />
      </div>

      {/* Pill in the middle */}
      <div
        style={{
          position: 'absolute',
          top: 380,
          left: '50%',
          transform: 'translateX(-50%) scale(2.6)',
          transformOrigin: 'top center',
        }}
      >
        <BarDriver state={pillState} elapsedMs={elapsedMs} />
      </div>

      {/* Transcribed text appears below, dictation-style */}
      <div
        style={{
          position: 'absolute',
          top: 720,
          left: '50%',
          transform: 'translateX(-50%)',
          width: 920,
          minHeight: 360,
          padding: '32px 36px',
          background: tokens.bg1,
          border: `1px solid ${tokens.border}`,
          borderRadius: 22,
          color: tokens.fgHi,
          fontSize: 30,
          lineHeight: 1.45,
          fontFamily: fonts.sans,
        }}
      >
        {transcript}
        {!showMessage && (
          <div style={{ color: tokens.fgDim, fontSize: 22, fontStyle: 'italic' }}>
            Listening… nothing leaves this machine.
          </div>
        )}
      </div>

      <Captions
        cues={[
          { text: 'Watch the network.', at: 0.5, until: 4.0 },
          { text: 'Stays at zero.', at: 4.2, until: 6.5 },
          { text: 'Audio never leaves your PC.', at: 6.7, until: 9.0 },
        ]}
        bottom={120}
      />

      <EndCard tagline="Local-only dictation. For Windows." startAt={9.5} />
    </SocialFrame>
  );
}
