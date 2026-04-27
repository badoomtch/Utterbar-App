/**
 * feature-local-privacy — 8 s. Replaces the v1 static-toggles version
 * (toggles all OFF was visually a non-event — hard to make a buying
 * argument out of negative space).
 *
 * The new story: a labelled "Your PC" panel on the left holds four
 * pills — Audio, Transcripts, Vocabulary, History — connected by a
 * thick cyan loop that stays inside the panel. A thin grey outbound
 * line goes to a small "utterbar.com" panel on the right and lights
 * up only for the four genuine network calls (licence, updates,
 * models, pricing). This makes the precise privacy claim visible:
 * voice / output / personal data stay local; tiny housekeeping
 * traffic doesn't.
 *
 * Timeline:
 *   0.0 – 1.5   "Your PC" panel fades in with the four data pills.
 *   1.5 – 4.5   The local loop animates around the pills (cyan trace).
 *   4.5 – 7.0   The outbound line lights up briefly, four times, each
 *               time labelled with what's being sent (licence check,
 *               update check, model download, pricing).
 *   7.0 – 8.0   Both elements settle; bottom caption "Audio, transcripts,
 *               vocab, history — never leave your PC." holds.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween } from '../util/anim';

const LOCAL_PILLS = ['Audio', 'Transcripts', 'Vocabulary', 'History'];

type OutboundEvent = {
  startSeconds: number;
  durationSeconds: number;
  label: string;
};

// Four genuine outbound calls the app actually makes. Each one fires
// once during the 4.5–7.0 window so the viewer reads "small list, fully
// enumerated" rather than "trust us, it's fine".
const OUTBOUND_EVENTS: OutboundEvent[] = [
  { startSeconds: 4.6, durationSeconds: 0.55, label: 'Update check' },
  { startSeconds: 5.2, durationSeconds: 0.55, label: 'Licence check' },
  { startSeconds: 5.8, durationSeconds: 0.55, label: 'Pricing' },
  { startSeconds: 6.4, durationSeconds: 0.55, label: 'Model download' },
];

export function FeatureLocalPrivacy() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Panel fade-in.
  const panelOpacity = tween(frame, 0, f(0.8), 0, 1);

  // Local loop trace — animates around the local panel between 1.5s and
  // 4.5s. We use stroke-dashoffset to draw the path progressively.
  const loopProgress = tween(frame, f(1.5), f(4.5), 0, 1);

  // Active outbound event (if any) — checks each event window in turn.
  let activeOutbound: OutboundEvent | null = null;
  let outboundIntensity = 0;
  for (const ev of OUTBOUND_EVENTS) {
    const startF = f(ev.startSeconds);
    const endF = f(ev.startSeconds + ev.durationSeconds);
    if (frame >= startF && frame <= endF) {
      activeOutbound = ev;
      // Triangle wave so the line "pulses" — peaks at the midpoint.
      const t = (frame - startF) / (endF - startF);
      outboundIntensity = 1 - Math.abs(2 * t - 1);
      break;
    }
  }

  // After the last outbound event, hold the closing caption.
  const captionOpacity = tween(frame, f(6.95), f(7.6), 0, 1);

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Eyebrow at the top. */}
      <div
        style={{
          position: 'absolute',
          top: 60,
          left: '50%',
          transform: 'translateX(-50%)',
          fontFamily: fonts.mono,
          fontSize: 15,
          letterSpacing: '0.18em',
          textTransform: 'uppercase',
          color: tokens.muted,
          opacity: panelOpacity,
        }}
      >
        Local by default
      </div>

      {/* Two panels side-by-side: Your PC on the left (large) and
          utterbar.com on the right (small). Connecting line between
          them is the carrier for outbound events. */}
      <div
        style={{
          position: 'absolute',
          top: 140,
          left: 0,
          right: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          gap: 60,
          padding: '0 80px',
          opacity: panelOpacity,
        }}
      >
        {/* === YOUR PC panel === */}
        <YourPcPanel pills={LOCAL_PILLS} loopProgress={loopProgress} />

        {/* === Connecting line + outbound flash === */}
        <ConnectingLine activeLabel={activeOutbound?.label ?? null} intensity={outboundIntensity} />

        {/* === utterbar.com panel === */}
        <RemotePanel highlight={outboundIntensity} />
      </div>

      {/* Closing caption. Lands once all four outbound events have
          fired — frames the four small calls as the ENTIRE outbound
          surface. */}
      <div
        style={{
          position: 'absolute',
          bottom: 80,
          left: '50%',
          transform: 'translateX(-50%)',
          textAlign: 'center',
          opacity: captionOpacity,
        }}
      >
        <div
          style={{
            fontSize: 28,
            fontWeight: 500,
            color: tokens.fgHi,
            letterSpacing: '-0.01em',
            lineHeight: 1.3,
          }}
        >
          Audio, transcripts, vocab, history — <span style={{ color: tokens.accent }}>never leave your PC</span>.
        </div>
        <div
          style={{
            marginTop: 14,
            fontSize: 17,
            color: tokens.muted,
            letterSpacing: '-0.005em',
          }}
        >
          The only outbound traffic is the four small checks above.
        </div>
      </div>
    </AbsoluteFill>
  );
}

