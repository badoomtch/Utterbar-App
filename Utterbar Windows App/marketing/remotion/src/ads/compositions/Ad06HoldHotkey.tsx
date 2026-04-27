/**
 * ad-06-hold-hotkey — 8s. The "hotkey is the instrument" beat. The
 * Ctrl+Space chord is shown front-and-centre; pressing it summons the
 * pill, releasing it dismisses. Three quick reps — the third spawns
 * a clean transcription that holds on screen.
 *
 * Different angle from anything in the existing pack: this ad treats
 * the hotkey itself as the protagonist. Useful as a 6-second loop on
 * the website hero.
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
import { HotkeyBadge } from '../scenes/HotkeyBadge';

const REPS = [
  { pressAt: 0.6, releaseAt: 1.4 },
  { pressAt: 1.8, releaseAt: 2.6 },
  { pressAt: 3.0, releaseAt: 5.5 },
];

const TRANSCRIPT = "Coffee at half past — bring the laptop.";

export function Ad06HoldHotkey() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const rep = REPS.find((r) => frame >= f(r.pressAt) && frame < f(r.releaseAt));
  const pressed = !!rep;
  const inThirdRep = frame >= f(REPS[2].pressAt) && frame < f(REPS[2].releaseAt);

  const pillState: 'idle' | 'listening' = pressed ? 'listening' : 'idle';
  const elapsedMs = pressed && rep ? ((frame - f(rep.pressAt)) / fps) * 1000 : 0;

  // Show pill only while pressed; on third rep, transcript appears below
  // after release.
  const showTranscript = frame >= f(REPS[2].releaseAt);
  const transcript = showTranscript
    ? humanType(TRANSCRIPT, frame, f(REPS[2].releaseAt + 0.1), fps, 0xfeed)
    : '';

  return (
    <SocialFrame variant="canvas">
      {/* Hotkey badge — centred, the protagonist */}
      <div
        style={{
          position: 'absolute',
          top: 360,
          left: '50%',
          transform: `translateX(-50%) scale(${pressed ? 1.06 : 1})`,
          transition: 'none',
        }}
      >
        <HotkeyBadge pressed={pressed} />
      </div>

      {/* Pill above the badge — only when pressed */}
      {pressed && (
        <div
          style={{
            position: 'absolute',
            top: 200,
            left: '50%',
            transform: 'translateX(-50%) scale(1.8)',
            transformOrigin: 'top center',
          }}
        >
          <BarDriver state={pillState} elapsedMs={elapsedMs} />
        </div>
      )}

      {/* Transcript that lands after the third release */}
      {showTranscript && (
        <div
          style={{
            position: 'absolute',
            top: 580,
            left: '50%',
            transform: 'translateX(-50%)',
            width: 920,
            padding: '24px 30px',
            background: tokens.bg1,
            border: `1px solid ${tokens.accent}55`,
            borderRadius: 18,
            color: tokens.fgHi,
            fontSize: 38,
            lineHeight: 1.4,
            fontFamily: fonts.sans,
            textAlign: 'center',
            opacity: tween(frame, f(REPS[2].releaseAt), f(REPS[2].releaseAt + 0.3), 0, 1),
          }}
        >
          {transcript || '…'}
        </div>
      )}

      <Captions
        cues={[
          { text: 'Hold to talk.', at: 0.3, until: 2.8 },
          { text: 'Release to send.', at: 5.6, until: 7.0 },
        ]}
        bottom={140}
      />

      <EndCard tagline="One key. Anywhere on Windows." startAt={7.0} />
    </SocialFrame>
  );
}
