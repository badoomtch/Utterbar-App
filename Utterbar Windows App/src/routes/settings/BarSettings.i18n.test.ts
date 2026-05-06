import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'BarSettings.tsx'), 'utf8');

// Strip line + block comments before grepping so any prose commentary
// inside BarSettings.tsx that legitimately mentions words like
// "Visibility" or "Reset" doesn't false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('BarSettings i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]settings['"]\)/);
  });

  it('does not embed pinned old card / row / option literal props or strings', () => {
    // Narrow allow-list. JSX prop attribute values + the inline option
    // labels that previously lived in the VISIBILITY_OPTIONS array.
    const forbidden = [
      'title="Utterbar"',
      'subtitle="The floating pill that lives on top of your desktop. Tune what it looks like and when it shows up."',
      'title="Visibility"',
      'title="Appearance"',
      'title="Position"',
      'label="Size"',
      'label="Accent outline"',
      'label="Reset position"',
      // Old VISIBILITY_OPTIONS entries — pre-localisation each was an
      // inline `label: '...'` / `desc: '...'` pair on a const array.
      "label: 'Always visible'",
      "label: 'Only while recording'",
      "label: 'Never show'",
      // Three representative description openings — pin migrating
      // descriptions alongside labels so a future regression that
      // converts only labels but forgets descriptions fails immediately.
      "desc: 'The Utterbar sits at the top of your screen",
      'description="Compact shrinks the pill footprint',
      'description="Snap the Utterbar back to the top-centre',
      // Reset button JSX-text shape (the whitespace-padded literal that
      // used to live as `>Reset position<` after the icon).
      '>Reset position<',
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `BarSettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the page header and card titles through bar_settings.* keys', () => {
    expect(SOURCE).toMatch(/bar_settings\.title/);
    expect(SOURCE).toMatch(/bar_settings\.subtitle/);
    expect(SOURCE).toMatch(/bar_settings\.visibility\.title/);
    expect(SOURCE).toMatch(/bar_settings\.appearance\.title/);
    expect(SOURCE).toMatch(/bar_settings\.position\.title/);
  });

  it('routes representative option / row keys through bar_settings.* keys', () => {
    // Visibility tile labels resolved per-id at render time.
    expect(SOURCE).toMatch(/bar_settings\.visibility\.\$\{[a-zA-Z._]*?\}\.label/);
    expect(SOURCE).toMatch(/bar_settings\.visibility\.\$\{[a-zA-Z._]*?\}\.description/);
    // Size chip labels resolved per-id from `appearance.size.options.<id>`.
    expect(SOURCE).toMatch(/bar_settings\.appearance\.size\.options\.\$\{[a-zA-Z._]*?\}/);
    expect(SOURCE).toMatch(/bar_settings\.appearance\.size\.label/);
    expect(SOURCE).toMatch(/bar_settings\.appearance\.outline\.label/);
    expect(SOURCE).toMatch(/bar_settings\.position\.reset\.label/);
    expect(SOURCE).toMatch(/bar_settings\.position\.reset\.button/);
  });

  it('preserves the underlying settings field names and reset command', () => {
    // The structural settings field shape is unchanged — the localisation
    // only swaps display labels. Each control still reads `g.<field>` /
    // writes via `update('<field>', v)` against the `general.<field>`
    // slot on Settings, and the Position card still fires
    // `invoke('reset_floating_position')`. Pin every load-bearing
    // identifier so a future regression that renames a settings field or
    // changes the reset command fails immediately.
    const fields = ['bar_visibility', 'bar_size', 'bar_outline'];
    for (const field of fields) {
      expect(
        SOURCE.includes(field),
        `BarSettings.tsx must keep the settings field name '${field}' intact`,
      ).toBe(true);
    }
    expect(SOURCE).toMatch(/invoke\(['"]reset_floating_position['"]\)/);
  });
});
