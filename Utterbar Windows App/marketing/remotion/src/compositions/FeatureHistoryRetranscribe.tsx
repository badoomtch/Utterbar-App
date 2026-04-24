/**
 * feature-history-retranscribe — 9 s. Zoomed onto the History detail
 * pane — the right-hand card where you re-run a past dictation through
 * a different Mode. Cursor flies in, opens the "Reprocess as"
 * dropdown, picks "Meeting", clicks Run, the Processed block rewrites
 * itself from Voice-to-text into the bulleted meeting note.
 *
 * No sidebar chrome, no search bar, no list — those are the app shell
 * around this card. The card itself is what sells the feature, so it
 * gets the whole frame.
 *
 * Timeline (30 fps):
 *   0.0–0.5  idle (Voice-to-text processed, dropdown closed)
 *   0.5–1.1  cursor flies in from off-frame to the dropdown
 *   1.15     click — dropdown panel opens
 *   1.2–2.1  panel finishes opening, user hovers options
 *   2.1–2.6  cursor slides down to "Meeting" row (panel highlights)
 *   2.7      click Meeting — label updates, panel closes
 *   2.9–3.4  cursor slides right to Run button
 *   3.5      click Run — processed block starts rewriting
 *   3.6–6.4  typewriter rewrites block as bulleted meeting note
 *   6.4–7.0  cursor exits off-frame
 *   7.0–9.0  hold on the result
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween, typewriter } from '../util/anim';
import { Cursor, ClickPulse, cursorPath } from '../util/Cursor';

const RAW =
  'so basically we need to push the launch to Tuesday because the QA team flagged two issues with the auth flow';
const PROCESSED_VOICE =
  'So basically we need to push the launch to Tuesday because the QA team flagged two issues with the auth flow.';
const PROCESSED_MEETING =
  '• Launch pushed to Tuesday\n• QA flagged 2 issues in auth flow\n• Fix before announcement';

// ---- layout ---------------------------------------------------------------
const CARD_W = 1480;
const CARD_H = 880;
const CARD_X = (1920 - CARD_W) / 2; // 220
const CARD_Y = (1080 - CARD_H) / 2; // 100
const CARD_PAD = 56;

// Reprocess-row geometry (in comp-space pixels). Derived from the
// typography above it: 56-pad + 72-title + 28-gap + 28-sub + 40-gap
// = row-top. Row height is 100 px, so row-centre-Y = row-top + 50.
const HEADER_BLOCK_H = 72 + 14 + 34 + 40; // title + gap + sub + trailing gap
const REPROCESS_ROW_TOP = CARD_Y + CARD_PAD + HEADER_BLOCK_H;
const REPROCESS_ROW_H = 100;
const REPROCESS_ROW_CY = REPROCESS_ROW_TOP + REPROCESS_ROW_H / 2;

// Column that contains the dropdown trigger.
const DROPDOWN_X_LEFT = CARD_X + CARD_PAD + 240;   // after "Reprocess as" label
const DROPDOWN_W = 700;
const DROPDOWN_CX = DROPDOWN_X_LEFT + DROPDOWN_W / 2;

// Run button centre.
const RUN_CX = CARD_X + CARD_W - CARD_PAD - 80;

// Dropdown panel geometry (below the trigger, with a 10 px gap).
const PANEL_TOP = REPROCESS_ROW_TOP + REPROCESS_ROW_H + 10;
const PANEL_ROW_H = 66;
// Row Y-centres inside the opened panel (three options).
const panelRowCy = (i: number) => PANEL_TOP + 8 + i * PANEL_ROW_H + PANEL_ROW_H / 2;

// Off-frame anchor.
const CURSOR_OFF_X = 1820;
const CURSOR_OFF_Y = 1080;

export function FeatureHistoryRetranscribe() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // --- timing --------------------------------------------------------------
  const cursorInStart = f(0.5);
  const cursorInEnd = f(1.1);
  const clickDropdownAt = f(1.15);
  const panelOpenEnd = f(1.7);
  const cursorToMeetingStart = f(2.1);
  const cursorToMeetingEnd = f(2.6);
  const clickMeetingAt = f(2.7);
  const cursorToRunStart = f(2.9);
  const cursorToRunEnd = f(3.4);
  const clickRunAt = f(3.5);
  const rewriteStart = f(3.6);
  const rewriteEnd = f(6.4);
  const cursorOutStart = f(6.4);
  const cursorOutEnd = f(7.0);

  // --- state ---------------------------------------------------------------
  // Dropdown label: shows "Meeting" the moment the user clicks it.
  const selectedLabel = frame >= clickMeetingAt ? 'Meeting' : 'Voice-to-text';
  // Panel is open from dropdown-click until meeting-click.
  const panelOpen =
    frame >= clickDropdownAt && frame < clickMeetingAt;
  const panelOpenT = tween(frame, clickDropdownAt, panelOpenEnd, 0, 1);
  // Hover highlight moves to "Meeting" once the cursor arrives.
  const hoverMeeting = frame >= cursorToMeetingEnd - 4 && frame < clickMeetingAt + 3;

  // Processed text — voice until Run, then rewrite into bulleted meeting.
  const showMeeting = frame >= clickRunAt;
  const processedText = showMeeting
    ? typewriter(PROCESSED_MEETING, frame, rewriteStart, rewriteEnd)
    : PROCESSED_VOICE;
  const processedHighlight =
    showMeeting && frame < rewriteEnd + f(0.6);

  // --- cursor path ---------------------------------------------------------
  const cursor = cursorPath(frame, [
    { x: CURSOR_OFF_X, y: CURSOR_OFF_Y, t: cursorInStart },
    { x: DROPDOWN_CX, y: REPROCESS_ROW_CY, t: cursorInEnd },
    { x: DROPDOWN_CX, y: REPROCESS_ROW_CY, t: cursorToMeetingStart },
    { x: DROPDOWN_CX, y: panelRowCy(2), t: cursorToMeetingEnd },
    { x: DROPDOWN_CX, y: panelRowCy(2), t: cursorToRunStart },
    { x: RUN_CX, y: REPROCESS_ROW_CY, t: cursorToRunEnd },
    { x: RUN_CX, y: REPROCESS_ROW_CY, t: cursorOutStart },
    { x: CURSOR_OFF_X, y: CURSOR_OFF_Y, t: cursorOutEnd },
  ]);
  const cursorVisible = frame >= cursorInStart && frame <= cursorOutEnd + 1;

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* History detail card */}
      <div
        style={{
          position: 'absolute',
          left: CARD_X,
          top: CARD_Y,
          width: CARD_W,
          height: CARD_H,
          background: tokens.bg1,
          borderRadius: 28,
          padding: CARD_PAD,
          border: `1px solid ${tokens.border}`,
          boxSizing: 'border-box',
          display: 'flex',
          flexDirection: 'column',
          gap: 24,
          boxShadow: '0 32px 80px rgba(0,0,0,0.5)',
        }}
      >
        {/* Header — date + mode + app. Anchored so layout constants
            above match reality. */}
        <div>
          <div
            style={{
              fontSize: 56,
              fontWeight: 700,
              color: tokens.fgHi,
              letterSpacing: '-0.025em',
              lineHeight: 1,
            }}
          >
            {selectedLabel}
          </div>
          <div
            style={{
              fontSize: 24,
              color: tokens.muted,
              marginTop: 14,
              fontFamily: fonts.mono,
            }}
          >
            Thu 23 Apr 2026 · 14:02 · slack.exe · #release
          </div>
        </div>

        {/* Reprocess row — label + dropdown + Run button */}
        <div
          style={{
            position: 'relative',
            height: REPROCESS_ROW_H,
            background: tokens.bg2,
            borderRadius: 18,
            padding: '0 24px',
            display: 'flex',
            alignItems: 'center',
            gap: 20,
            border: `1px solid ${tokens.border}`,
          }}
        >
          <div style={{ fontSize: 24, color: tokens.accent, flexShrink: 0 }}>✨</div>
          <div
            style={{
              fontSize: 26,
              color: tokens.fg,
              flexShrink: 0,
              fontWeight: 500,
            }}
          >
            Reprocess as
          </div>
          {/* Dropdown trigger */}
          <div
            style={{
              flex: 1,
              background: tokens.bg1,
              borderRadius: 12,
              padding: '14px 22px',
              fontSize: 28,
              color: tokens.fgHi,
              border: `2px solid ${panelOpen ? 'rgba(255,255,255,0.35)' : tokens.borderStrong}`,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              transition: 'border 120ms linear',
            }}
          >
            <span>{selectedLabel}</span>
            <span style={{ color: tokens.fgDim, fontSize: 22 }}>▾</span>
          </div>
          {/* Run button */}
          <div
            style={{
              padding: '14px 34px',
              background: frame >= clickRunAt ? tokens.accent : tokens.bg1,
              color: frame >= clickRunAt ? tokens.bg0 : tokens.fgHi,
              border: `1px solid ${frame >= clickRunAt ? tokens.accent : tokens.borderStrong}`,
              borderRadius: 12,
              fontSize: 26,
              fontWeight: 500,
              fontFamily: fonts.sans,
              flexShrink: 0,
              boxShadow:
                frame >= cursorToRunEnd - 4 && frame < clickRunAt + 6
                  ? `0 0 0 2px rgba(34, 211, 238, 0.55)`
                  : 'none',
              transition: 'background 120ms linear, color 120ms linear',
            }}
          >
            Run
          </div>
        </div>

        {/* Dropdown panel — absolutely positioned below the row so it
            overlays the transcripts without shifting layout. */}
        {panelOpen && (
          <div
            style={{
              position: 'absolute',
              left: DROPDOWN_X_LEFT - CARD_X,
              top: PANEL_TOP - CARD_Y,
              width: DROPDOWN_W,
              background: tokens.bg2,
              borderRadius: 14,
              padding: 8,
              border: `1px solid ${tokens.borderStrong}`,
              opacity: panelOpenT,
              transform: `translateY(${(1 - panelOpenT) * -6}px)`,
              zIndex: 3,
              boxShadow: '0 18px 40px rgba(0,0,0,0.6)',
            }}
          >
            {['Voice-to-text', 'Message', 'Meeting'].map((o, i) => {
              const isHover = hoverMeeting && o === 'Meeting';
              return (
                <div
                  key={o}
                  style={{
                    height: PANEL_ROW_H - 8,
                    display: 'flex',
                    alignItems: 'center',
                    padding: '0 16px',
                    borderRadius: 10,
                    background: isHover ? tokens.bg1 : 'transparent',
                    color: isHover ? tokens.fgHi : tokens.fg,
                    fontSize: 26,
                    transition: 'background 100ms linear',
                  }}
                >
                  {o}
                </div>
              );
            })}
          </div>
        )}

        {/* Transcripts — raw on top, processed below. */}
        <Block label="Raw transcript" text={RAW} />
        <Block label="Processed" text={processedText} highlight={processedHighlight} />
      </div>

      {/* Click pulses — sit over the cursor targets */}
      <ClickPulse frame={frame} startFrame={clickDropdownAt} x={DROPDOWN_CX} y={REPROCESS_ROW_CY} />
      <ClickPulse frame={frame} startFrame={clickMeetingAt} x={DROPDOWN_CX} y={panelRowCy(2)} />
      <ClickPulse frame={frame} startFrame={clickRunAt} x={RUN_CX} y={REPROCESS_ROW_CY} />

      {cursorVisible && <Cursor x={cursor.x} y={cursor.y} />}
    </AbsoluteFill>
  );
}

function Block({
  label,
  text,
  highlight = false,
}: {
  label: string;
  text: string;
  highlight?: boolean;
}) {
  return (
    <div>
      <div
        style={{
          fontFamily: fonts.mono,
          fontSize: 18,
          fontWeight: 500,
          letterSpacing: '0.14em',
          textTransform: 'uppercase',
          color: tokens.muted,
          marginBottom: 10,
        }}
      >
        {label}
      </div>
      <div
        style={{
          background: tokens.bg2,
          borderRadius: 14,
          padding: '22px 26px',
          fontSize: 26,
          lineHeight: 1.55,
          color: tokens.fg,
          whiteSpace: 'pre-wrap',
          border: `2px solid ${highlight ? 'rgba(34, 211, 238, 0.55)' : 'transparent'}`,
          boxShadow: highlight
            ? `0 0 28px rgba(34, 211, 238, 0.35), 0 0 0 3px rgba(34, 211, 238, 0.12)`
            : 'none',
          transition: 'border 160ms linear, box-shadow 160ms linear',
          minHeight: 100,
        }}
      >
        {text}
      </div>
    </div>
  );
}
