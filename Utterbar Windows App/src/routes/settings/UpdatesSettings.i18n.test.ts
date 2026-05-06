import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'UpdatesSettings.tsx'), 'utf8');

// Strip line + block comments before grepping so prose commentary in
// UpdatesSettings.tsx (which legitimately mentions things like
// "stable", "beta", "release notes", or "Subscribed") doesn't
// false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('UpdatesSettings i18n source-grep guard', () => {
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
      'title="Updates"',
      'subtitle="Choose your channel,',
      // Beta-channel warning JSX text.
      ">You're on the beta channel.<",
      'Beta builds are pre-release.',
      // Channel card props + the inline UPDATE_CHANNELS array entries.
      'title="Channel"',
      'label="Update channel"',
      "label: 'Stable'",
      "label: 'Beta'",
      // Auto-check row props.
      'label="Check for updates automatically"',
      'description="Checks the update feed on launch',
      // Check-now row: the pre-localisation backtick template + the
      // 4 dynamic description literals + the button states.
      "${updateInfo.current} → ${updateInfo.available}",
      "`You're up to date.`",
      "Couldn't reach the update server.",
      "'Asks the update server right now,",
      "? 'Checking…' : 'Check now'",
      'label="Check now"',
      // Release-notes card title + body literals.
      "What's new in ${version}",
      '"What\'s new in this version"',
      "'Release notes for the version you're running.",
      'No release notes bundled for this version.',
      // Release-email card props.
      'title="Release updates"',
      'description="Get a short email when a new Utterbar',
      'label="Email me when there\'s a new release"',
      // ReleaseEmailCard subscribe form / status literals.
      'placeholder="you@example.com"',
      "? 'Sending…' : 'Subscribe'",
      // Subscribed-as banner: the pre-localisation `Subscribed as ${current}`
      // shape + the Change-email button JSX text.
      '>Subscribed as ',
      '>Change email<',
      // Inline error messages — these now live in the catalogue.
      "'Enter your email to subscribe.'",
      "That doesn't look like an email address",
      "Couldn't reach the server.",
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `UpdatesSettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the page header and visible chrome through updates.* keys', () => {
    expect(SOURCE).toMatch(/updates\.title/);
    expect(SOURCE).toMatch(/updates\.subtitle/);
    expect(SOURCE).toMatch(/updates\.beta_warning\.heading/);
    expect(SOURCE).toMatch(/updates\.beta_warning\.body/);
    expect(SOURCE).toMatch(/updates\.channel\.title/);
    expect(SOURCE).toMatch(/updates\.channel\.description/);
    expect(SOURCE).toMatch(/updates\.channel\.row_label/);
    // Channel chip labels resolved per-id at render time.
    expect(SOURCE).toMatch(/updates\.channel\.options\.\$\{[^}]*?\}/);
    expect(SOURCE).toMatch(/updates\.channel\.auto_check\.label/);
    expect(SOURCE).toMatch(/updates\.channel\.auto_check\.description/);
    expect(SOURCE).toMatch(/updates\.channel\.check_now\.label/);
    expect(SOURCE).toMatch(/updates\.channel\.check_now\.description_idle/);
    expect(SOURCE).toMatch(/updates\.channel\.check_now\.description_available/);
    expect(SOURCE).toMatch(/updates\.channel\.check_now\.description_up_to_date/);
    expect(SOURCE).toMatch(/updates\.channel\.check_now\.description_failed/);
    expect(SOURCE).toMatch(/updates\.channel\.check_now\.checking/);
    expect(SOURCE).toMatch(/updates\.channel\.check_now\.button/);
    expect(SOURCE).toMatch(/updates\.release_notes\.title_with_version/);
    expect(SOURCE).toMatch(/updates\.release_notes\.title_fallback/);
    expect(SOURCE).toMatch(/updates\.release_notes\.description/);
    expect(SOURCE).toMatch(/updates\.release_notes\.empty/);
    expect(SOURCE).toMatch(/updates\.release_email\.title/);
    expect(SOURCE).toMatch(/updates\.release_email\.description/);
    expect(SOURCE).toMatch(/updates\.release_email\.row\.label/);
    expect(SOURCE).toMatch(/updates\.release_email\.row\.description/);
    expect(SOURCE).toMatch(/updates\.release_email\.input_placeholder/);
    expect(SOURCE).toMatch(/updates\.release_email\.subscribe/);
    expect(SOURCE).toMatch(/updates\.release_email\.sending/);
    expect(SOURCE).toMatch(/updates\.release_email\.subscribed_as/);
    expect(SOURCE).toMatch(/updates\.release_email\.change_email/);
    expect(SOURCE).toMatch(/updates\.release_email\.errors\.empty/);
    expect(SOURCE).toMatch(/updates\.release_email\.errors\.invalid/);
    expect(SOURCE).toMatch(/updates\.release_email\.errors\.fallback/);
  });

  it('passes interpolation values through t() at the call sites', () => {
    // Pin every interpolated key + the variable-name shape it expects.
    // The check-now description carries both `current` and `available`;
    // the release-notes title carries `version`; the subscribed-as
    // banner carries `email`. A future regression that drops any
    // argument fails at build time, not by leaking a literal mustache
    // template to users.
    expect(SOURCE).toMatch(
      /updates\.channel\.check_now\.description_available['"][\s\S]{0,120}current:[\s\S]{0,40}available:/,
    );
    expect(SOURCE).toMatch(
      /updates\.release_notes\.title_with_version['"][\s\S]{0,80}version/,
    );
    expect(SOURCE).toMatch(
      /updates\.release_email\.subscribed_as['"][\s\S]{0,80}email:/,
    );
  });

  it('keeps backend-emitted updater feed and subscription values verbatim', () => {
    // updateInfo.current and updateInfo.available are version strings
    // emitted by the Rust updater feed. The bundled release-notes
    // markdown comes from `current_release_notes`. The server-returned
    // subscription `msg` is backend prose. The OrliqError envelope's
    // `message` field is also backend prose. None of these strings
    // are wrapped in t(); they flow through verbatim. Pin the
    // structural usages positively, plus a couple of negative pins.
    expect(SOURCE).toMatch(/updateInfo\?\.current/);
    expect(SOURCE).toMatch(/updateInfo\.available/);
    expect(SOURCE).toMatch(/setNotes\(/);
    // The release-notes <ReleaseNotes md={notes} /> call: notes flow
    // straight through from Rust to the inline markdown renderer.
    expect(SOURCE).toMatch(/<ReleaseNotes md=\{notes\}/);
    // Server-returned subscription message: status text comes from
    // `msg` (Rust string return value).
    expect(SOURCE).toMatch(/text:\s*msg\b/);
    // OrliqError envelope's message field is rendered verbatim when
    // the looksLikeOrliqError check passes.
    expect(SOURCE).toMatch(/env\.message!/);
    // The user-supplied email value `current` is rendered verbatim
    // through the catalogued subscribed_as template — pin the
    // interpolation shape.
    expect(SOURCE).toMatch(/email:\s*current/);
  });

  it('preserves the load-bearing IPC commands and update-store wiring', () => {
    // The structural updater + subscription flow is unchanged — every
    // Tauri command name and the useUpdate store contract must
    // survive the localisation. Pin each so a future regression that
    // renames a command or breaks the check-now loop fails
    // immediately.
    const commands = ['current_release_notes', 'subscribe_to_release_updates'];
    for (const cmd of commands) {
      expect(
        SOURCE.includes(cmd),
        `UpdatesSettings.tsx must keep the IPC command '${cmd}' intact`,
      ).toBe(true);
    }
    // The useUpdate store hooks the page reads.
    expect(SOURCE).toMatch(/useUpdate\(\(x\)\s*=>\s*x\.checking\)/);
    expect(SOURCE).toMatch(/useUpdate\(\(x\)\s*=>\s*x\.lastManualOutcome\)/);
    expect(SOURCE).toMatch(/useUpdate\(\(x\)\s*=>\s*x\.info\)/);
    expect(SOURCE).toMatch(/useUpdate\(\(x\)\s*=>\s*x\.check\)/);
    // The manual-check call still flags `manual: true`.
    expect(SOURCE).toMatch(/runCheck\(\{ manual: true \}\)/);
  });

  it('preserves the underlying settings field names and channel ids', () => {
    // The localisation only swaps display labels. Each control still
    // reads `s.advanced.update_channel` / `g.check_updates_automatically`
    // / `g.releases_email` and writes via `patch({ ... })` against
    // the matching slot on Settings. Pin every load-bearing field
    // name so a future regression that renames a settings field
    // fails immediately.
    const fields = ['update_channel', 'check_updates_automatically', 'releases_email'];
    for (const field of fields) {
      expect(
        SOURCE.includes(field),
        `UpdatesSettings.tsx must keep the settings field name '${field}' intact`,
      ).toBe(true);
    }
    // Channel ids `'stable'` and `'beta'` are the Rust enum values
    // written back to update_channel; they must stay structural.
    expect(SOURCE).toMatch(/UPDATE_CHANNEL_IDS\s*=\s*\[\s*'stable',\s*'beta'\s*\]/);
  });
});
