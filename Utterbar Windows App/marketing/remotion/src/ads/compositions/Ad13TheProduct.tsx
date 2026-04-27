/**
 * ad-13-the-product — 30.0s. The hero ad.
 *
 * Unlike Ads 01–12, which each argue one feature loudly, this one is a
 * continuous product film: pill stays visible throughout, supporting
 * elements crossfade underneath. Six beats:
 *
 *   0.0 –  3.5  Hook            pill idle, copy "Press." / "Speak."
 *   3.5 –  8.5  Dictation       pill listening + raw transcript types in
 *   8.5 – 14.0  Smart shaping   pill processing → success, shaped panel slides in
 *  14.0 – 19.5  Vocabulary      pill listening, vocab chip teaches "Ddraig"
 *  19.5 – 24.5  Local proof     pill listening, network monitor at 0 KB
 *  24.5 – 30.0  EndCard         held screenshottable finale (4.5s + 1s pad)
 *
 * Stage layout: pill always at y≈340, scaled 2.6× — the hero. Supporting
 * panels slot into the y≈900–1500 zone underneath with crossfades on the
 * panel boundary. Pill never moves, never fades.
 *
 * Cyan rationing: at most 3 cyan moments per beat (pill LED + waveform
 * counts as 2; supporting element's accent strip is the third). EndCard
 * tagline / URL is a separate viewport so its cyan budget is fresh.
 */
import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { BarDriver } from '../../bar/BarDriver';
import { tokens, fonts } from '../../design/tokens';
import { tween } from '../../util/anim';
import { SocialFrame } from '../frame/SocialFrame';
import { Captions } from '../frame/Captions';
import { EndCard } from '../frame/EndCard';
import { TranscriptPanel } from '../scenes/TranscriptPanel';
import { VocabularyChip } from '../scenes/VocabularyChip';
import { NetworkMonitor } from '../scenes/NetworkMonitor';

const RAW_TEXT =
  "Push the staging fix went out twelve minutes ago. Webhook errors started showing up after the 14:10 deploy.";

const SHAPED_TEXT =
  "Push the staging fix went out twelve minutes ago.\n\nWebhook errors started showing up after the 14:10 deploy.";

