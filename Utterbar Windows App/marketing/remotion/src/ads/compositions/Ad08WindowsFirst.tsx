/**
 * ad-08-windows-first — 10s. The positioning ad. Half the frame is a
 * Mac with a "voice → text" tool that's clearly Mac-only (Superwhisper
 * shaped, never named). Half is a Windows desktop with the Utterbar
 * pill. The cursor on the Windows side fires the hotkey; the cursor
 * on the Mac side does nothing because there's nothing for it to do.
 *
 * Different angle from anything in the existing pack: explicit
 * positioning vs. the Mac incumbent, the way BRIEF.md §1 frames it.
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

  const showPill = frame >= f(2.0);
  const pillState: 'idle' | 'listening' = frame >= f(3.0) && frame < f(6.0) ? 'listening' : 'idle';
  const elapsedMs = pillState === 'listening' ? ((frame - f(3.0)) / fps) * 1000 : 0;

  const winFade = tween(frame, f(0.4), f(1.4), 0, 1);
  const macFade = tween(frame, f(0.0), f(0.8), 0, 1);

  return (
    <SocialFrame variant="canvas">
      <Eyebrow text="Windows-first since day one" />

      {/* MAC half — TOP. Greyed out, dimmer, "no app" badge */}
      <div
        style={{
          position: 'absolute',
          top: 200,
          left: '50%',
          transform: 'translateX(-50%)',
          width: 940,
          opacity: macFade,
        }}
      >
        <FauxLaptop label="macOS" filled>
          <div
            style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: tokens.fgDim,
              fontSize: 22,
              fontFamily: fonts.mono,
              letterSpacing: '0.08em',
            }}
          >
            ※ Mac dictation tools live elsewhere ※
          </div>
        </FauxLaptop>
      </div>

      {/* WINDOWS half — BOTTOM. The pill lives here */}
      <div
        style={{
          position: 'absolute',
          top: 1080,
          left: '50%',
          transform: 'translateX(-50%)',
          width: 940,
          opacity: winFade,
        }}
      >
        <FauxLaptop label="Windows 11" highlight>
          {showPill && (
            <div
              style={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%) scale(1.6)',
              }}
            >
              <BarDriver state={pillState} elapsedMs={elapsedMs} />
            </div>
          )}
          {!showPill && (
            <div
              style={{
                flex: 1,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: tokens.muted,
                fontSize: 22,
              }}
            >
              Press Ctrl + Space
            </div>
          )}
        </FauxLaptop>
      </div>

      <Captions
        cues={[
          { text: 'Mac has Superwhisper.', at: 0.5, until: 3.4 },
          { text: 'Windows has Utterbar.', at: 3.6, until: 6.5 },
          { text: 'Built for Windows from day one.', at: 6.7, until: 9.0 },
        ]}
        bottom={120}
      />

      <EndCard tagline="Dictation, native to Windows." startAt={9.0} />
    </SocialFrame>
  );
}

function Eyebrow({ text }: { text: string }) {
  return (
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
      {text}
    </div>
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
        border: `1px solid ${highlight ? `${tokens.accent}55` : tokens.border}`,
        borderRadius: 22,
        overflow: 'hidden',
        boxShadow: highlight ? `0 0 60px ${tokens.accent}1f` : 'none',
        opacity: filled ? 0.55 : 1,
      }}
    >
      <div
        style={{
          padding: '14px 20px',
          borderBottom: `1px solid ${tokens.border}`,
          display: 'flex',
          alignItems: 'center',
          gap: 10,
        }}
      >
        <div style={{ width: 12, height: 12, borderRadius: 999, background: 'rgba(255,255,255,0.15)' }} />
        <div style={{ width: 12, height: 12, borderRadius: 999, background: 'rgba(255,255,255,0.15)' }} />
        <div style={{ width: 12, height: 12, borderRadius: 999, background: 'rgba(255,255,255,0.15)' }} />
        <div
          style={{
            marginLeft: 14,
            color: tokens.muted,
            fontFamily: fonts.mono,
            fontSize: 14,
            letterSpacing: '0.08em',
            textTransform: 'uppercase',
          }}
        >
          {label}
        </div>
      </div>
      <div
        style={{
          height: 600,
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
