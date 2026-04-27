/**
 * ad-03-local-only — 13s. v3 fixes the typewriter outcome — the
 * transcript SNAPS in fully formed at 4.9s and holds for 3.6s
 * before the end card. Pill scale 2.6→2.0.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { NetworkMonitor } from '../scenes/NetworkMonitor';

const PRIVATE = "Q2 numbers off by £180k against forecast — don't flag before Friday's board call.";

export function Ad03LocalOnly() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  let pillState: 'idle' | 'listening' | 'processing' = 'idle';
  if (frame >= f(0.2) && frame < f(4.4)) pillState = 'listening';
  else if (frame >= f(4.4) && frame < f(4.9)) pillState = 'processing';
  const elapsedMs = pillState === 'listening' ? ((frame - f(0.2)) / fps) * 1000 : 0;

  // Transcript snaps in at 4.9 (not typewritten — that's how dictation
  // results actually appear) and sits readable for ~3.6s.
  const showTranscript = frame >= f(4.9);

  return (
    <SocialFrame variant="canvas">
      <div style={{ position: 'absolute', top: 100, left: '50%', transform: 'translateX(-50%)' }}>
        <NetworkMonitor bytes={0} />
      </div>

      <div style={{ position: 'absolute', top: 600, left: '50%', transform: 'translateX(-50%) scale(2.0)', transformOrigin: 'top center' }}>
        <BarDriver state={pillState} elapsedMs={elapsedMs} />
      </div>

      <div
        style={{
          position: 'absolute',
          top: 920,
          left: '50%',
          transform: 'translateX(-50%)',
          width: 1020,
          minHeight: 460,
          padding: '36px 40px',
          background: tokens.bg1,
          border: `1px solid ${tokens.border}`,
          borderRadius: 26,
          color: tokens.fgHi,
          fontSize: 44,
          lineHeight: 1.4,
          fontFamily: fonts.sans,
          fontWeight: 500,
        }}
      >
        {showTranscript ? PRIVATE : (
          <div style={{ color: tokens.fgDim, fontSize: 36, fontStyle: 'italic' }}>
            Listening… nothing leaves this machine.
          </div>
        )}
      </div>

      <Captions
        cues={[
          { text: 'Watch the network.', at: 0.2, until: 3.0 },
          { text: 'Stays at zero.',     at: 3.2, until: 4.6 },
          { text: 'Audio stays local.', at: 5.0, until: 8.4 },
        ]}
      />

      <EndCard tagline="Local-only dictation." startAt={8.5} />
    </SocialFrame>
  );
}
