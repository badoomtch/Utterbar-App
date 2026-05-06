import { describe, expect, it, beforeAll } from 'vitest';

import i18n from './index';
import barEn from './locales/en/bar.json';
import commonEn from './locales/en/common.json';
import onboardingEn from './locales/en/onboarding.json';
import settingsEn from './locales/en/settings.json';

beforeAll(async () => {
  // The bootstrap module calls `init()` synchronously, but i18next resolves
  // its initial language on a microtask. Awaiting `loadNamespaces` here
  // makes the assertions deterministic regardless of import order.
  await i18n.loadNamespaces(['bar', 'common', 'onboarding', 'settings']);
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

  it('resolves every General settings key', () => {
    // Pinned list of every settings.general.* key consumed by
    // GeneralSettings.tsx. Adding/renaming a key here without updating
    // the catalogue (or vice versa) fails the test before users see a
    // missing-key fallback at runtime.
    const keys = [
      'general.title',
      'general.startup.title',
      'general.startup.launch_on_login.label',
      'general.startup.launch_on_login.description',
      'general.startup.start_minimised.label',
      'general.startup.start_minimised.description',
      'general.startup.close_to_tray.label',
      'general.startup.close_to_tray.description',
      'general.appearance.title',
      'general.appearance.theme.label',
      'general.appearance.theme.description',
      'general.appearance.theme.options.obsidian',
      'general.appearance.theme.options.graphite',
      'general.appearance.theme.options.paper',
      'general.appearance.accent.label',
      'general.appearance.accent.description',
      'general.appearance.custom_accent.title_pro',
      'general.appearance.custom_accent.title_free',
      'general.appearance.custom_accent.aria',
      'general.feedback.title',
      'general.feedback.sound_cues.label',
      'general.feedback.sound_cues.description',
      'general.feedback.sound_profile.label',
      'general.feedback.sound_profile.description',
      'general.feedback.sound_profile.options.default.label',
      'general.feedback.sound_profile.options.default.description',
      'general.feedback.sound_profile.options.classic.label',
      'general.feedback.sound_profile.options.classic.description',
      'general.feedback.sound_profile.options.subtle.label',
      'general.feedback.sound_profile.options.subtle.description',
      'general.feedback.sound_profile.options.chime.label',
      'general.feedback.sound_profile.options.chime.description',
      'general.feedback.notifications.label',
      'general.feedback.notifications.description',
      'general.upgrade.custom_accent.feature',
      'general.upgrade.custom_accent.description',
      'general.upgrade.custom_accent.bullets.any_hex',
      'general.upgrade.custom_accent.bullets.auto_tint_shade',
      'general.upgrade.custom_accent.bullets.presets_free',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('App language description still distinguishes interface from dictation', () => {
    // Hard-pin the honesty contract: the App language picker copy must
    // make explicit that dictation language lives elsewhere. A future
    // copy edit that softens this loses the contract; this test fails
    // loudly so the change is deliberate.
    const desc = String(
      i18n.getResource('en', 'settings', 'general.app_language.description'),
    );
    expect(desc).toContain('Interface language');
    expect(desc).toContain('Dictation language');
    expect(desc).toContain('Models');
  });

  it('resolves every Settings nav section label', () => {
    // Pinned 12-section list. The Settings sidebar order is part of
    // the brand IA; renaming or reordering must update both this list
    // and `settings_nav.ts` together.
    const sectionIds = [
      'general',
      'bar',
      'audio',
      'models',
      'shortcuts',
      'text_output',
      'accessibility',
      'privacy',
      'advanced',
      'licence',
      'updates',
      'about',
    ];
    for (const id of sectionIds) {
      const value = i18n.getResource('en', 'settings', `nav.${id}.label`);
      expect(value, `settings.json missing nav.${id}.label`).toBeTruthy();
      expect(typeof value, `nav.${id}.label must be a string`).toBe('string');
    }
  });

  it('resolves the Settings search input copy', () => {
    expect(i18n.getResource('en', 'settings', 'nav.search.placeholder')).toBe('Search settings');
    expect(i18n.getResource('en', 'settings', 'nav.search.clear')).toBe('Clear search');
    // The no-match string is interpolated against `{{query}}`; assert
    // the template carries the placeholder so a future copy edit
    // can't drop the substitution silently.
    const noMatch = String(i18n.getResource('en', 'settings', 'nav.search.no_match'));
    expect(noMatch).toContain('{{query}}');
  });

  it('renders nav.search.no_match with the user query interpolated', () => {
    const out = i18n.t('nav.search.no_match', { ns: 'settings', query: 'banana' });
    expect(out).toContain('banana');
    expect(out).not.toContain('{{');
  });

  it('resolves the Workspace and Settings group labels', () => {
    expect(i18n.getResource('en', 'settings', 'nav.group.workspace')).toBe('Workspace');
    expect(i18n.getResource('en', 'settings', 'nav.group.settings')).toBe('Settings');
  });

  it('resolves at least one representative key for every onboarding step', () => {
    // Pinned 12-step list. The Onboarding `STEPS` const order is part of
    // the user journey; this test fails loudly if any step's translation
    // key disappears or the catalogue gets reshuffled.
    const keys = [
      'welcome.heading',
      'privacy.title',
      'updates.title',
      'theme.title',
      'model.title',
      'gpu.title',
      'mic.title',
      'baseline.title',
      'trigger.title',
      'shortcut.title',
      'bar.title',
      'ready.heading',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'onboarding', key);
      expect(value, `onboarding.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `onboarding.json key ${key} must be string`).toBe('string');
    }
  });

  it('resolves the model-download phase keys', () => {
    // The download UI surfaces these mid-install. Pinning them prevents
    // a silent missing-key fallback from leaking the raw catalogue path
    // into the progress line.
    const keys = [
      'model.download.connecting',
      'model.download.downloading',
      'model.download.verifying',
      'model.download.extracting',
      'model.download.done',
      'model.download.error_summary',
      'model.download.error_generic',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'onboarding', key);
      expect(value, `onboarding.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `onboarding.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders model.download.downloading with rate + eta interpolated', () => {
    const out = i18n.t('model.download.downloading', {
      ns: 'onboarding',
      rate: '12 MB',
      eta: '38 s',
    });
    expect(out).toContain('12 MB');
    expect(out).toContain('38 s');
    // Crucially: the raw key + Mustache placeholders must NOT leak through.
    expect(out).not.toContain('{{');
    expect(out).not.toContain('model.download.downloading');
  });

  it('exposes the localised typing-test sentence pool', () => {
    // BaselineStep reads `baseline.sentences` via i18next `returnObjects`
    // and falls back to the in-source pool only if the result isn't an
    // array. Pin both contracts here.
    const value = i18n.t('baseline.sentences', {
      ns: 'onboarding',
      returnObjects: true,
    });
    expect(Array.isArray(value)).toBe(true);
    expect((value as readonly string[]).length).toBeGreaterThan(0);
  });

  it('resolves every chrome key consumed by the app shell', () => {
    // Pinned list of every common.chrome.* key the shell actually reads.
    // Adding/renaming a key here without updating the catalogue (or vice
    // versa) fails the test before users see a missing-key fallback at
    // runtime.
    const keys = [
      'chrome.nav.dashboard',
      'chrome.nav.history',
      'chrome.nav.meetings',
      'chrome.nav.modes',
      'chrome.nav.vocabulary',
      'chrome.nav.files',
      'chrome.footer.strap',
      'chrome.tier.pro',
      'chrome.tier.free_cta',
      'chrome.update_toast.title',
      'chrome.update_toast.install',
      'chrome.update_toast.installing',
      'chrome.update_toast.later',
      'chrome.update_toast.error_generic',
      'chrome.update_toast.post_title',
      'chrome.update_toast.post_body',
      'chrome.update_toast.post_link',
      'chrome.update_toast.dismiss_aria',
      'chrome.upgrade_modal.feature_prefix',
      'chrome.upgrade_modal.pro_word',
      'chrome.upgrade_modal.feature_suffix',
      'chrome.upgrade_modal.period.one_time',
      'chrome.upgrade_modal.period.monthly',
      'chrome.upgrade_modal.period.yearly',
      'chrome.upgrade_modal.not_now',
      'chrome.upgrade_modal.upgrade',
      'chrome.pro_nudge.feature_prefix',
      'chrome.pro_nudge.pro_word',
      'chrome.pro_nudge.feature_suffix',
      'chrome.pro_nudge.learn_more',
      'chrome.managed_hint.default',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'common', key);
      expect(value, `common.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `common.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders chrome.update_toast.post_title with the version interpolated', () => {
    const out = i18n.t('chrome.update_toast.post_title', {
      ns: 'common',
      version: '0.19.0',
    });
    expect(out).toContain('0.19.0');
    // Crucially: the raw key + Mustache placeholders must NOT leak through.
    expect(out).not.toContain('{{');
    expect(out).not.toContain('chrome.update_toast.post_title');
  });

  it('resolves every Text Output settings key', () => {
    // Pinned list of every settings.text_output.* key consumed by
    // TextOutputSettings.tsx. Adding/renaming a key here without updating
    // the catalogue (or vice versa) fails the test before users see a
    // missing-key fallback at runtime.
    const keys = [
      'text_output.title',
      'text_output.subtitle',
      'text_output.delivery.title',
      'text_output.delivery.paste_automatically.label',
      'text_output.delivery.paste_automatically.description',
      'text_output.delivery.restore_clipboard.label',
      'text_output.delivery.restore_clipboard.description',
      'text_output.delivery.keystroke_fallback.label',
      'text_output.delivery.keystroke_fallback.description',
      'text_output.formatting.title',
      'text_output.formatting.auto_capitalise.label',
      'text_output.formatting.auto_capitalise.description',
      'text_output.formatting.trim_trailing_space.label',
      'text_output.formatting.trim_trailing_space.description',
      'text_output.formatting.auto_space_on_paste.label',
      'text_output.formatting.auto_space_on_paste.description',
      'text_output.formatting.numbers_as_digits.label',
      'text_output.formatting.numbers_as_digits.description',
      'text_output.formatting.smart_rewrite.label',
      'text_output.formatting.smart_rewrite.description',
      'text_output.formatting.shift_to_auto_send.label',
      'text_output.formatting.shift_to_auto_send.description',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('catalogues are non-empty (placeholder check)', () => {
    // Defensive against a bad merge that empties a JSON file. ESM
    // imports compile-time pin them; this is the sanity belt.
    expect(Object.keys(barEn.phase).length).toBeGreaterThan(0);
    expect(Object.keys(barEn.button).length).toBeGreaterThan(0);
    expect(Object.keys(barEn.mic_mute).length).toBeGreaterThan(0);
    expect(Object.keys(commonEn.language).length).toBeGreaterThan(0);
    expect(Object.keys(commonEn.chrome).length).toBeGreaterThan(0);
    expect(Object.keys(commonEn.chrome.nav).length).toBeGreaterThan(0);
    expect(Object.keys(commonEn.chrome.update_toast).length).toBeGreaterThan(0);
    expect(Object.keys(commonEn.chrome.upgrade_modal).length).toBeGreaterThan(0);
    expect(Object.keys(commonEn.chrome.pro_nudge).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.general).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.general.startup).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.general.appearance).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.general.feedback).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.general.upgrade).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.nav).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.nav.group).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.nav.search).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.text_output).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.text_output.delivery).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.text_output.formatting).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.welcome).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.model).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.model.download).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.gpu).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.baseline).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.ready).length).toBeGreaterThan(0);
    expect(onboardingEn.baseline.sentences.length).toBeGreaterThan(0);
  });
});