export function Ad13TheProduct() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = (s: number) => Math.round(s * fps);

  // -------- Beat boundaries (in seconds) --------
  const B1_END = 3.5;   // hook
  const B2_END = 8.5;   // dictation
  const B3_END = 14.0;  // shaping
  const B4_END = 19.5;  // vocabulary
  const B5_END = 24.5;  // local proof
  const ENDCARD = 24.5; // endcard takes over

  // -------- Pill state machine --------
  // Idle through the hook, listening through dictation, processing for
  // a beat then idle while shaped panel holds, listening through vocab
  // and local-proof beats. The pill is never offscreen until EndCard.
  let pillState: 'idle' | 'listening' | 'processing' | 'success' = 'idle';
  let listeningStartFrame = 0;

  if (frame < f(B1_END)) {
    pillState = 'idle';
  } else if (frame < f(B2_END)) {
    // Dictation beat
    pillState = 'listening';
    listeningStartFrame = f(B1_END);
  } else if (frame < f(B2_END) + 12) {
    // Brief processing punch — 0.4s
    pillState = 'processing';
  } else if (frame < f(B2_END) + 30) {
    // Brief success flash — 0.6s
    pillState = 'success';
  } else if (frame < f(B3_END)) {
    pillState = 'idle';
  } else if (frame < f(B4_END)) {
    pillState = 'listening';
    listeningStartFrame = f(B3_END);
  } else if (frame < f(B5_END)) {
    pillState = 'listening';
    listeningStartFrame = f(B4_END);
  } else {
    pillState = 'idle'; // EndCard covers everything anyway
  }

  const elapsedMs =
    pillState === 'listening' ? ((frame - listeningStartFrame) / fps) * 1000 : 0;

  // -------- Supporting panel timings --------
  // Beat 2: raw transcript panel fades in just after listening starts,
  // text builds character-by-character, holds until processing.
  const rawStart = f(B1_END) + 6; // 0.2s after pill enters listening
  const rawFadeIn = tween(frame, rawStart - 6, rawStart + 6, 0, 1);
  const rawFadeOut = tween(frame, f(B2_END) + 22, f(B2_END) + 32, 1, 0);
  const rawOpacity = Math.min(rawFadeIn, rawFadeOut);

  // Beat 3: shaped transcript slides in after the processing/success punch.
  const shapedStart = f(B2_END) + 36;
  const shapedFadeIn = tween(frame, shapedStart, shapedStart + 14, 0, 1);
  const shapedSlide = tween(frame, shapedStart, shapedStart + 16, 32, 0);
  const shapedFadeOut = tween(frame, f(B3_END) + 6, f(B3_END) + 16, 1, 0);
  const shapedOpacity = Math.min(shapedFadeIn, shapedFadeOut);

  // Beat 4: vocabulary chip
  const chipStart = f(B3_END) + 18;
  const chipLocalFrame = frame - chipStart;
  const chipFadeOut = tween(frame, f(B4_END) + 0, f(B4_END) + 10, 1, 0);

  // Beat 5: network monitor
  const netStart = f(B4_END) + 8;
  const netFadeIn = tween(frame, netStart, netStart + 14, 0, 1);
  const netSlide = tween(frame, netStart, netStart + 16, 32, 0);
  const netFadeOut = tween(frame, f(B5_END) - 18, f(B5_END) - 4, 1, 0);
  const netOpacity = Math.min(netFadeIn, netFadeOut);

  // -------- Pill entrance --------
  // Pill fades up over the first 0.7s. Fade only — never moves, never
  // scales. The brief asks for "no slow generic opener" so we get to
  // the product fast: 0 → 1 in 21 frames is ~0.7s, fast enough that
  // it feels deliberate but the viewer sees the product on frame 1.
  const pillFadeIn = tween(frame, 0, 18, 0, 1);

  return (
    <SocialFrame variant="canvas">
      {/* Pill — fixed position, never moves throughout the ad. Scale 2.0×
          is the "safe" cap established by Ad01 — at 2.6× the listening
          state's expanded chrome (timer + waveform + stop + close + cog)
          overflows the 1080px frame edges. 2.0× holds the full pill at
          all six states the ad cycles through. */}
      <div
        style={{
          position: 'absolute',
          top: 360,
          left: '50%',
          transform: 'translateX(-50%) scale(2.0)',
          transformOrigin: 'top center',
          opacity: pillFadeIn,
        }}
      >
        <BarDriver state={pillState} elapsedMs={elapsedMs} />
      </div>

      {/* Stage zone — supporting panels live here. Each beat sits in
          its own absolute layer at top:0 of the stage; the layer is
          full-width with flex-centred children so the panels (which
          have intrinsic widths < the stage width) sit dead-centre
          horizontally. y=900 clears the pill's drop shadow. */}
      {rawOpacity > 0 && (
        <div
          style={{
            position: 'absolute',
            top: 900,
            left: 0,
            right: 0,
            display: 'flex',
            justifyContent: 'center',
            opacity: rawOpacity,
          }}
        >
          <TranscriptPanel
            mode="raw"
            text={RAW_TEXT}
            startFrame={rawStart}
            seed={0xdda1}
          />
        </div>
      )}

      {shapedOpacity > 0 && (
        <div
          style={{
            position: 'absolute',
            top: 900,
            left: 0,
            right: 0,
            display: 'flex',
            justifyContent: 'center',
            opacity: shapedOpacity,
            transform: `translateY(${shapedSlide}px)`,
          }}
        >
          <TranscriptPanel mode="shaped" text={SHAPED_TEXT} startFrame={shapedStart} />
        </div>
      )}

      {chipLocalFrame >= 0 && chipFadeOut > 0 && (
        <div
          style={{
            position: 'absolute',
            top: 980,
            left: 0,
            right: 0,
            display: 'flex',
            justifyContent: 'center',
            opacity: chipFadeOut,
          }}
        >
          <VocabularyChip
            localFrame={chipLocalFrame}
            wrongWord="dragon"
            targetWord="Ddraig"
          />
        </div>
      )}

      {netOpacity > 0 && (
        <div
          style={{
            position: 'absolute',
            top: 960,
            left: 0,
            right: 0,
            display: 'flex',
            justifyContent: 'center',
            opacity: netOpacity,
            transform: `translateY(${netSlide}px)`,
          }}
        >
          <NetworkMonitor bytes={0} />
        </div>
      )}

      {/* Captions — short, verb-led, never overlapping the pill or panels.
          Bottom 130px from frame edge per the Captions component default,
          which sits in the thumb-zone safe area. */}
      <Captions
        cues={[
          { text: 'Press.', at: 0.6, until: 2.0 },
          { text: 'Speak.', at: 2.2, until: 3.4 },
          { text: 'Done.', at: 4.5, until: 7.5 },
          { text: 'Shaped to mode.', at: 9.5, until: 13.5 },
          { text: 'It learns your words.', at: 15.0, until: 19.0 },
          { text: 'All on your PC.', at: 20.5, until: 24.0 },
        ]}
      />

      {/* End card — held 5.5s. The composition's durationInFrames sets
          the final cap. Tagline pairs the brand promise with the
          ownership angle: your voice, your machine. No em dash. */}
      <EndCard tagline="Your voice. Your machine." startAt={ENDCARD} />
    </SocialFrame>
  );
}
