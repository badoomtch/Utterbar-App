import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'MainShell.tsx'), 'utf8');

// Strip line + block comments before grepping so the existing prose
// commentary doesn't false-trigger on words like "Workspace" or
// "Search settings" that appear inside the architectural comments.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('MainShell i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]settings['"]\)/);
  });

  it('consumes the buildSettingsNav helper', () => {
    // The static NAV_SETTINGS const has been moved to settings_nav.ts.
    // If anyone reintroduces the inline definition, this fails.
    expect(SOURCE).toMatch(/buildSettingsNav/);
    expect(SOURCE).not.toMatch(/const NAV_SETTINGS/);
  });

  it('does not embed the pinned old NavGroup / search literals as plain prop strings', () => {
    // Narrow allow-list. Each entry is the literal *as a JSX prop or
    // attribute value* (with the prop name and quotes), so the grep
    // does not trip on the same word appearing inside a t() key, a
    // comment, or NAV_PRIMARY (Workspace nav, intentionally still
    // hardcoded in this branch).
    const forbidden = [
      'label="Workspace"',
      'label="Settings"',
      'placeholder="Search settings"',
      'aria-label="Clear search"',
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `MainShell.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the no-match search hint through t(...)', () => {
    // The previous shape was the template `\`No settings match
    // "${settingsQuery.trim()}"\``. The new path resolves
    // `nav.search.no_match` via `t(..., { query })`. Pin both halves
    // so a future regression fails.
    expect(SOURCE).not.toMatch(/`No settings match "/);
    expect(SOURCE).toMatch(/nav\.search\.no_match/);
  });
});
