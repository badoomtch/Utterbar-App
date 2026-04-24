/**
 * feature-file-transcription — 8 s. Just the Files route's main card —
 * dropzone at the top, queue of three jobs underneath. No AppShell
 * sidebar, no page header; the site puts the textual copy next to the
 * video so the video shouldn't compete. Jobs stagger in, progress, and
 * flip to done. Ends on all three complete for a clean loop.
 *
 * No cursor here — the interaction that matters (dropping files) has
 * already happened; the reveal IS the UI doing its job. Adding a
 * cursor that doesn't click anything would muddy the beat.
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { tween } from '../util/anim';

const JOBS = [
  { name: 'team-standup-2026-04-22.mp3', size: '18.4 MB', duration: 1824 },
  { name: 'interview-kiran.m4a', size: '42.1 MB', duration: 2736 },
  { name: 'design-crit-0412.mov', size: '612 MB', duration: 3480 },
];

// ---- layout ---------------------------------------------------------------
const CARD_W = 1680;
const CARD_H = 900;
const CARD_X = (1920 - CARD_W) / 2; // 120
const CARD_Y = (1080 - CARD_H) / 2; // 90
const CARD_PAD = 56;

export function FeatureFileTranscription() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
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
          gap: 32,
          boxShadow: '0 32px 80px rgba(0,0,0,0.5)',
        }}
      >
        {/* Section header */}
        <div>
          <div
            style={{
              fontSize: 56,
              fontWeight: 700,
              color: tokens.fgHi,
              letterSpacing: '-0.025em',
              lineHeight: 1,
              marginBottom: 14,
            }}
          >
            Files
          </div>
          <div style={{ fontSize: 24, color: tokens.muted }}>
            Drop audio or video — transcripts save next to the original as .utterbar.txt
          </div>
        </div>

        {/* Dropzone — big target, matches the app's real dashed-outline
            variant with accent upload icon. */}
        <div
          style={{
            border: `3px dashed ${tokens.borderStrong}`,
            borderRadius: 22,
            padding: 44,
            textAlign: 'center',
            background: 'rgba(255,255,255,0.02)',
          }}
        >
          <div
            style={{
              width: 96,
              height: 96,
              borderRadius: 20,
              background: tokens.bg2,
              margin: '0 auto 20px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              border: `1px solid ${tokens.border}`,
            }}
          >
            <svg
              width={44}
              height={44}
              viewBox="0 0 24 24"
              fill="none"
              stroke={tokens.accent}
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
          </div>
          <div style={{ fontSize: 32, fontWeight: 500, color: tokens.fgHi, letterSpacing: '-0.01em' }}>
            Drop files here, or click to browse
          </div>
          <div style={{ fontSize: 22, color: tokens.muted, marginTop: 12, fontFamily: fonts.mono }}>
            .mp3 · .wav · .m4a · .flac · .ogg · .mp4 · .mov · .mkv
          </div>
        </div>

        {/* Queue */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 14, minHeight: 0 }}>
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 20,
              letterSpacing: '0.14em',
              textTransform: 'uppercase',
              color: tokens.muted,
            }}
          >
            Queue · {JOBS.length} files
          </div>
          {JOBS.map((job, i) => {
            // Stagger entry 0.3 s apart; each job runs 2.0 s then flips done.
            const enterAt = f(0.4 + i * 0.35);
            const startAt = f(1.0 + i * 0.7);
            const endAt = startAt + f(2.2);
            const opacity = tween(frame, enterAt, enterAt + 10, 0, 1);
            const progress = Math.min(1, Math.max(0, (frame - startAt) / (endAt - startAt)));
            const done = frame >= endAt;
            return (
              <div
                key={job.name}
                style={{
                  background: tokens.bg2,
                  borderRadius: 16,
                  padding: '22px 26px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 22,
                  opacity,
                  border: `1px solid ${tokens.border}`,
                }}
              >
                <div
                  style={{
                    width: 56,
                    height: 56,
                    borderRadius: 12,
                    background: done ? 'rgba(52, 211, 153, 0.15)' : 'rgba(34, 211, 238, 0.12)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                  }}
                >
                  {done ? (
                    <svg
                      width={26}
                      height={26}
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke={tokens.moss}
                      strokeWidth="3"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    >
                      <path d="M5 12l4 4L19 6" />
                    </svg>
                  ) : (
                    <svg width={22} height={22} viewBox="0 0 24 24" fill="none">
                      <circle cx="12" cy="12" r="10" stroke={tokens.accent} strokeOpacity="0.25" strokeWidth="3" />
                      <path
                        d="M12 2a10 10 0 0 1 10 10"
                        stroke={tokens.accent}
                        strokeWidth="3"
                        strokeLinecap="round"
                        fill="none"
                      />
                    </svg>
                  )}
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      fontSize: 28,
                      color: tokens.fgHi,
                      fontWeight: 500,
                      letterSpacing: '-0.01em',
                      fontFamily: fonts.mono,
                    }}
                  >
                    {job.name}
                  </div>
                  <div
                    style={{
                      fontFamily: fonts.mono,
                      fontSize: 20,
                      color: tokens.muted,
                      marginTop: 6,
                    }}
                  >
                    {job.size} · {formatSecs(job.duration)}
                    {done
                      ? ' · saved as .utterbar.txt'
                      : progress > 0
                        ? ` · ${Math.round(progress * 100)} %`
                        : ' · queued'}
                  </div>
                </div>
                {/* Progress bar */}
                <div
                  style={{
                    width: 220,
                    height: 8,
                    background: tokens.bg1,
                    borderRadius: 999,
                    overflow: 'hidden',
                    flexShrink: 0,
                  }}
                >
                  <div
                    style={{
                      width: `${progress * 100}%`,
                      height: '100%',
                      background: done ? tokens.moss : tokens.accent,
                      transition: 'background 120ms linear',
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

function formatSecs(s: number) {
  const m = Math.floor(s / 60);
  const r = s % 60;
  return `${m}:${r.toString().padStart(2, '0')}`;
}
