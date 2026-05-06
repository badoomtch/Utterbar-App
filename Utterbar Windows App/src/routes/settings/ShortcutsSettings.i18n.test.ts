import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'ShortcutsSettings.tsx'), 'utf8');

// Strip line + block comments before grepping so any prose commentary
// inside ShortcutsSettings.tsx that legitimately mentions words like
// "Shortcuts" or "capture" doesn't false-trigger the guard.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('ShortcutsSettings i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]settings['"]\)/);
  });

  it('does not embed pinned old visible literal props or strings', () => {
    // Narrow allow-list. JSX prop attribute values + the inline labels
    // that previously lived as bare string literals in the JSX tree.
    // Comments are stripped above so the contract documentation in JSX
    // comments doesn't false-trigger this guard.
    const forbidden = [
      'title="Shortcuts"',
      'subtitle="Global keybindings. Press any key combination in any app to trigger."',
      // Pre-localisation conflict tooltip used a backtick template,
      // which we now route through t('shortcuts.conflict.tooltip').
      'title={`Conflicts with another binding on',
      // Capture-button text shape — was a bare 'Press keys…' /
      // 'Unbound' literal.
      "'Press keys…'",
      "'Unbound'",
      // Clear button — was a JSX text node `>Clear<`.
      '>Clear<',
      // Esc tip — was a JSX text node "Tip: press" + " to cancel
      // capture." around the <Kbd>Esc</Kbd>. Pin both halves.
      'Tip: press <Kbd>',
      '> to cancel capture.',
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `ShortcutsSettings.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the page header and visible chrome through shortcuts.* keys', () => {
    expect(SOURCE).toMatch(/shortcuts\.title/);
    expect(SOURCE).toMatch(/shortcuts\.subtitle/);
    expect(SOURCE).toMatch(/shortcuts\.capture\.press_keys/);
    expect(SOURCE).toMatch(/shortcuts\.capture\.unbound/);
    expect(SOURCE).toMatch(/shortcuts\.buttons\.clear/);
    expect(SOURCE).toMatch(/shortcuts\.conflict\.tooltip/);
    expect(SOURCE).toMatch(/shortcuts\.tip\.prefix/);
    expect(SOURCE).toMatch(/shortcuts\.tip\.suffix/);
  });

  it('passes the accelerator through the conflict tooltip interpolation', () => {
    // The mustache placeholder lives in settings.json; the JSX must
    // pass `{ accelerator: ... }` to t() so the catalogue's
    // `{{accelerator}}` is substituted at render time. Pin the wiring
    // shape so a future regression that drops the substitution is
    // caught at build time, not by a confused user.
    expect(SOURCE).toMatch(
      /shortcuts\.conflict\.tooltip['"][\s\S]{0,80}accelerator:\s*conflict\.accelerator/,
    );
  });

  it('keeps backend-owned slot.label and slot.description rendering verbatim', () => {
    // The Rust `list_shortcuts` command emits both slot.label and
    // slot.description. They must continue to flow into <Row label=
    // {slot.label} description={slot.description}> with no t() wrapping
    // so the future error/event-codes branch can move backend prose
    // into a single coherent backend-side catalogue. Pin the JSX shape.
    expect(SOURCE).toMatch(/label=\{slot\.label\}/);
    expect(SOURCE).toMatch(/description=\{slot\.description\}/);
    // And — crucially — neither prop value is a t() call.
    expect(SOURCE).not.toMatch(/label=\{t\(['"]shortcuts[^'"]*slot/);
    expect(SOURCE).not.toMatch(/description=\{t\(['"]shortcuts[^'"]*slot/);
  });

  it('preserves the load-bearing IPC commands and capture state', () => {
    // The structural shortcut binding flow is unchanged — every Tauri
    // command name and every shortcut-id-keyed accelerator path must
    // survive the localisation. Pin each so a future regression that
    // renames a command or breaks the capture machinery fails
    // immediately.
    const commands = [
      'list_shortcuts',
      'bind_shortcut',
      'unbind_shortcut',
      'detect_conflicts',
    ];
    for (const cmd of commands) {
      expect(
        SOURCE.includes(cmd),
        `ShortcutsSettings.tsx must keep the IPC command '${cmd}' intact`,
      ).toBe(true);
    }
    // Capture state machinery: starting capture, the Escape early-exit,
    // the final bind payload shape.
    expect(SOURCE).toMatch(/setCapturing\(/);
    expect(SOURCE).toMatch(/'Escape'/);
    expect(SOURCE).toMatch(/trigger_style/);
  });
});
