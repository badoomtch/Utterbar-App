/**
 * feature-file-transcription — 7 s. The Files route's main surface —
 * dashed dropzone at the top, a short queue below it showing three
 * jobs moving through queued → running → done.
 *
 * Regenerated 2026-04-24. Same concept as before but dropped the outer
 * AppShell chrome (navigation rail, title bar) — the composition is
 * bigger with less padding, so the progress bars and job names actually
 * read at mobile widths.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween } from '../util/anim';

type Status = 'queued' | 'running' | 'done';

const JOBS: { name: string; kind: string; seconds: number }[] = [
  { name: 'Team standup — 2026-04-18.mp4', kind: 'Video', seconds: 1820 },
  { name: 'Podcast draft — ep 14.m4a', kind: 'Audio', seconds: 2680 },
  { name: 'Interview — Jane Doe.wav', kind: 'Audio', seconds: 3140 },
];

function statusAt(i: number, frame: number, fps: number): { status: Status; progress: number } {
  const f = (s: number) => Math.round(s * fps);
  // Timeline (seconds):
  // 0.0–0.8  all queued
  // 0.8–2.5  job 0 running (0→1), jobs 1,2 queued
  // 2.5–2.6  job 0 done
  // 2.6–4.2  job 1 running, job 2 queued
  // 4.2–4.3  job 1 done
  // 4.3–5.7  job 2 running
  // 5.7–7.0  all done
  if (i === 0) {
    if (frame < f(0.8)) return { status: 'queued', progress: 0 };
    if (frame < f(2.5)) return { status: 'running', progress: (frame - f(0.8)) / (f(2.5) - f(0.8)) };
    return { status: 'done', progress: 1 };
  }
  if (i === 1) {
    if (frame < f(2.6)) return { status: 'queued', progress: 0 };
    if (frame < f(4.2)) return { status: 'running', progress: (frame - f(2.6)) / (f(4.2) - f(2.6)) };
    return { status: 'done', progress: 1 };
  }
  // i === 2
  if (frame < f(4.3)) return { status: 'queued', progress: 0 };
  if (frame < f(5.7)) return { status: 'running', progress: (frame - f(4.3)) / (f(5.7) - f(4.3)) };
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

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
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
            return (
              <div
                key={j.name}
                style={{
                  padding: '20px 24px',
                  borderTop: i === 0 ? 'none' : `1px solid ${tokens.border}`,
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
    </AbsoluteFill>
  );
}
