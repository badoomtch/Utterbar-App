import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'Onboarding.tsx'), 'utf8');

// Strip line + block comments before grepping so the existing prose
// commentary inside Onboarding.tsx (which legitimately discusses words
// like "Welcome" or "Verifying") doesn't false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('Onboarding i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]onboarding['"]\)/);
  });

  it('does not embed pinned old onboarding literals as plain JSX strings', () => {
    // Narrow allow-list. Each entry is the literal *as it appeared in the
    // pre-localisation source*: a step title rendered inside a `<h1>`,
    // a body string passed to the Hero component, a button label, or a
    // direct JSX text node. The grep does not trip on the same word
    // appearing inside a t() string key, a comment, or a different
    // wrapper.
    //
    // Keep the list tight and tied to confirmed user-visible surfaces.
    const forbidden = [
      "title: 'Welcome to Utterbar'",
      "title: 'Everything local, always'",
      "title: 'Update checks'",
      "title: 'Pick a theme'",
      "title: 'Choose your ASR model'",
      "title: 'Use your GPU?'",
      "title: 'Pick and test your microphone'",
      "title: 'Your typing speed'",
      "title: 'How would you like to trigger recording?'",
      "title: 'Bind a recording shortcut'",
      "title: 'Where Utterbar lives'",
      "title: 'Ready to dictate'",
      'title="Everything stays on your machine"',
      'title="Bind a recording shortcut"',
      '>Choose your ASR model<',
      "'Verifying checksum…'",
      "'Connecting…'",
      "'Download failed. Check your connection and try again.'",
      ">Welcome<",
      "'Speak now, release to finish.'",
      "'Speak now, press again to stop.'",
      "'Processing your speech…'",
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `Onboarding.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('still renders Rust-emitted m.description verbatim (deferred backend prose)', () => {
    // The model-picker description paragraph reads `m.description`, sourced
    // from `ModelInfo.description` in the Rust domain layer. Localising
    // backend prose is deferred to a future error/event-codes branch.
    // This test pins the deferral so the comment + verbatim render can't
    // silently disappear.
    expect(SOURCE).toMatch(/m\.description/);
    expect(SOURCE).toMatch(/shortDescription\s*\(\s*m\.id\s*,\s*m\.description\s*\)/);
  });

  it('routes the model-download phase strings through onboarding keys', () => {
    // Old shape was a hand-rolled template (`Downloading · ${rateLabel} ·
    // ${etaLabel}`). New shape resolves through `model.download.*` keys.
    // Pin both halves so a future regression fails.
    expect(SOURCE).not.toMatch(/`Downloading · \$\{/);
    expect(SOURCE).not.toMatch(/`Extracting · \$\{/);
    expect(SOURCE).toMatch(/model\.download\.downloading/);
    expect(SOURCE).toMatch(/model\.download\.verifying/);
    expect(SOURCE).toMatch(/model\.download\.extracting/);
    expect(SOURCE).toMatch(/model\.download\.error_generic/);
  });

  it('reuses settings:general.appearance.theme.options for theme chip labels', () => {
    // The theme step deliberately resolves chip labels through the existing
    // settings namespace rather than duplicating them in onboarding.json.
    // Pin the cross-namespace contract so a future cleanup pass can't
    // silently fork the strings.
    expect(SOURCE).toMatch(/general\.appearance\.theme\.options\./);
  });

  it('routes the new dictation-language step through onboarding.language.* keys', () => {
    // The dictation-language step (`DictationLanguageStep`) writes
    // `models.whisper_language` and is the only frontend touch point
    // for the user's chosen dictation language. The picker labels
    // come from `lib/dictationLanguages.ts` via the shared `common.*`
    // namespace; the surrounding chrome lives under
    // `onboarding.language.*`. Pin every key the step actually
    // resolves so a future copy edit can't drift away from the
    // catalogue.
    expect(SOURCE).toMatch(/language\.title/);
    expect(SOURCE).toMatch(/language\.subtitle/);
    expect(SOURCE).toMatch(/language\.picker_label/);
    expect(SOURCE).toMatch(/language\.picker_description/);
    expect(SOURCE).toMatch(/language\.warning\.english_only_title/);
    expect(SOURCE).toMatch(/language\.warning\.english_only_body/);
    expect(SOURCE).toMatch(/language\.warning\.no_multilingual_title/);
    expect(SOURCE).toMatch(/language\.warning\.no_multilingual_body/);
    expect(SOURCE).toMatch(/language\.switch_to_multilingual/);
  });

  it('writes the dictation choice to models.whisper_language, never to general.app_language', () => {
    // The contract: dictation language is `models.whisper_language`,
    // app interface language is `general.app_language`. They must
    // stay decoupled. Pin the positive shape (DictationLanguageStep
    // patches `models: { ..., whisper_language: code }`) and a
    // negative pin against `app_language: ` written from the new
    // step. The latter would catch a future regression that couples
    // the two.
    expect(SOURCE).toMatch(/whisper_language:\s*code/);
    // The new step must not patch general.app_language. (The existing
    // GeneralSettings page is the only legitimate writer; that page
    // is unaffected by this branch.)
    expect(SOURCE).not.toMatch(/general:\s*\{[^}]*app_language:[^}]*whisper_language/);
  });

  it('uses the pure recommendation helper for language-aware model picking', () => {
    // The recommendation logic lives in
    // `routes/onboarding_model_recommendation.ts` so it can be unit-
    // tested in pure Node. Pin both the import shape and the live
    // call site so a future refactor that inlines the logic back
    // into the JSX (and loses test coverage) fails immediately.
    expect(SOURCE).toMatch(/from ['"]@\/routes\/onboarding_model_recommendation['"]/);
    expect(SOURCE).toMatch(/recommendModelForLanguage\(/);
  });

  it('passes the offending model display name through the english-only warning interpolation', () => {
    // m.display_name is Rust-emitted and stays verbatim; the
    // surrounding warning prose interpolates it as `{{model}}`. Pin
    // the wiring so a future copy edit can't drop the model name and
    // leave a literal mustache template visible to users.
    expect(SOURCE).toMatch(
      /language\.warning\.english_only_body['"][\s\S]{0,200}model:/,
    );
  });
});
