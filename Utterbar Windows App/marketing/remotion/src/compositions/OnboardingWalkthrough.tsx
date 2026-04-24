/**
 * onboarding-walkthrough — 12 s. Strips through three onboarding steps
 * at 4 s each: Welcome → Privacy promise → Model pick → the "Ready"
 * payoff frame. Every step matches the real onboarding layout from
 * `src/routes/Onboarding.tsx` (header with logo + step dots, footer,
 * centred body panel).
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { UtterbarLogoMark } from '@/components/bar/UtterbarLogoMark';
import { UtterbarWordmark } from '@/components/bar/UtterbarWordmark';
import { BarDriver } from '../bar/BarDriver';
import { Scene } from '../util/Scene';
import { tween } from '../util/anim';

const STEPS = 9;

export function OnboardingWalkthrough() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const beatLen = f(4);
  const beat = Math.min(2, Math.floor(frame / beatLen));
  const stepIdx = beat === 0 ? 0 : beat === 1 ? 1 : 8; // welcome / privacy / ready

  return (
    <Scene background={tokens.bg0}>
      {/* Mock onboarding window frame */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <div
          style={{
            width: 1440,
            height: 880,
            background: tokens.bg0,
            borderRadius: 18,
            border: `1px solid ${tokens.border}`,
            display: 'flex',
            flexDirection: 'column',
            overflow: 'hidden',
            boxShadow: '0 32px 80px rgba(0,0,0,0.55)',
            fontFamily: fonts.sans,
          }}
        >
          {/* Header */}
          <header style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '34px 44px 0' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
              <UtterbarLogoMark size={40} />
              {/* Real Wordmark reads its ink colour from CSS
                  `currentColor` — wrap so the token applies. */}
              <span style={{ color: tokens.fgHi, display: 'inline-flex' }}>
                <UtterbarWordmark size={28} />
              </span>
            </div>
            <div style={{ display: 'flex', gap: 6 }}>
              {Array.from({ length: STEPS }).map((_, i) => (
                <span
                  key={i}
                  style={{
                    height: 6,
                    borderRadius: 999,
                    width: i <= stepIdx ? 36 : 18,
                    background: i <= stepIdx ? tokens.accent : tokens.border,
                    transition: 'width 240ms',
                  }}
                />
              ))}
            </div>
          </header>

          {/* Body — swap by beat */}
          <main style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 48 }}>
            {beat === 0 && <WelcomeBody frame={frame} />}
            {beat === 1 && <PrivacyBody frame={frame} beatStart={beatLen} />}
            {beat === 2 && <ReadyBody frame={frame} beatStart={2 * beatLen} fps={fps} />}
          </main>

          {/* Footer */}
          <footer
            style={{
              borderTop: `1px solid ${tokens.border}`,
              padding: '22px 44px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              fontSize: 22,
              color: tokens.muted,
            }}
          >
            <span>Back</span>
            <div
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 10,
                padding: '14px 28px',
                background: tokens.accent,
                color: '#06121A',
                borderRadius: 999,
                fontSize: 22,
                fontWeight: 500,
              }}
            >
              {beat === 2 ? 'Start dictating' : 'Continue'}
              <span style={{ fontSize: 24 }}>→</span>
            </div>
          </footer>
        </div>
      </div>
    </Scene>
  );
}

function WelcomeBody({ frame }: { frame: number }) {
  const scale = tween(frame, 6, 22, 0.6, 1);
  const opacity = tween(frame, 6, 22, 0, 1);
  return (
    <div style={{ textAlign: 'center' }}>
      <div
        style={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: 32,
          transform: `scale(${scale})`,
          opacity,
        }}
      >
        <UtterbarLogoMark size={140} active />
        <span style={{ color: tokens.fgHi, display: 'inline-flex' }}>
          <UtterbarWordmark size={96} />
        </span>
      </div>
      <div style={{ marginTop: 56, fontSize: 48, fontWeight: 700, color: tokens.fgHi, letterSpacing: '-0.02em' }}>
        Welcome
      </div>
      <div style={{ marginTop: 20, fontSize: 24, color: tokens.muted, maxWidth: 700, margin: '20px auto 0', lineHeight: 1.5 }}>
        A premium dictation app for Windows that runs entirely on your machine. We'll get you set up in under two minutes.
      </div>
    </div>
  );
}

function PrivacyBody({ frame, beatStart }: { frame: number; beatStart: number }) {
  const local = frame - beatStart;
  const opacity = tween(local, 4, 18, 0, 1);
  return (
    <div
      style={{
        width: '100%',
        maxWidth: 820,
        borderRadius: 28,
        background: tokens.bg1,
        padding: 56,
        textAlign: 'center',
        opacity,
        border: `1px solid ${tokens.border}`,
      }}
    >
      <div
        style={{
          margin: '0 auto',
          width: 100,
          height: 100,
          borderRadius: 22,
          background: `color-mix(in oklab, ${tokens.accent} 14%, ${tokens.bg2})`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <svg width={52} height={52} viewBox="0 0 24 24" fill="none" stroke={tokens.accent} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <rect x="3" y="11" width="18" height="11" rx="2" />
          <path d="M7 11V7a5 5 0 0 1 10 0v4" />
        </svg>
      </div>
      <div style={{ marginTop: 36, fontSize: 44, fontWeight: 700, letterSpacing: '-0.02em', color: tokens.fgHi, lineHeight: 1.1 }}>
        Everything stays on your machine
      </div>
      <div style={{ marginTop: 22, fontSize: 24, color: tokens.muted, lineHeight: 1.5 }}>
        No cloud transcription. No cloud post-processing. No telemetry by default. Your recordings, transcripts, and preferences are stored only on this PC.
      </div>
    </div>
  );
}

function ReadyBody({ frame, beatStart, fps }: { frame: number; beatStart: number; fps: number }) {
  const local = frame - beatStart;
  const pillState = local < fps ? 'idle' : local < 3 * fps ? 'listening' : 'success';
  return (
    <div
      style={{
        width: '100%',
        maxWidth: 880,
        borderRadius: 28,
        background: tokens.bg1,
        padding: 56,
        textAlign: 'center',
        border: `1px solid ${tokens.border}`,
      }}
    >
      <div style={{ fontSize: 44, fontWeight: 700, letterSpacing: '-0.02em', color: tokens.fgHi, lineHeight: 1.1 }}>
        You're ready
      </div>
      <div style={{ marginTop: 16, fontSize: 22, color: tokens.muted, lineHeight: 1.5 }}>
        Press the shortcut, speak, and your words will land in the box below.
      </div>
      <div style={{ marginTop: 32, display: 'flex', justifyContent: 'center' }}>
        <div style={{ transform: 'scale(1.3)', transformOrigin: 'center' }}>
          <BarDriver
            state={pillState as 'idle' | 'listening' | 'success'}
            elapsedMs={Math.max(0, (local - fps) / fps) * 1000}
          />
        </div>
      </div>
      <div
        style={{
          marginTop: 40,
          padding: '22px 26px',
          background: tokens.bg2,
          borderRadius: 14,
          fontSize: 22,
          color: tokens.fg,
          textAlign: 'left',
          minHeight: 96,
          lineHeight: 1.5,
          border: `1px solid ${tokens.border}`,
        }}
      >
        {pillState === 'success' ? 'Your first dictation landed here.' : 'Your dictated text will appear here…'}
      </div>
    </div>
  );
}
