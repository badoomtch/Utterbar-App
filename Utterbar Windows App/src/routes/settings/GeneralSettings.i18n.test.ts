import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'GeneralSettings.tsx'), 'utf8');

// Strip line + block comments before grepping so the existing prose
// commentary (which legitimately discusses words like "Theme" or
// "Sound cues") doesn't false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('GeneralSettings i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    // If this import vanishes the file is back to hardcoded English.
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]settings['"]\)/);
  });

  it('does not embed pinned old card titles or row labels as plain literals', () => {
    // Narrow allow-list of literals that previously lived as JSX
    // attribute strings on Card / Row / PageHeader / UpgradeModal.
    // Each entry is the literal *as the attribute value* (with quotes
    // and the prop name) so the grep doesn't false-trigger on the
    // word appearing inside a t(...) string key or a comment.
    //
    // Keep this list tight. Anything outside General settings is out
    // of scope for this guard.
    const forbidden = [
      'title="General"',
      'title="Startup"',
      'title="Appearance"',
      'title="Feedback"',
      'label="Launch on login"',
      'label="Start minimised"',
      'label="Close to tray"',
      'label="Theme"',
      'label="Accent"',
      'label="Sound cues"',
      'label="Sound profile"',
      'label="System notifications"',
      'feature="Custom accent colours"',
      'aria-label="Custom accent colour"',
      "'Pick any colour (Pro)'",
      "'Custom accent is a Pro feature'",
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `GeneralSettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the Pro upgrade-modal bullets through t(...)', () => {
    // The previous shape was a hardcoded array of three English
    // sentences. The new path uses the `general.upgrade.custom_accent.
    // bullets.*` keys; the test pins both halves so a future regression
    // (either re-introducing the literals OR dropping the keys) fails.
    expect(SOURCE).not.toMatch(/'Pick any hex colour from the native Windows colour picker'/);
    expect(SOURCE).toMatch(/general\.upgrade\.custom_accent\.bullets\.any_hex/);
    expect(SOURCE).toMatch(/general\.upgrade\.custom_accent\.bullets\.auto_tint_shade/);
    expect(SOURCE).toMatch(/general\.upgrade\.custom_accent\.bullets\.presets_free/);
  });

  it('App language card still renders the honest dictation-language description key', () => {
    // The picker copy must keep pointing at the i18n key whose value
    // says "Dictation language is set separately in Models." The
    // i18n.test.ts companion test pins the value itself.
    expect(SOURCE).toMatch(/general\.app_language\.description/);
  });
});
