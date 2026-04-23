/**
 * Remotion composition registry. Every video + still that ships in the
 * marketing asset pack appears here exactly once. `render.mjs` iterates
 * this list by id, so adding a new asset is "add a <Composition/> +
 * mention its id in the manifest and you're done".
 *
 * Video conventions:
 *   • 1920 × 1080, 30 fps (matches the website player + poster aspect).
 *   • Durations in frames, not seconds — Remotion's native unit.
 *   • All IDs are kebab-case and stable: they're baked into the manifest
 *     and filename pattern (`<id>.mp4`, `<id>-poster.png`).
 *
 * Still conventions:
 *   • 2560 × 1600 single-frame. Render via `remotion still --frame=0`.
 *   • Also kebab-case; filename pattern (`<id>.png`).
 */
import { Composition } from 'remotion';
// Side-effect CSS — order matters. globals.css MUST come first so the
// Tailwind layers + token vars land before app.css's Remotion-only
// overrides (bg-0 under the pill, floating-shell bleed-through). We
// import globals.css directly as a sibling rather than letting app.css
// @import it, because `@remotion/bundler/css-loader` disables the
// postcss-loader chain on CSS @imports (see app.css's header comment for
// the full rationale). Top-level JS imports are the only entry path that
// reliably runs postcss-loader, which is where `@tailwind base/etc.`
// actually get expanded into utility rules.
import '../../../src/styles/globals.css';
import './app.css';
import './design/fonts';

// Pre-register the real app's FloatingBar so the stills composition can
// reach it via BarDriver. Importing here rather than lazily keeps the
// bundle graph explicit.
import { StateParitySheet, parityStillDefaults } from './compositions/StateParitySheet';

// Videos
import { HeroPillLoop } from './compositions/HeroPillLoop';
import { HowItWorks } from './compositions/HowItWorks';
import { FeaturePressSpeakDone } from './compositions/FeaturePressSpeakDone';
import { FeatureWorksAnywhere } from './compositions/FeatureWorksAnywhere';
import { FeatureModes } from './compositions/FeatureModes';
import { FeatureSystemAudio } from './compositions/FeatureSystemAudio';
import { FeatureVocabulary } from './compositions/FeatureVocabulary';
import { FeatureLocalPrivacy } from './compositions/FeatureLocalPrivacy';
import { FeatureFileTranscription } from './compositions/FeatureFileTranscription';
import { FeatureHistoryRetranscribe } from './compositions/FeatureHistoryRetranscribe';
import { FeatureCustomiseBar } from './compositions/FeatureCustomiseBar';
import { OnboardingWalkthrough } from './compositions/OnboardingWalkthrough';

// Stills
import {
  Press01FloatingBarIdle,
  Press02FloatingBarListening,
  Press03MainShellDashboard,
  Press04Vocabulary,
  Press05FileTranscription,
} from './compositions/stills/Stills';

const VIDEO = { width: 1920, height: 1080, fps: 30 };
const STILL = { width: 2560, height: 1600, fps: 30 };

// Durations in seconds, resolved to frames at 30 fps. Kept in one place so
// render.mjs can derive per-asset metadata from the same constants.
const SEC = (s: number) => Math.round(s * VIDEO.fps);

export function RemotionRoot() {
  return (
    <>
      {/* ---------- VIDEOS ---------- */}
      <Composition
        id="hero-pill-loop"
        component={HeroPillLoop}
        durationInFrames={SEC(6)}
        {...VIDEO}
      />
      <Composition
        id="how-it-works-walkthrough"
        component={HowItWorks}
        durationInFrames={SEC(12)}
        {...VIDEO}
      />
      <Composition
        id="feature-press-speak-done"
        component={FeaturePressSpeakDone}
        durationInFrames={SEC(5)}
        {...VIDEO}
      />
      <Composition
        id="feature-works-anywhere"
        component={FeatureWorksAnywhere}
        durationInFrames={SEC(10)}
        {...VIDEO}
      />
      <Composition
        id="feature-modes"
        component={FeatureModes}
        durationInFrames={SEC(8)}
        {...VIDEO}
      />
      <Composition
        id="feature-system-audio"
        component={FeatureSystemAudio}
        durationInFrames={SEC(7)}
        {...VIDEO}
      />
      <Composition
        id="feature-vocabulary"
        component={FeatureVocabulary}
        durationInFrames={SEC(8)}
        {...VIDEO}
      />
      <Composition
        id="feature-local-privacy"
        component={FeatureLocalPrivacy}
        durationInFrames={SEC(8)}
        {...VIDEO}
      />
      <Composition
        id="feature-file-transcription"
        component={FeatureFileTranscription}
        durationInFrames={SEC(8)}
        {...VIDEO}
      />
      <Composition
        id="feature-history-retranscribe"
        component={FeatureHistoryRetranscribe}
        durationInFrames={SEC(9)}
        {...VIDEO}
      />
      <Composition
        id="feature-customise-bar"
        component={FeatureCustomiseBar}
        durationInFrames={SEC(8)}
        {...VIDEO}
      />
      <Composition
        id="onboarding-walkthrough"
        component={OnboardingWalkthrough}
        durationInFrames={SEC(12)}
        {...VIDEO}
      />

      {/* Parity sheet — six floating-bar states (idle / warming /
          listening / processing / success / error) stacked vertically
          on bg-0 for the refactor's sign-off step. Rendered as a single
          still; not shipped to the website. */}
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
        {...STILL}
      />
      <Composition
        id="press-02-floating-bar-listening"
        component={Press02FloatingBarListening}
        durationInFrames={1}
        {...STILL}
      />
      <Composition
        id="press-03-main-shell-dashboard"
        component={Press03MainShellDashboard}
        durationInFrames={1}
        {...STILL}
      />
      <Composition
        id="press-04-vocabulary"
        component={Press04Vocabulary}
        durationInFrames={1}
        {...STILL}
      />
      <Composition
        id="press-05-file-transcription"
        component={Press05FileTranscription}
        durationInFrames={1}
        {...STILL}
      />
    </>
  );
}
