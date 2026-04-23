/**
 * how-it-works-walkthrough — 12 s explainer rebuilt as pure UI-in-motion.
 * No Callouts, no eyebrows, no taglines — each beat is one zoomed UI
 * element doing the thing it does. The site puts the textual "Press.
 * Speak. Done." copy next to the video; the video just shows it.
 *
 * Three 4-second beats:
 *   1. Ctrl + Space kbd chips, huge and centred. Space animates a press.
 *   2. FloatingBar, huge and centred, cycling idle → listening → processing.
 *   3. Host chat window mid-frame with the bar pinned above — transcript
 *      types into the composer, caret blinks, bar flashes "Inserted".
 *
 * Background is tokens.bg0 end-to-end — vanishes into the site section.
 * Ends on the resting "Inserted" state so the loop reads as "the words
 * landed" rather than restarting an ad.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import { HostWindow } from '../chrome/HostWindow';
import { pulse, tween, typewriter } from '../util/anim';

const TRANSCRIPT =
  "Hey team — quick status update on the release. We're on track for Friday, final QA passes start tomorrow.";

export function HowItWorks() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const beat1 = f(4);
  const beat2 = f(8);
  // end = f(12)

  const inBeat1 = frame < beat1;
  const inBeat2 = frame >= beat1 && frame < beat2;
  const inBeat3 = frame >= beat2;

  // Beat 1 — kbd press. Space drops 6 px between 1.4 s and 1.8 s, holds
  // for 0.3 s (so it reads as a real press), releases by 2.2 s.
  const pressDepth =
    tween(frame, f(1.4), f(1.8), 0, 6) - tween(frame, f(2.1), f(2.4), 0, 6);
  const pressed = pressDepth > 0.5;

  // Beat 2 — bar state machine. Idle 0.4s, listen 2.6s, processing 0.6s,
  // success 0.4s — sits on success so beat boundary reads cleanly.
  const b2 = frame - beat1;
  const barStateB2: 'idle' | 'listening' | 'processing' | 'success' =
    b2 < f(0.4)
      ? 'idle'
      : b2 < f(3.0)
        ? 'listening'
        : b2 < f(3.6)
          ? 'processing'
          : 'success';
  const elapsedB2 = barStateB2 === 'listening' ? (b2 - f(0.4)) / fps : 0;

  // Beat 3 — typewriter the transcript, caret blinks throughout. Bar
  // flashes success as the typing completes.
  const typeStart = beat2 + f(0.6);
  const typeEnd = beat2 + f(3.2);
  const typed = inBeat3 ? typewriter(TRANSCRIPT, frame, typeStart, typeEnd) : '';
  const caretOn = pulse(frame, 15) > 0.5;
  const b3 = frame - beat2;
  const barStateB3: 'listening' | 'processing' | 'success' =
    b3 < f(2.6) ? 'listening' : b3 < f(3.2) ? 'processing' : 'success';

  // Cross-beat fade — 12 fr crossfade so beat boundaries don't hard-cut.
  const opacity1 = tween(frame, beat1 - 10, beat1, 1, 0);
  const opacity2In = tween(frame, beat1 - 6, beat1 + 6, 0, 1);
  const opacity2Out = tween(frame, beat2 - 10, beat2, 1, 0);
  const opacity3In = tween(frame, beat2 - 6, beat2 + 6, 0, 1);

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Ambient cyan wash — continuous across beats so the background
          doesn't flash between scenes. */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: '50%',
          transform: 'translate(-50%, -50%)',
          width: 1400,
          height: 700,
          background: `radial-gradient(ellipse at center, ${tokens.accent}22 0%, transparent 65%)`,
          filter: 'blur(90px)',
        }}
      />

      {/* BEAT 1 — Ctrl + Space kbd chips */}
      {inBeat1 && (
        <AbsoluteFill
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: 28,
            opacity: opacity1,
          }}
        >
          <Kbd>Ctrl</Kbd>
          <span
            style={{
              color: tokens.fgDim,
              fontSize: 72,
              fontWeight: 300,
              lineHeight: 1,
            }}
          >
            +
          </span>
          <Kbd pressed={pressed}>Space</Kbd>
        </AbsoluteFill>
      )}

      {/* BEAT 2 — floating bar, big and centred */}
      {inBeat2 && (
        <AbsoluteFill
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            opacity: Math.min(opacity2In, opacity2Out),
          }}
        >
          <div style={{ transform: 'scale(1.8)', transformOrigin: 'center' }}>
            <BarDriver state={barStateB2} elapsedMs={elapsedB2 * 1000} />
          </div>
        </AbsoluteFill>
      )}

      {/* BEAT 3 — chat window receives the transcript; bar pinned above */}
      {inBeat3 && (
        <AbsoluteFill
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            opacity: opacity3In,
          }}
        >
          <div style={{ position: 'relative' }}>
            <HostWindow variant="chat" title="Messages · Team" width={1360} height={620}>
              <ChatBody text={typed} caret={caretOn} />
            </HostWindow>
            <div
              style={{
                position: 'absolute',
                top: -44,
                left: '50%',
                transform: 'translateX(-50%) scale(1.1)',
                transformOrigin: 'top center',
              }}
            >
              <BarDriver state={barStateB3} />
            </div>
          </div>
        </AbsoluteFill>
      )}
    </AbsoluteFill>
  );
}

