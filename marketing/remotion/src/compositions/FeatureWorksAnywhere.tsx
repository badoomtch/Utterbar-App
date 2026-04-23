/**
 * feature-works-anywhere — 10 s sequence showing the floating pill
 * dictating text into THREE different Windows apps, one after another:
 * browser, chat, editor. The pill stays pinned to the top of the screen;
 * the host window behind it transforms each beat.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import { HostWindow } from '../chrome/HostWindow';
import { WindowsDesktop } from '../chrome/WindowsDesktop';
import { pulse, tween, typewriter } from '../util/anim';

const BEATS = [
  {
    variant: 'browser' as const,
    title: 'Gmail · Compose',
    transcript:
      "Thanks for sending this through — I've had a look and the dates on slide twelve should be the 9th of June, not the 16th.",
    accent: '#ea4335',
  },
  {
    variant: 'chat' as const,
    title: 'Slack · #release',
    transcript:
      "Quick heads up — 0.6.1 is cut and the installer is up on R2. Auto-update toast lands on launch.",
    accent: '#7c3aed',
  },
  {
    variant: 'editor' as const,
    title: 'Obsidian · Research Notes',
    transcript:
      "Feature hypothesis: users who complete onboarding in under two minutes convert at 3× the rate of users who drop off between steps four and six.",
    accent: '#a855f7',
  },
];

export function FeatureWorksAnywhere() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);
  const beatLen = f(3.33); // 3 beats across 10 s

  const beatIdx = Math.min(2, Math.floor(frame / beatLen));
  const beat = BEATS[beatIdx];
  const localFrame = frame - beatIdx * beatLen;

  // Within each beat: 0.2 s idle → 1.6 s listening → 0.3 s processing → 1.2 s result
  const listenStart = f(0.2);
  const listenEnd = f(1.8);
  const procEnd = f(2.1);

  let barState: 'idle' | 'listening' | 'processing' | 'success' = 'idle';
  if (localFrame < listenStart) barState = 'idle';
  else if (localFrame < listenEnd) barState = 'listening';
  else if (localFrame < procEnd) barState = 'processing';
  else barState = 'success';

  const elapsedSecs = barState === 'listening' ? (localFrame - listenStart) / fps : 0;
  const typed = typewriter(beat.transcript, localFrame, procEnd, procEnd + f(1.0));

  // Cross-fade host window between beats.
  const crossIn = tween(frame, beatIdx * beatLen - 6, beatIdx * beatLen + 6, 0, 1);

  return (
    <AbsoluteFill style={{ fontFamily: fonts.sans }}>
      <WindowsDesktop accent={beat.accent}>
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            paddingBottom: 48,
          }}
        >
          <div style={{ opacity: crossIn }}>
            <HostWindow variant={beat.variant} title={beat.title} width={1500} height={820} accent={beat.accent}>
              <HostBody variant={beat.variant} typed={typed} />
            </HostWindow>
          </div>
        </div>

        {/* Floating bar pinned near the top of the screen. Bumped wider
            + scaled up so the bar reads clearly at a 400 px mobile
            render alongside the host window. */}
        <div
          style={{
            position: 'absolute',
            top: 36,
            left: '50%',
            transform: 'translateX(-50%) scale(1.15)',
            transformOrigin: 'top center',
          }}
        >
          <BarDriver state={barState} elapsedMs={elapsedSecs * 1000} />
        </div>
      </WindowsDesktop>
    </AbsoluteFill>
  );
}

function HostBody({ variant, typed }: { variant: 'browser' | 'chat' | 'editor'; typed: string }) {
  if (variant === 'browser') {
    return (
      <div style={{ padding: '40px 56px', color: '#E8EAED', fontFamily: fonts.sans, height: '100%', boxSizing: 'border-box' }}>
        <div style={{ fontSize: 22, color: '#9AA0A6', marginBottom: 14 }}>
          To: priya@utterbar.app &nbsp;·&nbsp; Cc: team@utterbar.app
        </div>
        <div style={{ fontSize: 36, fontWeight: 500, marginBottom: 28, color: '#E8EAED' }}>Re: Slide deck</div>
        <div style={{ fontSize: 30, lineHeight: 1.55, color: '#D0D3D9' }}>
          Hi Priya,
          <br />
          <br />
          {typed}
          <span style={{ display: 'inline-block', width: 3, height: 30, background: tokens.accent, marginLeft: 2, verticalAlign: -4 }} />
        </div>
      </div>
    );
  }
  if (variant === 'chat') {
    return (
      <div
        style={{
          padding: '32px 40px',
          color: '#E8EAED',
          fontFamily: fonts.sans,
          display: 'flex',
          flexDirection: 'column',
          gap: 20,
          height: '100%',
          boxSizing: 'border-box',
        }}
      >
        <MsgBubble who="Priya" color="#7c3aed" text="ETA on 0.6.1?" />
        <MsgBubble who="Kiran" color="#0ea371" text="running the checklist rn" />
        <div
          style={{
            marginTop: 'auto',
            background: '#2A2B2E',
            borderRadius: 14,
            padding: '24px 28px',
            minHeight: 120,
            fontSize: 30,
            lineHeight: 1.5,
            color: '#E8EAED',
          }}
        >
          {typed}
          <span style={{ display: 'inline-block', width: 3, height: 30, background: tokens.accent, marginLeft: 2, verticalAlign: -4 }} />
        </div>
      </div>
    );
  }
  // editor
  return (
    <div
      style={{
        fontFamily: fonts.mono,
        fontSize: 28,
        lineHeight: 1.65,
        color: '#D0D3D9',
        padding: '40px 56px',
        height: '100%',
        boxSizing: 'border-box',
      }}
    >
      <div style={{ color: '#9AA0A6' }}># Research Notes — 23 April</div>
      <div style={{ marginTop: 26 }}>
        {typed}
        <span style={{ display: 'inline-block', width: 12, height: 30, background: tokens.accent, marginLeft: 2, verticalAlign: -4 }} />
      </div>
    </div>
  );
}

function MsgBubble({ who, color, text }: { who: string; color: string; text: string }) {
  return (
    <div style={{ display: 'flex', gap: 16, alignItems: 'flex-start' }}>
      <div
        style={{
          width: 52,
          height: 52,
          borderRadius: 999,
          background: color,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 22,
          fontWeight: 700,
          color: '#fff',
          flexShrink: 0,
        }}
      >
        {who[0]}
      </div>
      <div>
        <div style={{ fontSize: 20, fontWeight: 500, color: tokens.muted, marginBottom: 4 }}>{who}</div>
        <div style={{ fontSize: 28 }}>{text}</div>
      </div>
    </div>
  );
}
