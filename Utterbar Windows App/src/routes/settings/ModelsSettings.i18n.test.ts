import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'ModelsSettings.tsx'), 'utf8');

// Strip line + block comments before grepping so the pane's prose
// commentary (which legitimately discusses words like "Whisper",
// "Recommended", or "languages") doesn't false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('ModelsSettings i18n source-grep guard (scoped)', () => {
  // Scope note: this branch localises only the page header + the
  // Whisper-language card chrome — the surfaces most exposed by the
  // new dictation-language onboarding flow. Full ModelsSettings
  // localisation (per-row install chrome, Meeting Mode picker, the
  // keep-warm card, ProNudge passthroughs, etc.) is deferred to a
  // follow-up branch because the pane is dense with backend-emitted
  // prose (m.display_name, m.description, m.version, m.languages,
  // engine labels, ENGINE_LABEL, file sizes) that must continue to
  // render verbatim.

  it('uses useTranslation from react-i18next in the settings namespace', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]settings['"]\)/);
  });

  it('imports the shared dictation-language registry instead of carrying its own list', () => {
    // The pre-branch source had a 26-entry inline
    // WHISPER_LANGUAGE_OPTIONS array. The shared registry in
    // lib/dictationLanguages is now the single source of truth for
    // both this picker and the new onboarding step. Pin the import +
    // the live consumer call so a future regression that re-inlines
    // the list (and forks the two surfaces) fails immediately.
    expect(SOURCE).toMatch(/from ['"]@\/lib\/dictationLanguages['"]/);
    expect(SOURCE).toMatch(/DICTATION_LANGUAGES\.map/);
  });

  it('routes the page header and Whisper-language card chrome through models.* keys', () => {
    expect(SOURCE).toMatch(/models\.title/);
    expect(SOURCE).toMatch(/models\.subtitle/);
    expect(SOURCE).toMatch(/models\.whisper_language\.title/);
    expect(SOURCE).toMatch(/models\.whisper_language\.description/);
  });

  it('does not embed the localised header / whisper-language literals', () => {
    // Narrow allow-list: only the four strings this branch moved into
    // the catalogue. Other ModelsSettings.tsx surfaces (per-row
    // Install chrome, Meeting Mode card, Keep-warm card, etc.) are
    // out of scope for this branch and intentionally not covered.
    const forbidden = [
      'title="Models"',
      'subtitle="Parakeet ASR models.',
      'title="Whisper language"',
      'description="Used by Whisper models only.',
      // The pre-branch inline picker entries — pin a representative
      // sample to catch a regression that re-introduces the array.
      "label: 'Auto-detect (recommended)'",
      "label: 'English'",
      "label: 'Spanish'",
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `ModelsSettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('keeps backend-emitted ModelInfo fields rendering verbatim', () => {
    // The model catalogue exposes m.display_name, m.description,
    // m.version, m.languages, m.size_bytes — all Rust-emitted prose
    // or backend identifiers. None of them are wrapped in t().
    // Localising backend prose is deferred to a future error/event-
    // codes branch; this pin makes sure that boundary stays clean
    // through scoped header/Whisper-language localisation.
    expect(SOURCE).toMatch(/m\.display_name/);
    expect(SOURCE).toMatch(/m\.description/);
    expect(SOURCE).toMatch(/m\.version/);
    expect(SOURCE).toMatch(/m\.languages/);
    expect(SOURCE).toMatch(/m\.size_bytes/);
    // None of those should appear inside a t() catalogue path.
    expect(SOURCE).not.toMatch(/t\(['"]models[^'"]*m\.display_name/);
    expect(SOURCE).not.toMatch(/t\(['"]models[^'"]*m\.description/);
  });

  it('preserves the load-bearing IPC commands and settings field reads', () => {
    // The structural Models flow is unchanged — every Tauri command
    // name and every settings field this pane reads/writes survives
    // the localisation. Pin each so a future regression that renames
    // a command or breaks the runtime-status / install round-trip
    // fails immediately.
    const commands = ['list_available_models', 'asr_runtime_status', 'install_model'];
    for (const cmd of commands) {
      expect(
        SOURCE.includes(cmd),
        `ModelsSettings.tsx must keep the IPC command '${cmd}' intact`,
      ).toBe(true);
    }
    // models.whisper_language is still the structural field the Whisper
    // language picker reads/writes; the labels resolve through t() but
    // the values written back are the raw ISO codes from the registry.
    expect(SOURCE).toMatch(/whisper_language:\s*lang/);
    expect(SOURCE).toMatch(/default_asr_model_id/);
    expect(SOURCE).toMatch(/meeting_asr_model_id/);
    expect(SOURCE).toMatch(/warm_duration_secs/);
  });

  it('routes the language-compatibility warning through models.language_warning.* keys', () => {
    // Settings → Models surfaces the same banner shape as onboarding
    // when whisper_language is non-English and the default model is
    // English-only. Pin every key the warning resolves so a future
    // copy edit can't drift away from the catalogue.
    expect(SOURCE).toMatch(/models\.language_warning\.english_only_title/);
    expect(SOURCE).toMatch(/models\.language_warning\.english_only_body/);
    expect(SOURCE).toMatch(/models\.language_warning\.no_multilingual_title/);
    expect(SOURCE).toMatch(/models\.language_warning\.no_multilingual_body/);
    expect(SOURCE).toMatch(/models\.language_warning\.switch_to_multilingual/);
    expect(SOURCE).toMatch(/models\.language_warning\.install_multilingual_hint/);
  });

  it('reuses the shared onboarding helper for the language-compatibility check', () => {
    // The recommendation logic lives in
    // `routes/onboarding_model_recommendation.ts` so it can be unit-
    // tested in pure Node and so onboarding + Settings stay in
    // lock-step. Pin both the import shape and the live call site so
    // a future refactor that inlines the rules into the JSX (and
    // forks the two surfaces) fails immediately.
    expect(SOURCE).toMatch(/from ['"]@\/routes\/onboarding_model_recommendation['"]/);
    expect(SOURCE).toMatch(/recommendModelForLanguage\(/);
    // isMultilingualOnly() gates the warning so English / Auto users
    // never see anything; the dictation-language registry is the
    // single source of truth for what counts as "needs multilingual".
    expect(SOURCE).toMatch(/isMultilingualOnly\(/);
  });

  it('passes the offending display_name through the english-only warning interpolation', () => {
    // selectedDefaultModel.display_name is Rust-emitted prose and
    // stays verbatim; the warning body interpolates it as `{{model}}`.
    // Pin the wiring so a future regression that drops the model
    // name and leaves a literal mustache template fails at build
    // time.
    expect(SOURCE).toMatch(
      /models\.language_warning\.english_only_body['"][\s\S]{0,200}model:/,
    );
    // The fallback chain must not wrap display_name in t() — that
    // would silently rename a Rust-owned string in the UI.
    expect(SOURCE).not.toMatch(/t\(['"][^'"]*display_name/);
  });

  it('switches to the helper-recommended model id, never patches general.app_language', () => {
    // The one-click switch button calls updateDefaultModel with the
    // helper's recommendedId. Pin the wiring shape positively, then
    // negative-pin against any general.app_language patch from this
    // file (interface language is `general.app_language`, dictation
    // language is `models.whisper_language`; the contract must stay
    // separated).
    expect(SOURCE).toMatch(
      /updateDefaultModel\(\s*settingsRecommendation\.recommendedId\s*\)/,
    );
    expect(SOURCE).not.toMatch(/app_language/);
  });

  it('keeps the multilingual contract structural via languages.includes("multi")', () => {
    // The multilingual filter is the single contract identifier
    // (services/models.rs marks Whisper variants with
    // `languages: vec!["multi"]`; English-only models declare
    // `["en"]`). The Settings warning gate-checks this on the
    // selected model directly. Pin the structural string so the
    // contract identifier doesn't drift.
    expect(SOURCE).toContain("languages.includes('multi')");
  });
});
