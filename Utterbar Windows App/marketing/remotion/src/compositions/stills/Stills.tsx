/**
 * Press stills — 2200 × 1400 single-frame compositions. Each is framed
 * tight on its subject so the visible UI is large at any display size.
 * No AppShell chrome (nav rails, window frames, sidebars), no radial
 * washes — every still lives directly on the site's --bg-0 so the PNG
 * drops into any page section without a visible seam.
 *
 *   press-01-floating-bar-idle       — pill in Ready state, close shot
 *   press-02-floating-bar-listening  — pill mid-recording, 00:04 timer
 *   press-03-main-shell-dashboard    — Dashboard metrics + recent activity card
 *   press-04-vocabulary              — vocabulary + replacements cards
 *   press-05-file-transcription      — dropzone + 3 jobs, no shell
 *
 * All built from the same tokens + BarDriver as the videos, so any
 * component change propagates automatically.
 */
import React from 'react';
import { AbsoluteFill } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { BarDriver } from '../../bar/BarDriver';

// ---------- press-01 — floating bar idle ---------------------------------
export function Press01FloatingBarIdle() {
  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      <AbsoluteFill
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <div style={{ transform: 'scale(3.8)', transformOrigin: 'center' }}>
          <BarDriver state="idle" />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
}

// ---------- press-02 — floating bar listening ----------------------------
export function Press02FloatingBarListening() {
  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      <AbsoluteFill
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <div style={{ transform: 'scale(3.8)', transformOrigin: 'center' }}>
          <BarDriver state="listening" elapsedMs={4000} />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
}

// ---------- press-03 — main shell dashboard (card only) ------------------
export function Press03MainShellDashboard() {
  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans, padding: '110px 120px 90px' }}>
      <div style={{ fontFamily: fonts.mono, fontSize: 18, letterSpacing: '0.18em', textTransform: 'uppercase', color: tokens.muted, marginBottom: 10 }}>
        Dashboard
      </div>
      <div style={{ fontSize: 52, fontWeight: 700, color: tokens.fgHi, letterSpacing: '-0.02em', marginBottom: 8 }}>
        Your time saved, at a glance.
      </div>
      <div style={{ fontSize: 22, color: tokens.muted, marginBottom: 48 }}>
        Every dictation is logged locally so you can see what the app is saving you.
      </div>

      {/* Key metrics — large tiles */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 22, marginBottom: 42 }}>
        <BigMetric value="47" label="Today" accent />
        <BigMetric value="23 min" label="Saved today" />
        <BigMetric value="312" label="This week" />
        <BigMetric value="4 h 18 m" label="Saved this week" moss />
      </div>

      {/* Recent activity block */}
      <div style={{ background: tokens.bg1, borderRadius: 22, padding: '32px 36px', border: `1px solid ${tokens.border}` }}>
        <div style={{ fontSize: 24, fontWeight: 700, color: tokens.fgHi, marginBottom: 4 }}>Recent activity</div>
        <div style={{ fontSize: 15, color: tokens.muted, marginBottom: 24 }}>Last five dictations.</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
          {RECENT.map((r, i) => (
            <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 18 }}>
              <div style={{ width: 10, height: 10, borderRadius: '50%', background: tokens.accent, flexShrink: 0 }} />
              <div
                style={{
                  flex: 1,
                  fontSize: 18,
                  color: tokens.fg,
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                }}
              >
                {r.text}
              </div>
              <div style={{ fontFamily: fonts.mono, fontSize: 13, color: tokens.muted, textTransform: 'uppercase', letterSpacing: '0.1em' }}>
                {r.mode} · {r.duration}
              </div>
            </div>
          ))}
        </div>
      </div>
    </AbsoluteFill>
  );
}

