import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'PrivacySettings.tsx'), 'utf8');

// Strip line + block comments before grepping so prose commentary in
// PrivacySettings.tsx (which legitimately mentions things like
// "Privacy", "context", "Teams", or filesystem paths) doesn't
// false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('PrivacySettings i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]settings['"]\)/);
  });

  it('does not embed pinned old visible literal props or strings', () => {
    // Narrow allow-list. Each entry is a literal that lived in the
    // pre-localisation source and would re-appear if a future edit
    // bypassed t(). Comments are stripped above so prose commentary
    // mentioning these tokens doesn't false-trigger this guard.
    const forbidden = [
      // PageHeader props.
      'title="Privacy"',
      'subtitle="Utterbar processes everything on-device.',
      // ManagedHint body.
      '>Some privacy controls on this device are managed by your administrator.',
      // Context-capture card props + per-row labels/descriptions.
      'title="Context capture"',
      'description="Which signals may a mode use to tailor its output?"',
      'label="Allow active-app context"',
      'label="Allow selection context"',
      'label="Allow clipboard context"',
      'description="App executable and window title."',
      'description="Currently selected text in the foreground app."',
      'description="Most recently copied text."',
      // Audio retention card props.
      'title="Audio retention"',
      'label="Save recordings"',
      'label="Delete recordings after"',
      // ProNudge prop literals.
      'feature="Audio retention"',
      'description="Keep a local WAV of every dictation',
      // Slider readout — was a bare ` days` template suffix.
      '} days',
      // Media card props.
      'title="Media"',
      'label="Pause music while recording"',
      'label="Mute conferencing app output while recording"',
      // Data-locations card props + per-row label literals (the old
      // [[k, v]] tuples used capitalised prose strings as the label
      // — those are now keyed off stable ids and resolved through
      // t()).
      'title="Data locations"',
      "['Config'",
      "['Database'",
      "['Recordings'",
      "['Logs'",
      // Conferencing-apps editor chrome.
      '>Apps to silence<',
      '>Reset to defaults<',
      '>No apps configured.<',
      'placeholder="e.g. Teams.exe"',
      '>Add\n',
      // The aria-label backtick template.
      'aria-label={`Remove ${name}`}',
      // The four VALIDATION_MESSAGES dictionary string values + the
      // original fallback message — these now live in the catalogue.
      "'Enter an executable name.'",
      "'Name is too long (max 64 characters).'",
      "'That app is already in the list.'",
      "'Defaults are still loading — try again in a moment.'",
      "'That name isn’t valid.'",
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `PrivacySettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the page header and visible chrome through privacy.* keys', () => {
    expect(SOURCE).toMatch(/privacy\.title/);
    expect(SOURCE).toMatch(/privacy\.subtitle/);
    expect(SOURCE).toMatch(/privacy\.managed_hint/);
    // Context capture sub-tree.
    expect(SOURCE).toMatch(/privacy\.context\.title/);
    expect(SOURCE).toMatch(/privacy\.context\.description/);
    expect(SOURCE).toMatch(/privacy\.context\.active_app\.label/);
    expect(SOURCE).toMatch(/privacy\.context\.selection\.label/);
    expect(SOURCE).toMatch(/privacy\.context\.clipboard\.label/);
    // Audio retention sub-tree.
    expect(SOURCE).toMatch(/privacy\.audio_retention\.title/);
    expect(SOURCE).toMatch(/privacy\.audio_retention\.save_recordings\.label/);
    expect(SOURCE).toMatch(/privacy\.audio_retention\.delete_after\.label/);
    expect(SOURCE).toMatch(/privacy\.audio_retention\.delete_after\.value/);
    // Media sub-tree.
    expect(SOURCE).toMatch(/privacy\.media\.title/);
    expect(SOURCE).toMatch(/privacy\.media\.pause_music\.label/);
    expect(SOURCE).toMatch(/privacy\.media\.mute_conferencing\.label/);
    // Conferencing-apps editor.
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.title/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.description_prefix/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.using_defaults/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.using_custom/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.edit_locked/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.reset_to_defaults/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.empty/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.input_placeholder/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.add/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.remove_aria/);
    // The validator-reason -> message lookup: pin per-id resolution
    // with a template-literal key plus the fallback path. The
    // substitution body can carry a TypeScript cast (`as Foo`) so the
    // regex must accept any non-`}` content inside `${...}`.
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.errors\.\$\{[^}]*?\}/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.errors\.fallback/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.errors\.invalid_characters/);
    expect(SOURCE).toMatch(/privacy\.conferencing_apps\.errors\.defaults_loading/);
    // Data-locations rows resolved per-id from a tuple list.
    expect(SOURCE).toMatch(/privacy\.data_locations\.title/);
    expect(SOURCE).toMatch(/privacy\.data_locations\.rows\.\$\{[^}]*?\}/);
  });

  it('passes interpolation values through t() at the call sites', () => {
    // The slider readout passes `{ days }`; the remove-button
    // aria-label passes `{ name }`. Pin both wiring shapes so a future
    // regression that drops an interpolation argument fails at build
    // time, not by leaking a literal mustache template to users.
    expect(SOURCE).toMatch(
      /privacy\.audio_retention\.delete_after\.value['"][\s\S]{0,80}days:/,
    );
    expect(SOURCE).toMatch(
      /privacy\.conferencing_apps\.remove_aria['"][\s\S]{0,80}name/,
    );
  });

  it('keeps backend-emitted DataPaths and conferencing-app names verbatim', () => {
    // The DataPaths values (paths.config / paths.data / paths.models /
    // paths.recordings / paths.logs) are filesystem paths emitted by
    // the Rust `get_data_paths` command and must continue to render
    // verbatim inside the <code> block. The conferencing-app `name`
    // tags likewise render the user-/backend-supplied executable
    // string verbatim inside its <span> tag. Pin both shapes
    // positively, plus a negative pin that the path values aren't
    // wrapped in t() (would silently rename the user's filesystem
    // paths in the UI).
    expect(SOURCE).toMatch(/paths\.config/);
    expect(SOURCE).toMatch(/paths\.data/);
    expect(SOURCE).toMatch(/paths\.models/);
    expect(SOURCE).toMatch(/paths\.recordings/);
    expect(SOURCE).toMatch(/paths\.logs/);
    expect(SOURCE).not.toMatch(/<code[^>]*>\{t\(['"]privacy[^'"]*paths\./);
    // Conferencing-app `name` rendered between the opening <span> and
    // the inline remove button.
    expect(SOURCE).toMatch(/\{name\}\s*\n?\s*<button/);
  });

  it('preserves the load-bearing IPC commands and validator contract', () => {
    // The structural privacy / conferencing-mute flow is unchanged —
    // every Tauri command name must survive the localisation. The
    // reason-id list pinned in the source must also remain a strict
    // subset of the validator-emitted ids (the fallback path handles
    // unknown future ids gracefully without blowing up).
    const commands = [
      'get_data_paths',
      'get_default_conferencing_apps',
      'validate_conferencing_app_name',
    ];
    for (const cmd of commands) {
      expect(
        SOURCE.includes(cmd),
        `PrivacySettings.tsx must keep the IPC command '${cmd}' intact`,
      ).toBe(true);
    }
    // The four canonical validator reason ids the editor recognises.
    expect(SOURCE).toMatch(/'empty'/);
    expect(SOURCE).toMatch(/'invalid_characters'/);
    expect(SOURCE).toMatch(/'too_long'/);
    expect(SOURCE).toMatch(/'duplicate'/);
  });

  it('preserves the underlying privacy settings field names', () => {
    // The localisation only swaps display labels. Each control still
    // reads `s.privacy.<field>` (or via the loose `p` index helper)
    // and writes via `update('<field>', v)` against the
    // `privacy.<field>` slot on Settings. Pin every load-bearing
    // field name so a future regression that renames a settings
    // field fails immediately.
    const fields = [
      'allow_context_active_app',
      'allow_context_selection',
      'allow_context_clipboard',
      'retain_audio',
      'retain_audio_days',
      'pause_media_while_recording',
      'mute_conferencing_while_recording',
      'conferencing_apps_override',
    ];
    for (const field of fields) {
      expect(
        SOURCE.includes(field),
        `PrivacySettings.tsx must keep the settings field name '${field}' intact`,
      ).toBe(true);
    }
  });
});
