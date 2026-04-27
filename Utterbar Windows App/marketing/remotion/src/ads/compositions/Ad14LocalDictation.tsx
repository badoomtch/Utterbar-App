/**
 * ad-14-local-dictation — 30.0 s, 1920×1080 landscape master.
 *
 * The hero ad. Eight tight shots, each tied to one feature with a
 * visible cause and result. Designed to land on the website as the
 * primary marketing asset and on YouTube/landscape feeds.
 *
 * Shot map (all timings in seconds, frame == seconds × 30):
 *   0.0  – 2.5   1. Hook         pill idle → listening, "Speak."
 *   2.5  – 7.0   2. Dictation    pill + text field, sentence pastes in
 *   7.0  – 11.0  3. Modes        Message vs Meeting transformation
 *  11.0  – 15.0  4. Vocabulary   Sara → Sarah correction
 *  15.0  – 19.0  5. Files        drop, queue, transcript appears
 *  19.0  – 23.0  6. History      previous transcript opens
 *  23.0  – 27.0  7. Privacy      Models route, all-local proof
 *  27.0  – 30.0  8. Close        wordmark + CTA
 *
 * Real-app reuse: every shot composes the genuine BarDriver, the real
 * design tokens, and the same fonts/colours as the shipping app. The
 * file does NOT embed the longer feature-* compositions because each
 * of those is paced for 7–12s and the brief gives 4–4.5s per shot —
 * a truncated narrative reads worse than a fresh tighter scene.
 *
 * Pacing constraints baked in:
 *   - At most 3 cyan elements per shot (design system §1.6)
 *   - Pill always large enough to read on phone (>= 1.65× scale)
 *   - Each shot has one focal element; supporting UI is under-stated
 *   - No camera drift, no decorative motion. One transition per beat.
 */
import React from 'react';
import { AbsoluteFill, Sequence, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween, typewriter } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';
import { humanType } from '../frame/humanType';

// ---- copy bank ------------------------------------------------------------

const DICTATED_SENTENCE = 'Hi Sarah, just checking whether Tuesday at 3 still works for you.';

const MESSAGE_RAW =
  'we need to push the launch to tuesday because qa flagged two auth issues and we want them fixed before marketing pings the list';
const MESSAGE_SHAPED =
  'Quick heads up. Pushing launch to Tuesday. QA flagged two auth issues we want fixed before marketing sends the announcement.';
const MEETING_SHAPED =
  '• Launch pushed to Tuesday\n• QA flagged 2 auth issues\n• Fix before marketing announce';

// Vocabulary correction — the brief specifies Sara → Sarah explicitly.
const VOCAB_BEFORE = 'Hi Sara, can you review the deck before Friday?';
const VOCAB_AFTER  = 'Hi Sarah, can you review the deck before Friday?';

// History excerpt — short, plausible, reads as "real past work".
const HISTORY_ENTRIES = [
  { title: 'Hi Sarah, just checking whether Tuesday at 3', when: '2 min ago',  mode: 'Voice to Text' },
  { title: 'Quick heads up. Pushing launch to Tuesday',   when: '14 min ago', mode: 'Message' },
  { title: 'Standup notes, auth migration green',          when: '1 hr ago',   mode: 'Meeting' },
  { title: 'Reply to Marcus on the staging deploy',        when: '3 hr ago',   mode: 'Message' },
];

const HISTORY_DETAIL_BODY =
  'Hi Sarah, just checking whether Tuesday at 3 still works for you. Happy to push to Wednesday if it suits better.';

// File transcription job rows — same shape as the shipping Files route.
const FILE_JOBS = [
  { name: 'Team standup, 2026-04-18.mp4', kind: 'Video', dur: '30:20' },
  { name: 'Podcast draft, ep 14.m4a',     kind: 'Audio', dur: '44:40' },
  { name: 'Interview, Jane Doe.wav',      kind: 'Audio', dur: '52:20' },
];

const FILE_TRANSCRIPT =
  'Right, standup. Three things to land before Friday. First, the auth migration is good to go. QA flagged two cases yesterday but both turned out to be the test fixtures, not production.';

// Models — three real shipping engines, names from the live catalogue.
const MODELS = [
  { name: 'Parakeet TDT', size: '1.2 GB', engine: 'Sherpa-ONNX',  note: 'fastest · English' },
  { name: 'Whisper Base', size: '142 MB', engine: 'Sherpa-ONNX',  note: 'multilingual' },
  { name: 'Whisper Small',size: '466 MB', engine: 'Sherpa-ONNX',  note: 'higher accuracy' },
];

