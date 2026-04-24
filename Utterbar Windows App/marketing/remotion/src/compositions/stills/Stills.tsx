/**
 * Press stills — five single-frame compositions at 2560×1600 for the
 * press page + link embeds. Each is an <AbsoluteFill/> with a static
 * layout; `durationInFrames = 1` so they render as a PNG via
 * `remotion still`.
 *
 *   press-01-floating-bar-idle        — pill alone on bg-0, Ready state
 *   press-02-floating-bar-listening   — pill in full listening, 00:04 timer
 *   press-03-main-shell-dashboard     — full app shell, Dashboard route
 *   press-04-vocabulary               — app shell, Vocabulary route
 *   press-05-file-transcription       — app shell, Files route with jobs
 *
 * These re-use the same primitives + chrome the video compositions do,
 * so any token change propagates here automatically.
 */
import React from 'react';
import { AbsoluteFill } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { BarDriver } from '../../bar/BarDriver';
import { AppShell } from '../../chrome/AppShell';

// -------- press-01 --------------------------------------------------------
export function Press01FloatingBarIdle() {
  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: '50%',
          transform: 'translate(-50%, -50%)',
          width: 1400,
          height: 500,
          background: `radial-gradient(ellipse at center, ${tokens.accent}22 0%, transparent 65%)`,
          filter: 'blur(80px)',
        }}
      />
      <AbsoluteFill style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <div style={{ transform: 'scale(2.4)' }}>
          <BarDriver state="idle" />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
}

// -------- press-02 --------------------------------------------------------
export function Press02FloatingBarListening() {
  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: '50%',
          transform: 'translate(-50%, -50%)',
          width: 1400,
          height: 500,
          background: `radial-gradient(ellipse at center, ${tokens.accent}44 0%, transparent 65%)`,
          filter: 'blur(80px)',
        }}
      />
      <AbsoluteFill style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <div style={{ transform: 'scale(2.4)' }}>
          <BarDriver state="listening" elapsedMs={4000} />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
}

// -------- press-03 (dashboard) --------------------------------------------
export function Press03MainShellDashboard() {
  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <AppShell active="dashboard" width={2400} height={1500} tier="pro">
        <DashboardBody />
      </AppShell>
    </AbsoluteFill>
  );
}

