import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'ManagedHint.tsx'), 'utf8');

function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('ManagedHint i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]common['"]\)/);
  });

  it('does not embed the default fallback literal directly', () => {
    // The previous shape was the inline literal
    // `children ?? 'Disabled by your administrator.'`. The new path
    // resolves the fallback via `t('chrome.managed_hint.default')` so
    // the only remaining occurrence of that literal is inside the
    // bundled `common.json`, not the consuming TSX.
    expect(SOURCE).not.toMatch(/'Disabled by your administrator\.'/);
    expect(SOURCE).toMatch(/chrome\.managed_hint\.default/);
  });

  it('still accepts caller-provided children as the override', () => {
    // The `children ?? <fallback>` pattern is the contract that lets
    // callers pass a fully custom message. Pin both halves so a future
    // refactor that drops the children parameter fails.
    expect(SOURCE).toMatch(/children\s*\?\?/);
  });
});