// =========================================================================
//   Top-level composition — composes 8 shots via <Sequence>
// =========================================================================

export function Ad14LocalDictation() {
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  return (
    <AbsoluteFill
      style={{
        background: tokens.bg0,
        fontFamily: fonts.sans,
        color: tokens.fg,
      }}
    >
      {/* Subtle atmospheric wash — top-right cyan, bottom-left cyan, dies
          out into the brand plate. Same recipe as the website hero glow,
          calibrated to be present everywhere without competing with the UI. */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          background: `
            radial-gradient(60% 50% at 92% 6%, ${tokens.accent}1f 0%, transparent 65%),
            radial-gradient(50% 45% at 8% 96%, ${tokens.accent3}1c 0%, transparent 70%)
          `,
          pointerEvents: 'none',
        }}
      />

      <Sequence from={f(0.0)}  durationInFrames={f(2.5)}>  <Shot1Hook />          </Sequence>
      <Sequence from={f(2.5)}  durationInFrames={f(4.5)}>  <Shot2Dictation />     </Sequence>
      <Sequence from={f(7.0)}  durationInFrames={f(4.0)}>  <Shot3Modes />         </Sequence>
      <Sequence from={f(11.0)} durationInFrames={f(4.0)}>  <Shot4Vocabulary />    </Sequence>
      <Sequence from={f(15.0)} durationInFrames={f(4.0)}>  <Shot5Files />         </Sequence>
      <Sequence from={f(19.0)} durationInFrames={f(4.0)}>  <Shot6History />       </Sequence>
      <Sequence from={f(23.0)} durationInFrames={f(4.0)}>  <Shot7Privacy />       </Sequence>
      <Sequence from={f(27.0)} durationInFrames={f(3.0)}>  <Shot8Close />         </Sequence>
    </AbsoluteFill>
  );
}

// =========================================================================
//   Shot 1 — Hook (0.0 – 2.5s)
// =========================================================================

function Shot1Hook() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Pill enters listening at f(0.7), elapsed clock starts then.
  const listening = frame >= f(0.7);
  const state: 'idle' | 'listening' = listening ? 'listening' : 'idle';
  const elapsedMs = listening ? ((frame - f(0.7)) / fps) * 1000 : 0;

  const pillFade = tween(frame, 0, 12, 0, 1);
  const eyebrowFade = tween(frame, f(0.2), f(0.8), 0, 1);
  const headlineFade = tween(frame, f(0.4), f(1.0), 0, 1);
  const headlineSlide = tween(frame, f(0.4), f(1.0), 16, 0);

  return (
    <AbsoluteFill style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 80 }}>
        <div style={{ transform: 'scale(2.0)', transformOrigin: 'center', opacity: pillFade }}>
          <BarDriver state={state} elapsedMs={elapsedMs} />
        </div>

        <div style={{ textAlign: 'center', marginTop: 60 }}>
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 22,
              letterSpacing: '0.22em',
              textTransform: 'uppercase',
              color: tokens.muted,
              marginBottom: 22,
              opacity: eyebrowFade,
            }}
          >
            Utterbar for Windows
          </div>
          <div
            style={{
              fontSize: 168,
              fontWeight: 700,
              color: tokens.fgHi,
              letterSpacing: '-0.04em',
              lineHeight: 1,
              opacity: headlineFade,
              transform: `translateY(${headlineSlide}px)`,
            }}
          >
            Speak.
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
}

// =========================================================================
//   Shot 2 — Dictation into a real app (2.5 – 7.0s)
// =========================================================================