function DashboardBody() {
  return (
    <div style={{ padding: '36px 48px', fontFamily: fonts.sans }}>
      <div style={{ fontSize: 30, fontWeight: 700, color: tokens.fgHi, letterSpacing: '-0.02em' }}>Dashboard</div>
      <div style={{ fontSize: 15, color: tokens.muted, marginTop: 6 }}>
        Your time saved, at a glance.
      </div>

      {/* Today/Week strip */}
      <StripHeader label="Today · this week" />
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 16, marginTop: 16 }}>
        <MetricCard value="47" label="Dictations today" accent />
        <MetricCard value="23 min" label="Time saved today" />
        <MetricCard value="312" label="This week" />
        <MetricCard value="4 h 18 m" label="Time saved this week" />
      </div>

      <StripHeader label="Lifetime" />
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 16, marginTop: 16 }}>
        <MetricCard value="12,488" label="Total dictations" />
        <MetricCard value="18 h 42 m" label="Time saved" positive />
        <MetricCard value="99.2 %" label="Successful inserts" />
        <MetricCard value="55 wpm" label="Typing baseline" />
      </div>

      {/* Recent activity */}
      <div style={{ marginTop: 28, background: tokens.bg1, borderRadius: 18, padding: 28 }}>
        <div style={{ fontSize: 18, fontWeight: 700, color: tokens.fgHi, marginBottom: 4 }}>Recent activity</div>
        <div style={{ fontSize: 13, color: tokens.muted, marginBottom: 18 }}>Last 10 dictations.</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          {RECENT.map((r, i) => (
            <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
              <div style={{ width: 8, height: 8, borderRadius: 999, background: tokens.accent, flexShrink: 0 }} />
              <div style={{ flex: 1, fontSize: 14, color: tokens.fg, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {r.text}
              </div>
              <div style={{ fontFamily: fonts.mono, fontSize: 11, color: tokens.fgDim }}>
                {r.ago} · {r.mode} · {r.duration}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

const RECENT = [
  { text: "Quick heads up — pushing launch to Tuesday. QA flagged two auth-flow issues.", ago: '3m ago', mode: 'Message', duration: '00:08' },
  { text: "Feature hypothesis: users who complete onboarding in under two minutes…", ago: '14m ago', mode: 'Voice-to-text', duration: '00:12' },
  { text: '• Launch pushed to Tuesday  • QA flagged 2 issues in auth flow', ago: '38m ago', mode: 'Meeting', duration: '00:22' },
  { text: "Thanks for the review — addressed all the comments and re-requested.", ago: '1h ago', mode: 'Message', duration: '00:05' },
  { text: "So basically we need to push the launch because QA flagged two issues…", ago: '2h ago', mode: 'Voice-to-text', duration: '00:09' },
];

function StripHeader({ label }: { label: string }) {
  return (
    <div
      style={{
        marginTop: 28,
        fontFamily: fonts.sans,
        fontSize: 11,
        fontWeight: 500,
        letterSpacing: '0.14em',
        textTransform: 'uppercase',
        color: tokens.muted,
      }}
    >
      {label}
    </div>
  );
}

function MetricCard({
  value,
  label,
  accent = false,
  positive = false,
}: {
  value: string;
  label: string;
  accent?: boolean;
  positive?: boolean;
}) {
  const colour = accent ? tokens.accent : positive ? tokens.moss : tokens.fgHi;
  return (
    <div style={{ background: tokens.bg1, borderRadius: 18, padding: 22, position: 'relative' }}>
      {accent && (
        <span
          style={{
            position: 'absolute',
            left: 0,
            top: 20,
            bottom: 20,
            width: 2,
            borderRadius: 999,
            background: tokens.accent,
          }}
        />
      )}
      <div style={{ fontSize: 32, fontWeight: 700, color: colour, letterSpacing: '-0.01em', paddingLeft: 6 }}>
        {value}
      </div>
      <div style={{ marginTop: 8, fontSize: 12, color: tokens.muted, paddingLeft: 6 }}>{label}</div>
    </div>
  );
}

// -------- press-04 (vocabulary) -------------------------------------------
export function Press04Vocabulary() {
  const WORDS = ['Utterbar', 'Parakeet', 'sherpa-onnx', 'Tauri', 'WASAPI', 'DirectML', 'monkeytype', 'R2', 'DPAPI', 'NSIS', 'ed25519', 'framer-motion'];
  const REPL = [
    { from: 'ty', to: 'thank you' },
    { from: 'brb', to: 'be right back' },
    { from: 'qa', to: 'QA' },
    { from: 'sig', to: '—\nJay · Utterbar' },
  ];
  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <AppShell active="vocabulary" width={2400} height={1500} tier="pro">
        <div style={{ padding: '36px 48px' }}>
          <div style={{ fontSize: 30, fontWeight: 700, color: tokens.fgHi, letterSpacing: '-0.02em' }}>Vocabulary</div>
          <div style={{ fontSize: 15, color: tokens.muted, marginTop: 6 }}>
            Teach Utterbar your jargon. Applied on-device, before the transcript lands.
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 20, marginTop: 32 }}>
            <div style={{ background: tokens.bg1, borderRadius: 18, padding: 28 }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 4 }}>
                <div style={{ fontSize: 18, fontWeight: 700, color: tokens.fgHi }}>Custom words</div>
                <div style={{ fontFamily: fonts.mono, fontSize: 12, color: tokens.muted }}>
                  {WORDS.length} / 50
                </div>
              </div>
              <div style={{ fontSize: 13, color: tokens.muted, marginBottom: 20 }}>
                Brand names, jargon, rare proper nouns — the ASR biases toward these.
              </div>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
                {WORDS.map((w) => (
                  <div
                    key={w}
                    style={{
                      padding: '6px 12px',
                      background: tokens.bg2,
                      borderRadius: 999,
                      fontSize: 13,
                      fontFamily: fonts.mono,
                      color: tokens.fgHi,
                    }}
                  >
                    {w}
                  </div>
                ))}
              </div>
            </div>

            <div style={{ background: tokens.bg1, borderRadius: 18, padding: 28 }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 4 }}>
                <div style={{ fontSize: 18, fontWeight: 700, color: tokens.fgHi }}>Text replacements</div>
                <div style={{ fontFamily: fonts.mono, fontSize: 12, color: tokens.muted }}>
                  {REPL.length} / 25
                </div>
              </div>
              <div style={{ fontSize: 13, color: tokens.muted, marginBottom: 20 }}>
                Shortcuts that expand automatically — signatures, common phrases, "qa" → "QA".
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                {REPL.map((r) => (
                  <div
                    key={r.from}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 12,
                      padding: '12px 14px',
                      background: tokens.bg2,
                      borderRadius: 10,
                      fontSize: 14,
                    }}
                  >
                    <span style={{ fontFamily: fonts.mono, color: tokens.accent }}>{r.from}</span>
                    <span style={{ color: tokens.fgDim }}>→</span>
                    <span style={{ color: tokens.fg, whiteSpace: 'pre-wrap' }}>{r.to}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </AppShell>
    </AbsoluteFill>
  );
}

// -------- press-05 (files) -------------------------------------------------
export function Press05FileTranscription() {
  const JOBS = [
    { name: 'team-standup-2026-04-22.mp3', size: '18.4 MB', duration: '30:24', status: 'done' as const, pct: 100 },
    { name: 'interview-kiran.m4a', size: '42.1 MB', duration: '45:36', status: 'running' as const, pct: 62 },
    { name: 'design-crit-0412.mov', size: '612 MB', duration: '58:00', status: 'queued' as const, pct: 0 },
    { name: 'podcast-ep-014-master.wav', size: '1.2 GB', duration: '1:02:11', status: 'queued' as const, pct: 0 },
  ];
  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <AppShell active="files" width={2400} height={1500} tier="pro">
        <div style={{ padding: '36px 48px' }}>
          <div style={{ fontSize: 30, fontWeight: 700, color: tokens.fgHi, letterSpacing: '-0.02em' }}>Files</div>
          <div style={{ fontSize: 15, color: tokens.muted, marginTop: 6 }}>
            Drop audio or video files — Utterbar transcribes them locally and saves each transcript next to the original as .utterbar.txt.
          </div>

          <div
            style={{
              marginTop: 28,
              border: `2px dashed ${tokens.borderStrong}`,
              borderRadius: 16,
              padding: 56,
              textAlign: 'center',
              background: 'rgba(255,255,255,0.015)',
            }}
          >
            <div
              style={{
                width: 64,
                height: 64,
                borderRadius: 16,
                background: tokens.bg1,
                margin: '0 auto 16px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <svg width={28} height={28} viewBox="0 0 24 24" fill="none" stroke={tokens.accent} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="17 8 12 3 7 8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
            </div>
            <div style={{ fontSize: 17, fontWeight: 500, color: tokens.fgHi }}>
              Drop files here, or click to browse
            </div>
            <div style={{ fontSize: 12, color: tokens.muted, marginTop: 8, fontFamily: fonts.mono }}>
              .mp3 · .wav · .m4a · .flac · .ogg · .mp4 · .mov · .mkv
            </div>
          </div>

          <div style={{ marginTop: 28, display: 'flex', flexDirection: 'column', gap: 10 }}>
            <div
              style={{
                fontFamily: fonts.mono,
                fontSize: 11,
                letterSpacing: '0.14em',
                textTransform: 'uppercase',
                color: tokens.muted,
              }}
            >
              Queue · {JOBS.length} files
            </div>
            {JOBS.map((j) => {
              const colour =
                j.status === 'done' ? tokens.moss : j.status === 'running' ? tokens.accent : tokens.fgDim;
              return (
                <div
                  key={j.name}
                  style={{
                    background: tokens.bg1,
                    borderRadius: 12,
                    padding: '14px 20px',
                    display: 'flex',
                    alignItems: 'center',
                    gap: 16,
                  }}
                >
                  <div
                    style={{
                      width: 40,
                      height: 40,
                      borderRadius: 8,
                      background: tokens.bg2,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                    }}
                  >
                    {j.status === 'done' ? (
                      <svg width={18} height={18} viewBox="0 0 24 24" fill="none" stroke={tokens.moss} strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M5 12l4 4L19 6" />
                      </svg>
                    ) : j.status === 'running' ? (
                      <svg width={16} height={16} viewBox="0 0 24 24">
                        <circle cx="12" cy="12" r="10" fill="none" stroke={tokens.accent} strokeOpacity="0.25" strokeWidth="3" />
                        <path d="M12 2 a 10 10 0 0 1 10 10" fill="none" stroke={tokens.accent} strokeWidth="3" strokeLinecap="round" />
                      </svg>
                    ) : (
                      <div style={{ width: 8, height: 8, borderRadius: 999, background: tokens.fgDim }} />
                    )}
                  </div>
                  <div style={{ flex: 1 }}>
                    <div style={{ fontSize: 14, color: tokens.fgHi, fontWeight: 500 }}>{j.name}</div>
                    <div style={{ fontFamily: fonts.mono, fontSize: 11, color: tokens.muted, marginTop: 2 }}>
                      {j.size} · {j.duration} · {j.status === 'done' ? 'saved as .utterbar.txt' : j.status === 'running' ? `${j.pct} %` : 'queued'}
                    </div>
                  </div>
                  <div style={{ width: 160, height: 4, background: tokens.bg2, borderRadius: 999, overflow: 'hidden' }}>
                    <div style={{ width: `${j.pct}%`, height: '100%', background: colour }} />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </AppShell>
    </AbsoluteFill>
  );
}
