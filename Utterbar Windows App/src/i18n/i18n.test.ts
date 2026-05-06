import { describe, expect, it, beforeAll } from 'vitest';

import i18n from './index';
import barEn from './locales/en/bar.json';
import commonEn from './locales/en/common.json';
import settingsEn from './locales/en/settings.json';

beforeAll(async () => {
  // The bootstrap module calls `init()` synchronously, but i18next resolves
  // its initial language on a microtask. Awaiting `loadNamespaces` here
  // makes the assertions deterministic regardless of import order.
  await i18n.loadNamespaces(['bar', 'common', 'settings']);
});

describe('i18n English foundation', () => {
  it('initialises with English as the active language', () => {
    expect(i18n.language).toBe('en');
  });

  it('resolves every floating-bar phase key', () => {
    // Pinned list. If a phase label is added or renamed in StatePill.tsx,
    // this list — and the bar.json catalogue — must be updated together.
    const keys = [
      'phase.ready',
      'phase.initialising_model',
      'phase.recording',
      'phase.processing',
      'phase.transcribing_meeting',
      'phase.transcribing_meeting_with_progress',
      'phase.saved',
      'phase.auto_stopped_5_min',
      'phase.inserted',
      'phase.error',
      'phase.cancelled',
      'phase.no_audio_detected',
      'unit.chunk_one',
      'unit.chunk_other',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'bar', key);
      expect(value, `bar.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `bar.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders the meeting-progress template with interpolated values', () => {
    const out = i18n.t('phase.transcribing_meeting_with_progress', {
      ns: 'bar',
      current: 3,
      total: 11,
    });
    expect(out).toContain('3');
    expect(out).toContain('11');
    // Crucially: the raw key must NOT leak through — that is the most
    // common failure mode (namespace not registered, key not loaded).
    expect(out).not.toContain('{{');
    expect(out).not.toContain('phase.transcribing_meeting_with_progress');
  });

  it('renders the saved-meeting template with interpolated chunk word', () => {
    const single = i18n.t('phase.saved', {
      ns: 'bar',
      time: '0:09',
      modelSegment: '',
      chunks: 1,
      chunkWord: 'chunk',
      chars: '142',
    });
    expect(single).toContain('1 chunk ');
    expect(single).not.toContain('1 chunks');

    const many = i18n.t('phase.saved', {
      ns: 'bar',
      time: '5:01',
      modelSegment: ' · Whisper Medium',
      chunks: 11,
      chunkWord: 'chunks',
      chars: '799',
    });
    expect(many).toContain('11 chunks');
    expect(many).toContain('Whisper Medium');
  });

  it('settings catalogue exposes the App language row keys', () => {
    expect(i18n.getResource('en', 'settings', 'general.app_language.label')).toBe('App language');
    expect(
      i18n.getResource('en', 'settings', 'general.app_language.description'),
    ).toContain('Dictation language');
  });

  it('common catalogue has an English language label', () => {
    expect(i18n.getResource('en', 'common', 'language.english')).toBe('English');
  });

  it('resolves every floating-bar button key', () => {
    // Pinned list. If a button is added, renamed, or removed from
    // FloatingBar.tsx, this list — and bar.json — must move together.
    const keys = [
      'button.record',
      'button.stop',
      'button.cancel',
      'button.settings',
      'button.copy_last_message',
      'button.copied',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'bar', key);
      expect(value, `bar.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `bar.json key ${key} must be string`).toBe('string');
    }
  });

  it('exposes the mic-mute Confirmed button label', () => {
    // The frontend-owned button copy is localised here; the banner
    // message itself is still Rust-emitted prose (see MicMuteBanner.tsx)
    // and stays unlocalised in this branch.
    expect(i18n.getResource('en', 'bar', 'mic_mute.confirmed')).toBe('Confirmed');
  });

  it('catalogues are non-empty (placeholder check)', () => {
    // Defensive against a bad merge that empties a JSON file. ESM
    // imports compile-time pin them; this is the sanity belt.
    expect(Object.keys(barEn.phase).length).toBeGreaterThan(0);
    expect(Object.keys(barEn.button).length).toBeGreaterThan(0);
    expect(Object.keys(barEn.mic_mute).length).toBeGreaterThan(0);
    expect(Object.keys(commonEn.language).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.general).length).toBeGreaterThan(0);
  });
});
