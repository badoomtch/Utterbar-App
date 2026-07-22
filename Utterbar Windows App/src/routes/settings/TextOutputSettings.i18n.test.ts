import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'TextOutputSettings.tsx'), 'utf8');

// Strip line + block comments before grepping so any prose commentary
// inside TextOutputSettings.tsx that legitimately mentions words like
// "Delivery" or "Formatting" doesn't false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('TextOutputSettings i18n source-grep guard', () => {
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
      'title="Text Output"',
      'subtitle="How dictated text reaches your active app."',
      'title="Delivery"',
      'title="Formatting"',
      'label="Paste automatically"',
      'label="Restore previous clipboard"',
      'label="Type it out if paste is blocked"',
      'label="Auto-capitalise first letter"',
      'label="Trim trailing whitespace"',
      'label="Auto-space between dictations"',
      'label="Numbers as digits"',
      'label="Smart rewrite"',
      'label="Hold Shift to auto-send"',
      // Three representative description openings — pin migrating descriptions
      // alongside labels so a future regression that converts only labels but
      // forgets descriptions fails immediately.
      'description="After transcription,',
      'description="Make sure the inserted text starts',
      'description="If you\'re holding Shift',
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `TextOutputSettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the page header and card titles through text_output.* keys', () => {
    expect(SOURCE).toMatch(/text_output\.title/);
    expect(SOURCE).toMatch(/text_output\.subtitle/);
    expect(SOURCE).toMatch(/text_output\.delivery\.title/);
    expect(SOURCE).toMatch(/text_output\.formatting\.title/);
  });

  it('routes a representative row label and description through text_output.* keys', () => {
    expect(SOURCE).toMatch(/text_output\.delivery\.paste_automatically\.label/);
    expect(SOURCE).toMatch(/text_output\.delivery\.paste_automatically\.description/);
    expect(SOURCE).toMatch(/text_output\.delivery\.clipboard_restore_delay\.label/);
    expect(SOURCE).toMatch(/text_output\.delivery\.clipboard_restore_delay\.description/);
    expect(SOURCE).toMatch(/text_output\.delivery\.clipboard_restore_delay\.value/);
    expect(SOURCE).toMatch(/text_output\.formatting\.smart_rewrite\.label/);
    expect(SOURCE).toMatch(/text_output\.formatting\.smart_rewrite\.description/);
    expect(SOURCE).toMatch(/text_output\.formatting\.shift_to_auto_send\.label/);
  });

  it('preserves the underlying settings field names that drive the patch wiring', () => {
    // The structural settings field shape is unchanged — the localisation
    // only swaps display labels. Each toggle still reads `<obj>.<field>`
    // and writes via `update('<field>', v)` against the
    // `text_output.<field>` slot on Settings. Pin every field name so a
    // future refactor that renames a settings field without updating the
    // patch wiring fails immediately.
    const fields = [
      'paste_automatically',
      'restore_clipboard',
      'clipboard_restore_delay_ms',
      'keystroke_fallback',
      'auto_capitalise',
      'trim_trailing_space',
      'auto_space_on_paste',
      'numbers_as_digits',
      'smart_rewrite',
      'shift_to_auto_send',
    ];
    for (const field of fields) {
      expect(
        SOURCE.includes(field),
        `TextOutputSettings.tsx must keep the settings field name '${field}' intact`,
      ).toBe(true);
    }
  });

  it('keeps the clipboard restore delay slider within the supported range', () => {
    expect(SOURCE).toMatch(/type="range"/);
    expect(SOURCE).toMatch(/min=\{250\}/);
    expect(SOURCE).toMatch(/max=\{1500\}/);
    expect(SOURCE).toMatch(/step=\{250\}/);
    expect(SOURCE).toMatch(/disabled=\{!to\.restore_clipboard\}/);
  });
});
