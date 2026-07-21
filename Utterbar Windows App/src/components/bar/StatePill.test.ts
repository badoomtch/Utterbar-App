import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const RAW = readFileSync(resolve(HERE, 'StatePill.tsx'), 'utf8');

// Strip line comments + block comments before grepping. Comments
// historically explain the *meaning* of the old phase labels (e.g. "the
// 'Ready' bug on Paper") and must not trigger the guard. The regex is
// intentionally simple — it does not handle commented-out comments
// inside strings, but TS source rarely contains them and the test
// catches the overwhelming-common case.
const SOURCE = RAW.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');

describe('StatePill source-grep guard', () => {
  it('uses useTranslation from react-i18next', () => {
    // If this import vanishes, the file is back to hardcoded English. The
    // guard intentionally checks the import line, not a t( call, because
    // a future refactor that aliases t() through a wrapper should still
    // pass as long as the i18n hook is the underlying source.
    expect(SOURCE).toMatch(/from ['"]react-i18next['"]/);
    expect(SOURCE).toMatch(/useTranslation\(['"]bar['"]\)/);
  });

  it('does not embed the pinned old phase labels as plain string literals', () => {
    // Narrow allow-list of forbidden literals: only the exact phrases that
    // used to live inside the `describe()` switch in StatePill.tsx. Each
    // entry is a quoted string literal with the surrounding quote marks,
    // so the test does not false-positive on the same word appearing in
    // a comment or in this test file's commentary.
    //
    // Keep this list tight. Anything outside the StatePill `describe()`
    // is out of scope for this guard.
    const forbidden = [
      "'Ready'",
      "'Initialising model'",
      "'Recording'",
      "'Processing'",
      "'Transcribing meeting…'",
      "'Inserted'",
      "'Auto-stopped at 5 min'",
      "'Error'",
      "'Cancelled'",
      "'No audio detected'",
    ];
    for (const literal of forbidden) {
      expect(
        SOURCE.includes(literal),
        `StatePill.tsx must not contain the bare literal ${literal}; route it through t(...)`,
      ).toBe(false);
    }
  });

  it('routes the meeting-progress phrase through a translation key', () => {
    // The previous template literal `Transcribing meeting · ${current}/...`
    // is gone; the new path uses `t('phase.transcribing_meeting_with_progress', ...)`.
    expect(SOURCE).not.toMatch(/`Transcribing meeting · \$\{/);
    expect(SOURCE).toMatch(/phase\.transcribing_meeting_with_progress/);
  });

  it('surfaces a backend delivery error instead of replacing it with generic copy', () => {
    expect(SOURCE).toMatch(/case 'error':[\s\S]*phase\.message \|\| t\('phase\.error'\)/);
  });
});