function BigMetric({ value, label, accent = false, moss = false }: { value: string; label: string; accent?: boolean; moss?: boolean }) {
  const colour = accent ? tokens.accent : moss ? tokens.moss : tokens.fgHi;
  return (
    <div
      style={{
        background: tokens.bg1,
        borderRadius: 20,
        padding: '28px 30px',
        position: 'relative',
        border: `1px solid ${tokens.border}`,
      }}
    >
      {accent && (
        <span
          style={{
            position: 'absolute',
            left: 0,
            top: 28,
            bottom: 28,
            width: 3,
            borderRadius: 999,
            background: tokens.accent,
          }}
        />
      )}
      <div style={{ fontSize: 46, fontWeight: 700, color: colour, letterSpacing: '-0.02em', paddingLeft: 10 }}>{value}</div>
      <div style={{ marginTop: 8, fontSize: 14, color: tokens.muted, paddingLeft: 10, fontFamily: fonts.mono, textTransform: 'uppercase', letterSpacing: '0.14em' }}>
        {label}
      </div>
    </div>
  );
}

const RECENT = [
  { text: "Quick heads up — pushing launch to Tuesday. QA flagged two auth-flow issues.", mode: 'Message', duration: '00:08' },
  { text: "Feature hypothesis: users who complete onboarding in under two minutes…", mode: 'Voice-to-text', duration: '00:12' },
  { text: '• Launch pushed to Tuesday  • QA flagged 2 issues in auth flow', mode: 'Meeting', duration: '00:22' },
  { text: "Thanks for the review — addressed all the comments and re-requested.", mode: 'Message', duration: '00:05' },
  { text: "So basically we need to push the launch because QA flagged two issues…", mode: 'Voice-to-text', duration: '00:09' },
];

// ---------- press-04 — vocabulary (card only) ----------------------------
export function Press04Vocabulary() {
  const WORDS = ['Utterbar', 'Parakeet TDT', 'Jay Parry', 'Cloudways', 'sherpa-onnx', 'WASAPI', 'DirectML', 'R2', 'Ed25519', 'Tauri'];
  const REPL = [
    { from: 'J parrie', to: 'Jay Parry' },
    { from: 'utter bar', to: 'Utterbar' },
    { from: 'ty', to: 'thank you' },
    { from: 'brb', to: 'be right back' },
  ];
  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans, padding: 120 }}>
      <div style={{ fontFamily: fonts.mono, fontSize: 18, letterSpacing: '0.18em', textTransform: 'uppercase', color: tokens.muted, marginBottom: 10 }}>
        Vocabulary
      </div>
      <div style={{ fontSize: 52, fontWeight: 700, color: tokens.fgHi, letterSpacing: '-0.02em', marginBottom: 8 }}>
        Teach Utterbar your jargon.
      </div>
      <div style={{ fontSize: 22, color: tokens.muted, marginBottom: 48 }}>
        Your words bias the decoder. Replacements fire after transcription, before output.
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1.2fr 1fr', gap: 28 }}>
        <div style={{ background: tokens.bg1, borderRadius: 22, padding: '32px 36px', border: `1px solid ${tokens.border}` }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 6 }}>
            <div style={{ fontSize: 24, fontWeight: 700, color: tokens.fgHi }}>Custom words</div>
            <div style={{ fontFamily: fonts.mono, fontSize: 14, color: tokens.muted }}>{WORDS.length} / 50</div>
          </div>
          <div style={{ fontSize: 15, color: tokens.muted, marginBottom: 26 }}>
            Brand names, jargon, rare proper nouns — the decoder biases toward these.
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10 }}>
            {WORDS.map((w) => (
              <div
                key={w}
                style={{
                  padding: '10px 18px',
                  background: tokens.bg2,
                  borderRadius: 999,
                  fontSize: 18,
                  fontFamily: fonts.mono,
                  color: tokens.fgHi,
                }}
              >
                {w}
              </div>
            ))}
          </div>
        </div>

        <div style={{ background: tokens.bg1, borderRadius: 22, padding: '32px 36px', border: `1px solid ${tokens.border}` }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 6 }}>
            <div style={{ fontSize: 24, fontWeight: 700, color: tokens.fgHi }}>Replacements</div>
            <div style={{ fontFamily: fonts.mono, fontSize: 14, color: tokens.muted }}>{REPL.length} / 25</div>
          </div>
          <div style={{ fontSize: 15, color: tokens.muted, marginBottom: 24 }}>
            Literal or regex swaps applied to every mode.
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {REPL.map((r) => (
              <div
                key={r.from}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 12,
                  padding: '14px 18px',
                  background: tokens.bg2,
                  borderRadius: 12,
                  fontSize: 16,
                }}
              >
                <span style={{ fontFamily: fonts.mono, color: tokens.fg }}>{r.from}</span>
                <span style={{ color: tokens.fgDim }}>→</span>
                <span style={{ fontFamily: fonts.mono, color: tokens.accent2 }}>{r.to}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
}