// Kbd chip — scaled up for mobile 400 px render. 64 pt glyphs, 180 px min
// width so "Space" fills the chip without feeling cramped.
function Kbd({
  children,
  pressed = false,
}: {
  children: React.ReactNode;
  pressed?: boolean;
}) {
  return (
    <div
      style={{
        minWidth: pressed ? 260 : 280,
        padding: '38px 52px',
        fontFamily: fonts.mono,
        fontSize: 64,
        fontWeight: 500,
        color: pressed ? tokens.bg0 : tokens.fgHi,
        background: pressed ? tokens.accent : tokens.bg1,
        border: `2px solid ${pressed ? tokens.accent : tokens.borderStrong}`,
        borderRadius: 20,
        boxShadow: pressed
          ? `inset 0 4px 8px rgba(0,0,0,0.25)`
          : `0 4px 0 ${tokens.border}, 0 10px 24px rgba(0,0,0,0.45)`,
        transform: `translateY(${pressed ? 6 : 0}px)`,
        textAlign: 'center',
        letterSpacing: '-0.01em',
      }}
    >
      {children}
    </div>
  );
}

function ChatBody({ text, caret }: { text: string; caret: boolean }) {
  return (
    <div
      style={{
        padding: '36px 44px',
        fontFamily: fonts.sans,
        color: tokens.fg,
        display: 'flex',
        flexDirection: 'column',
        gap: 22,
        height: '100%',
        boxSizing: 'border-box',
      }}
    >
      <Bubble who="Priya" color="#7c3aed" text="can we ship Friday?" />
      {/* Composer — where dictated text lands. Sized big enough that
          a 400 px mobile render still reads every word. */}
      <div
        style={{
          marginTop: 'auto',
          background: '#2A2B2E',
          borderRadius: 14,
          padding: '24px 28px',
          minHeight: 120,
          fontSize: 32,
          lineHeight: 1.5,
          color: '#E8EAED',
        }}
      >
        {text}
        {caret && (
          <span
            style={{
              display: 'inline-block',
              width: 3,
              height: 34,
              verticalAlign: -6,
              marginLeft: 2,
              background: tokens.accent,
            }}
          />
        )}
      </div>
    </div>
  );
}

function Bubble({ who, color, text }: { who: string; color: string; text: string }) {
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
          fontWeight: 700,
          fontSize: 22,
          color: '#fff',
          flexShrink: 0,
        }}
      >
        {who[0]}
      </div>
      <div>
        <div style={{ fontSize: 20, color: tokens.muted, fontWeight: 500, marginBottom: 4 }}>
          {who}
        </div>
        <div style={{ fontSize: 28, color: tokens.fg }}>{text}</div>
      </div>
    </div>
  );
}