function YourPcPanel({
  pills,
  loopProgress,
}: {
  pills: string[];
  loopProgress: number;
}) {
  const panelW = 600;
  const panelH = 480;
  // Path coordinates for the cyan loop — runs around the panel just
  // inside the border, plus a soft inward kink that touches each pill.
  // Computed lazily so the animation stays declarative.
  const inset = 36;
  const x1 = inset;
  const y1 = inset;
  const x2 = panelW - inset;
  const y2 = panelH - inset;
  const corner = 24;
  const pathD = [
    `M ${x1 + corner} ${y1}`,
    `L ${x2 - corner} ${y1}`,
    `Q ${x2} ${y1} ${x2} ${y1 + corner}`,
    `L ${x2} ${y2 - corner}`,
    `Q ${x2} ${y2} ${x2 - corner} ${y2}`,
    `L ${x1 + corner} ${y2}`,
    `Q ${x1} ${y2} ${x1} ${y2 - corner}`,
    `L ${x1} ${y1 + corner}`,
    `Q ${x1} ${y1} ${x1 + corner} ${y1}`,
    'Z',
  ].join(' ');
  // Approximate path length so the dashoffset reveal lines up with the
  // visible loop. Rectangle perimeter plus the four rounded corners.
  const perim = 2 * ((x2 - x1) + (y2 - y1));

  return (
    <div
      style={{
        position: 'relative',
        width: panelW,
        height: panelH,
        borderRadius: 24,
        background: tokens.bg1,
        border: `1px solid ${tokens.border}`,
        flexShrink: 0,
      }}
    >
      {/* Animated cyan loop trace */}
      <svg
        width={panelW}
        height={panelH}
        style={{ position: 'absolute', top: 0, left: 0, pointerEvents: 'none' }}
      >
        <path
          d={pathD}
          fill="none"
          stroke={tokens.accent}
          strokeWidth={3}
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeDasharray={perim}
          strokeDashoffset={perim * (1 - loopProgress)}
          opacity={0.72}
        />
      </svg>

      {/* Panel header */}
      <div
        style={{
          position: 'absolute',
          top: 28,
          left: 36,
          fontSize: 16,
          fontFamily: fonts.mono,
          letterSpacing: '0.12em',
          textTransform: 'uppercase',
          color: tokens.muted,
        }}
      >
        Your PC
      </div>

      {/* Data pills laid out two-up */}
      <div
        style={{
          position: 'absolute',
          top: 78,
          left: 36,
          right: 36,
          bottom: 36,
          display: 'grid',
          gridTemplateColumns: '1fr 1fr',
          gap: 18,
          alignContent: 'center',
        }}
      >
        {pills.map((p) => (
          <div
            key={p}
            style={{
              padding: '22px 24px',
              borderRadius: 14,
              background: tokens.bg2 ?? 'rgba(255,255,255,0.04)',
              border: `1px solid ${tokens.border}`,
              fontSize: 22,
              fontWeight: 500,
              color: tokens.fgHi,
              letterSpacing: '-0.01em',
            }}
          >
            {p}
          </div>
        ))}
      </div>
    </div>
  );
}

function ConnectingLine({
  activeLabel,
  intensity,
}: {
  activeLabel: string | null;
  intensity: number;
}) {
  const lineW = 220;
  return (
    <div
      style={{
        position: 'relative',
        width: lineW,
        height: 480,
        flexShrink: 0,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      {/* Base grey line — always there, faint */}
      <div
        style={{
          position: 'absolute',
          left: 0,
          right: 0,
          top: '50%',
          height: 2,
          background: tokens.border,
          transform: 'translateY(-1px)',
        }}
      />
      {/* Pulsing accent overlay — only visible during outbound events */}
      <div
        style={{
          position: 'absolute',
          left: 0,
          right: 0,
          top: '50%',
          height: 2,
          background: tokens.accent,
          opacity: 0.15 + intensity * 0.85,
          transform: `translateY(-1px) scaleX(${0.4 + intensity * 0.6})`,
          transformOrigin: 'center',
          filter: `drop-shadow(0 0 ${4 + intensity * 8}px rgba(34, 211, 238, ${intensity * 0.5}))`,
        }}
      />
      {/* Floating outbound label — only present during an event */}
      {activeLabel && intensity > 0.1 && (
        <div
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: `translate(-50%, calc(-50% - ${30 + intensity * 12}px))`,
            background: tokens.bg1,
            border: `1px solid rgba(34, 211, 238, ${0.4 + intensity * 0.4})`,
            padding: '6px 12px',
            borderRadius: 999,
            fontFamily: fonts.mono,
            fontSize: 13,
            letterSpacing: '0.05em',
            color: tokens.accent,
            whiteSpace: 'nowrap',
            opacity: intensity,
          }}
        >
          {activeLabel}
        </div>
      )}
    </div>
  );
}

function RemotePanel({ highlight }: { highlight: number }) {
  const panelW = 320;
  const panelH = 200;
  return (
    <div
      style={{
        position: 'relative',
        width: panelW,
        height: panelH,
        borderRadius: 20,
        background: tokens.bg1,
        border: `1px solid ${
          highlight > 0.05
            ? `rgba(34, 211, 238, ${0.3 + highlight * 0.4})`
            : tokens.border
        }`,
        flexShrink: 0,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 28,
        textAlign: 'center',
      }}
    >
      <div
        style={{
          fontFamily: fonts.mono,
          fontSize: 14,
          letterSpacing: '0.12em',
          textTransform: 'uppercase',
          color: tokens.muted,
          marginBottom: 8,
        }}
      >
        Outbound to
      </div>
      <div
        style={{
          fontFamily: fonts.mono,
          fontSize: 22,
          fontWeight: 600,
          color: tokens.fgHi,
          letterSpacing: '-0.01em',
        }}
      >
        utterbar.com
      </div>
      <div
        style={{
          marginTop: 14,
          fontSize: 14,
          color: tokens.muted,
          lineHeight: 1.5,
        }}
      >
        Update / licence checks,
        <br />
        catalogue + pricing.
      </div>
    </div>
  );
}
