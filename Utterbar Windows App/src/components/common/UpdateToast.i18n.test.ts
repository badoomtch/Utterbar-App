import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'UpdateToast.tsx'), 'utf8');

// Strip line + block comments before grepping so the existing prose
// commentary inside UpdateToast.tsx doesn't false-trigger on words like
// "Install" or "Dismiss" that appear inside the architectural comments.
function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
}

const SOURCE = stripComments(RAW);

describe('UpdateToast i18n source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    expect(RAW).toMatch(/from ['"]react-i18next['"]/);
    expect(RAW).toMatch(/useTranslation\(['"]common['"]\)/);
  });

  it('does not embed pinned old toast literals as plain prop strings or JSX text', () => {
    // Narrow allow-list. Each entry is the literal *as it appeared
    // pre-localisation*: a JSX text node, a string passed as a prop, or
    // a return value from `extractErrorMessage`. The grep does not trip
    // on the same word appearing inside a t() key, a comment, or
    // unrelated source.
    const forbidden = [
      '>Update available<',
      "'Install & restart'",
      "'Installing…'",
      ">Later<",
      "'Update failed. Try again in a moment.'",
      ">You're on the latest version.<",
      ">See what's new →<",
      'aria-label="Dismiss"',
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `UpdateToast.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('renders the post-update title via t() with version interpolation', () => {
    // The previous shape was the template `Updated to v${justUpdatedTo}`
    // rendered as JSX text. The new path resolves
    // `chrome.update_toast.post_title` with `{ version }`.
    expect(SOURCE).not.toMatch(/Updated to v\$\{justUpdatedTo\}/);
    expect(SOURCE).toMatch(/chrome\.update_toast\.post_title/);
    expect(SOURCE).toMatch(/version: justUpdatedTo/);
  });

  it('still renders firstNotesLine(info.notes) verbatim (deferred backend prose)', () => {
    // `UpdateInfo.notes` is Rust-emitted release-note prose served by
    // the admin updater feed. Backend prose stays verbatim until the
    // future error/event-codes branch lands. This test pins the
    // deferral so the comment + verbatim render can't silently
    // disappear.
    expect(SOURCE).toMatch(/firstNotesLine\(info\??\.notes\)/);
  });

  it('preserves the OrliqError.message passthrough in extractErrorMessage', () => {
    // The Rust-side OrliqError shape `{ code, message }` flows through
    // verbatim — only the frontend "Update failed" fallback is
    // localised. Pin both halves so a future regression that translates
    // `anyE.message` (or drops the OrliqError shape detection) fails.
    expect(SOURCE).toMatch(/return anyE\.message;/);
    expect(SOURCE).toMatch(/chrome\.update_toast\.error_generic/);
  });
});
