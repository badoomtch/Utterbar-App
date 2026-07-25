import { describe, expect, it, beforeAll } from 'vitest';

import i18n from '@/i18n';
import { buildSettingsNav } from './settings_nav';
import { filterSettingsByQuery, settingsSearchMatchSummary } from './settings_search';

beforeAll(async () => {
  await i18n.loadNamespaces(['settings']);
});

// Build the nav once for the test run — the helper depends only on the
// active i18n language, which is `en` for the entire suite.
const NAV = buildSettingsNav(i18n.getFixedT('en', 'settings'));

const labelsOf = (xs: ReadonlyArray<{ label: string }>) => xs.map((x) => x.label);

describe('buildSettingsNav: shape', () => {
  it('returns 11 sections in the canonical IA order', () => {
    expect(labelsOf(NAV)).toEqual([
      'General',
      'Utterbar',
      'Audio',
      'Models',
      'Shortcuts',
      'Text Output',
      'Accessibility',
      'Privacy',
      'Advanced',
      'Updates',
      'About',
    ]);
  });

  it('every section carries a route, icon, keywords, and matches', () => {
    for (const section of NAV) {
      expect(section.to.startsWith('/settings/'), `${section.label} route`).toBe(true);
      expect(section.icon, `${section.label} icon`).toBeTruthy();
      expect(Array.isArray(section.keywords), `${section.label} keywords`).toBe(true);
      expect(Array.isArray(section.matches), `${section.label} matches`).toBe(true);
      expect(section.matches.length, `${section.label} matches non-empty`).toBeGreaterThan(0);
    }
  });
});

// Behaviour parity contract: every English search example listed in
// the brief must still return the same section / hint as the
// hand-rolled NAV_SETTINGS that lived in MainShell.tsx pre-helper.
describe('Settings search behaviour parity (English)', () => {
  it('language → General / App language', () => {
    expect(labelsOf(filterSettingsByQuery(NAV, 'language'))).toContain('General');
    const general = NAV.find((x) => x.label === 'General')!;
    expect(settingsSearchMatchSummary(general, 'language')).toBe('App language');
  });

  it('smart rewrite → Text Output / Smart rewrite', () => {
    expect(labelsOf(filterSettingsByQuery(NAV, 'smart rewrite'))).toEqual(['Text Output']);
    const textOutput = NAV.find((x) => x.label === 'Text Output')!;
    expect(settingsSearchMatchSummary(textOutput, 'smart rewrite')).toBe('Smart rewrite');
  });

  it('filler words → Text Output / Smart rewrite', () => {
    expect(labelsOf(filterSettingsByQuery(NAV, 'filler words'))).toEqual(['Text Output']);
    const textOutput = NAV.find((x) => x.label === 'Text Output')!;
    expect(settingsSearchMatchSummary(textOutput, 'filler words')).toBe('Smart rewrite');
  });

  it('code dictation → Text Output / Smart rewrite', () => {
    expect(labelsOf(filterSettingsByQuery(NAV, 'code dictation'))).toEqual(['Text Output']);
    const textOutput = NAV.find((x) => x.label === 'Text Output')!;
    expect(settingsSearchMatchSummary(textOutput, 'code dictation')).toBe('Smart rewrite');
  });

  it('teams mute → Privacy / Mute conferencing app output', () => {
    // Both Privacy and Advanced carry a "teams mute" alias (Privacy on
    // the conferencing-output mute, Advanced on the experimental
    // microphone mute). The contract from the brief is that "teams
    // mute" finds Privacy / Mute conferencing app output, so this
    // test pins the Privacy hit specifically. Advanced is allowed to
    // also match.
    const hits = labelsOf(filterSettingsByQuery(NAV, 'teams mute'));
    expect(hits).toContain('Privacy');
    const privacy = NAV.find((x) => x.label === 'Privacy')!;
    expect(settingsSearchMatchSummary(privacy, 'teams mute')).toBe(
      'Mute conferencing app output',
    );
  });

  it('apps to silence → Privacy / Apps to silence', () => {
    expect(labelsOf(filterSettingsByQuery(NAV, 'apps to silence'))).toEqual(['Privacy']);
    const privacy = NAV.find((x) => x.label === 'Privacy')!;
    expect(settingsSearchMatchSummary(privacy, 'apps to silence')).toBe('Apps to silence');
  });

  it('gpu → Advanced / GPU acceleration', () => {
    expect(labelsOf(filterSettingsByQuery(NAV, 'gpu'))).toContain('Advanced');
    const advanced = NAV.find((x) => x.label === 'Advanced')!;
    // Production NAV gives "gpu" two match-label hits inside Advanced —
    // GPU acceleration (its main row) and CPU only (because the latter
    // aliases include "no gpu"). The summary joins up to two hits with
    // ", ", so the contract is "starts with GPU acceleration", not
    // "exactly GPU acceleration". Pre-refactor MainShell behaved
    // identically; the simplified fixture in settings_search.test.ts
    // omits the second hit.
    const summary = settingsSearchMatchSummary(advanced, 'gpu');
    expect(summary).toBeDefined();
    expect(summary!.startsWith('GPU acceleration')).toBe(true);
  });

  it('launch startup → General / Launch at startup', () => {
    expect(labelsOf(filterSettingsByQuery(NAV, 'launch startup'))).toEqual(['General']);
    const general = NAV.find((x) => x.label === 'General')!;
    expect(settingsSearchMatchSummary(general, 'launch startup')).toBe('Launch at startup');
  });

  it('empty query returns the full list (preserves About version sublabel path)', () => {
    // The empty-query path is what feeds the About-row "v0.x.y" sublabel
    // in MainShell.tsx — `filterSettingsByQuery` returns every section
    // unchanged, so the `.map(... item.to === '/settings/about' ? ...)`
    // branch can attach the version string. Pin the contract here at
    // the helper layer; the MainShell branch logic is unchanged.
    expect(filterSettingsByQuery(NAV, '')).toEqual(NAV);
    expect(filterSettingsByQuery(NAV, '   ')).toEqual(NAV);
  });

  it('empty query summary returns undefined (no per-row hint, version path is free)', () => {
    // The MainShell shell only attaches the App version sublabel when
    // `settingsSearchMatchSummary` is NOT consulted — i.e. when the
    // query is empty. Confirm that contract from the helper side.
    const general = NAV.find((x) => x.label === 'General')!;
    expect(settingsSearchMatchSummary(general, '')).toBeUndefined();
    expect(settingsSearchMatchSummary(general, '   ')).toBeUndefined();
  });

  it('unknown query returns no items (no hint, no false positive)', () => {
    expect(filterSettingsByQuery(NAV, 'banana republic')).toEqual([]);
  });
});
