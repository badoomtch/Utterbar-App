/**
 * Composition registry for the social-ad pack. Vertical 1080×1920, 30 fps.
 * Strictly additive — does not touch ../Root.tsx.
 *
 * Naming: ad-NN-<slug>. NN is stable, so reorderings here don't break
 * any external scripts that target a specific id. ad-NN doubles as the
 * shoot-list ordering reference in ./README.md.
 */
import { Composition } from 'remotion';
// Side-effect CSS — paths are relative to src/ads/, one level deeper than
// Root.tsx, hence the extra ../ on every entry.
import '../../../../src/styles/globals.css';
import '../app.css';
import '../design/fonts';

import { Ad01StopTyping } from './compositions/Ad01StopTyping';
import { Ad02VoiceIsFaster } from './compositions/Ad02VoiceIsFaster';
import { Ad03LocalOnly } from './compositions/Ad03LocalOnly';
import { Ad04TalksToAnyAI } from './compositions/Ad04TalksToAnyAI';
import { Ad05ThirtyLanguages } from './compositions/Ad05ThirtyLanguages';
import { Ad06HoldHotkey } from './compositions/Ad06HoldHotkey';
import { Ad07TeachVocabulary } from './compositions/Ad07TeachVocabulary';
import { Ad08WindowsFirst } from './compositions/Ad08WindowsFirst';
import { Ad09Manifesto } from './compositions/Ad09Manifesto';
import { Ad10PriceOnce } from './compositions/Ad10PriceOnce';
import { Ad11NoAccount } from './compositions/Ad11NoAccount';
import { Ad12FoundersNote } from './compositions/Ad12FoundersNote';

const VERTICAL = { width: 1080, height: 1920, fps: 30 } as const;
const SEC = (s: number) => Math.round(s * VERTICAL.fps);

export function AdsRoot() {
  return (
    <>
      <Composition id="ad-01-stop-typing"        component={Ad01StopTyping}       durationInFrames={SEC(12)} {...VERTICAL} />
      <Composition id="ad-02-voice-is-faster"    component={Ad02VoiceIsFaster}    durationInFrames={SEC(11)} {...VERTICAL} />
      <Composition id="ad-03-local-only"         component={Ad03LocalOnly}        durationInFrames={SEC(10)} {...VERTICAL} />
      <Composition id="ad-04-talks-to-any-ai"    component={Ad04TalksToAnyAI}     durationInFrames={SEC(13)} {...VERTICAL} />
      <Composition id="ad-05-thirty-languages"   component={Ad05ThirtyLanguages}  durationInFrames={SEC(12)} {...VERTICAL} />
      <Composition id="ad-06-hold-hotkey"        component={Ad06HoldHotkey}       durationInFrames={SEC(8)}  {...VERTICAL} />
      <Composition id="ad-07-teach-vocabulary"   component={Ad07TeachVocabulary}  durationInFrames={SEC(12)} {...VERTICAL} />
      <Composition id="ad-08-windows-first"      component={Ad08WindowsFirst}     durationInFrames={SEC(10)} {...VERTICAL} />
      <Composition id="ad-09-manifesto"          component={Ad09Manifesto}        durationInFrames={SEC(30)} {...VERTICAL} />
      <Composition id="ad-10-price-once"         component={Ad10PriceOnce}        durationInFrames={SEC(9)}  {...VERTICAL} />
      <Composition id="ad-11-no-account"         component={Ad11NoAccount}        durationInFrames={SEC(11)} {...VERTICAL} />
      <Composition id="ad-12-founders-note"      component={Ad12FoundersNote}     durationInFrames={SEC(20)} {...VERTICAL} />
    </>
  );
}
