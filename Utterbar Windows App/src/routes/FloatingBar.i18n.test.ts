import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW_FLOATING = readFileSync(resolve(HERE, 'FloatingBar.tsx'), 'utf8');
const RAW_BANNER = readFileSync(
  resolve(HERE, '..', 'components', 'bar', 'MicMuteBanner.tsx'),
  'utf8',
);

// Strip line + block comments before grepping so the existing prose
// commentary (which legitimately discusses words like "Stop" or
// "Confirmed") doesn't false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const FLOATING = stripComments(RAW_FLOATING);
const BANNER = stripComments(RAW_BANNER);

describe('FloatingBar i18n source-grep guard', () => {
  it('FloatingBar.tsx uses useTranslation from react-i18next', () => {
    // If this import vanishes, the file is back to hardcoded English.
    expect(RAW_FLOATING).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW_FLOATING).toMatch(/useTranslation\(['"]bar['"]\)/);
  });

  it('FloatingBar.tsx does not embed pinned old button labels as plain literals', () => {
    // Narrow allow-list. Each entry is the literal label *as a quoted
    // BarButton prop* — so the grep does not trip on the words appearing
    // inside JSX prose, comments, or test fixtures elsewhere in the file.
    //
    // Keep this list tight. Anything outside the action cluster is out
    // of scope for this guard.
    const forbidden = [
      'label="Stop"',
      'label="Cancel"',
      'label="Record"',
      'label="Settings"',
      "label={copied ? 'Copied' : 'Copy last message'}",
    ];
    for (const literal of forbidden) {
      expect(
        FLOATING.includes(literal),
        `FloatingBar.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('MicMuteBanner.tsx uses useTranslation from react-i18next', () => {
    expect(RAW_BANNER).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW_BANNER).toMatch(/useTranslation\(['"]bar['"]\)/);
  });

  it('MicMuteBanner.tsx routes the Confirmed button through t(...)', () => {
    // The button text used to be the literal `>Confirmed<`. The new
    // path is `>{t('mic_mute.confirmed')}<`. The grep checks for the
    // old shape directly so a future copy-paste regression fails.
    expect(BANNER).not.toMatch(/>\s*Confirmed\s*</);
    expect(BANNER).toMatch(/mic_mute\.confirmed/);
  });
});
