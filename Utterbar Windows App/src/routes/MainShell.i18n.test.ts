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
  it('uses useTranslation from react-i18next for both settings and common namespaces', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]settings['"]\)/);
    // The chrome branch added a second hook for the workspace nav,
    // footer strap, and TierLine copy.
    expect(RAW).toMatch(/useTranslation\(['"]common['"]\)/);
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
    // comment, or other surfaces.
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

  it('routes NAV_PRIMARY labels through chrome.nav.* keys', () => {
    // The previous shape was inline `label: 'Dashboard'` etc. on each
    // NAV_PRIMARY entry. The new path stores `labelKey` and resolves at
    // render time. Pin both halves so a regression that re-inlines the
    // labels fails immediately.
    const forbiddenInlineLabels = [
      "label: 'Dashboard'",
      "label: 'History'",
      "label: 'Meetings'",
      "label: 'Modes'",
      "label: 'Vocabulary'",
      "label: 'Files'",
    ];
    for (const literal of forbiddenInlineLabels) {
      expect(
        SOURCE.includes(literal),
        `MainShell.tsx must not contain the bare literal ${literal}; route it through chrome.nav.*`,
      ).toBe(false);
    }
    expect(SOURCE).toMatch(/chrome\.nav\.dashboard/);
    expect(SOURCE).toMatch(/chrome\.nav\.files/);
  });

  it('routes the footer strap through chrome.footer.strap', () => {
    // The previous shape was the literal `· local only` directly inside
    // a JSX template. The new path resolves the strap word via t().
    expect(SOURCE).not.toMatch(/`v\$\{[^}]+\}` : 'v…'\} · local only/);
    expect(SOURCE).toMatch(/chrome\.footer\.strap/);
  });

  it('routes TierLine labels through chrome.tier.* keys', () => {
    // The previous shape rendered `Pro` and `Upgrade to Pro →` as bare
    // JSX text nodes. New path resolves both via t().
    expect(SOURCE).not.toMatch(/>\s*Upgrade to Pro\s+→\s*</);
    expect(SOURCE).toMatch(/chrome\.tier\.pro/);
    expect(SOURCE).toMatch(/chrome\.tier\.free_cta/);
  });
});
