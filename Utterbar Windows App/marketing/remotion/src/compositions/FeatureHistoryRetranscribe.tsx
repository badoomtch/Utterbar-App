/**
 * feature-history-retranscribe — 7.5 s. Close shot of the History
 * detail card. A past dictation that was captured in Voice-to-text mode
 * (raw, rambly) gets re-run through Meeting mode — the output block
 * animates its content from prose to a three-bullet note.
 *
 * Regenerated 2026-04-24 — previously sat inside the AppShell chrome
 * with the history list on the left, which meant the actual detail
 * pane was a quarter of the frame. Now the detail card IS the frame.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween, typewriter } from '../util/anim';

const RAW =
  "so basically we need to push the launch to Tuesday because QA flagged two issues with the auth flow and we want them fixed before marketing pings the list";
const BULLETED = '• Launch pushed to Tuesday\n• QA flagged 2 issues in auth flow\n• Fix before marketing announcement';

const MODES: { id: string; label: string }[] = [
  { id: 'voice-to-text', label: 'Voice-to-text' },
  { id: 'message', label: 'Message' },
  { id: 'meeting', label: 'Meeting' },
];

export function FeatureHistoryRetranscribe() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Timeline:
  // 0.0–0.8  card fade in, raw output visible
  // 0.8–2.0  hold
  // 2.0–2.5  user clicks Meeting chip (selection transitions)
  // 2.5–3.0  processing flash on the output card
  // 3.0–5.5  bulleted output typewriters in, raw fades out
  // 5.5–7.5  hold on final state
  const cardIn = tween(frame, f(0.2), f(0.9), 0, 1);
  const selected = frame < f(2.1) ? 0 : 2; // v-to-t -> meeting
  const selTransition = tween(frame, f(2.0), f(2.4), 0, 1);
  const processing = frame >= f(2.4) && frame < f(3.0);
  const rawFade = tween(frame, f(2.8), f(3.2), 1, 0);
  const bulletTyped = typewriter(BULLETED, frame, f(3.0), f(5.3));

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Meta strip at top — date + duration, like the real detail header */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 70,
          transform: 'translateX(-50%)',
          width: 1080,
          maxWidth: '90%',
          opacity: cardIn,
          display: 'flex',
          alignItems: 'baseline',
          gap: 22,
        }}
      >
        <div style={{ fontSize: 26, color: tokens.fgHi, fontWeight: 600, letterSpacing: '-0.01em' }}>
          Today · 11:48
        </div>
        <div style={{ fontFamily: fonts.mono, fontSize: 18, color: tokens.muted, letterSpacing: '0.12em', textTransform: 'uppercase' }}>
          00:18 · 32 words
        </div>
      </div>

      {/* Source transcript block */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 130,
          transform: 'translateX(-50%)',
          width: 1080,
          maxWidth: '90%',
          opacity: cardIn,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 17,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 12,
          }}
        >
          Source transcript
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 18,
            border: `1px solid ${tokens.border}`,
            padding: '22px 28px',
            fontStyle: 'italic',
            fontSize: 20,
            lineHeight: 1.55,
            color: tokens.fg,
          }}
        >
          &ldquo;{RAW}&rdquo;
        </div>
      </div>

      {/* Mode chips row */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 360,
          transform: 'translateX(-50%)',
          width: 1080,
          maxWidth: '90%',
          display: 'flex',
          alignItems: 'center',
          gap: 12,
          opacity: cardIn,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 17,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginRight: 6,
          }}
        >
          Reprocess with
        </div>
        {MODES.map((m, i) => {
          const active = selected === i;
          return (
            <div
              key={m.id}
              style={{
                padding: '10px 18px',
                borderRadius: 999,
                border: `1px solid ${active ? `rgba(34,211,238,${0.3 + selTransition * 0.4})` : tokens.border}`,
                background: active ? `rgba(34,211,238,${0.08 + selTransition * 0.08})` : tokens.bg1,
                fontSize: 18,
                color: active ? tokens.accent2 : tokens.fg,
              }}
            >
              {m.label}
            </div>
          );
        })}
      </div>

      {/* Output card — either raw (fading out) or bulleted (typing in). */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 430,
          transform: 'translateX(-50%)',
          width: 1080,
          maxWidth: '90%',
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 17,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 12,
            display: 'flex',
            alignItems: 'center',
            gap: 10,
          }}
        >
          Processed output
          {processing && (
            <span
              style={{
                display: 'inline-block',
                padding: '2px 10px',
                borderRadius: 999,
                fontSize: 18,
                background: `rgba(34,211,238,0.12)`,
                color: tokens.accent2,
                letterSpacing: '0.14em',
              }}
            >
              PROCESSING
            </span>
          )}
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 18,
            border: `1px solid ${
              processing ? `rgba(34,211,238,0.35)` : tokens.border
            }`,
            padding: '26px 30px',
            fontSize: 22,
            lineHeight: 1.6,
            color: tokens.fg,
            position: 'relative',
            minHeight: 240,
          }}
        >
          <div style={{ opacity: rawFade, whiteSpace: 'pre-line' }}>{RAW}</div>
          <div
            style={{
              position: 'absolute',
              top: 26,
              left: 30,
              right: 30,
              opacity: 1 - rawFade,
              whiteSpace: 'pre-line',
            }}
          >
            {bulletTyped}
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
}
