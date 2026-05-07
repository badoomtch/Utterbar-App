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
});
