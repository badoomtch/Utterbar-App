import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'AccessibilitySettings.tsx'), 'utf8');

// Strip line + block comments before grepping so any prose commentary
// inside AccessibilitySettings.tsx that legitimately mentions words
// like "Motion" or "Reduce" doesn't false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('AccessibilitySettings i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]settings['"]\)/);
  });

  it('does not embed pinned old card / row literal props', () => {
    // Narrow allow-list. Each entry is the literal *as a JSX prop or
    // attribute value* (with the prop name and quotes), so the grep
    // does not trip on the same word appearing inside a t() key, a
    // comment, or unrelated source.
    const forbidden = [
      'title="Accessibility"',
      'subtitle="Make Utterbar\'s main window more comfortable to read, hear, and move through."',
      'title="UI scale"',
      'title="Motion"',
      'title="Contrast and focus"',
      'title="Transparency"',
      'label="Scale"',
      'label="Fine tune"',
      'label="Reduce motion"',
      'label="High contrast"',
      'label="Larger focus rings"',
      'label="Reduce transparency"',
      // Three representative description openings — pin migrating descriptions
      // alongside labels so a future regression that converts only labels but
      // forgets descriptions fails immediately.
      'description="Pick a preset or drag the slider',
      'description="Minimise animations across Utterbar',
      'description="Push dim greys toward full white',
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `AccessibilitySettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the page header and card titles through accessibility.* keys', () => {
    expect(SOURCE).toMatch(/accessibility\.title/);
    expect(SOURCE).toMatch(/accessibility\.subtitle/);
    expect(SOURCE).toMatch(/accessibility\.ui_scale\.title/);
    expect(SOURCE).toMatch(/accessibility\.motion\.title/);
    expect(SOURCE).toMatch(/accessibility\.contrast\.title/);
    expect(SOURCE).toMatch(/accessibility\.transparency\.title/);
  });

  it('routes representative row labels and descriptions through accessibility.* keys', () => {
    expect(SOURCE).toMatch(/accessibility\.ui_scale\.scale\.label/);
    expect(SOURCE).toMatch(/accessibility\.ui_scale\.scale\.description/);
    expect(SOURCE).toMatch(/accessibility\.ui_scale\.fine_tune\.label/);
    expect(SOURCE).toMatch(/accessibility\.motion\.reduce_motion\.label/);
    expect(SOURCE).toMatch(/accessibility\.contrast\.high_contrast\.label/);
    expect(SOURCE).toMatch(/accessibility\.contrast\.larger_focus_rings\.label/);
    expect(SOURCE).toMatch(/accessibility\.transparency\.reduce_transparency\.label/);
  });

  it('routes the percentage readout through the value template', () => {
    // Both the chip presets (`{pct}%`) and the slider readout (`{uiScale}%`)
    // used to render a bare `${value}%` template literal. New path resolves
    // through `t('accessibility.ui_scale.value', { value })` so a future
    // locale can rephrase the format.
    expect(SOURCE).toMatch(/accessibility\.ui_scale\.value/);
    // Old shapes — the chip preset form `{pct}%<` and the readout
    // `{uiScale}%<` — must be gone. Pin both.
    expect(SOURCE).not.toMatch(/\{pct\}%<\/button>/);
    expect(SOURCE).not.toMatch(/\{uiScale\}%</);
  });

  it('preserves the underlying settings field names that drive the patch wiring', () => {
    // The structural settings field shape is unchanged — the localisation
    // only swaps display labels. Each control still reads `g.<field>` /
    // `Boolean(g.<field>)` and writes via `update('<field>', v)` against
    // the `general.<field>` slot on Settings. Pin every field name so a
    // future refactor that renames a settings field without updating the
    // patch wiring fails immediately.
    const fields = [
      'reduce_motion',
      'ui_scale',
      'high_contrast',
      'larger_focus_rings',
      'reduce_transparency',
    ];
    for (const field of fields) {
      expect(
        SOURCE.includes(field),
        `AccessibilitySettings.tsx must keep the settings field name '${field}' intact`,
      ).toBe(true);
    }
  });
});
