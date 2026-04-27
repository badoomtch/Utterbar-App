/**
 * ad-11-no-account — 13s. v3: each sign-in wall holds 1.0s (was
 * fine), then Utterbar takes over. Transcript SNAPS in (was
 * typewritten) and holds for ~3s. Pill scale 2.6→2.0.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { SignInWall } from '../scenes/SignInWall';

const COMPETITORS = [
  'Cloud Dictation Co.',
  'EnterpriseVoice',
  'StreamingTranscribe',
];

const BEAT = 1.0;
const TRANSCRIPT = 'Hi Marcus, the deck will be ready by five.';

export function Ad11NoAccount() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const showCompetitorWalls = frame < f(COMPETITORS.length * BEAT);

  const utterbarStart = f(COMPETITORS.length * BEAT + 0.3);
  const showUtterbar = frame >= utterbarStart;
  const utterFade = tween(frame, utterbarStart, utterbarStart + 10, 0, 1);

  const pillStart = utterbarStart;
  const pillState: 'idle' | 'listening' | 'processing' =
    frame >= pillStart && frame < pillStart + f(2.0) ? 'listening'
    : frame >= pillStart + f(2.0) && frame < pillStart + f(2.4) ? 'processing'
    : 'idle';
  const pillElapsed = pillState === 'listening' ? ((frame - pillStart) / fps) * 1000 : 0;

  const transcriptStart = pillStart + f(2.4);
  const showTranscript = frame >= transcriptStart;

  const wallOps = COMPETITORS.map((_, i) => {
    const start = f(i * BEAT);
    const end = f((i + 1) * BEAT);
    return Math.min(
      tween(frame, start, start + 4, 0, 1),
      tween(frame, end - 6, end, 1, 0),
    );
  });

  return (
    <SocialFrame variant="canvas">
      {showCompetitorWalls && (
        <>
          {COMPETITORS.map((c, i) => (
            <div key={c} style={{ opacity: wallOps[i] }}>
              <SignInWall product={c} />
            </div>
          ))}
        </>
      )}

      {showUtterbar && (
        <div style={{ opacity: utterFade }}>
          <div style={{ position: 'absolute', top: 240, left: '50%', transform: 'translateX(-50%) scale(2.0)', transformOrigin: 'top center' }}>
            <BarDriver state={pillState} elapsedMs={pillElapsed} />
          </div>

          <div
            style={{
              position: 'absolute',
              top: 720,
              left: '50%',
              transform: 'translateX(-50%)',
              width: 1020,
              minHeight: 320,
              padding: '40px 44px',
              background: tokens.bg1,
              border: `1px solid ${tokens.border}`,
              borderRadius: 26,
              color: tokens.fgHi,
              fontFamily: fonts.sans,
              fontSize: 50,
              lineHeight: 1.4,
              fontWeight: 600,
            }}
          >
            {showTranscript ? TRANSCRIPT : (
              <span style={{ color: tokens.fgDim, fontStyle: 'italic', fontSize: 38 }}>
                Just dictate. No account needed.
              </span>
            )}
          </div>
        </div>
      )}

      <Captions
        cues={[
          { text: 'Sign in.',    at: 0.2, until: 0.9 },
          { text: 'Sign in.',    at: 1.2, until: 1.9 },
          { text: 'Sign in.',    at: 2.2, until: 2.9 },
          { text: 'No account.', at: 3.5, until: 8.4 },
        ]}
      />

      <EndCard tagline="No account. Just dictate." startAt={8.5} />
    </SocialFrame>
  );
}