// ---------- press-05 — files (card only) ---------------------------------
export function Press05FileTranscription() {
  const JOBS: { name: string; kind: string; duration: string; status: 'done' | 'running' | 'queued'; pct: number }[] = [
    { name: 'Team standup — 2026-04-22.mp3', kind: 'Audio', duration: '30:24', status: 'done', pct: 100 },
    { name: 'Podcast — ep 14 draft.m4a', kind: 'Audio', duration: '45:36', status: 'running', pct: 62 },
    { name: 'Interview — Jane Doe.wav', kind: 'Audio', duration: '52:11', status: 'queued', pct: 0 },
  ];
  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans, padding: 120 }}>
      <div style={{ fontFamily: fonts.mono, fontSize: 18, letterSpacing: '0.18em', textTransform: 'uppercase', color: tokens.muted, marginBottom: 10 }}>
        Files
      </div>
      <div style={{ fontSize: 52, fontWeight: 700, color: tokens.fgHi, letterSpacing: '-0.02em', marginBottom: 8 }}>
        Drop audio or video, get clean text.
      </div>
      <div style={{ fontSize: 22, color: tokens.muted, marginBottom: 40 }}>
        WAV, MP3, M4A, MP4, MKV — transcribed locally to .txt, .srt or .vtt.
      </div>

      {/* Dropzone */}
      <div
        style={{
          height: 220,
          borderRadius: 24,
          background: 'rgba(34, 211, 238, 0.04)',
          border: `3px dashed rgba(34,211,238,0.45)`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          flexDirection: 'column',
          gap: 12,
          marginBottom: 36,
        }}
      >
        <div style={{ fontSize: 28, color: tokens.fgHi, fontWeight: 500 }}>Drop files here</div>
        <div style={{ fontSize: 16, color: tokens.muted }}>
          Jobs process one at a time — never in the cloud.
        </div>
      </div>

      {/* Queue */}
      <div style={{ background: tokens.bg1, borderRadius: 22, border: `1px solid ${tokens.border}`, overflow: 'hidden' }}>
        {JOBS.map((j, i) => {
          const colour = j.status === 'done' ? tokens.moss : j.status === 'running' ? tokens.accent : tokens.muted;
          return (
            <div
              key={j.name}
              style={{
                padding: '24px 32px',
                borderTop: i === 0 ? 'none' : `1px solid ${tokens.border}`,
                display: 'flex',
                flexDirection: 'column',
                gap: 14,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 20 }}>
                <div style={{ display: 'flex', alignItems: 'baseline', gap: 16, flex: 1, minWidth: 0 }}>
                  <span
                    style={{
                      fontSize: 22,
                      color: tokens.fgHi,
                      whiteSpace: 'nowrap',
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      maxWidth: 900,
                    }}
                  >
                    {j.name}
                  </span>
                  <span style={{ fontFamily: fonts.mono, fontSize: 13, color: tokens.fgDim, letterSpacing: '0.12em', textTransform: 'uppercase' }}>
                    {j.kind} · {j.duration}
                  </span>
                </div>
                <span style={{ fontFamily: fonts.mono, fontSize: 13, color: colour, letterSpacing: '0.14em', textTransform: 'uppercase' }}>
                  ● {j.status}
                </span>
              </div>
              <div style={{ height: 5, background: 'rgba(255,255,255,0.06)', borderRadius: 3, overflow: 'hidden' }}>
                <div style={{ width: `${j.pct}%`, height: '100%', background: colour }} />
              </div>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
}
