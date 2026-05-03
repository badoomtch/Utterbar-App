import { describe, expect, it } from 'vitest';

import {
  filterSettingsByQuery,
  settingsSearchMatchSummary,
  type SettingsSearchItem,
} from './settings_search';

// Mirror of MainShell's NAV_SETTINGS, kept narrow on purpose. We don't
// want this test to track every keyword change — only the contract that
// search v2 promises: deep matches against `matches` (label + aliases),
// multi-token AND, summary returns the user-facing control name.
const FIXTURES: SettingsSearchItem[] = [
  {
    label: 'General',
    keywords: ['theme'],
    matches: [
      'Theme',
      { label: 'Launch at startup', aliases: ['launch', 'startup', 'autostart'] },
    ],
  },
  {
    label: 'Text Output',
    keywords: ['punctuation'],
    matches: [
      { label: 'Paste automatically', aliases: ['paste'] },
      { label: 'Auto-capitalise', aliases: ['capitalise', 'capitalize'] },
      {
        label: 'Smart rewrite',
        aliases: ['rewrite', 'filler', 'filler words', 'um', 'uh', 'code dictation', 'code'],
      },
      { label: 'Numbers as digits', aliases: ['numbers', 'digits'] },
    ],
  },
  {
    label: 'Privacy',
    keywords: ['recordings'],
    matches: [
      {
        label: 'Mute conferencing app output',
        aliases: ['mute', 'teams', 'discord', 'zoom', 'slack', 'conferencing'],
      },
      { label: 'Apps to silence', aliases: ['apps to silence', 'silence list'] },
    ],
  },
  {
    label: 'Licence',
    keywords: ['licence'],
    matches: [
      { label: 'Activate Pro', aliases: ['activate', 'activation', 'pro'] },
      { label: 'Air-gapped activation', aliases: ['air-gapped', 'air gapped', 'airgapped', 'offline'] },
    ],
  },
  {
    label: 'Advanced',
    keywords: ['experimental'],
    matches: [
      { label: 'GPU acceleration', aliases: ['gpu', 'acceleration', 'directml', 'cuda'] },
      { label: 'DirectML', aliases: ['directml', 'dml'] },
      { label: 'CPU only', aliases: ['cpu'] },
    ],
  },
  {
    label: 'Shortcuts',
    keywords: ['hotkey', 'global shortcut'],
    matches: [
      { label: 'Toggle recording hotkey', aliases: ['toggle', 'record'] },
    ],
  },
];

const labelsOf = (xs: SettingsSearchItem[]) => xs.map((x) => x.label);

describe('filterSettingsByQuery', () => {
  it('empty query returns all items', () => {
    expect(filterSettingsByQuery(FIXTURES, '')).toEqual(FIXTURES);
    expect(filterSettingsByQuery(FIXTURES, '   ')).toEqual(FIXTURES);
  });

  it('multi-token query requires every token to match somewhere', () => {
    expect(labelsOf(filterSettingsByQuery(FIXTURES, 'smart rewrite'))).toEqual([
      'Text Output',
    ]);
  });

  it('finds Text Output via "smart rewrite"', () => {
    expect(labelsOf(filterSettingsByQuery(FIXTURES, 'smart rewrite'))).toContain(
      'Text Output',
    );
  });

  it('finds Text Output via "filler words" (alias match)', () => {
    expect(labelsOf(filterSettingsByQuery(FIXTURES, 'filler words'))).toEqual([
      'Text Output',
    ]);
  });

  it('finds Privacy via "teams mute"', () => {
    expect(labelsOf(filterSettingsByQuery(FIXTURES, 'teams mute'))).toEqual([
      'Privacy',
    ]);
  });

  it('finds Privacy via "apps to silence"', () => {
    expect(labelsOf(filterSettingsByQuery(FIXTURES, 'apps to silence'))).toEqual([
      'Privacy',
    ]);
  });

  it('finds Licence via "air gapped"', () => {
    expect(labelsOf(filterSettingsByQuery(FIXTURES, 'air gapped'))).toEqual([
      'Licence',
    ]);
  });

  it('finds Advanced via "gpu"', () => {
    expect(labelsOf(filterSettingsByQuery(FIXTURES, 'gpu'))).toEqual(['Advanced']);
  });

  it('finds General via "launch startup"', () => {
    expect(labelsOf(filterSettingsByQuery(FIXTURES, 'launch startup'))).toEqual([
      'General',
    ]);
  });

  it('unknown query returns no items', () => {
    expect(filterSettingsByQuery(FIXTURES, 'banana republic')).toEqual([]);
  });

  it('is case-insensitive', () => {
    expect(labelsOf(filterSettingsByQuery(FIXTURES, 'SMART REWRITE'))).toEqual([
      'Text Output',
    ]);
  });
});

