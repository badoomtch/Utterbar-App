/**
 * feature-multilingual — 12 s. Same one user, same one hotkey, four
 * languages in sequence. The pill sits at the top in listening state
 * throughout (because in real use the language doesn't make the bar
 * change shape — it just changes what comes out the bottom). Below the
 * pill, a language chip cycles EN → FR → JA → ES, and each language's
 * transcript types itself out in turn into the field.
 *
 * The point isn't to teach you Whisper's language coverage — it's to
 * make a French / Japanese / Spanish viewer think "oh, that's me too"
 * inside two seconds. The cycling chip does the audience-broadening
 * lift; the on-PC framing in the eyebrow does the privacy reassurance.
 *
 * Timeline (each segment = 3s):
 *   0.0 –  3.0   EN sentence types in
 *   3.0 –  6.0   FR sentence types in
 *   6.0 –  9.0   JA sentence types in
 *   9.0 – 12.0   ES sentence types in
 *
 * The field clears between segments via a 0.4s fade rather than a hard
 * cut so it reads as "the same field, the same bar, just different
 * language".
 */
import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { tokens, fonts } from '../design/tokens';
import { BarDriver } from '../bar/BarDriver';
import { tween, typewriter } from '../util/anim';

type LangSegment = {
  code: string;
  label: string;
  /** Native-script label so a viewer who reads that script gets a hit
   *  of recognition without translating. Falls back to `label` when the
   *  script equals the Latin label (English, Spanish, French). */
  native?: string;
  sentence: string;
};

const SEGMENTS: LangSegment[] = [
  {
    code: 'EN',
    label: 'English',
    sentence: 'Speak in any language. Get clean text in any Windows app.',
  },
  {
    code: 'FR',
    label: 'French',
    native: 'Français',
    sentence: 'Parlez dans la langue que vous voulez. Le texte arrive proprement, partout sur Windows.',
  },
  {
    code: 'JA',
    label: 'Japanese',
    native: '日本語',
    sentence: 'どの言語でも話してください。Windowsのどのアプリにもきれいなテキストが届きます。',
  },
  {
    code: 'ES',
    label: 'Spanish',
    native: 'Español',
    sentence: 'Habla en cualquier idioma. El texto aparece limpio en cualquier app de Windows.',
  },
];

// Each segment runs for 3 seconds: 2.4s typing + 0.6s settle/fade.
const SEGMENT_SECONDS = 3.0;
const TYPE_FRACTION = 0.8; // typing finishes at 80% of each segment
const FADE_OUT_SECONDS = 0.4;

export function FeatureMultilingual() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // Which segment is on screen right now.
  const segIndex = Math.min(
    SEGMENTS.length - 1,
    Math.floor(frame / f(SEGMENT_SECONDS)),
  );
  const seg = SEGMENTS[segIndex]!;
  const segStartFrame = segIndex * f(SEGMENT_SECONDS);
  const segLocalFrame = frame - segStartFrame;
  const segLength = f(SEGMENT_SECONDS);

  // Typewriter window inside the segment.
  const typeStart = segStartFrame;
  const typeEnd = segStartFrame + Math.round(segLength * TYPE_FRACTION);
  const typed = typewriter(seg.sentence, frame, typeStart, typeEnd);

  // Fade the field text out in the last 0.4s of every non-final
  // segment so the next language doesn't crash-cut over it.
  const fadeOutStart = segStartFrame + segLength - f(FADE_OUT_SECONDS);
  const isLastSegment = segIndex === SEGMENTS.length - 1;
  const fieldOpacity = isLastSegment
    ? 1
    : tween(frame, fadeOutStart, segStartFrame + segLength, 1, 0);

  // Chip enters with a small upward slide and accent glow on each
  // segment change. Without this every segment looks identical at the
  // top of the frame and the eye doesn't notice the language switched.
  const chipEnter = tween(segLocalFrame, 0, f(0.4), 0, 1);

  // Pill stays in listening throughout — it's the same dictation flow,
  // just with different output. The elapsed counter resets per
  // segment so the timer reads naturally.
  const elapsed = segLocalFrame / fps;

  return (
    <AbsoluteFill style={{ background: tokens.bg0, fontFamily: fonts.sans }}>
      {/* Pill at the top — same as feature-works-anywhere for visual
          continuity across the features page. */}
      <div
        style={{
          position: 'absolute',
          top: 90,
          left: '50%',
          transform: 'translateX(-50%)',
        }}
      >
        <div style={{ transform: 'scale(1.85)', transformOrigin: 'top center' }}>
          <BarDriver state="listening" elapsedMs={elapsed * 1000} />
        </div>
      </div>

      {/* Language chip — sits between pill and field. ISO-code on the
          left in mono, full label on the right in sans. The chip itself
          is on bg1 with an accent ring so it reads as the "active"
          element of the frame. */}
      <div
        style={{
          position: 'absolute',
          top: 290,
          left: '50%',
          transform: `translate(-50%, ${(1 - chipEnter) * 12}px)`,
          opacity: chipEnter,
          display: 'flex',
          alignItems: 'center',
          gap: 14,
          padding: '12px 22px 12px 16px',
          background: tokens.bg1,
          border: `1.5px solid rgba(34, 211, 238, ${0.35 + chipEnter * 0.4})`,
          borderRadius: 999,
          boxShadow: `0 0 0 ${6 * chipEnter}px rgba(34, 211, 238, ${0.06 * chipEnter})`,
        }}
      >
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            fontWeight: 700,
            letterSpacing: '0.08em',
            color: tokens.accent,
            background: 'rgba(34, 211, 238, 0.12)',
            padding: '4px 10px',
            borderRadius: 8,
          }}
        >
          {seg.code}
        </div>
        <div
          style={{
            fontSize: 18,
            fontWeight: 500,
            color: tokens.fgHi,
            letterSpacing: '-0.005em',
          }}
        >
          {seg.label}
          {seg.native && seg.native !== seg.label && (
            <span style={{ color: tokens.muted, marginLeft: 10, fontWeight: 400 }}>
              · {seg.native}
            </span>
          )}
        </div>
      </div>

      {/* Eyebrow — frames the demo as "30+ languages, all on-device" so
          the viewer reads the cycling chip as a sample, not the entire
          set. Sits above the field on the bg-0 canvas, mono-uppercase in
          the same idiom as the works-anywhere eyebrow. */}
      <div
        style={{
          position: 'absolute',
          top: 380,
          left: '50%',
          transform: 'translateX(-50%)',
          fontFamily: fonts.mono,
          fontSize: 15,
          letterSpacing: '0.18em',
          textTransform: 'uppercase',
          color: tokens.muted,
        }}
      >
        30+ languages — all on your PC
      </div>

      {/* Output field — the same field every segment writes into. */}
      <div
        style={{
          position: 'absolute',
          left: '50%',
          top: 440,
          transform: 'translateX(-50%)',
          width: 1100,
          maxWidth: '88%',
        }}
      >
        <div
          style={{
            background: tokens.bg1,
            borderRadius: 16,
            padding: '32px 36px',
            border: `1px solid ${tokens.border}`,
            minHeight: 240,
            fontSize: 28,
            lineHeight: 1.55,
            color: tokens.fg,
            opacity: fieldOpacity,
            transition: 'opacity 0.05s linear',
          }}
        >
          {typed}
        </div>
      </div>
    </AbsoluteFill>
  );
}
