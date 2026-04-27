/**
 * ad-14-local-dictation — 45.0 s, 1920×1080 landscape master.
 *
 * The hero ad. Eight readable shots, each tied to one feature with a
 * visible cause and a visible result that gets enough screen time to
 * actually be read.
 *
 * Pacing principle (from the v2 rewrite): every block of copy gets enough
 * hold to be read twice. First read is comprehension, second is the
 * take-away. The ad is ~50% longer than v1 because v1's 4-second shots
 * couldn't fit the reading load. Better long and readable than short and
 * skimmed past.
 *
 * Shot map (timings are global seconds; frame == seconds × 30):
 *   0.0  – 3.5   1. Hook         pill idle → listening, "Speak." holds
 *   3.5  – 11.5  2. Dictation    sentence pastes in, then 5 s of hold on the result
 *   11.5 – 18.5  3. Modes        Message types first, holds, then Meeting types
 *   18.5 – 24.5  4. Vocabulary   before / chip teach / after, three readable beats
 *   24.5 – 30.5  5. Files        queue 2 s, transcript modal holds 3.5 s
 *   30.5 – 36.0  6. History      list, highlight, detail panel holds 2.5 s
 *   36.0 – 41.0  7. Privacy      models stagger in, 0 KB monitor holds 3 s
 *   41.0 – 45.0  8. Close        wordmark + tagline + CTA
 *
 * Real-app reuse: every shot composes the genuine BarDriver, the real
 * design tokens, and the same fonts/colours as the shipping app. Card
 * shapes match the live Modes / Files / History / Models routes.
 */
import React from 'react';
import { AbsoluteFill, Sequence, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../../design/tokens';
import { tween, typewriter } from '../../util/anim';
import { BarDriver } from '../../bar/BarDriver';

// ---- copy bank (kept short on purpose) -----------------------------------

const DICTATED_SENTENCE = 'Hi Sarah, just checking whether Tuesday at 3 still works for you.';

// Shot 3 — drop the raw "you said" preamble entirely. The two cards make
// the point themselves; an extra paragraph triples the reading load.
const MESSAGE_SHAPED =
  'Quick heads up. Pushing launch to Tuesday. QA flagged two auth issues we want fixed first.';
const MEETING_SHAPED =
  '• Launch pushed to Tuesday\n• QA flagged 2 auth issues\n• Fix before marketing announce';

// Shot 4 — short, punchy.
const VOCAB_BEFORE = 'Hi Sara, can you review the deck before Friday?';
const VOCAB_AFTER  = 'Hi Sarah, can you review the deck before Friday?';

// Shot 5 — transcript trimmed from 4 sentences to 1, so the modal
// holds long enough for the eye to read the whole thing.
const FILE_TRANSCRIPT =
  'Right, standup. Three things to land before Friday. First, the auth migration is good to go.';

const FILE_JOBS = [
  { name: 'Team standup, 2026-04-18.mp4', kind: 'Video', dur: '30:20' },
  { name: 'Podcast draft, ep 14.m4a',     kind: 'Audio', dur: '44:40' },
  { name: 'Interview, Jane Doe.wav',      kind: 'Audio', dur: '52:20' },
];

// Shot 6 — three entries (was four), shorter detail body.
const HISTORY_ENTRIES = [
  { title: 'Hi Sarah, just checking whether Tuesday', when: '2 min ago',  mode: 'Voice to Text' },
  { title: 'Quick heads up. Pushing launch',         when: '14 min ago', mode: 'Message' },
  { title: 'Standup notes, auth migration green',    when: '1 hr ago',   mode: 'Meeting' },
];
const HISTORY_DETAIL_BODY =
  'Hi Sarah, just checking whether Tuesday at 3 still works for you.';

// Shot 7 — three real shipping engines, names from the live catalogue.
const MODELS = [
  { name: 'Parakeet TDT', size: '1.2 GB', engine: 'Sherpa-ONNX', note: 'fastest · English' },
  { name: 'Whisper Base', size: '142 MB', engine: 'Sherpa-ONNX', note: 'multilingual' },
  { name: 'Whisper Small',size: '466 MB', engine: 'Sherpa-ONNX', note: 'higher accuracy' },
];

// =========================================================================
//   Top-level composition — 8 shots via <Sequence>
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
      {/* Atmospheric wash, top-right cyan + bottom-left cyan. Same recipe
          as the website hero glow; calibrated to be present everywhere
          without competing with the UI. Same alpha as the v1 — that part
          worked, the rest of v1 didn't. */}
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

      <Sequence from={f(0.0)}  durationInFrames={f(3.5)}>  <Shot1Hook />          </Sequence>
      <Sequence from={f(3.5)}  durationInFrames={f(8.0)}>  <Shot2Dictation />     </Sequence>
      <Sequence from={f(11.5)} durationInFrames={f(7.0)}>  <Shot3Modes />         </Sequence>
      <Sequence from={f(18.5)} durationInFrames={f(6.0)}>  <Shot4Vocabulary />    </Sequence>
      <Sequence from={f(24.5)} durationInFrames={f(6.0)}>  <Shot5Files />         </Sequence>
      <Sequence from={f(30.5)} durationInFrames={f(5.5)}>  <Shot6History />       </Sequence>
      <Sequence from={f(36.0)} durationInFrames={f(5.0)}>  <Shot7Privacy />       </Sequence>
      <Sequence from={f(41.0)} durationInFrames={f(4.0)}>  <Shot8Close />         </Sequence>
    </AbsoluteFill>
  );
}