describe('settingsSearchMatchSummary', () => {
  const textOutput = FIXTURES.find((x) => x.label === 'Text Output')!;
  const privacy = FIXTURES.find((x) => x.label === 'Privacy')!;
  const licence = FIXTURES.find((x) => x.label === 'Licence')!;
  const general = FIXTURES.find((x) => x.label === 'General')!;
  const advanced = FIXTURES.find((x) => x.label === 'Advanced')!;
  const shortcuts = FIXTURES.find((x) => x.label === 'Shortcuts')!;

  it('returns undefined for empty query', () => {
    expect(settingsSearchMatchSummary(textOutput, '')).toBeUndefined();
    expect(settingsSearchMatchSummary(textOutput, '   ')).toBeUndefined();
  });

  it('"smart rewrite" → Smart rewrite (literal label)', () => {
    expect(settingsSearchMatchSummary(textOutput, 'smart rewrite')).toBe(
      'Smart rewrite',
    );
  });

  it('"filler words" → Smart rewrite (via alias)', () => {
    expect(settingsSearchMatchSummary(textOutput, 'filler words')).toBe(
      'Smart rewrite',
    );
  });

  it('"code dictation" → Smart rewrite (via alias)', () => {
    expect(settingsSearchMatchSummary(textOutput, 'code dictation')).toBe(
      'Smart rewrite',
    );
  });

  it('"teams mute" → Mute conferencing app output (via aliases)', () => {
    expect(settingsSearchMatchSummary(privacy, 'teams mute')).toBe(
      'Mute conferencing app output',
    );
  });

  it('"apps to silence" → Apps to silence (literal label)', () => {
    expect(settingsSearchMatchSummary(privacy, 'apps to silence')).toBe(
      'Apps to silence',
    );
  });

  it('"air gapped" → Air-gapped activation', () => {
    expect(settingsSearchMatchSummary(licence, 'air gapped')).toBe(
      'Air-gapped activation',
    );
  });

  it('"launch startup" → Launch at startup', () => {
    expect(settingsSearchMatchSummary(general, 'launch startup')).toBe(
      'Launch at startup',
    );
  });

  it('"gpu" → GPU acceleration', () => {
    // "gpu" is in two match labels' aliases (GPU acceleration AND
    // DirectML's "directml" alias doesn't help here, but DirectML's
    // own aliases include neither "gpu" nor "directml" alone for this
    // query). GPU acceleration is the first hit.
    expect(settingsSearchMatchSummary(advanced, 'gpu')).toBe('GPU acceleration');
  });

  it('falls back to a keyword when no match label hits', () => {
    // "global shortcut" only appears in Shortcuts.keywords. Neither
    // "global" nor "shortcut" appears in the single match label
    // "Toggle recording hotkey" or its aliases. So summary falls back
    // to the matching keyword.
    expect(settingsSearchMatchSummary(shortcuts, 'global shortcut')).toBe(
      'global shortcut',
    );
  });

  it('returns undefined when nothing hits', () => {
    expect(settingsSearchMatchSummary(textOutput, 'banana')).toBeUndefined();
  });

  it('returns at most two match labels', () => {
    const item: SettingsSearchItem = {
      label: 'Synthetic',
      keywords: [],
      matches: [
        { label: 'One alpha', aliases: ['one'] },
        { label: 'One beta', aliases: ['one'] },
        { label: 'One gamma', aliases: ['one'] },
        { label: 'One delta', aliases: ['one'] },
      ],
    };
    const summary = settingsSearchMatchSummary(item, 'one');
    expect(summary).toBeDefined();
    expect(summary!.split(', ').length).toBe(2);
  });

  it('joins multiple hits with ", "', () => {
    const item: SettingsSearchItem = {
      label: 'Synthetic',
      keywords: [],
      matches: [
        { label: 'One alpha', aliases: ['one'] },
        { label: 'One beta', aliases: ['one'] },
      ],
    };
    expect(settingsSearchMatchSummary(item, 'one')).toBe('One alpha, One beta');
  });

  it('plain string match (no aliases) still works', () => {
    // The General fixture has the bare string 'Theme' as a match label.
    expect(settingsSearchMatchSummary(general, 'theme')).toBe('Theme');
  });
});
