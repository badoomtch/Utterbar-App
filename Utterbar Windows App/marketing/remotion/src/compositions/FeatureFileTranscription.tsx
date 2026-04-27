/**
 * feature-file-transcription — 12 s. Two-act narrative:
 *
 *   Act 1 (0–7s): the Files route's queue moves three jobs through
 *   queued → running → done.
 *
 *   Act 2 (7–12s): the user "clicks" a Done row. The transcript modal
 *   slides up showing real text content, with the Copy / Show in folder
 *   actions visible. This addresses the "what happens after?" question
 *   the v1 composition left dangling.
 *
 * Frame is 1440×900; both acts share the same canvas. Act 2 dims the
 * Act 1 surface and overlays the modal — same visual model as the live
 * app, where the transcript modal sits over the route on a scrim.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween } from '../util/anim';

type Status = 'queued' | 'running' | 'done';

const JOBS: { name: string; kind: string; seconds: number }[] = [
  { name: 'Team standup — 2026-04-18.mp4', kind: 'Video', seconds: 1820 },
  { name: 'Podcast draft — ep 14.m4a',     kind: 'Audio', seconds: 2680 },
  { name: 'Interview — Jane Doe.wav',      kind: 'Audio', seconds: 3140 },
];

// Pre-written transcript that lands when the user opens the modal.
// First few paragraphs of the team-standup job. Real-feeling content
// (specific names, specific decisions) so the viewer reads it as an
// actual transcript, not lorem ipsum.
const TRANSCRIPT = [
  'Right, standup. Quick one today — three things to land before Friday.',
  '',
  'First, the auth migration is good to go. QA flagged two edge cases yesterday but both turned out to be the test fixtures, not the production path. I rebuilt the fixtures and reran — green across the board.',
  '',
  'Second, the pricing endpoint. We agreed last week to cache it for 60 seconds at the edge. That landed in the Tuesday deploy. Watching the response times now and the p95 has dropped from 280ms to 14ms — Cloudflare is doing all the work.',
  '',
  'Third, marketing. They want copy for the new comparison page by Thursday. I\'ve got a draft in the doc; please leave comments. I\'ll do another pass tomorrow once everyone\'s read it.',
  '',
  'That\'s it from me. Anyone blocked?',
].join('\n');

function statusAt(i: number, frame: number, fps: number): { status: Status; progress: number } {
  const f = (s: number) => Math.round(s * fps);
  // Compress the queue narrative into the first 6.5s so Act 2 has room.
  // 0.0–0.7  all queued
  // 0.7–2.3  job 0 running (0→1), jobs 1,2 queued
  // 2.3–2.4  job 0 done
  // 2.4–4.0  job 1 running, job 2 queued
  // 4.0–4.1  job 1 done
  // 4.1–5.5  job 2 running
  // 5.5–6.5  all done; row 0 highlights pre-click
  if (i === 0) {
    if (frame < f(0.7)) return { status: 'queued',  progress: 0 };
    if (frame < f(2.3)) return { status: 'running', progress: (frame - f(0.7)) / (f(2.3) - f(0.7)) };
    return { status: 'done', progress: 1 };
  }
  if (i === 1) {
    if (frame < f(2.4)) return { status: 'queued',  progress: 0 };
    if (frame < f(4.0)) return { status: 'running', progress: (frame - f(2.4)) / (f(4.0) - f(2.4)) };
    return { status: 'done', progress: 1 };
  }
  if (frame < f(4.1)) return { status: 'queued',  progress: 0 };
  if (frame < f(5.5)) return { status: 'running', progress: (frame - f(4.1)) / (f(5.5) - f(4.1)) };
  return { status: 'done', progress: 1 };
}

function Duration({ seconds }: { seconds: number }) {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return (
    <span style={{ fontFamily: fonts.mono, fontSize: 17, color: tokens.muted }}>
      {m}:{s.toString().padStart(2, '0')}
    </span>
  );
}

function StatusChip({ status }: { status: Status }) {
  const conf =
    status === 'running'
      ? { label: 'Running', color: tokens.accent }
      : status === 'done'
        ? { label: 'Done', color: tokens.moss }
        : { label: 'Queued', color: tokens.muted };
  return (
    <span
      style={{
        fontFamily: fonts.mono,
        fontSize: 16,
        textTransform: 'uppercase',
        letterSpacing: '0.14em',
        color: conf.color,
      }}
    >
      ● {conf.label}
    </span>
  );
}

export function FeatureFileTranscription() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const dropIn = tween(frame, f(0.15), f(0.7), 0, 1);
  const dropHover = tween(frame, f(0.3), f(0.6), 0, 1);

  // Modal arc:
  //   6.8 – 7.0   click highlight on row 0
  //   7.0 – 7.5   modal slides up + scrim fades in
  //   7.5 – 11.0  modal hold; "Copied!" label flashes around 9.0s
  //   11.0 – 12.0 modal sits; composition ends on it (no exit anim
  //               because trailing frames may not always render)
  const modalProgress = tween(frame, f(7.0), f(7.5), 0, 1);
  const scrimOpacity = modalProgress * 0.55;
  const rowHighlight = tween(frame, f(6.6), f(7.0), 0, 1);

  // Copy button "click" highlight + 1.4s "Copied!" tooltip flash.
  const copiedFlash =
    tween(frame, f(8.6), f(8.8), 0, 1) * tween(frame, f(9.8), f(10.0), 1, 0);

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* === ACT 1 SURFACE — Files route === */}
      {/* Dimmed slightly when the modal is up, never fully removed so the
          viewer reads the modal as "on top of" rather than "instead of". */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          opacity: 1 - modalProgress * 0.35,
          filter: modalProgress > 0.1 ? `blur(${modalProgress * 1.5}px)` : 'none',
        }}
      >
        {/* Header strip */}
        <div
          style={{
            position: 'absolute',
            left: '50%',
            top: 70,
            transform: 'translateX(-50%)',
            textAlign: 'center',
            opacity: dropIn,
          }}
        >
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 18,
              letterSpacing: '0.18em',
              textTransform: 'uppercase',
              color: tokens.muted,
            }}
          >
            Files
          </div>
        </div>

        {/* Dropzone */}
        <div
          style={{
            position: 'absolute',
            left: '50%',
            top: 130,
            transform: 'translateX(-50%)',
            width: 1080,
            maxWidth: '90%',
            height: 180,
            borderRadius: 20,
            background: 'rgba(34, 211, 238, 0.04)',
            border: `2px dashed rgba(34, 211, 238, ${0.3 + dropHover * 0.3})`,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            flexDirection: 'column',
            gap: 10,
            opacity: dropIn,
          }}
        >
          <div style={{ fontSize: 22, color: tokens.fgHi, fontWeight: 500 }}>
            Drop audio or video files here
          </div>
          <div style={{ fontSize: 18, color: tokens.muted }}>
            WAV, MP3, M4A, MP4, MKV — transcribed locally to .txt, .srt or .vtt
          </div>
        </div>

        {/* Queue */}
        <div
          style={{
            position: 'absolute',
            left: '50%',
            top: 360,
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
              marginBottom: 14,
            }}
          >
            Queue
          </div>
          <div
            style={{
              background: tokens.bg1,
              borderRadius: 18,
              border: `1px solid ${tokens.border}`,
              overflow: 'hidden',
            }}
          >
            {JOBS.map((j, i) => {
              const { status, progress } = statusAt(i, frame, fps);
              const highlightRow = i === 0 && rowHighlight > 0.05 && modalProgress < 0.15;
              return (
                <div
                  key={j.name}
                  style={{
                    padding: '20px 24px',
                    borderTop: i === 0 ? 'none' : `1px solid ${tokens.border}`,
                    background: highlightRow
                      ? `rgba(34, 211, 238, ${rowHighlight * 0.08})`
                      : 'transparent',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: 10,
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 16 }}>
                    <div style={{ display: 'flex', alignItems: 'baseline', gap: 12, flex: 1, minWidth: 0 }}>
                      <span
                        style={{
                          fontSize: 18,
                          color: tokens.fgHi,
                          whiteSpace: 'nowrap',
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          maxWidth: 520,
                        }}
                      >
                        {j.name}
                      </span>
                      <span
                        style={{
                          fontFamily: fonts.mono,
                          fontSize: 16,
                          textTransform: 'uppercase',
                          letterSpacing: '0.14em',
                          color: tokens.fgDim,
                        }}
                      >
                        {j.kind}
                      </span>
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
                      <Duration seconds={j.seconds} />
                      <StatusChip status={status} />
                    </div>
                  </div>
                  <div
                    style={{
                      height: 4,
                      borderRadius: 2,
                      background: 'rgba(255,255,255,0.06)',
                      overflow: 'hidden',
                    }}
                  >
                    <div
                      style={{
                        width: `${progress * 100}%`,
                        height: '100%',
                        background: status === 'done' ? tokens.moss : tokens.accent,
                      }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>

      {/* === ACT 2 SCRIM === */}
      {scrimOpacity > 0.01 && (
        <div
          style={{
            position: 'absolute',
            inset: 0,
            background: `rgba(0, 0, 0, ${scrimOpacity})`,
          }}
        />
      )}

      {/* === ACT 2 MODAL === */}
      {modalProgress > 0.01 && (
        <div
          style={{
            position: 'absolute',
            left: '50%',
            top: '50%',
            transform: `translate(-50%, calc(-50% + ${(1 - modalProgress) * 24}px))`,
            opacity: modalProgress,
            width: 980,
            maxWidth: '88%',
            maxHeight: 720,
            background: tokens.bg1,
            borderRadius: 18,
            border: `1px solid ${tokens.border}`,
            boxShadow: '0 20px 60px rgba(0, 0, 0, 0.5)',
            display: 'flex',
            flexDirection: 'column',
            overflow: 'hidden',
          }}
        >
          {/* Modal header — filename + path */}
          <div
            style={{
              padding: '18px 24px 14px 24px',
              display: 'flex',
              alignItems: 'flex-start',
              justifyContent: 'space-between',
              gap: 16,
            }}
          >
            <div style={{ minWidth: 0, flex: 1 }}>
              <div
                style={{
                  fontSize: 18,
                  fontWeight: 500,
                  color: tokens.fgHi,
                  whiteSpace: 'nowrap',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                }}
              >
                Team standup — 2026-04-18.mp4
              </div>
              <div
                style={{
                  marginTop: 4,
                  fontFamily: fonts.mono,
                  fontSize: 13,
                  color: tokens.muted,
                  whiteSpace: 'nowrap',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                }}
              >
                C:\Users\you\Documents\Recordings\Team standup — 2026-04-18.mp4
              </div>
            </div>
            <div
              style={{
                fontSize: 18,
                lineHeight: 1,
                color: tokens.muted,
                paddingTop: 4,
              }}
            >
              ✕
            </div>
          </div>

          {/* Transcript pane */}
          <div
            style={{
              flex: 1,
              minHeight: 380,
              maxHeight: 420,
              overflow: 'hidden',
              borderTop: `1px solid ${tokens.border}`,
              borderBottom: `1px solid ${tokens.border}`,
              background: tokens.bg0,
              padding: '24px 28px',
              fontFamily: fonts.sans,
              fontSize: 16,
              lineHeight: 1.7,
              color: tokens.fg,
              whiteSpace: 'pre-wrap',
              // Subtle reveal of the transcript text — fades in over 0.6s
              // from the modal-open mark so the eye reads modal first,
              // text second.
              opacity: tween(frame, f(7.5), f(8.1), 0, 1),
            }}
          >
            {TRANSCRIPT}
          </div>

          {/* Footer — Copy + Show in folder */}
          <div
            style={{
              padding: '14px 18px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              gap: 10,
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, position: 'relative' }}>
              {/* Copy button — flashes accent when "clicked" at 8.7s */}
              <div
                style={{
                  position: 'relative',
                  padding: '8px 14px',
                  borderRadius: 10,
                  background: copiedFlash > 0.05
                    ? `rgba(34, 211, 238, ${0.18 + copiedFlash * 0.2})`
                    : tokens.accent,
                  color: copiedFlash > 0.05 ? tokens.accent : '#06121A',
                  fontSize: 14,
                  fontWeight: 600,
                  border: copiedFlash > 0.05 ? `1px solid ${tokens.accent}` : 'none',
                }}
              >
                {copiedFlash > 0.5 ? 'Copied' : 'Copy'}
              </div>
              <div
                style={{
                  padding: '8px 14px',
                  borderRadius: 10,
                  background: 'transparent',
                  color: tokens.fg,
                  fontSize: 14,
                  fontWeight: 500,
                  border: `1px solid ${tokens.border}`,
                }}
              >
                Show in folder
              </div>
              {/* Floating "Copied!" pill — visible during the flash window */}
              {copiedFlash > 0.05 && (
                <div
                  style={{
                    position: 'absolute',
                    top: -36,
                    left: 4,
                    background: tokens.bg1,
                    border: `1px solid rgba(34, 211, 238, ${0.4 + copiedFlash * 0.3})`,
                    padding: '4px 10px',
                    borderRadius: 999,
                    fontSize: 12,
                    color: tokens.accent,
                    fontFamily: fonts.mono,
                    letterSpacing: '0.05em',
                    opacity: copiedFlash,
                  }}
                >
                  ✓ in clipboard
                </div>
              )}
            </div>
            <div
              style={{
                fontSize: 13,
                color: tokens.muted,
                fontFamily: fonts.mono,
                letterSpacing: '0.02em',
              }}
            >
              Delete
            </div>
          </div>
        </div>
      )}
    </AbsoluteFill>
  );
}