// =========================================================================
//   Shot 1 — Hook (0.0 – 3.5s)
//
//   Pill enters listening at 0.7 s, the elapsed clock starts then.
//   "Speak." holds for ~2.5 seconds before the cut — long enough to
//   read it twice, with a beat for the eye to drift from the headline
//   to the listening pill above and watch the waveform breathe.
// =========================================================================

function Shot1Hook() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const listening = frame >= f(0.7);
  const state: 'idle' | 'listening' = listening ? 'listening' : 'idle';
  const elapsedMs = listening ? ((frame - f(0.7)) / fps) * 1000 : 0;

  const pillFade = tween(frame, 0, 12, 0, 1);
  const eyebrowFade = tween(frame, f(0.2), f(0.8), 0, 1);
  const headlineFade = tween(frame, f(0.5), f(1.1), 0, 1);
  const headlineSlide = tween(frame, f(0.5), f(1.1), 18, 0);
  // Soft fade-out on the last 12 frames so the cut isn't a hard slam.
  const exitFade = tween(frame, f(3.1), f(3.5), 1, 0);

  return (
    <AbsoluteFill
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        opacity: exitFade,
      }}
    >
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
//   Shot 2 — Dictation (3.5 – 11.5s, 8.0s total)
//
//   The headline moment. The viewer sees the floating bar above an empty
//   email field, the bar transitions through listening → processing →
//   the sentence pastes in fast (0.7 s, real-app speed), and then the
//   field with the dictated text + accent glow holds for 5+ seconds.
//
//   Internal timeline (relative to shot start):
//     0.0 – 0.6   pill in listening (entered from previous shot)
//     0.6 – 2.4   listening, timer ticks 0:00 → 0:02
//     2.4 – 3.0   processing
//     3.0 – 3.7   text pastes in (typewriter, fast)
//     3.7 – 7.0   HOLD: full sentence, pill back to idle, glow at peak
//     7.0 – 7.6   glow fades, exit fade begins
//     7.6 – 8.0   exit fade
// =========================================================================

