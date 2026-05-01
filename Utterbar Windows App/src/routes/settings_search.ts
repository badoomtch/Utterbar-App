/**
 * Pure filter function for the Settings search input. Lives in its own
 * module so MainShell.tsx remains a components-only file (Vite fast
 * refresh requires component files to export only components, and the
 * shell would otherwise mix `MainShell` + helper exports).
 *
 * Empty / whitespace-only query returns the full list unchanged so the
 * empty-search state is identical to the no-search state. Otherwise we
 * match case-insensitively against the visible label and the curated
 * keyword list. Each token in a multi-word query must hit somewhere —
 * "system audio" finds Audio because "system" hits "system audio" in
 * the keywords AND "audio" hits the label.
 */
export function filterSettingsByQuery<
  T extends { label: string; keywords: readonly string[] },
>(items: readonly T[], query: string): T[] {
  const trimmed = query.trim().toLowerCase();
  if (trimmed.length === 0) return items.slice();
  const tokens = trimmed.split(/\s+/).filter((t) => t.length > 0);
  if (tokens.length === 0) return items.slice();
  return items.filter((item) => {
    const haystack = [
      item.label.toLowerCase(),
      ...item.keywords.map((k) => k.toLowerCase()),
    ].join(' \n ');
    return tokens.every((tok) => haystack.includes(tok));
  });
}