function Shot2Dictation() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Internal timeline (relative to shot start, which is 2.5s global):
  //   0.0 – 0.5   pill enters listening (ramping up from previous shot)
  //   0.5 – 3.2   listening, timer ticks 0:00 → 0:02
  //   3.2 – 3.6   processing
  //   3.6 – 4.1   text pastes in (typewriter)
  //   4.1 – 4.5   pill returns idle, accent glow fades on field
  let state: 'idle' | 'listening' | 'processing' = 'listening';
  const elapsedListenStart = 0;
  if (frame < f(3.2)) state = 'listening';
  else if (frame < f(3.6)) state = 'processing';
  else state = 'idle';

  const elapsedMs = state === 'listening' ? ((frame - elapsedListenStart) / fps) * 1000 : 0;

  // Paste in over 0.5s — fast enough that it reads as "boom, there it is".
  const typed = typewriter(DICTATED_SENTENCE, frame, f(3.6), f(4.1));

  // Field accent glow: rises as text lands, holds, fades.
  const successGlow =
    tween(frame, f(3.6), f(4.0), 0, 1) * tween(frame, f(4.1), f(4.4), 1, 0);

  // Caption fade
  const capFade = Math.min(
    tween(frame, f(0.2), f(0.6), 0, 1),
    tween(frame, f(4.0), f(4.4), 1, 0),
  );

  return (
    <AbsoluteFill>
      {/* Pill — top-centre, large */}
      <div
        style={{
          position: 'absolute',
          top: 90,
          left: '50%',
          transform: 'translateX(-50%) scale(1.85)',
          transformOrigin: 'top center',
        }}
      >
        <BarDriver state={state} elapsedMs={elapsedMs} />
      </div>

      {/* Email-style text field below the pill */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 360,
          transform: 'translateX(-50%)',
          width: 1280,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.16em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 16,
          }}
        >
          To: Sarah Wright
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 18,
            padding: '40px 44px',
            border: `1px solid ${
              successGlow > 0.05
                ? `rgba(34, 211, 238, ${0.15 + successGlow * 0.55})`
                : tokens.border
            }`,
            minHeight: 320,
            fontSize: 38,
            lineHeight: 1.45,
            color: tokens.fg,
            boxShadow:
              successGlow > 0.05
                ? `0 0 0 ${8 * successGlow}px rgba(34, 211, 238, ${0.1 * successGlow})`
                : 'none',
          }}
        >
          {typed || (
            <span style={{ color: tokens.fgDim }}>
              <span
                style={{
                  display: 'inline-block',
                  width: 3,
                  height: 38,
                  background: tokens.accent,
                  verticalAlign: '-7px',
                  marginRight: 4,
                  opacity: Math.floor(frame / 14) % 2 ? 1 : 0,
                }}
              />
            </span>
          )}
        </div>
      </div>

      {/* Caption */}
      <Caption text="Dictate into any app." opacity={capFade} />
    </AbsoluteFill>
  );
}

// =========================================================================
//   Shot 3 — Same thought, different mode (7.0 – 11.0s)
// =========================================================================

function Shot3Modes() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Two-card grid: Message vs Meeting. Message lands immediately, Meeting
  // slides in 0.6s later so the eye registers the mode shift, not just
  // "two cards exist".
  const messageEnter = tween(frame, f(0.2), f(0.7), 0, 1);
  const meetingEnter = tween(frame, f(0.8), f(1.4), 0, 1);
  const meetingSlide = tween(frame, f(0.8), f(1.4), 24, 0);

  const messageBody = typewriter(MESSAGE_SHAPED, frame, f(0.7), f(1.6));
  const meetingBody = typewriter(MEETING_SHAPED, frame, f(1.4), f(2.4));

  const capFade = Math.min(
    tween(frame, f(0.4), f(0.8), 0, 1),
    tween(frame, f(3.6), f(4.0), 1, 0),
  );

  return (
    <AbsoluteFill>
      {/* The raw line at the top — what the user actually said */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 110,
          transform: 'translateX(-50%)',
          width: 1500,
          textAlign: 'center',
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 16,
          }}
        >
          You said
        </div>
        <div
          style={{
            fontSize: 28,
            lineHeight: 1.45,
            color: tokens.fg,
            fontStyle: 'italic',
            opacity: 0.9,
          }}
        >
          &ldquo;{MESSAGE_RAW}&rdquo;
        </div>
      </div>

      {/* Two mode cards side by side — narrow gap, big body text */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 360,
          transform: 'translateX(-50%)',
          width: 1500,
          display: 'grid',
          gridTemplateColumns: '1fr 1fr',
          gap: 32,
        }}
      >
        <ModeCard
          name="Message"
          desc="Cleaned for chat"
          body={messageBody}
          opacity={messageEnter}
          slideY={0}
        />
        <ModeCard
          name="Meeting"
          desc="Bullet structure"
          body={meetingBody}
          opacity={meetingEnter}
          slideY={meetingSlide}
        />
      </div>

      <Caption text="Change the output, not the workflow." opacity={capFade} />
    </AbsoluteFill>
  );
}