function Shot2Dictation() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  let state: 'listening' | 'processing' | 'idle' = 'listening';
  if (frame < f(2.4)) state = 'listening';
  else if (frame < f(3.0)) state = 'processing';
  else state = 'idle';

  const elapsedMs = state === 'listening' ? (frame / fps) * 1000 : 0;

  const typed = typewriter(DICTATED_SENTENCE, frame, f(3.0), f(3.7));

  // Glow rises with the paste (3.0 → 3.5), holds at full from 3.5 → 7.0,
  // fades 7.0 → 7.6.
  const glowRise = tween(frame, f(3.0), f(3.5), 0, 1);
  const glowHold = tween(frame, f(7.0), f(7.6), 1, 0);
  const successGlow = Math.min(glowRise, glowHold);

  // Caption fades in early, holds through the entire shot, fades out
  // before the cut. The caption is the durable label.
  const capFade = Math.min(
    tween(frame, f(0.4), f(0.9), 0, 1),
    tween(frame, f(7.4), f(7.9), 1, 0),
  );

  // Whole-shot exit fade for the cut.
  const exitFade = tween(frame, f(7.6), f(8.0), 1, 0);

  // Caret blink before the paste — visible 0.0 → 3.0, blinks every 14 frames.
  const showCaret = frame < f(3.0);
  const caretOn = Math.floor(frame / 14) % 2 === 0;

  return (
    <AbsoluteFill style={{ opacity: exitFade }}>
      {/* Pill, top centre, large */}
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

      {/* Email-style field */}
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
                ? `rgba(34, 211, 238, ${0.20 + successGlow * 0.55})`
                : tokens.border
            }`,
            minHeight: 320,
            fontSize: 38,
            lineHeight: 1.45,
            color: tokens.fg,
            boxShadow:
              successGlow > 0.05
                ? `0 0 0 ${10 * successGlow}px rgba(34, 211, 238, ${0.10 * successGlow})`
                : 'none',
          }}
        >
          {typed}
          {showCaret && !typed && (
            <span
              style={{
                display: 'inline-block',
                width: 3,
                height: 38,
                background: tokens.accent,
                verticalAlign: '-7px',
                opacity: caretOn ? 1 : 0,
              }}
            />
          )}
        </div>
      </div>

      <Caption text="Dictate into any app." opacity={capFade} />
    </AbsoluteFill>
  );
}

// =========================================================================
//   Shot 3 — Modes (11.5 – 18.5s, 7.0s total)
//
//   Both cards visible from the start, but only one types at a time.
//   The eye gets to focus.
//
//   Internal timeline:
//     0.0 – 0.6   both cards fade up empty
//     0.6 – 1.7   Message body types in
//     1.7 – 4.0   HOLD on Message readable, Meeting still empty
//     4.0 – 5.0   Meeting body types in
//     5.0 – 6.4   HOLD on both readable
//     6.4 – 7.0   exit fade
// =========================================================================

function Shot3Modes() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const cardsFade = tween(frame, f(0.1), f(0.6), 0, 1);
  // Message starts the typewriter at 0.6, finishes at 1.7 (1.1 s of typing)
  const messageBody = typewriter(MESSAGE_SHAPED, frame, f(0.6), f(1.7));
  // Meeting starts at 4.0, finishes at 5.0
  const meetingBody = typewriter(MEETING_SHAPED, frame, f(4.0), f(5.0));

  // Active-card highlight — the card currently typing has a stronger
  // accent rail. Message glows during 0.6–1.7, then Meeting glows
  // during 4.0–5.0.
  const messageActive = frame >= f(0.6) && frame < f(2.0);
  const meetingActive = frame >= f(4.0) && frame < f(5.3);

  const capFade = Math.min(
    tween(frame, f(0.4), f(0.9), 0, 1),
    tween(frame, f(6.4), f(6.9), 1, 0),
  );
  const exitFade = tween(frame, f(6.5), f(7.0), 1, 0);

  return (
    <AbsoluteFill style={{ opacity: exitFade }}>
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 230,
          transform: 'translateX(-50%)',
          width: 1500,
          display: 'grid',
          gridTemplateColumns: '1fr 1fr',
          gap: 36,
          opacity: cardsFade,
        }}
      >
        <ModeCard
          name="Message"
          desc="Cleaned for chat"
          body={messageBody}
          active={messageActive}
        />
        <ModeCard
          name="Meeting"
          desc="Bullet structure"
          body={meetingBody}
          active={meetingActive}
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
  active,
}: {
  name: string;
  desc: string;
  body: string;
  active: boolean;
}) {
  return (
    <div
      style={{
        background: tokens.bg1,
        borderRadius: 22,
        padding: '40px 44px',
        position: 'relative',
        border: `1px solid ${active ? `rgba(34, 211, 238, 0.30)` : tokens.border}`,
        minHeight: 480,
        transition: 'border-color 0.2s',
      }}
    >
      <span
        style={{
          position: 'absolute',
          left: 0,
          top: 36,
          bottom: 36,
          width: active ? 4 : 3,
          borderRadius: 999,
          background: tokens.accent,
          opacity: active ? 1 : 0.5,
        }}
      />
      <div
        style={{
          fontSize: 36,
          fontWeight: 700,
          color: tokens.fgHi,
          marginBottom: 6,
          paddingLeft: 22,
          letterSpacing: '-0.01em',
        }}
      >
        {name}
      </div>
      <div
        style={{
          fontFamily: fonts.mono,
          fontSize: 18,
          letterSpacing: '0.16em',
          textTransform: 'uppercase',
          color: tokens.muted,
          marginBottom: 28,
          paddingLeft: 22,
        }}
      >
        {desc}
      </div>
      <div
        style={{
          fontSize: 26,
          lineHeight: 1.55,
          color: tokens.fg,
          whiteSpace: 'pre-line',
          paddingLeft: 22,
          minHeight: 240,
        }}
      >
        {body}
      </div>
    </div>
  );
}

// =========================================================================
//   Shot 4 — Vocabulary (18.5 – 24.5s, 6.0s total)
//
//   Three readable beats. Sentences appear instantly (no slow typewriter
//   for the wrong sentence — that burned viewer time on the problem).
//
//   Internal timeline:
//     0.0 – 0.4   before sentence appears (the previous dictation)
//     0.4 – 1.8   HOLD on before — viewer reads "Hi Sara, can you..."
//     1.8 – 2.4   chip slides in
//     2.4 – 3.6   HOLD on chip — strike-through builds, "→ Sarah ✓" lands
//     3.6 – 4.2   after sentence appears
//     4.2 – 5.6   HOLD on after — viewer reads "Hi Sarah, can you..."
//     5.6 – 6.0   exit fade
// =========================================================================

function Shot4Vocabulary() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const beforeFade = tween(frame, f(0.1), f(0.4), 0, 1);

  const chipFade = Math.min(
    tween(frame, f(1.7), f(2.2), 0, 1),
    tween(frame, f(5.6), f(6.0), 1, 0),
  );
  const chipSlide = tween(frame, f(1.7), f(2.4), 18, 0);
  const strikeWidth = tween(frame, f(2.2), f(2.7), 0, 100);
  const arrowFade = tween(frame, f(2.4), f(2.9), 0, 1);

  const afterFade = tween(frame, f(3.6), f(4.1), 0, 1);
  const afterSlide = tween(frame, f(3.6), f(4.1), 14, 0);
  // Brief accent pulse on the after card so the eye is drawn there
  const afterPulse = tween(frame, f(3.7), f(4.2), 0, 1) * tween(frame, f(4.6), f(5.4), 1, 0);

  const capFade = Math.min(
    tween(frame, f(0.4), f(0.9), 0, 1),
    tween(frame, f(5.4), f(5.9), 1, 0),
  );
  const exitFade = tween(frame, f(5.6), f(6.0), 1, 0);

  return (
    <AbsoluteFill style={{ opacity: exitFade }}>
      {/* Before sentence */}
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
          {VOCAB_BEFORE}
        </div>
      </div>

      {/* Vocab chip */}
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

      {/* After sentence */}
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
            border: `1px solid rgba(34, 211, 238, ${0.30 + afterPulse * 0.30})`,
            fontSize: 38,
            lineHeight: 1.4,
            color: tokens.fg,
            boxShadow: `0 0 0 ${4 + afterPulse * 6}px rgba(34, 211, 238, ${0.06 + afterPulse * 0.08})`,
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
//   Shot 5 — Files (24.5 – 30.5s, 6.0s total)
//
//   Internal timeline:
//     0.0 – 0.5   Files route fades up, three queued rows
//     0.5 – 1.5   Top row goes Running with progress bar
//     1.5 – 2.0   Top row goes Done
//     2.0 – 2.5   Modal slides up over the route
//     2.5 – 5.5   HOLD: transcript readable, route blurred behind
//     5.5 – 6.0   exit fade
// =========================================================================

function Shot5Files() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const dropFade = tween(frame, 0, 14, 0, 1);

  const row0Progress =
    frame < f(0.5) ? 0 :
    frame < f(1.7) ? Math.min(1, (frame - f(0.5)) / (f(1.7) - f(0.5))) :
    1;
  const row0Done = frame >= f(1.7);

  const modalProgress = tween(frame, f(2.0), f(2.5), 0, 1);
  const scrim = modalProgress * 0.55;

  const capFade = Math.min(
    tween(frame, f(0.6), f(1.1), 0, 1),
    tween(frame, f(5.4), f(5.9), 1, 0),
  );
  const exitFade = tween(frame, f(5.5), f(6.0), 1, 0);

  return (
    <AbsoluteFill style={{ opacity: exitFade }}>
      {/* Files route — dimmed when modal is up */}
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
              ? (row0Done ? 'Done' : (frame >= f(0.5) ? 'Running' : 'Queued'))
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
                  gridTemplateColumns: '1fr 200px 200px 180px',
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

      {/* Scrim + transcript modal */}
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
            top: 280,
            transform: `translate(-50%, ${(1 - modalProgress) * 30}px)`,
            opacity: modalProgress,
            width: 1300,
            background: tokens.bg2,
            border: `1px solid ${tokens.borderStrong}`,
            borderRadius: 22,
            padding: '40px 48px',
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
              marginBottom: 16,
            }}
          >
            Transcript · Team standup
          </div>
          <div
            style={{
              fontSize: 32,
              lineHeight: 1.55,
              color: tokens.fg,
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
//   Shot 6 — History (30.5 – 36.0s, 5.5s total)
//
//   Internal timeline:
//     0.0 – 0.6   list fades up, three entries
//     0.6 – 1.6   HOLD: viewer reads the list
//     1.6 – 2.0   top row gets selected highlight
//     2.0 – 2.5   detail panel slides in from the right
//     2.5 – 5.0   HOLD: detail body readable, Copy + Reprocess buttons
//     5.0 – 5.5   exit fade
// =========================================================================

function Shot6History() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const listFade = tween(frame, 0, 14, 0, 1);
  const rowHighlight = tween(frame, f(1.6), f(2.0), 0, 1);

  const detailFade = tween(frame, f(1.9), f(2.4), 0, 1);
  const detailSlide = tween(frame, f(1.9), f(2.4), 30, 0);

  const capFade = Math.min(
    tween(frame, f(0.4), f(0.9), 0, 1),
    tween(frame, f(4.9), f(5.4), 1, 0),
  );
  const exitFade = tween(frame, f(5.0), f(5.5), 1, 0);

  return (
    <AbsoluteFill style={{ opacity: exitFade }}>
      {/* History list (left half) */}
      <div
        style={{
          position: 'absolute',
          left: 100,
          top: 200,
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
                  padding: '28px 32px',
                  borderBottom: i < HISTORY_ENTRIES.length - 1 ? `1px solid ${tokens.border}` : 'none',
                  background: selected ? `rgba(34, 211, 238, ${0.10 * rowHighlight})` : 'transparent',
                  borderLeft: selected ? `3px solid ${tokens.accent}` : '3px solid transparent',
                }}
              >
                <div
                  style={{
                    fontSize: 24,
                    color: tokens.fgHi,
                    fontWeight: 500,
                    marginBottom: 10,
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
          top: 200,
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
            padding: '36px 40px',
            minHeight: 320,
            boxShadow: '0 0 0 4px rgba(34, 211, 238, 0.06)',
          }}
        >
          <div style={{ fontSize: 30, lineHeight: 1.55, color: tokens.fg, marginBottom: 32 }}>
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
//   Shot 7 — Privacy (36.0 – 41.0s, 5.0s total)
//
//   Internal timeline:
//     0.0 – 1.5   three model rows stagger in
//     1.5 – 2.0   network monitor slides in
//     2.0 – 4.5   HOLD: three models with Installed badges, monitor at 0 KB
//     4.5 – 5.0   exit fade
// =========================================================================

function Shot7Privacy() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const fade = tween(frame, 0, 14, 0, 1);
  const rowEnter = MODELS.map((_, i) => f(0.2 + i * 0.4));

  const monitorFade = tween(frame, f(1.5), f(2.0), 0, 1);
  const monitorSlide = tween(frame, f(1.5), f(2.0), 24, 0);

  const capFade = Math.min(
    tween(frame, f(0.6), f(1.1), 0, 1),
    tween(frame, f(4.4), f(4.9), 1, 0),
  );
  const exitFade = tween(frame, f(4.5), f(5.0), 1, 0);

  return (
    <AbsoluteFill style={{ opacity: exitFade }}>
      {/* Models list — left side, three rows */}
      <div
        style={{
          position: 'absolute',
          left: 100,
          top: 180,
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

      {/* Network monitor — right side */}
      <div
        style={{
          position: 'absolute',
          right: 100,
          top: 240,
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

      <Caption text="No cloud dictation." opacity={capFade} />
    </AbsoluteFill>
  );
}

// =========================================================================
//   Shot 8 — Close (41.0 – 45.0s, 4.0s total)
//
//   Internal timeline:
//     0.0 – 0.6   pill fades up in idle
//     0.6 – 1.4   wordmark rises in
//     1.0 – 1.8   tagline rises in
//     1.6 – 2.4   CTA URL rises in
//     2.4 – 4.0   HOLD: full close card readable
// =========================================================================

function Shot8Close() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  const pillFade = tween(frame, 0, 14, 0, 1);
  const wordmarkFade = tween(frame, f(0.6), f(1.4), 0, 1);
  const wordmarkSlide = tween(frame, f(0.6), f(1.4), 18, 0);
  const taglineFade = tween(frame, f(1.0), f(1.8), 0, 1);
  const ctaFade = tween(frame, f(1.6), f(2.4), 0, 1);

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
  opacity,
}: {
  text: string;
  opacity: number;
}) {
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
          fontSize: 56,
          fontWeight: 700,
          letterSpacing: '-0.02em',
          lineHeight: 1.1,
        }}
      >
        {text}
      </div>
    </div>
  );
}
