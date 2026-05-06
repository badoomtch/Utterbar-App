import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'UpgradeModal.tsx'), 'utf8');

function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('UpgradeModal i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]common['"]\)/);
  });

  it('does not embed pinned old period / button / framing literals', () => {
    // Narrow allow-list. JSX-text literals only — the same words inside
    // t() keys / comments / unrelated source pass through.
    const forbidden = [
      '>one-time<',
      '>per month<',
      '>per year<',
      '>Not now<',
      '>Upgrade<',
      // Old shape of the framing sentence: `{feature} is a <span ...>Pro</span> feature`
      'is a <span className="text-accent">Pro</span> feature',
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `UpgradeModal.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('preserves the accent-coloured Pro span via prefix/pro/suffix split', () => {
    // The accent-coloured `<span className="text-accent">` wrapping the
    // word "Pro" is the visual hook of the framing sentence. The new
    // path keeps the span and resolves the pro word via t() — the
    // surrounding sentence is split into prefix + suffix keys so the
    // locale can rephrase without breaking the span.
    expect(SOURCE).toMatch(/text-accent/);
    expect(SOURCE).toMatch(/chrome\.upgrade_modal\.pro_word/);
    expect(SOURCE).toMatch(/chrome\.upgrade_modal\.feature_prefix/);
    expect(SOURCE).toMatch(/chrome\.upgrade_modal\.feature_suffix/);
  });

  it('routes period suffixes and primary buttons through chrome.upgrade_modal.*', () => {
    expect(SOURCE).toMatch(/chrome\.upgrade_modal\.period\.one_time/);
    expect(SOURCE).toMatch(/chrome\.upgrade_modal\.period\.monthly/);
    expect(SOURCE).toMatch(/chrome\.upgrade_modal\.period\.yearly/);
    expect(SOURCE).toMatch(/chrome\.upgrade_modal\.not_now/);
    expect(SOURCE).toMatch(/chrome\.upgrade_modal\.upgrade/);
  });
});
