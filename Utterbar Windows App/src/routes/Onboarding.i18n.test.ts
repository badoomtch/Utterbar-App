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
});
