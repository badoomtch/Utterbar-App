import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'ProNudge.tsx'), 'utf8');

function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('ProNudge i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]common['"]\)/);
  });

  it('does not embed pinned old framing or learn-more literals', () => {
    const forbidden = [
      // Old shape: `{feature} is a <span className="text-accent">Pro</span> feature.`
      'is a <span className="text-accent">Pro</span> feature.',
      '>Learn more →<',
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `ProNudge.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('preserves the accent-coloured Pro span via prefix/pro/suffix split', () => {
    expect(SOURCE).toMatch(/text-accent/);
    expect(SOURCE).toMatch(/chrome\.pro_nudge\.pro_word/);
    expect(SOURCE).toMatch(/chrome\.pro_nudge\.feature_prefix/);
    expect(SOURCE).toMatch(/chrome\.pro_nudge\.feature_suffix/);
    expect(SOURCE).toMatch(/chrome\.pro_nudge\.learn_more/);
  });
});
