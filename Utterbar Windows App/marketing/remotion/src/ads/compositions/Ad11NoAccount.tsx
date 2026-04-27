/**
 * ad-11-no-account — 11s. Three competitor-shaped sign-in walls flash
 * by, each with a stamp-in "Create your account" friction. Then the
 * Utterbar pill appears with no friction — install, hotkey, dictate.
 *
 * Different angle from anything in the existing pack: addresses the
 * "I'm so tired of accounts" objection — leans into BRIEF's no-account
 * stance as the differentiator.
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
import { humanType } from '../frame/humanType';

const COMPETITORS = [
  'Cloud Dictation Co.',
  'EnterpriseVoice',
  'StreamingTranscribe',
];

const BEAT = 1.4;

export function Ad11NoAccount() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Show three sign-in walls back-to-back
  const wallIdx = Math.min(COMPETITORS.length - 1, Math.floor(frame / fps / BEAT));
  const showCompetitorWalls = frame < f(COMPETITORS.length * BEAT);

  // Then Utterbar takes over: pill + dictation
  const utterbarStart = f(COMPETITORS.length * BEAT + 0.3);
  const showUtterbar = frame >= utterbarStart;
  const utterFade = tween(frame, utterbarStart, utterbarStart + 14, 0, 1);

  const pillStart = utterbarStart;
  const pillState: 'idle' | 'listening' | 'processing' =
    frame >= pillStart && frame < pillStart + f(2.4)
      ? 'listening'
      : frame >= pillStart + f(2.4) && frame < pillStart + f(3.0)
      ? 'processing'
      : 'idle';
  const pillElapsed = pillState === 'listening' ? ((frame - pillStart) / fps) * 1000 : 0;

  const transcriptStart = pillStart + f(3.0);
  const transcript = frame >= transcriptStart
    ? humanType('Hi Marcus, the deck will be ready by five.', frame, transcriptStart, fps, 0xacab)
    : '';

  const wallOps = COMPETITORS.map((_, i) => {
    const start = f(i * BEAT);
    const end = f((i + 1) * BEAT);
    return Math.min(
      tween(frame, start, start + 6, 0, 1),
      tween(frame, end - 8, end, 1, 0),
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
            Or, you know.
          </div>

          <div
            style={{
              position: 'absolute',
              top: 320,
              left: '50%',
              transform: 'translateX(-50%) scale(2.4)',
              transformOrigin: 'top center',
            }}
          >
            <BarDriver state={pillState} elapsedMs={pillElapsed} />
          </div>

          <div
            style={{
              position: 'absolute',
              top: 720,
              left: '50%',
              transform: 'translateX(-50%)',
              width: 920,
              minHeight: 200,
              padding: '32px 36px',
              background: tokens.bg1,
              border: `1px solid ${tokens.border}`,
              borderRadius: 22,
              color: tokens.fgHi,
              fontFamily: fonts.sans,
              fontSize: 30,
              lineHeight: 1.45,
            }}
          >
            {transcript || (
              <span style={{ color: tokens.fgDim, fontStyle: 'italic' }}>
                Just dictate. No account, no email, no nothing.
              </span>
            )}
          </div>
        </div>
      )}

      <Captions
        cues={[
          { text: 'Sign in.', at: 0.4, until: 1.4 },
          { text: 'Sign in.', at: 1.6, until: 2.8 },
          { text: 'Sign in.', at: 3.0, until: 4.2 },
          { text: 'No account needed.', at: 5.0, until: 9.5 },
        ]}
        bottom={120}
      />

      <EndCard tagline="No account. Just dictate." startAt={9.5} />
    </SocialFrame>
  );
}
