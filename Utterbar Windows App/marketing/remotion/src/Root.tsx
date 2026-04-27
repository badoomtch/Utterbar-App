/**
 * Remotion composition registry. Every video + still that ships in the
 * marketing asset pack appears here exactly once. `render.mjs` iterates
 * this list by id, so adding a new asset is "add a <Composition/> +
 * mention its id in the manifest and you're done".
 *
 * Frame sizes (2026-04-24 regenerate):
 *   • Videos — 1440 × 900, 30 fps. Dropped from 1920×1080; the old size
 *     left the UI floating small inside a lot of empty background, and
 *     the site never displayed them full-bleed so the headroom was
 *     wasted. Smaller frame + same encode = bigger UI at display size.
 *   • Stills — 2200 × 1400. Same reasoning; also OG-card friendly at 2×.
 *
 * All compositions render on --bg-0 (#0A0E17) so the output seams
 * invisibly into the site's actual page background.
 */
import { Composition } from 'remotion';
// Side-effect CSS — order matters. globals.css MUST come first so the
// Tailwind layers + token vars land before app.css's Remotion-only
// overrides. See app.css header comment for why this can't move.
import '../../../src/styles/globals.css';
import './app.css';
import './design/fonts';

import { StateParitySheet, parityStillDefaults } from './compositions/StateParitySheet';

// Videos
import { HeroPillLoop } from './compositions/HeroPillLoop';
import { FeaturePressSpeakDone } from './compositions/FeaturePressSpeakDone';
import { FeatureWorksAnywhere } from './compositions/FeatureWorksAnywhere';
import { FeatureModes } from './compositions/FeatureModes';
import { FeatureMultilingual } from './compositions/FeatureMultilingual';
import { FeatureSystemAudio } from './compositions/FeatureSystemAudio';
import { FeatureTalkToAI } from './compositions/FeatureTalkToAI';
import { FeatureVocabulary } from './compositions/FeatureVocabulary';
import { FeatureLocalPrivacy } from './compositions/FeatureLocalPrivacy';
import { FeatureFileTranscription } from './compositions/FeatureFileTranscription';
import { FeatureHistoryRetranscribe } from './compositions/FeatureHistoryRetranscribe';
import { FeatureCustomiseBar } from './compositions/FeatureCustomiseBar';

// Stills
import {
  Press01FloatingBarIdle,
  Press02FloatingBarListening,
  Press03MainShellDashboard,
  Press04Vocabulary,
  Press05FileTranscription,
} from './compositions/stills/Stills';

// Two video aspects:
//   BANNER — 1440×520 (~2.77:1). Pill-only compositions. The pill
//     is ~120px tall at its natural size; with 2.6x scale it's ~312px.
//     A 520-tall frame gives ~100px breathing room top + bottom so the
//     pill reads as centred on a tight banner, not floating in a huge
//     box. This is the fix for the "pill-lost-in-a-box" regression on
//     the first pass.
//   UI — 1440×900 (16:10). Compositions with a card / settings panel /
//     queue / transcript — they need the vertical headroom.
const VIDEO_BANNER = { width: 1600, height: 520, fps: 30 };
const VIDEO_UI = { width: 1440, height: 900, fps: 30 };
// Stills come in two shapes:
//   - Pill stills (press-01, press-02) — square-ish so the pill sits
//     centred with breathing room top + bottom.
//   - UI stills (dashboard, vocabulary, files) — content is stacked
//     top-down, so the frame is sized to hug the content height
//     rather than leaving a tall empty stripe at the bottom.
const STILL_PILL = { width: 2200, height: 1400, fps: 30 };
const STILL_UI_DASHBOARD = { width: 2200, height: 1080, fps: 30 };
const STILL_UI_VOCAB = { width: 2200, height: 1280, fps: 30 };
const STILL_UI_FILES = { width: 2200, height: 1380, fps: 30 };

const SEC = (s: number) => Math.round(s * VIDEO_UI.fps);

export function RemotionRoot() {
  return (
    <>
      {/* ---------- VIDEOS ---------- */}
      {/* Pill-only banner-aspect compositions. Frame hugs the pill. */}
      <Composition
        id="hero-pill-loop"
        component={HeroPillLoop}
        durationInFrames={SEC(10)}
        {...VIDEO_BANNER}
      />
      <Composition
        id="feature-press-speak-done"
        component={FeaturePressSpeakDone}
        durationInFrames={SEC(4)}
        {...VIDEO_BANNER}
      />

      {/* UI-body compositions — need the vertical space for card + body. */}
      <Composition
        id="feature-works-anywhere"
        component={FeatureWorksAnywhere}
        durationInFrames={SEC(8)}
        {...VIDEO_UI}
      />
      <Composition
        id="feature-modes"
        component={FeatureModes}
        durationInFrames={SEC(7)}
        {...VIDEO_UI}
      />
      <Composition
        id="feature-multilingual"
        component={FeatureMultilingual}
        durationInFrames={SEC(12)}
        {...VIDEO_UI}
      />
      <Composition
        id="feature-system-audio"
        component={FeatureSystemAudio}
        durationInFrames={SEC(6)}
        {...VIDEO_UI}
      />
      <Composition
        id="feature-vocabulary"
        component={FeatureVocabulary}
        durationInFrames={SEC(7)}
        {...VIDEO_UI}
      />
      <Composition
        id="feature-local-privacy"
        component={FeatureLocalPrivacy}
        durationInFrames={SEC(8)}
        {...VIDEO_UI}
      />
      <Composition
        id="feature-file-transcription"
        component={FeatureFileTranscription}
        durationInFrames={SEC(12)}
        {...VIDEO_UI}
      />
      <Composition
        id="feature-talk-to-ai"
        component={FeatureTalkToAI}
        durationInFrames={SEC(10)}
        {...VIDEO_UI}
      />
      <Composition
        id="feature-history-retranscribe"
        component={FeatureHistoryRetranscribe}
        durationInFrames={SEC(7.5)}
        {...VIDEO_UI}
      />
      {/* customise-bar is also pill-only, banner aspect. */}
      <Composition
        id="feature-customise-bar"
        component={FeatureCustomiseBar}
        durationInFrames={SEC(6)}
        {...VIDEO_BANNER}
      />

      {/* Parity sheet — six floating-bar states for the refactor sign-off
          step. Rendered as a single still; not shipped to the website. */}
      <Composition
        id="state-parity-sheet"
        component={StateParitySheet}
        durationInFrames={1}
        width={1920}
        height={1080}
        fps={30}
        defaultProps={parityStillDefaults}
      />

      {/* ---------- STILLS ---------- */}
      <Composition
        id="press-01-floating-bar-idle"
        component={Press01FloatingBarIdle}
        durationInFrames={1}
        {...STILL_PILL}
      />
      <Composition
        id="press-02-floating-bar-listening"
        component={Press02FloatingBarListening}
        durationInFrames={1}
        {...STILL_PILL}
      />
      <Composition
        id="press-03-main-shell-dashboard"
        component={Press03MainShellDashboard}
        durationInFrames={1}
        {...STILL_UI_DASHBOARD}
      />
      <Composition
        id="press-04-vocabulary"
        component={Press04Vocabulary}
        durationInFrames={1}
        {...STILL_UI_VOCAB}
      />
      <Composition
        id="press-05-file-transcription"
        component={Press05FileTranscription}
        durationInFrames={1}
        {...STILL_UI_FILES}
      />
    </>
  );
}
