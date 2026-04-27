/**
 * ad-08-windows-first — 11s. v3: pill demo extended to 5.5s, hold
 * the resolution for 1s before end card. Pill scale 2.0 (already).
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';

export function Ad08WindowsFirst() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const showPill = frame >= f(1.4);
  const pillState: 'idle' | 'listening' = frame >= f(2.2) && frame < f(5.5) ? 'listening' : 'idle';
  const elapsedMs = pillState === 'listening' ? ((frame - f(2.2)) / fps) * 1000 : 0;

  const macFade = tween(frame, f(0.0), f(0.5), 0, 1);
  const winFade = tween(frame, f(0.6), f(1.4), 0, 1);

  return (
    <SocialFrame variant="canvas">
      <div style={{ position: 'absolute', top: 100, left: '50%', transform: 'translateX(-50%)', width: 1020, opacity: macFade }}>
        <FauxLaptop label="macOS" filled>
          <div
            style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: tokens.fgDim,
              fontSize: 40,
              fontFamily: fonts.mono,
              letterSpacing: '0.06em',
              textAlign: 'center',
              padding: '0 40px',
              fontWeight: 500,
            }}
          >
            Mac dictation tools<br />live elsewhere
          </div>
        </FauxLaptop>
      </div>

      <div style={{ position: 'absolute', top: 1000, left: '50%', transform: 'translateX(-50%)', width: 1020, opacity: winFade }}>
        <FauxLaptop label="Windows 11" highlight>
          {showPill && (
            <div style={{ position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%) scale(2.0)' }}>
              <BarDriver state={pillState} elapsedMs={elapsedMs} />
            </div>
          )}
          {!showPill && (
            <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: tokens.muted, fontSize: 40, fontFamily: fonts.mono, fontWeight: 500 }}>
              Press Ctrl + Space
            </div>
          )}
        </FauxLaptop>
      </div>

      <Captions
        cues={[
          { text: 'Mac has Superwhisper.', at: 0.4, until: 2.7 },
          { text: 'Windows has Utterbar.', at: 2.9, until: 6.4 },
        ]}
      />

      <EndCard tagline="Native to Windows." startAt={6.5} />
    </SocialFrame>
  );
}

function FauxLaptop({
  label,
  filled,
  highlight,
  children,
}: {
  label: string;
  filled?: boolean;
  highlight?: boolean;
  children?: React.ReactNode;
}) {
  return (
    <div
      style={{
        background: tokens.bg1,
        border: `2px solid ${highlight ? `${tokens.accent}88` : tokens.border}`,
        borderRadius: 26,
        overflow: 'hidden',
        boxShadow: highlight ? `0 0 60px ${tokens.accent}33` : 'none',
        opacity: filled ? 0.55 : 1,
      }}
    >
      <div
        style={{
          padding: '20px 28px',
          borderBottom: `1px solid ${tokens.border}`,
          display: 'flex',
          alignItems: 'center',
          gap: 14,
        }}
      >
        <div style={{ width: 16, height: 16, borderRadius: 999, background: 'rgba(255,255,255,0.15)' }} />
        <div style={{ width: 16, height: 16, borderRadius: 999, background: 'rgba(255,255,255,0.15)' }} />
        <div style={{ width: 16, height: 16, borderRadius: 999, background: 'rgba(255,255,255,0.15)' }} />
        <div
          style={{
            marginLeft: 18,
            color: tokens.muted,
            fontFamily: fonts.mono,
            fontSize: 20,
            letterSpacing: '0.1em',
            textTransform: 'uppercase',
            fontWeight: 600,
          }}
        >
          {label}
        </div>
      </div>
      <div
        style={{
          height: 720,
          background: filled ? tokens.bg0 : `linear-gradient(180deg, ${tokens.bg0}, #08101a)`,
          position: 'relative',
          display: 'flex',
        }}
      >
        {children}
      </div>
    </div>
  );
}