function ModeCard({
  name,
  desc,
  body,
  opacity,
  slideY,
}: {
  name: string;
  desc: string;
  body: string;
  opacity: number;
  slideY: number;
}) {
  return (
    <div
      style={{
        background: tokens.bg1,
        borderRadius: 22,
        padding: '34px 38px',
        opacity,
        transform: `translateY(${slideY}px)`,
        position: 'relative',
        border: `1px solid ${tokens.border}`,
        minHeight: 300,
      }}
    >
      <span
        style={{
          position: 'absolute',
          left: 0,
          top: 30,
          bottom: 30,
          width: 3,
          borderRadius: 999,
          background: tokens.accent,
        }}
      />
      <div
        style={{
          fontSize: 30,
          fontWeight: 700,
          color: tokens.fgHi,
          marginBottom: 4,
          paddingLeft: 18,
          letterSpacing: '-0.01em',
        }}
      >
        {name}
      </div>
      <div
        style={{
          fontFamily: fonts.mono,
          fontSize: 17,
          letterSpacing: '0.16em',
          textTransform: 'uppercase',
          color: tokens.muted,
          marginBottom: 22,
          paddingLeft: 18,
        }}
      >
        {desc}
      </div>
      <div
        style={{
          fontSize: 24,
          lineHeight: 1.55,
          color: tokens.fg,
          whiteSpace: 'pre-line',
          paddingLeft: 18,
        }}
      >
        {body}
      </div>
    </div>
  );
}

// =========================================================================
//   Shot 4 — Vocabulary (11.0 – 15.0s)
// =========================================================================

function Shot4Vocabulary() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Beat 1 (0.0–1.5): Show before sentence with "Sara" wrong.
  // Beat 2 (1.5–2.5): Vocab chip slides in showing "Sara → Sarah" added.
  // Beat 3 (2.5–4.0): After sentence appears with "Sarah" correct.
  const beforeFade = Math.min(
    tween(frame, f(0.1), f(0.5), 0, 1),
    tween(frame, f(2.5), f(2.9), 1, 0),
  );
  const beforeText = humanType(VOCAB_BEFORE, frame, f(0.1), fps, 0xab12);

  const chipFade = Math.min(
    tween(frame, f(1.4), f(1.9), 0, 1),
    tween(frame, f(3.8), f(4.0), 1, 0),
  );
  const chipSlide = tween(frame, f(1.4), f(2.1), 18, 0);
  const strikeWidth = tween(frame, f(1.9), f(2.3), 0, 100);
  const arrowFade = tween(frame, f(2.0), f(2.4), 0, 1);

  const afterFade = tween(frame, f(2.6), f(3.1), 0, 1);
  const afterSlide = tween(frame, f(2.6), f(3.1), 14, 0);

  const capFade = Math.min(
    tween(frame, f(0.4), f(0.8), 0, 1),
    tween(frame, f(3.6), f(4.0), 1, 0),
  );

  return (
    <AbsoluteFill>
      {/* Before sentence — top */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 180,
          transform: 'translateX(-50%)',
          width: 1400,
          opacity: beforeFade,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 16,
          }}
        >
          Heard
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 18,
            padding: '28px 36px',
            border: `1px solid ${tokens.border}`,
            fontSize: 38,
            lineHeight: 1.4,
            color: tokens.fg,
          }}
        >
          {beforeText}
        </div>
      </div>

      {/* Vocab chip — middle */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 470,
          transform: `translate(-50%, ${chipSlide}px)`,
          opacity: chipFade,
          background: tokens.bg1,
          border: `1px solid ${tokens.border}`,
          borderLeft: `3px solid ${tokens.accent}`,
          borderRadius: 18,
          padding: '22px 32px',
          display: 'flex',
          alignItems: 'center',
          gap: 28,
        }}
      >
        <div
          style={{
            color: tokens.muted,
            fontSize: 16,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            fontWeight: 500,
          }}
        >
          Vocabulary
        </div>
        <div style={{ position: 'relative', display: 'inline-block' }}>
          <div style={{ fontSize: 36, fontWeight: 500, color: tokens.muted, lineHeight: 1 }}>
            Sara
          </div>
          <div
            style={{
              position: 'absolute',
              left: 0,
              top: '52%',
              height: 3,
              width: `${strikeWidth}%`,
              background: tokens.rose,
              borderRadius: 2,
            }}
          />
        </div>
        <div
          style={{
            opacity: arrowFade,
            display: 'flex',
            alignItems: 'center',
            gap: 22,
          }}
        >
          <div style={{ color: tokens.fgDim, fontSize: 26, fontFamily: fonts.mono }}>→</div>
          <div style={{ fontSize: 36, fontWeight: 600, color: tokens.fgHi, lineHeight: 1 }}>
            Sarah
          </div>
          <svg width={32} height={32} viewBox="0 0 32 32" fill="none">
            <circle cx={16} cy={16} r={15} stroke={tokens.moss} strokeWidth={1.6} fill="none" opacity={0.5} />
            <path
              d="M 9 17 L 14 22 L 23 12"
              stroke={tokens.moss}
              strokeWidth={2.6}
              strokeLinecap="round"
              strokeLinejoin="round"
              fill="none"
            />
          </svg>
        </div>
      </div>

      {/* After sentence — bottom */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 660,
          transform: `translate(-50%, ${afterSlide}px)`,
          width: 1400,
          opacity: afterFade,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.accent,
            marginBottom: 16,
          }}
        >
          Now
        </div>
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 18,
            padding: '28px 36px',
            border: `1px solid rgba(34, 211, 238, 0.35)`,
            fontSize: 38,
            lineHeight: 1.4,
            color: tokens.fg,
            boxShadow: '0 0 0 4px rgba(34, 211, 238, 0.08)',
          }}
        >
          {VOCAB_AFTER}
        </div>
      </div>

      <Caption text="Teach it your words." opacity={capFade} />
    </AbsoluteFill>
  );
}

