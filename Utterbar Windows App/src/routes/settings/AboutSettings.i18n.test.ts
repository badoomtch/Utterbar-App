import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
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

  it('preserves dynamic version, third-party body, and outbound URLs', () => {
    // The localisation only swaps display labels. The structural
    // identifiers below are load-bearing — `getVersion()` feeds the
    // dynamic version readout, `THIRD_PARTY_LICENCES_MD` is the
    // bundled licence body the `<pre>` block renders verbatim, and the
    // two outbound URLs are the footer's only behaviour. Pin every one
    // so a future regression that drops one fails immediately.
    expect(SOURCE).toMatch(/getVersion\(\)/);
    expect(SOURCE).toContain('THIRD_PARTY_LICENCES_MD');
    expect(SOURCE).toContain('https://ddraig.dev');
    expect(SOURCE).toContain('https://utterbar.com');
    // `openShell` is the fire-and-forget mechanism for both URLs and
    // must not silently get swapped for a Rust-IPC `invoke()`.
    expect(SOURCE).toMatch(/openShell\(['"]https:\/\/ddraig\.dev['"]\)/);
    expect(SOURCE).toMatch(/openShell\(['"]https:\/\/utterbar\.com['"]\)/);
  });

  it('passes the year through the copyright_prefix interpolation', () => {
    // The mustache placeholder lives in settings.json; the JSX must
    // pass `{ year: ... }` to t() so the catalogue's `{{year}}` is
    // substituted at render time. Pin the wiring shape.
    expect(SOURCE).toMatch(/about\.footer\.copyright_prefix['"][\s\S]{0,80}year:/);
    expect(SOURCE).toMatch(/new Date\(\)\.getFullYear\(\)/);
  });
});
