/**
 * Pure filter + summary functions for the Settings search input. Lives in
 * its own module so MainShell.tsx remains a components-only file (Vite
 * fast refresh requires component files to export only components, and
 * the shell would otherwise mix `MainShell` + helper exports).
 *
 * Empty / whitespace-only query returns the full list unchanged so the
 * empty-search state is identical to the no-search state. Otherwise we
 * match case-insensitively against the visible section label, the
 * curated keyword list, and the per-setting `matches` (control labels
 * + their alias terms). Each token in a multi-word query must hit
 * somewhere in the haystack — "system audio" finds Audio because
 * "system" hits the "system audio" keyword AND "audio" hits the label.
 *
 * `matches` carries the actual user-facing control names ("Smart
 * rewrite", "Apps to silence", "Air-gapped activation") plus an
 * optional list of `aliases` — alternate phrasings users type when
 * they're describing the control without using its label ("filler
 * words" → Smart rewrite). The summary helper picks one or two
 * matched control labels as a short hint under the nav row.
 */

export type SettingsSearchMatch =
  | string
  | { label: string; aliases?: readonly string[] };

export type SettingsSearchItem = {
  label: string;
  keywords: readonly string[];
  matches?: readonly SettingsSearchMatch[];
};

function tokenise(query: string): string[] {
  const trimmed = query.trim().toLowerCase();
  if (trimmed.length === 0) return [];
  return trimmed.split(/\s+/).filter((t) => t.length > 0);
}

function matchLabel(m: SettingsSearchMatch): string {
  return typeof m === 'string' ? m : m.label;
}

function matchAliases(m: SettingsSearchMatch): readonly string[] {
  return typeof m === 'string' ? [] : (m.aliases ?? []);
}

function haystackFor(item: SettingsSearchItem): string {
  const parts: string[] = [item.label.toLowerCase()];
  for (const k of item.keywords) parts.push(k.toLowerCase());
  for (const m of item.matches ?? []) {
    parts.push(matchLabel(m).toLowerCase());
    for (const a of matchAliases(m)) parts.push(a.toLowerCase());
  }
  return parts.join(' \n ');
}

export function filterSettingsByQuery<T extends SettingsSearchItem>(
  items: readonly T[],
  query: string,
): T[] {
  const tokens = tokenise(query);
  if (tokens.length === 0) return items.slice();
  return items.filter((item) => {
    const haystack = haystackFor(item);
    return tokens.every((tok) => haystack.includes(tok));
  });
}

const MAX_SUMMARY_HITS = 2;

/**
 * Pick up to two matched-control labels from `item.matches` that the
 * query touches, then format as a short hint string for the nav row's
 * subLabel slot.
 *
 * A match label is considered a hit if every query token appears
 * somewhere in (label + aliases) for that specific match. We do NOT
 * count section-level keywords here — those got the user to the
 * section in the first place but they don't tell us *which* control
 * the user is being pointed at.
 *
 * Behaviour:
 *   - empty/whitespace query                    → undefined
 *   - one or more matching labels (or aliases)  → up to two, joined by ", "
 *   - no match-label hit, but a keyword hits    → first matching keyword
 *   - nothing hits                              → undefined
 */
export function settingsSearchMatchSummary(
  item: SettingsSearchItem,
  query: string,
): string | undefined {
  const tokens = tokenise(query);
  if (tokens.length === 0) return undefined;

  // Pass 1: per-match-label haystack. The candidate hits if every
  // query token appears in (label + that label's own aliases). The
  // section's other keywords do NOT count here, so we don't return
  // every match label whenever a section-level keyword satisfies all
  // tokens.
  const hits: string[] = [];
  for (const m of item.matches ?? []) {
    const label = matchLabel(m);
    const candidateHaystack = [label.toLowerCase(), ...matchAliases(m).map((a) => a.toLowerCase())].join(' \n ');
    if (tokens.every((tok) => candidateHaystack.includes(tok))) {
      hits.push(label);
      if (hits.length >= MAX_SUMMARY_HITS) break;
    }
  }
  if (hits.length > 0) return hits.join(', ');

  // Pass 2: no match label hit, fall back to the first keyword that
  // contains at least one token. This is the "synonym" fallback — the
  // user typed "ASR" and we surface "ASR" because there's no specific
  // control labelled that.
  for (const k of item.keywords) {
    const kLc = k.toLowerCase();
    if (tokens.some((tok) => kLc.includes(tok))) {
      return k;
    }
  }

  return undefined;
}