// =========================================================================
//   Shot 5 — File transcription (15.0 – 19.0s)
// =========================================================================

function Shot5Files() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Beat 1 (0.0–1.0): Files dropzone visible, three queued rows.
  // Beat 2 (1.0–2.5): Top row goes running with progress bar, then done.
  // Beat 3 (2.5–4.0): Transcript modal slides up showing real text.
  const dropFade = tween(frame, 0, 14, 0, 1);

  const row0Progress =
    frame < f(1.0) ? 0 :
    frame < f(2.4) ? Math.min(1, (frame - f(1.0)) / (f(2.4) - f(1.0))) :
    1;
  const row0Done = frame >= f(2.4);

  const modalProgress = tween(frame, f(2.5), f(3.0), 0, 1);
  const scrim = modalProgress * 0.55;

  const capFade = Math.min(
    tween(frame, f(0.4), f(0.8), 0, 1),
    tween(frame, f(3.6), f(4.0), 1, 0),
  );

  return (
    <AbsoluteFill>
      {/* Files route surface — dimmed when modal is up, never fully gone */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          opacity: 1 - modalProgress * 0.45,
          filter: modalProgress > 0.1 ? `blur(${modalProgress * 1.5}px)` : 'none',
        }}
      >
        <div
          style={{
            position: 'absolute',
            left: '50%',
            top: 130,
            transform: 'translateX(-50%)',
            opacity: dropFade,
          }}
        >
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 18,
              letterSpacing: '0.18em',
              textTransform: 'uppercase',
              color: tokens.muted,
              textAlign: 'center',
            }}
          >
            Files
          </div>
        </div>

        {/* Job table */}
        <div
          style={{
            position: 'absolute',
            left: '50%',
            top: 230,
            transform: 'translateX(-50%)',
            width: 1400,
            background: tokens.bg1,
            border: `1px solid ${tokens.border}`,
            borderRadius: 22,
            overflow: 'hidden',
            opacity: dropFade,
          }}
        >
          {FILE_JOBS.map((job, i) => {
            const isRow0 = i === 0;
            const status = isRow0
              ? (row0Done ? 'Done' : (frame >= f(1.0) ? 'Running' : 'Queued'))
              : 'Queued';
            const statusColor =
              status === 'Done' ? tokens.moss :
              status === 'Running' ? tokens.accent :
              tokens.muted;
            return (
              <div
                key={i}
                style={{
                  display: 'grid',
                  gridTemplateColumns: '1fr 200px 200px 160px',
                  alignItems: 'center',
                  padding: '28px 36px',
                  borderBottom: i < FILE_JOBS.length - 1 ? `1px solid ${tokens.border}` : 'none',
                  gap: 24,
                }}
              >
                <div style={{ fontSize: 22, color: tokens.fgHi, fontWeight: 500 }}>{job.name}</div>
                <div style={{ fontSize: 18, color: tokens.muted, fontFamily: fonts.mono }}>{job.kind}</div>
                <div style={{ fontSize: 18, color: tokens.muted, fontFamily: fonts.mono }}>{job.dur}</div>
                <div
                  style={{
                    fontFamily: fonts.mono,
                    fontSize: 16,
                    letterSpacing: '0.14em',
                    textTransform: 'uppercase',
                    color: statusColor,
                  }}
                >
                  ● {status}
                  {isRow0 && status === 'Running' && (
                    <div
                      style={{
                        marginTop: 8,
                        height: 3,
                        background: 'rgba(34, 211, 238, 0.15)',
                        borderRadius: 2,
                        overflow: 'hidden',
                      }}
                    >
                      <div
                        style={{
                          height: '100%',
                          width: `${row0Progress * 100}%`,
                          background: tokens.accent,
                        }}
                      />
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Scrim + Transcript modal */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          background: `rgba(8, 12, 20, ${scrim})`,
          pointerEvents: 'none',
        }}
      />
      {modalProgress > 0 && (
        <div
          style={{
            position: 'absolute',
            left: '50%',
            top: 200,
            transform: `translate(-50%, ${(1 - modalProgress) * 30}px)`,
            opacity: modalProgress,
            width: 1300,
            background: tokens.bg2,
            border: `1px solid ${tokens.borderStrong}`,
            borderRadius: 22,
            padding: '36px 44px',
            boxShadow: '0 32px 96px rgba(0,0,0,0.6)',
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
            Transcript · Team standup
          </div>
          <div
            style={{
              fontSize: 28,
              lineHeight: 1.5,
              color: tokens.fg,
              maxHeight: 480,
              overflow: 'hidden',
            }}
          >
            {FILE_TRANSCRIPT}
          </div>
        </div>
      )}

      <Caption text="Transcribe recordings locally." opacity={capFade} />
    </AbsoluteFill>
  );
}

// =========================================================================
//   Shot 6 — History (19.0 – 23.0s)
// =========================================================================

function Shot6History() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Beat 1 (0.0–1.4): History list visible, four entries.
  // Beat 2 (1.4–2.0): Top row gets a "selected" highlight.
  // Beat 3 (2.0–4.0): Detail panel slides in to the right side.
  const listFade = tween(frame, 0, 14, 0, 1);
  const rowHighlight = tween(frame, f(1.2), f(1.6), 0, 1);

  const detailFade = tween(frame, f(1.9), f(2.4), 0, 1);
  const detailSlide = tween(frame, f(1.9), f(2.4), 30, 0);

  const capFade = Math.min(
    tween(frame, f(0.4), f(0.8), 0, 1),
    tween(frame, f(3.6), f(4.0), 1, 0),
  );

  return (
    <AbsoluteFill>
      {/* History list (left half) */}
      <div
        style={{
          position: 'absolute',
          left: 100,
          top: 130,
          width: 800,
          opacity: listFade,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 22,
          }}
        >
          History
        </div>
        <div
          style={{
            background: tokens.bg1,
            border: `1px solid ${tokens.border}`,
            borderRadius: 22,
            overflow: 'hidden',
          }}
        >
          {HISTORY_ENTRIES.map((entry, i) => {
            const selected = i === 0 && rowHighlight > 0;
            return (
              <div
                key={i}
                style={{
                  padding: '24px 28px',
                  borderBottom: i < HISTORY_ENTRIES.length - 1 ? `1px solid ${tokens.border}` : 'none',
                  background: selected ? `rgba(34, 211, 238, ${0.08 * rowHighlight})` : 'transparent',
                  borderLeft: selected ? `3px solid ${tokens.accent}` : '3px solid transparent',
                }}
              >
                <div
                  style={{
                    fontSize: 22,
                    color: tokens.fgHi,
                    fontWeight: 500,
                    marginBottom: 8,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                  }}
                >
                  {entry.title}…
                </div>
                <div
                  style={{
                    display: 'flex',
                    gap: 22,
                    fontSize: 16,
                    fontFamily: fonts.mono,
                    color: tokens.muted,
                  }}
                >
                  <span>{entry.when}</span>
                  <span>·</span>
                  <span>{entry.mode}</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Detail panel (right half) */}
      <div
        style={{
          position: 'absolute',
          left: 980,
          top: 130,
          width: 840,
          opacity: detailFade,
          transform: `translateX(${detailSlide}px)`,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.accent,
            marginBottom: 22,
          }}
        >
          Selected · Voice to Text
        </div>
        <div
          style={{
            background: tokens.bg1,
            border: `1px solid rgba(34, 211, 238, 0.30)`,
            borderRadius: 22,
            padding: '32px 36px',
            minHeight: 380,
            boxShadow: '0 0 0 4px rgba(34, 211, 238, 0.06)',
          }}
        >
          <div style={{ fontSize: 28, lineHeight: 1.55, color: tokens.fg, marginBottom: 28 }}>
            {HISTORY_DETAIL_BODY}
          </div>
          <div style={{ display: 'flex', gap: 16 }}>
            <ReuseButton label="Copy" />
            <ReuseButton label="Reprocess" emphasis />
          </div>
        </div>
      </div>

      <Caption text="Everything stays on your machine." opacity={capFade} />
    </AbsoluteFill>
  );
}

function ReuseButton({ label, emphasis }: { label: string; emphasis?: boolean }) {
  return (
    <div
      style={{
        background: emphasis ? tokens.accent : tokens.bg2,
        color: emphasis ? '#001018' : tokens.fgHi,
        fontFamily: fonts.sans,
        fontSize: 18,
        fontWeight: 600,
        padding: '12px 22px',
        borderRadius: 12,
        border: emphasis ? 'none' : `1px solid ${tokens.border}`,
        letterSpacing: '-0.005em',
      }}
    >
      {label}
    </div>
  );
}

// =========================================================================
//   Shot 7 — Local/private proof (23.0 – 27.0s)
// =========================================================================

function Shot7Privacy() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Show the Models route. Three rows, each with a green "Installed locally"
  // badge. Right side: a small "Network out" widget pinned at 0 KB.
  const fade = tween(frame, 0, 14, 0, 1);
  const rowEnter = MODELS.map((_, i) => f(0.3 + i * 0.18));

  const monitorFade = tween(frame, f(1.4), f(2.0), 0, 1);
  const monitorSlide = tween(frame, f(1.4), f(2.0), 24, 0);

  const capFade = Math.min(
    tween(frame, f(0.6), f(1.0), 0, 1),
    tween(frame, f(3.6), f(4.0), 1, 0),
  );

  return (
    <AbsoluteFill>
      {/* Models list — left/centre */}
      <div
        style={{
          position: 'absolute',
          left: 100,
          top: 140,
          width: 1100,
          opacity: fade,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            letterSpacing: '0.18em',
            textTransform: 'uppercase',
            color: tokens.muted,
            marginBottom: 22,
          }}
        >
          Speech models
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
          {MODELS.map((m, i) => {
            const op = tween(frame, rowEnter[i], rowEnter[i] + 8, 0, 1);
            const sy = tween(frame, rowEnter[i], rowEnter[i] + 12, 16, 0);
            return (
              <div
                key={i}
                style={{
                  background: tokens.bg1,
                  border: `1px solid ${tokens.border}`,
                  borderRadius: 18,
                  padding: '24px 30px',
                  display: 'grid',
                  gridTemplateColumns: '1fr auto auto',
                  alignItems: 'center',
                  gap: 22,
                  opacity: op,
                  transform: `translateY(${sy}px)`,
                }}
              >
                <div>
                  <div style={{ fontSize: 26, fontWeight: 600, color: tokens.fgHi, marginBottom: 4 }}>
                    {m.name}
                  </div>
                  <div
                    style={{
                      fontSize: 16,
                      fontFamily: fonts.mono,
                      color: tokens.muted,
                      letterSpacing: '0.04em',
                    }}
                  >
                    {m.engine} · {m.note}
                  </div>
                </div>
                <div
                  style={{
                    fontFamily: fonts.mono,
                    fontSize: 18,
                    color: tokens.muted,
                  }}
                >
                  {m.size}
                </div>
                <div
                  style={{
                    fontFamily: fonts.mono,
                    fontSize: 14,
                    letterSpacing: '0.16em',
                    textTransform: 'uppercase',
                    color: tokens.moss,
                    background: 'rgba(52, 211, 153, 0.12)',
                    padding: '8px 14px',
                    borderRadius: 10,
                  }}
                >
                  ● Installed
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Network monitor — small widget on the right */}
      <div
        style={{
          position: 'absolute',
          right: 100,
          top: 200,
          width: 560,
          opacity: monitorFade,
          transform: `translateY(${monitorSlide}px)`,
        }}
      >
        <div
          style={{
            background: tokens.bg1,
            border: `1px solid ${tokens.border}`,
            borderRadius: 18,
            padding: '28px 32px',
          }}
        >
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 12,
              marginBottom: 18,
              fontFamily: fonts.mono,
              fontSize: 16,
              letterSpacing: '0.14em',
              textTransform: 'uppercase',
              color: tokens.muted,
            }}
          >
            <span
              style={{
                width: 12,
                height: 12,
                borderRadius: 999,
                background: tokens.moss,
                boxShadow: `0 0 10px ${tokens.moss}`,
              }}
            />
            Network out · this device
          </div>
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 76,
              fontWeight: 600,
              color: tokens.fgHi,
              letterSpacing: '-0.02em',
              fontVariantNumeric: 'tabular-nums',
              lineHeight: 1,
            }}
          >
            0
            <span style={{ fontSize: 28, color: tokens.muted, marginLeft: 14 }}>KB sent</span>
          </div>
        </div>
      </div>

      <Caption
        primary="No cloud dictation."
        secondary="Models run locally."
        opacity={capFade}
      />
    </AbsoluteFill>
  );
}

// =========================================================================
//   Shot 8 — Close (27.0 – 30.0s)
// =========================================================================

function Shot8Close() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const pillFade = tween(frame, 0, 14, 0, 1);
  const wordmarkFade = tween(frame, f(0.4), f(1.0), 0, 1);
  const wordmarkSlide = tween(frame, f(0.4), f(1.0), 18, 0);
  const taglineFade = tween(frame, f(0.8), f(1.4), 0, 1);
  const ctaFade = tween(frame, f(1.4), f(2.0), 0, 1);

  return (
    <AbsoluteFill style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 56 }}>
        <div style={{ transform: 'scale(2.0)', transformOrigin: 'center', opacity: pillFade }}>
          <BarDriver state="idle" />
        </div>

        <div style={{ textAlign: 'center', marginTop: 80 }}>
          <div
            style={{
              fontSize: 156,
              fontWeight: 700,
              color: tokens.fgHi,
              letterSpacing: '-0.04em',
              lineHeight: 1,
              opacity: wordmarkFade,
              transform: `translateY(${wordmarkSlide}px)`,
            }}
          >
            Utterbar
          </div>
          <div
            style={{
              fontSize: 32,
              fontWeight: 400,
              color: tokens.muted,
              letterSpacing: '-0.005em',
              marginTop: 24,
              opacity: taglineFade,
            }}
          >
            Windows dictation that runs locally
          </div>
          <div
            style={{
              fontFamily: fonts.mono,
              fontSize: 26,
              color: tokens.accent,
              letterSpacing: '0.1em',
              marginTop: 36,
              opacity: ctaFade,
              textShadow: `0 0 22px ${tokens.accent}66`,
            }}
          >
            Download for Windows · utterbar.com
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
}

// =========================================================================
//   Shared caption component
// =========================================================================

function Caption({
  text,
  primary,
  secondary,
  opacity,
}: {
  text?: string;
  primary?: string;
  secondary?: string;
  opacity: number;
}) {
  // Two shapes: single line (text=) or primary+secondary (multiline).
  return (
    <div
      style={{
        position: 'absolute',
        left: 0,
        right: 0,
        bottom: 80,
        display: 'flex',
        justifyContent: 'center',
        opacity,
      }}
    >
      <div
        style={{
          textAlign: 'center',
          fontFamily: fonts.sans,
          color: tokens.fgHi,
          textShadow: '0 4px 24px rgba(0,0,0,0.85)',
        }}
      >
        {text && (
          <div style={{ fontSize: 56, fontWeight: 700, letterSpacing: '-0.02em', lineHeight: 1.1 }}>
            {text}
          </div>
        )}
        {primary && (
          <div style={{ fontSize: 56, fontWeight: 700, letterSpacing: '-0.02em', lineHeight: 1.1 }}>
            {primary}
          </div>
        )}
        {secondary && (
          <div
            style={{
              fontSize: 28,
              fontWeight: 400,
              color: tokens.muted,
              marginTop: 12,
              letterSpacing: '-0.005em',
            }}
          >
            {secondary}
          </div>
        )}
      </div>
    </div>
  );
}
