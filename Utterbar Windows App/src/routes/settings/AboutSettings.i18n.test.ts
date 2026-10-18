import { describe, expect, it } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'AboutSettings.tsx'), 'utf8');

// Strip line + block comments before grepping so any prose commentary
// inside AboutSettings.tsx that legitimately mentions the old footer
// wording (it deliberately documents the website-attribution contract)
// doesn't false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('AboutSettings i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]settings['"]\)/);
  });

  it('does not embed pinned old visible literal props or strings', () => {
    // Narrow allow-list. Each entry is a literal that lived in the
    // pre-localisation source and would re-appear if a future edit
    // bypassed t(). Comments are stripped above so the contract
    // documentation in JSX comments doesn't false-trigger this guard.
    const forbidden = [
      'title="About"',
      // The fallback shown while `getVersion()` is pending — was a bare
      // 'v…' string in the JSX, now resolved through t().
      "'v…'",
      // Tagline opening — unique enough to pin without false-triggering
      // on prose.
      'Local-only dictation for Windows. All audio and text processing',
      // Footer attribution shape: the literal "Built by " text + the
      // "Built in Wales" suffix + the visible "Ddraig Dev" button text +
      // the "utterbar.com" host token.
      'Utterbar · Built by',
      '· Built in Wales',
      '>Ddraig Dev',
      '>utterbar.com<',
      // Third-party card copy.
      '>Third-party licences<',
      'Open-source libraries and models bundled with Utterbar.',
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `AboutSettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the page header and visible card copy through about.* keys', () => {
    expect(SOURCE).toMatch(/about\.title/);
    expect(SOURCE).toMatch(/about\.tagline/);
    expect(SOURCE).toMatch(/about\.version_pending/);
    expect(SOURCE).toMatch(/about\.footer\.copyright_prefix/);
    expect(SOURCE).toMatch(/about\.footer\.attribution_link/);
    expect(SOURCE).toMatch(/about\.footer\.location_suffix/);
    expect(SOURCE).toMatch(/about\.footer\.website_link/);
    expect(SOURCE).toMatch(/about\.third_party\.title/);
    expect(SOURCE).toMatch(/about\.third_party\.description/);
  });

  it('preserves version and licences while linking to GitHub and current support', () => {
    // Product-home changes must not drop the version or required notices.
    expect(SOURCE).toMatch(/getVersion\(\)/);
    expect(SOURCE).toContain('THIRD_PARTY_LICENCES_MD');
    expect(SOURCE).toContain('mailto:jay@jayparry.me');
    expect(SOURCE).toContain('https://github.com/badoomtch/Utterbar-App');
    expect(SOURCE).not.toContain('https://ddraig.dev');
    expect(SOURCE).not.toContain('https://utterbar.com');
    expect(SOURCE).toMatch(/openShell\(['"]mailto:jay@jayparry\.me['"]\)/);
    expect(SOURCE).toMatch(/openShell\(['"]https:\/\/github\.com\/badoomtch\/Utterbar-App['"]\)/);
  });

  it('uses current project names and update host in every locale', () => {
    const root = resolve(HERE, '../../i18n/locales');
    for (const entry of readdirSync(root, { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;
      const settings = JSON.parse(readFileSync(resolve(root, entry.name, 'settings.json'), 'utf8'));
      const onboarding = JSON.parse(
        readFileSync(resolve(root, entry.name, 'onboarding.json'), 'utf8'),
      );
      expect(settings.about.footer.attribution_link, entry.name).toBe('Jay Parry');
      expect(settings.about.footer.website_link, entry.name).toBe('GitHub');
      expect(onboarding.updates.body_domain, entry.name).toBe('GitHub');
    }
  });

  it('passes the year through the copyright_prefix interpolation', () => {
    // The mustache placeholder lives in settings.json; the JSX must
    // pass `{ year: ... }` to t() so the catalogue's `{{year}}` is
    // substituted at render time. Pin the wiring shape.
    expect(SOURCE).toMatch(/about\.footer\.copyright_prefix['"][\s\S]{0,80}year:/);
    expect(SOURCE).toMatch(/new Date\(\)\.getFullYear\(\)/);
  });
});
