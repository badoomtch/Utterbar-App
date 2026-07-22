import { describe, expect, it, beforeAll, afterAll } from 'vitest';

import i18n, { applyAppLanguage } from './index';
import barEn from './locales/en/bar.json';
import commonEn from './locales/en/common.json';
import onboardingEn from './locales/en/onboarding.json';
import settingsEn from './locales/en/settings.json';
import barEs from './locales/es/bar.json';
import commonEs from './locales/es/common.json';
import onboardingEs from './locales/es/onboarding.json';
import settingsEs from './locales/es/settings.json';
import barFr from './locales/fr/bar.json';
import commonFr from './locales/fr/common.json';
import onboardingFr from './locales/fr/onboarding.json';
import settingsFr from './locales/fr/settings.json';
import barDe from './locales/de/bar.json';
import commonDe from './locales/de/common.json';
import onboardingDe from './locales/de/onboarding.json';
import settingsDe from './locales/de/settings.json';
import barIt from './locales/it/bar.json';
import commonIt from './locales/it/common.json';
import onboardingIt from './locales/it/onboarding.json';
import settingsIt from './locales/it/settings.json';
import barPt from './locales/pt/bar.json';
import commonPt from './locales/pt/common.json';
import onboardingPt from './locales/pt/onboarding.json';
import settingsPt from './locales/pt/settings.json';
import barNl from './locales/nl/bar.json';
import commonNl from './locales/nl/common.json';
import onboardingNl from './locales/nl/onboarding.json';
import settingsNl from './locales/nl/settings.json';

beforeAll(async () => {
  // The bootstrap module calls `init()` synchronously, but i18next resolves
  // its initial language on a microtask. Awaiting `loadNamespaces` here
  // makes the assertions deterministic regardless of import order.
  await i18n.loadNamespaces(['bar', 'common', 'onboarding', 'settings']);
});

/**
 * Recursively collects every leaf path from a nested catalogue object.
 * For arrays, only the array's path is recorded (not per-element); the
 * Spanish parity test then asserts the same array length on both sides.
 *
 * Used by the Spanish-locale parity tests below to guarantee Spanish has
 * exactly the same key surface as English, no extra keys, no missing keys.
 */
function collectPaths(obj: unknown, prefix = ''): string[] {
  if (obj === null || obj === undefined) return [prefix];
  if (Array.isArray(obj)) return [prefix];
  if (typeof obj === 'object') {
    const out: string[] = [];
    for (const [key, val] of Object.entries(obj as Record<string, unknown>)) {
      const path = prefix ? `${prefix}.${key}` : key;
      out.push(...collectPaths(val, path));
    }
    return out;
  }
  // String / number / boolean leaf
  return [prefix];
}

/**
 * Extracts every Mustache-style interpolation placeholder from a string,
 * sorted and deduplicated. Used to enforce that the Spanish translation
 * of every interpolated key carries the same `{{name}}` placeholders as
 * the English source. Order doesn't matter; presence does.
 */
function placeholdersOf(s: string): string[] {
  const matches = s.match(/\{\{[^}]+\}\}/g) ?? [];
  return Array.from(new Set(matches)).sort();
}

/**
 * Walks both catalogues in parallel and returns every leaf path that holds
 * a string value in EN. Lets the placeholder-parity test compare the EN
 * and ES strings at every interpolated key without manually enumerating
 * them.
 */
function collectStringLeaves(obj: unknown, prefix = ''): Array<[string, string]> {
  if (obj === null || obj === undefined) return [];
  if (typeof obj === 'string') return [[prefix, obj]];
  if (Array.isArray(obj)) return [];
  if (typeof obj === 'object') {
    const out: Array<[string, string]> = [];
    for (const [key, val] of Object.entries(obj as Record<string, unknown>)) {
      const path = prefix ? `${prefix}.${key}` : key;
      out.push(...collectStringLeaves(val, path));
    }
    return out;
  }
  return [];
}

function getByPath(obj: unknown, path: string): unknown {
  const parts = path.split('.');
  let cur: unknown = obj;
  for (const part of parts) {
    if (cur === null || cur === undefined || typeof cur !== 'object') return undefined;
    cur = (cur as Record<string, unknown>)[part];
  }
  return cur;
}

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
    // Pinned 13-step list (12 base + the dictation-language step
    // added in the language-aware-onboarding-models branch). The
    // Onboarding `STEPS` const order is part of the user journey;
    // this test fails loudly if any step's translation key disappears
    // or the catalogue gets reshuffled.
    const keys = [
      'welcome.heading',
      'privacy.title',
      'updates.title',
      'theme.title',
      'language.title',
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
      'text_output.delivery.clipboard_restore_delay.label',
      'text_output.delivery.clipboard_restore_delay.description',
      'text_output.delivery.clipboard_restore_delay.value',
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

  it('resolves every Accessibility settings key', () => {
    // Pinned list of every settings.accessibility.* key consumed by
    // AccessibilitySettings.tsx. Adding/renaming a key here without
    // updating the catalogue (or vice versa) fails the test before
    // users see a missing-key fallback at runtime.
    const keys = [
      'accessibility.title',
      'accessibility.subtitle',
      'accessibility.ui_scale.title',
      'accessibility.ui_scale.description',
      'accessibility.ui_scale.scale.label',
      'accessibility.ui_scale.scale.description',
      'accessibility.ui_scale.fine_tune.label',
      'accessibility.ui_scale.value',
      'accessibility.motion.title',
      'accessibility.motion.description',
      'accessibility.motion.reduce_motion.label',
      'accessibility.motion.reduce_motion.description',
      'accessibility.contrast.title',
      'accessibility.contrast.description',
      'accessibility.contrast.high_contrast.label',
      'accessibility.contrast.high_contrast.description',
      'accessibility.contrast.larger_focus_rings.label',
      'accessibility.contrast.larger_focus_rings.description',
      'accessibility.transparency.title',
      'accessibility.transparency.description',
      'accessibility.transparency.reduce_transparency.label',
      'accessibility.transparency.reduce_transparency.description',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders accessibility.ui_scale.value with the percentage interpolated', () => {
    // The chip presets and the slider readout share this template via
    // `t('accessibility.ui_scale.value', { value })`. Pin both halves so a
    // future regression that drops the `{{value}}` substitution fails.
    const out = i18n.t('accessibility.ui_scale.value', {
      ns: 'settings',
      value: 125,
    });
    expect(out).toContain('125');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('accessibility.ui_scale.value');
  });

  it('resolves every Bar settings key', () => {
    // Pinned list of every settings.bar_settings.* key consumed by
    // BarSettings.tsx. Adding/renaming a key here without updating the
    // catalogue (or vice versa) fails the test before users see a
    // missing-key fallback at runtime.
    const keys = [
      'bar_settings.title',
      'bar_settings.subtitle',
      'bar_settings.visibility.title',
      'bar_settings.visibility.always.label',
      'bar_settings.visibility.always.description',
      'bar_settings.visibility.while_recording.label',
      'bar_settings.visibility.while_recording.description',
      'bar_settings.visibility.hidden.label',
      'bar_settings.visibility.hidden.description',
      'bar_settings.appearance.title',
      'bar_settings.appearance.size.label',
      'bar_settings.appearance.size.description',
      'bar_settings.appearance.size.options.compact',
      'bar_settings.appearance.size.options.standard',
      'bar_settings.appearance.outline.label',
      'bar_settings.appearance.outline.description',
      'bar_settings.position.title',
      'bar_settings.position.reset.label',
      'bar_settings.position.reset.description',
      'bar_settings.position.reset.button',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('resolves every About settings key', () => {
    // Pinned list of every settings.about.* key consumed by
    // AboutSettings.tsx. Adding/renaming a key here without updating
    // the catalogue (or vice versa) fails the test before users see a
    // missing-key fallback at runtime.
    const keys = [
      'about.title',
      'about.tagline',
      'about.version_pending',
      'about.footer.copyright_prefix',
      'about.footer.attribution_link',
      'about.footer.location_suffix',
      'about.footer.website_link',
      'about.third_party.title',
      'about.third_party.description',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders about.footer.copyright_prefix with the year interpolated', () => {
    // The footer attribution mirrors the website footer's wording. Pin
    // the `{{year}}` substitution so a future copy edit can't drop the
    // interpolation silently and leave a literal mustache template
    // visible to users.
    const out = i18n.t('about.footer.copyright_prefix', {
      ns: 'settings',
      year: 2026,
    });
    expect(out).toContain('2026');
    expect(out).toContain('Utterbar');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('about.footer.copyright_prefix');
  });

  it('resolves every Shortcuts settings key', () => {
    // Pinned list of every settings.shortcuts.* key consumed by
    // ShortcutsSettings.tsx. Backend-owned slot.label and
    // slot.description still flow straight through from `list_shortcuts`
    // and are deliberately NOT catalogued — those move in the future
    // backend-prose / error-codes branch.
    const keys = [
      'shortcuts.title',
      'shortcuts.subtitle',
      'shortcuts.capture.press_keys',
      'shortcuts.capture.unbound',
      'shortcuts.buttons.clear',
      'shortcuts.conflict.tooltip',
      'shortcuts.tip.prefix',
      'shortcuts.tip.suffix',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders shortcuts.conflict.tooltip with the accelerator interpolated', () => {
    // The conflict-warning tooltip displays the colliding accelerator
    // (which itself comes from the Rust `detect_conflicts` payload).
    // Pin the `{{accelerator}}` substitution so a future copy edit can't
    // drop the interpolation silently.
    const out = i18n.t('shortcuts.conflict.tooltip', {
      ns: 'settings',
      accelerator: 'Ctrl+Alt+Space',
    });
    expect(out).toContain('Ctrl+Alt+Space');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('shortcuts.conflict.tooltip');
  });

  it('resolves every Audio settings key', () => {
    // Pinned list of every settings.audio.* key consumed by
    // AudioSettings.tsx. Backend/OS-owned device names and ids still
    // flow straight through from `list_input_devices`; only frontend-
    // owned chrome (page header, capture-source picker copy, the
    // Meeting-Mode callout, the input-device card chrome, and the
    // noise-handling card with its 4-level suppression registry)
    // is catalogued here.
    const keys = [
      'audio.title',
      'audio.subtitle',
      'audio.capture_source.title',
      'audio.capture_source.description',
      'audio.capture_source.options.microphone.label',
      'audio.capture_source.options.microphone.note',
      'audio.capture_source.options.system.label',
      'audio.capture_source.options.system.note',
      'audio.capture_source.options.mixed.label',
      'audio.capture_source.options.mixed.note',
      'audio.meeting_callout.title',
      'audio.meeting_callout.description',
      'audio.meeting_callout.link',
      'audio.input_device.title',
      'audio.input_device.microphone.label',
      'audio.input_device.microphone.description',
      'audio.input_device.system_default',
      'audio.input_device.system_default_caption',
      'audio.input_device.live_meter.label',
      'audio.input_device.live_meter.test',
      'audio.input_device.live_meter.stop',
      'audio.noise_handling.title',
      'audio.noise_handling.description',
      'audio.noise_handling.suppression.label',
      'audio.noise_handling.suppression.description',
      'audio.noise_handling.suppression.preview_tooltip',
      'audio.noise_handling.suppression.preview_hint',
      'audio.noise_handling.suppression.levels.off.label',
      'audio.noise_handling.suppression.levels.off.blurb',
      'audio.noise_handling.suppression.levels.light.label',
      'audio.noise_handling.suppression.levels.light.blurb',
      'audio.noise_handling.suppression.levels.balanced.label',
      'audio.noise_handling.suppression.levels.balanced.blurb',
      'audio.noise_handling.suppression.levels.aggressive.label',
      'audio.noise_handling.suppression.levels.aggressive.blurb',
      'audio.noise_handling.noise_gate.label',
      'audio.noise_handling.noise_gate.description',
      'audio.noise_handling.dynamic_normalisation.label',
      'audio.noise_handling.dynamic_normalisation.description',
      'audio.noise_handling.silence_removal.label',
      'audio.noise_handling.silence_removal.description',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders audio.noise_handling.suppression.preview_tooltip with the blurb interpolated', () => {
    // The preview tooltip composes the active level's blurb with the
    // " · Click to preview" suffix at render time. Pin the
    // `{{blurb}}` substitution so a future copy edit can't drop the
    // interpolation silently and leave a literal mustache template
    // visible to users.
    const out = i18n.t('audio.noise_handling.suppression.preview_tooltip', {
      ns: 'settings',
      blurb: 'Cuts fan hum, HVAC, and keyboard clatter between words. Best for most setups.',
    });
    expect(out).toContain('Cuts fan hum');
    expect(out).toContain('Click to preview');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('audio.noise_handling.suppression.preview_tooltip');
  });

  it('resolves every Privacy settings key', () => {
    // Pinned list of every settings.privacy.* key consumed by
    // PrivacySettings.tsx. Backend-emitted DataPaths values, the
    // user-supplied conferencing-app names, and the Rust validator's
    // stable reason ids stay verbatim — only frontend chrome and the
    // matching error-message catalogue are pinned here.
    const keys = [
      'privacy.title',
      'privacy.subtitle',
      'privacy.managed_hint',
      'privacy.context.title',
      'privacy.context.description',
      'privacy.context.active_app.label',
      'privacy.context.active_app.description',
      'privacy.context.selection.label',
      'privacy.context.selection.description',
      'privacy.context.clipboard.label',
      'privacy.context.clipboard.description',
      'privacy.audio_retention.title',
      'privacy.audio_retention.save_recordings.label',
      'privacy.audio_retention.save_recordings.description',
      'privacy.audio_retention.pro_nudge.feature',
      'privacy.audio_retention.pro_nudge.description',
      'privacy.audio_retention.delete_after.label',
      'privacy.audio_retention.delete_after.description',
      'privacy.audio_retention.delete_after.value',
      'privacy.media.title',
      'privacy.media.pause_music.label',
      'privacy.media.pause_music.description',
      'privacy.media.mute_conferencing.label',
      'privacy.media.mute_conferencing.description',
      'privacy.conferencing_apps.title',
      'privacy.conferencing_apps.description_prefix',
      'privacy.conferencing_apps.using_defaults',
      'privacy.conferencing_apps.using_custom',
      'privacy.conferencing_apps.edit_locked',
      'privacy.conferencing_apps.reset_to_defaults',
      'privacy.conferencing_apps.empty',
      'privacy.conferencing_apps.input_placeholder',
      'privacy.conferencing_apps.add',
      'privacy.conferencing_apps.remove_aria',
      'privacy.conferencing_apps.errors.empty',
      'privacy.conferencing_apps.errors.invalid_characters',
      'privacy.conferencing_apps.errors.too_long',
      'privacy.conferencing_apps.errors.duplicate',
      'privacy.conferencing_apps.errors.defaults_loading',
      'privacy.conferencing_apps.errors.fallback',
      'privacy.data_locations.title',
      'privacy.data_locations.rows.config',
      'privacy.data_locations.rows.database',
      'privacy.data_locations.rows.models',
      'privacy.data_locations.rows.recordings',
      'privacy.data_locations.rows.logs',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders privacy.audio_retention.delete_after.value with the day count interpolated', () => {
    // The retention slider readout reads `t(..., { days })`. Pin the
    // `{{days}}` substitution so a future copy edit can't drop it
    // silently and leave a literal mustache template visible.
    const out = i18n.t('privacy.audio_retention.delete_after.value', {
      ns: 'settings',
      days: 14,
    });
    expect(out).toContain('14');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('privacy.audio_retention.delete_after.value');
  });

  it('renders privacy.conferencing_apps.remove_aria with the app name interpolated', () => {
    // The remove-button aria-label is composed at render time per app.
    // Pin the `{{name}}` substitution so screen-reader text always
    // names the specific app being removed rather than a literal
    // template string.
    const out = i18n.t('privacy.conferencing_apps.remove_aria', {
      ns: 'settings',
      name: 'Teams.exe',
    });
    expect(out).toContain('Teams.exe');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('privacy.conferencing_apps.remove_aria');
  });

  it('resolves every Updates settings key', () => {
    // Pinned list of every settings.updates.* key consumed by
    // UpdatesSettings.tsx. Backend-emitted version strings, the
    // bundled release-notes markdown, server-returned subscription
    // messages, OrliqError envelope messages, and the user-supplied
    // email value all stay verbatim — only frontend chrome is pinned
    // here.
    const keys = [
      'updates.title',
      'updates.subtitle',
      'updates.beta_warning.heading',
      'updates.beta_warning.body',
      'updates.channel.title',
      'updates.channel.description',
      'updates.channel.row_label',
      'updates.channel.options.stable',
      'updates.channel.options.beta',
      'updates.channel.auto_check.label',
      'updates.channel.auto_check.description',
      'updates.channel.check_now.label',
      'updates.channel.check_now.description_idle',
      'updates.channel.check_now.description_available',
      'updates.channel.check_now.description_up_to_date',
      'updates.channel.check_now.description_failed',
      'updates.channel.check_now.checking',
      'updates.channel.check_now.button',
      'updates.release_notes.title_with_version',
      'updates.release_notes.title_fallback',
      'updates.release_notes.description',
      'updates.release_notes.empty',
      'updates.release_email.title',
      'updates.release_email.description',
      'updates.release_email.row.label',
      'updates.release_email.row.description',
      'updates.release_email.input_placeholder',
      'updates.release_email.subscribe',
      'updates.release_email.sending',
      'updates.release_email.subscribed_as',
      'updates.release_email.change_email',
      'updates.release_email.errors.empty',
      'updates.release_email.errors.invalid',
      'updates.release_email.errors.fallback',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders updates.channel.check_now.description_available with current and available interpolated', () => {
    // Both versions come from the Rust updater feed (updateInfo.current,
    // updateInfo.available) and stay verbatim. Pin both `{{current}}`
    // and `{{available}}` substitutions so a future copy edit can't
    // drop either silently.
    const out = i18n.t('updates.channel.check_now.description_available', {
      ns: 'settings',
      current: '0.20.0',
      available: '0.20.1',
    });
    expect(out).toContain('0.20.0');
    expect(out).toContain('0.20.1');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('updates.channel.check_now.description_available');
  });

  it('renders updates.release_notes.title_with_version with the version interpolated', () => {
    // The What's-new card title interpolates the running version.
    // Pin the `{{version}}` substitution so the catalogue can't drop
    // it and leave a literal mustache template visible in the title.
    const out = i18n.t('updates.release_notes.title_with_version', {
      ns: 'settings',
      version: '0.20.1',
    });
    expect(out).toContain('0.20.1');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('updates.release_notes.title_with_version');
  });

  it('renders updates.release_email.subscribed_as with the email interpolated', () => {
    // The "Subscribed as" line displays the user-supplied email
    // verbatim through the catalogued template. Pin the `{{email}}`
    // substitution so the email can never be dropped or replaced by
    // a literal placeholder.
    const out = i18n.t('updates.release_email.subscribed_as', {
      ns: 'settings',
      email: 'jay@example.com',
    });
    expect(out).toContain('jay@example.com');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('updates.release_email.subscribed_as');
  });

  it('resolves every Advanced settings key', () => {
    // Pinned list of every settings.advanced.* key consumed by
    // AdvancedSettings.tsx. Backend-emitted Diagnostics fields
    // (version/os/cpu_count/log_path/db_path/models_path), GPU
    // adapter names + VRAM numbers, the Rust-side backupError
    // string, and the basename split off the pending import path
    // all stay verbatim — only frontend chrome and the matching
    // status fragments are pinned here.
    const keys = [
      'advanced.title',
      'advanced.subtitle',
      'advanced.gpu.title',
      'advanced.gpu.description.checking',
      'advanced.gpu.description.no_runtime_ep',
      'advanced.gpu.description.detected',
      'advanced.gpu.description.no_gpu',
      'advanced.gpu.options.auto.title',
      'advanced.gpu.options.auto.hint',
      'advanced.gpu.options.direct_ml.title',
      'advanced.gpu.options.direct_ml.hint',
      'advanced.gpu.options.cpu.title',
      'advanced.gpu.options.cpu.hint',
      'advanced.gpu.disabled_suffix',
      'advanced.gpu.inactive_notice',
      'advanced.gpu.detected_adapters',
      'advanced.gpu.adapter_software_suffix',
      'advanced.gpu.adapter_vram_suffix',
      'advanced.diagnostics.title',
      'advanced.diagnostics.description',
      'advanced.diagnostics.rows.version',
      'advanced.diagnostics.rows.os',
      'advanced.diagnostics.rows.cpu_threads',
      'advanced.diagnostics.rows.onnx',
      'advanced.diagnostics.rows.llama',
      'advanced.diagnostics.rows.logs',
      'advanced.diagnostics.rows.database',
      'advanced.diagnostics.rows.models',
      'advanced.diagnostics.feature_state.compiled',
      'advanced.diagnostics.feature_state.disabled',
      'advanced.backup.title',
      'advanced.backup.description',
      'advanced.backup.include_recordings.label',
      'advanced.backup.include_recordings.description',
      'advanced.backup.export.label',
      'advanced.backup.export.description_idle',
      'advanced.backup.export.description_last',
      'advanced.backup.export.contents_nothing',
      'advanced.backup.export.exporting',
      'advanced.backup.export.button',
      'advanced.backup.export.dialog_title',
      'advanced.backup.export.filter_name',
      'advanced.backup.import.label',
      'advanced.backup.import.description',
      'advanced.backup.import.importing',
      'advanced.backup.import.button',
      'advanced.backup.import.dialog_title',
      'advanced.backup.import_modal.eyebrow',
      'advanced.backup.import_modal.heading',
      'advanced.backup.import_modal.body_prefix',
      'advanced.backup.import_modal.body_suffix',
      'advanced.backup.import_modal.warning',
      'advanced.backup.import_modal.cancel',
      'advanced.backup.import_modal.replace',
      'advanced.backup.import_modal.replacing',
      'advanced.reset.title',
      'advanced.reset.row.label',
      'advanced.reset.row.description',
      'advanced.reset.button',
      'advanced.reset.confirm',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders advanced.gpu.description.detected with the adapter name interpolated', () => {
    // The OS/driver-emitted adapter name flows verbatim through the
    // catalogued template's `{{name}}` substitution. Pin it so a
    // future copy edit can't drop the interpolation silently.
    const out = i18n.t('advanced.gpu.description.detected', {
      ns: 'settings',
      name: 'NVIDIA GeForce RTX 4070',
    });
    expect(out).toContain('NVIDIA GeForce RTX 4070');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('advanced.gpu.description.detected');
  });

  it('renders advanced.gpu.adapter_vram_suffix with the megabyte count interpolated', () => {
    // The VRAM count (already locale-formatted via toLocaleString) is
    // interpolated as `{{mb}}` into the suffix. Pin the substitution
    // so a future regression that drops the placeholder leaves
    // ", undefined MB VRAM" instead.
    const out = i18n.t('advanced.gpu.adapter_vram_suffix', {
      ns: 'settings',
      mb: '8,192',
    });
    expect(out).toContain('8,192');
    expect(out).toContain('MB VRAM');
    expect(out).not.toContain('{{');
  });

  it('renders advanced.backup.export.description_last with size + contents interpolated', () => {
    // The last-export hint composes a formatted byte-size and a
    // joined contents list. Pin both `{{size}}` and `{{contents}}`
    // substitutions so neither can be dropped silently.
    const out = i18n.t('advanced.backup.export.description_last', {
      ns: 'settings',
      size: '4.2 MB',
      contents: 'settings, modes, vocabulary, history',
    });
    expect(out).toContain('4.2 MB');
    expect(out).toContain('settings, modes, vocabulary, history');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('advanced.backup.export.description_last');
  });

  it('resolves every dictation-language onboarding key', () => {
    // Pinned list of every onboarding.language.* key consumed by
    // DictationLanguageStep + the language-aware warnings in
    // ModelPickStep. The dictation-language step writes
    // models.whisper_language; it does NOT touch general.app_language
    // (interface language stays English). Adding/renaming a key here
    // without updating the catalogue (or vice versa) fails the test
    // before users see a missing-key fallback at runtime.
    const keys = [
      'language.title',
      'language.subtitle',
      'language.picker_label',
      'language.picker_description',
      'language.warning.english_only_title',
      'language.warning.english_only_body',
      'language.warning.no_multilingual_title',
      'language.warning.no_multilingual_body',
      'language.switch_to_multilingual',
      // step_titles.language must exist alongside the existing 12 step
      // titles; the new step is rendered as a 13th progress pip.
      'step_titles.language',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'onboarding', key);
      expect(value, `onboarding.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `onboarding.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders language.warning.english_only_body with the model name interpolated', () => {
    // The warning composes the offending model's display_name (Rust-
    // emitted, stays verbatim) into the English-only-warning body.
    // Pin the `{{model}}` substitution so a future copy edit can't
    // drop the interpolation silently.
    const out = i18n.t('language.warning.english_only_body', {
      ns: 'onboarding',
      model: 'Parakeet TDT v3 (int8)',
    });
    expect(out).toContain('Parakeet TDT v3 (int8)');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('language.warning.english_only_body');
  });

  it('exposes a common.language label for every dictation-language registry entry', () => {
    // The shared dictation-language registry in lib/dictationLanguages
    // points each catalogue code at a `common.language.*` key. Pin
    // every code here so a future rename of the registry can't drift
    // away from the catalogue without failing.
    const codes = [
      'auto_detect',
      'english',
      'spanish',
      'french',
      'german',
      'italian',
      'portuguese',
      'dutch',
      'polish',
      'turkish',
      'russian',
      'arabic',
      'hindi',
      'japanese',
      'korean',
      'chinese',
      'indonesian',
      'thai',
      'vietnamese',
      'swedish',
      'norwegian',
      'danish',
      'finnish',
      'czech',
      'greek',
      'hebrew',
    ];
    for (const code of codes) {
      const value = i18n.getResource('en', 'common', `language.${code}`);
      expect(value, `common.json missing key language.${code}`).toBeTruthy();
      expect(typeof value, `common.json key language.${code} must be string`).toBe('string');
    }
  });

  it('resolves the new Models settings header + Whisper-language card keys', () => {
    // Scoped Models settings localisation: the page header + the
    // Whisper-language card chrome (which is the surface most exposed
    // by the new dictation-language flow). Full ModelsSettings.tsx
    // localisation is deliberately deferred to a follow-up branch
    // because most of that file is dense backend-emitted prose.
    const keys = [
      'models.title',
      'models.subtitle',
      'models.whisper_language.title',
      'models.whisper_language.description',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('resolves every Models language-compatibility warning key', () => {
    // The Settings → Models language-compatibility warning mirrors
    // the onboarding model-step banner: when whisper_language is
    // non-English and the default model is English-only, surface a
    // warning with a one-click switch to a multilingual model;
    // when no installed multilingual model exists, show an install
    // hint pointing at the model list. Pin every key the warning
    // resolves so a future copy edit can't drift away from the
    // catalogue.
    const keys = [
      'models.language_warning.english_only_title',
      'models.language_warning.english_only_body',
      'models.language_warning.no_multilingual_title',
      'models.language_warning.no_multilingual_body',
      'models.language_warning.switch_to_multilingual',
      'models.language_warning.install_multilingual_hint',
    ];
    for (const key of keys) {
      const value = i18n.getResource('en', 'settings', key);
      expect(value, `settings.json missing key ${key}`).toBeTruthy();
      expect(typeof value, `settings.json key ${key} must be string`).toBe('string');
    }
  });

  it('renders models.language_warning.english_only_body with the model name interpolated', () => {
    // The offending model's Rust-emitted display_name is interpolated
    // into the warning body as `{{model}}`. Pin the substitution so a
    // future copy edit can't drop it silently and leave a literal
    // mustache template visible to users. The display_name itself
    // stays verbatim (Rust-emitted prose, deferred).
    const out = i18n.t('models.language_warning.english_only_body', {
      ns: 'settings',
      model: 'Parakeet TDT v3 (int8)',
    });
    expect(out).toContain('Parakeet TDT v3 (int8)');
    expect(out).not.toContain('{{');
    expect(out).not.toContain('models.language_warning.english_only_body');
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
    expect(Object.keys(settingsEn.accessibility).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.accessibility.ui_scale).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.accessibility.motion).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.accessibility.contrast).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.accessibility.transparency).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.bar_settings).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.bar_settings.visibility).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.bar_settings.appearance).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.bar_settings.position).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.about).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.about.footer).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.about.third_party).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.shortcuts).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.shortcuts.capture).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.shortcuts.buttons).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.shortcuts.conflict).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.shortcuts.tip).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.audio).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.audio.capture_source).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.audio.capture_source.options).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.audio.meeting_callout).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.audio.input_device).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.audio.input_device.live_meter).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.audio.noise_handling).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.audio.noise_handling.suppression).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.audio.noise_handling.suppression.levels).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.privacy).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.privacy.context).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.privacy.audio_retention).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.privacy.media).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.privacy.conferencing_apps).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.privacy.conferencing_apps.errors).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.privacy.data_locations).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.privacy.data_locations.rows).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.updates).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.updates.beta_warning).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.updates.channel).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.updates.channel.options).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.updates.channel.auto_check).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.updates.channel.check_now).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.updates.release_notes).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.updates.release_email).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.updates.release_email.row).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.updates.release_email.errors).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.gpu).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.gpu.description).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.gpu.options).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.diagnostics).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.diagnostics.rows).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.diagnostics.feature_state).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.backup).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.backup.include_recordings).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.backup.export).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.backup.import).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.backup.import_modal).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.advanced.reset).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.models).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.models.whisper_language).length).toBeGreaterThan(0);
    expect(Object.keys(settingsEn.models.language_warning).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.welcome).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.model).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.model.download).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.gpu).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.baseline).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.ready).length).toBeGreaterThan(0);
    expect(onboardingEn.baseline.sentences.length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.language).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.language.warning).length).toBeGreaterThan(0);
  });
});

describe('i18n Spanish locale', () => {
  // Each pair: [namespace, English source, Spanish source]. Drives the
  // legacy Spanish-only parity + smoke tests below. Bulk locale parity
  // for every supported review-batch locale (es, fr, de, it, pt, nl)
  // lives in the next describe block.
  const NAMESPACES: Array<[string, unknown, unknown]> = [
    ['bar', barEn, barEs],
    ['common', commonEn, commonEs],
    ['onboarding', onboardingEn, onboardingEs],
    ['settings', settingsEn, settingsEs],
  ];

  it('every Spanish namespace has exactly the same key surface as English', () => {
    // Catalogue parity: every key path present in EN must exist in ES,
    // and ES must NOT introduce any extra key paths. Diff both sets and
    // surface every divergence with the namespace prefix so a missing
    // Spanish key is fixable in one read.
    for (const [ns, en, es] of NAMESPACES) {
      const enPaths = new Set(collectPaths(en));
      const esPaths = new Set(collectPaths(es));
      const missingInEs = [...enPaths].filter((p) => !esPaths.has(p));
      const extraInEs = [...esPaths].filter((p) => !enPaths.has(p));
      expect(missingInEs, `Spanish ${ns}.json is missing keys present in English`).toEqual([]);
      expect(extraInEs, `Spanish ${ns}.json has keys not present in English`).toEqual([]);
    }
  });

  it('every interpolated Spanish string contains the same placeholders as English', () => {
    // Placeholder parity. For every leaf string in the English catalogue
    // that contains at least one Mustache placeholder, the matching
    // Spanish string must contain exactly the same set of placeholders.
    // Otherwise we'd render with `undefined` substitutions or leak
    // literal `{{...}}` to users.
    for (const [ns, en, es] of NAMESPACES) {
      const leaves = collectStringLeaves(en);
      for (const [path, enValue] of leaves) {
        const enPlaceholders = placeholdersOf(enValue);
        if (enPlaceholders.length === 0) continue;
        const esValue = getByPath(es, path);
        expect(typeof esValue, `Spanish ${ns}.${path} is not a string`).toBe('string');
        const esPlaceholders = placeholdersOf(String(esValue));
        expect(
          esPlaceholders,
          `Spanish ${ns}.${path} placeholder set diverges from English (en=${JSON.stringify(enPlaceholders)} es=${JSON.stringify(esPlaceholders)})`,
        ).toEqual(enPlaceholders);
      }
    }
  });

  it('baseline.sentences in Spanish has the same array length as English', () => {
    // The typing-test pool is the only array of strings in the catalogues.
    // Different lengths would mean BaselineStep's pool-rotation logic
    // sees different cardinalities depending on UI language, which is
    // confusing to QA and breaks any deterministic-shuffle invariant
    // we may add later.
    expect(onboardingEs.baseline.sentences.length).toBe(onboardingEn.baseline.sentences.length);
  });

  describe('with i18n switched to Spanish', () => {
    beforeAll(async () => {
      applyAppLanguage('es');
      await i18n.loadNamespaces(['bar', 'common', 'onboarding', 'settings']);
    });

    afterAll(() => {
      // Restore English so the rest of the suite (and any later test
      // file in the same vitest worker) sees the canonical default.
      applyAppLanguage('en');
    });

    it('initialises with es as the active language tag', () => {
      expect(i18n.language).toBe('es');
    });

    it('resolves a representative key from every namespace', () => {
      // One sample per namespace, deliberately picked from different
      // shapes (chrome nav, bar phase, onboarding step title, settings
      // page title, settings whisper-language card title) so a missing
      // namespace fails the test loudly rather than silently returning
      // English fallback for the others.
      expect(i18n.t('chrome.nav.dashboard', { ns: 'common' })).toBe('Panel');
      expect(i18n.t('button.cancel', { ns: 'bar' })).toBe('Cancelar');
      expect(i18n.t('step_titles.language', { ns: 'onboarding' })).toContain('idioma');
      expect(i18n.t('general.title', { ns: 'settings' })).toBe('General');
      expect(i18n.t('models.whisper_language.title', { ns: 'settings' })).toContain('Whisper');
    });

    it('App language description still distinguishes interface from dictation in Spanish', () => {
      // Spanish equivalent of the honesty contract pinned for English.
      // The picker's description must keep the user clear on which
      // setting governs which language; a future Spanish copy edit
      // that softens this loses the contract here too.
      const desc = String(
        i18n.getResource('es', 'settings', 'general.app_language.description'),
      );
      expect(desc).toContain('interfaz');
      expect(desc).toContain('dictado');
      expect(desc).toContain('Modelos');
    });

    it('renders chrome.update_toast.post_title with the version interpolated in Spanish', () => {
      // The placeholder-parity test guarantees `{{version}}` exists in
      // the Spanish string. This test additionally verifies the
      // substitution itself runs cleanly and the raw mustache template
      // never reaches the user.
      const out = i18n.t('chrome.update_toast.post_title', {
        ns: 'common',
        version: '0.21.0',
      });
      expect(out).toContain('0.21.0');
      expect(out).not.toContain('{{');
      expect(out).not.toContain('chrome.update_toast.post_title');
    });

    it('English remains available as a fallback safety net', () => {
      // i18next's fallbackLng is still 'en'. Adding Spanish must not
      // change that — if a future ES key goes missing somehow despite
      // the parity test, users see the English string rather than a
      // raw catalogue path. i18next normalises a string config into
      // a one-element array internally, so this test accepts either
      // representation as long as the resolved fallback is `en`.
      const fb = i18n.options.fallbackLng;
      const tag = Array.isArray(fb) ? fb[0] : fb;
      expect(tag).toBe('en');
    });
  });

  it('switching app language does not touch the dictation-language registry', () => {
    // Dictation language lives at models.whisper_language and uses the
    // word-name keys under common.language.* (e.g. language.spanish).
    // App language uses the BCP-47 tags (language.es). Pin the
    // separation: applyAppLanguage('es') must not mutate or alias the
    // word-name keys, so the dictation-language picker continues to
    // render its full registry list independently of the app's UI tag.
    applyAppLanguage('es');
    // The word-name keys still resolve in Spanish (just translated to
    // their Spanish word, not aliased to the BCP-47 tag).
    expect(i18n.getResource('es', 'common', 'language.spanish')).toBeTruthy();
    expect(i18n.getResource('es', 'common', 'language.english')).toBeTruthy();
    // The new BCP-47 self-name key is independent and stays "Español"
    // regardless of UI locale (universal convention: language pickers
    // show each language in its own script).
    expect(i18n.getResource('en', 'common', 'language.es')).toBe('Español');
    expect(i18n.getResource('es', 'common', 'language.es')).toBe('Español');
    applyAppLanguage('en');
  });
});

describe('Settings → General app-language picker contract', () => {
  // Source-grep guards. These read GeneralSettings.tsx as text so a
  // future refactor that breaks the picker's contract fails CI before
  // shipping. Pinned facts:
  //
  //   1. The APP_LANGUAGES array contains every review-batch locale
  //      (en, es, fr, de, it, pt, nl). This list MUST be kept in sync
  //      with the AppLanguage Rust enum and the i18next resources.
  //   2. The picker writes general.app_language, NOT
  //      models.whisper_language. The two settings are independent;
  //      conflating them was the single biggest risk flagged in the
  //      Spanish-pilot brief.
  //   3. The chip label is rendered through tCommon(`language.${...}`)
  //      so labels stay catalogue-routed rather than hard-coded.

  const SOURCE: string = (() => {
    // Read at module-import time so the test list reads cleanly. The
    // helper is private to this describe block.
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const fs = require('node:fs') as typeof import('node:fs');
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const path = require('node:path') as typeof import('node:path');
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const url = require('node:url') as typeof import('node:url');
    const here = path.dirname(url.fileURLToPath(import.meta.url));
    const target = path.resolve(here, '..', 'routes', 'settings', 'GeneralSettings.tsx');
    return fs.readFileSync(target, 'utf8');
  })();

  it('APP_LANGUAGES includes every review-batch locale', () => {
    // Every BCP-47 code in the review batch must literally appear as a
    // string element of the APP_LANGUAGES array. We also pin the array
    // length (7 codes including English) so a sneaky `as const`
    // expansion that adds an extra locale without updating the i18n
    // resources fails this assertion.
    const expected: ReadonlyArray<string> = ['en', 'es', 'fr', 'de', 'it', 'pt', 'nl'];
    for (const code of expected) {
      expect(
        SOURCE,
        `APP_LANGUAGES must include literal '${code}'`,
      ).toMatch(new RegExp(`['"]${code}['"]`));
    }
  });

  it('picker writes general.app_language, never models.whisper_language', () => {
    // Pin the write target. The picker MUST call update('app_language', ...)
    // and MUST NOT touch whisper_language from this surface.
    expect(SOURCE).toContain("update('app_language', code)");
    expect(SOURCE).not.toContain('whisper_language');
  });

  it('picker label is routed through the i18n catalogue', () => {
    // The chip label resolves through tCommon(`language.${code...}`)
    // rather than being hard-coded. A future refactor that hard-codes
    // 'English'/'Español' loses translation coverage on non-English UI.
    expect(SOURCE).toMatch(/tCommon\(`language\./);
  });
});

describe('i18n review-batch locales', () => {
  // Bulk parity coverage for every review-batch locale (es, fr, de,
  // it, pt, nl) against English. Every locale must:
  //   - have exactly the same key surface as English in every namespace
  //   - preserve every Mustache placeholder unchanged
  //   - keep baseline.sentences as an array with the same length
  //   - resolve representative keys cleanly with i18n switched to it
  //   - NOT corrupt the dictation-language registry when activated
  //
  // Adding a new locale to the review batch means appending a row to
  // BULK_LOCALES below. The parity tests below pick up the new locale
  // automatically; the smoke test needs a representative-key entry
  // added to SMOKE_KEYS.

  const BULK_LOCALES: ReadonlyArray<{
    readonly tag: string;
    readonly bar: unknown;
    readonly common: unknown;
    readonly onboarding: unknown;
    readonly settings: unknown;
  }> = [
    { tag: 'es', bar: barEs, common: commonEs, onboarding: onboardingEs, settings: settingsEs },
    { tag: 'fr', bar: barFr, common: commonFr, onboarding: onboardingFr, settings: settingsFr },
    { tag: 'de', bar: barDe, common: commonDe, onboarding: onboardingDe, settings: settingsDe },
    { tag: 'it', bar: barIt, common: commonIt, onboarding: onboardingIt, settings: settingsIt },
    { tag: 'pt', bar: barPt, common: commonPt, onboarding: onboardingPt, settings: settingsPt },
    { tag: 'nl', bar: barNl, common: commonNl, onboarding: onboardingNl, settings: settingsNl },
  ];

  const EN_BY_NS: Record<string, unknown> = {
    bar: barEn,
    common: commonEn,
    onboarding: onboardingEn,
    settings: settingsEn,
  };

  it.each(BULK_LOCALES)(
    'locale $tag has exactly the same key surface as English in every namespace',
    ({ tag, bar, common, onboarding, settings }) => {
      const locales: Record<string, unknown> = { bar, common, onboarding, settings };
      for (const ns of ['bar', 'common', 'onboarding', 'settings'] as const) {
        const enPaths = new Set(collectPaths(EN_BY_NS[ns]));
        const localePaths = new Set(collectPaths(locales[ns]));
        const missing = [...enPaths].filter((p) => !localePaths.has(p));
        const extra = [...localePaths].filter((p) => !enPaths.has(p));
        expect(missing, `${tag}/${ns}.json is missing keys present in English`).toEqual([]);
        expect(extra, `${tag}/${ns}.json has keys not present in English`).toEqual([]);
      }
    },
  );

  it.each(BULK_LOCALES)(
    'locale $tag preserves every Mustache placeholder from English',
    ({ tag, bar, common, onboarding, settings }) => {
      const locales: Record<string, unknown> = { bar, common, onboarding, settings };
      for (const ns of ['bar', 'common', 'onboarding', 'settings'] as const) {
        const leaves = collectStringLeaves(EN_BY_NS[ns]);
        for (const [path, enValue] of leaves) {
          const enPlaceholders = placeholdersOf(enValue);
          if (enPlaceholders.length === 0) continue;
          const localeValue = getByPath(locales[ns], path);
          expect(typeof localeValue, `${tag}/${ns}.${path} is not a string`).toBe('string');
          const localePlaceholders = placeholdersOf(String(localeValue));
          expect(
            localePlaceholders,
            `${tag}/${ns}.${path} placeholder set diverges from English (en=${JSON.stringify(enPlaceholders)} ${tag}=${JSON.stringify(localePlaceholders)})`,
          ).toEqual(enPlaceholders);
        }
      }
    },
  );

  it.each(BULK_LOCALES)(
    'locale $tag baseline.sentences has the same array length as English',
    ({ tag, onboarding }) => {
      const ob = onboarding as { baseline: { sentences: readonly string[] } };
      expect(
        ob.baseline.sentences.length,
        `${tag}/onboarding.baseline.sentences should match English length`,
      ).toBe(onboardingEn.baseline.sentences.length);
    },
  );

  // One representative key per namespace per locale. The smoke test
  // resolves these with i18n switched to the locale's tag, fails loudly
  // if any namespace returns the raw catalogue path (which is what
  // i18next does when a namespace is missing or fails to load).
  const SMOKE_KEYS: ReadonlyArray<{
    readonly tag: string;
    readonly nav: { ns: 'common'; key: string };
    readonly bar: { ns: 'bar'; key: string };
    readonly step: { ns: 'onboarding'; key: string };
    readonly settings: { ns: 'settings'; key: string };
    readonly whisper: { ns: 'settings'; key: string };
  }> = BULK_LOCALES.map(({ tag }) => ({
    tag,
    nav: { ns: 'common', key: 'chrome.nav.dashboard' },
    bar: { ns: 'bar', key: 'button.cancel' },
    step: { ns: 'onboarding', key: 'step_titles.language' },
    settings: { ns: 'settings', key: 'general.title' },
    whisper: { ns: 'settings', key: 'models.whisper_language.title' },
  }));

  it.each(SMOKE_KEYS)(
    'locale $tag resolves a representative key from every namespace cleanly',
    async ({ tag, nav, bar, step, settings, whisper }) => {
      // applyAppLanguage is sync but i18next defers the change to a
      // microtask; awaiting loadNamespaces synchronises before the
      // assertions.
      applyAppLanguage(tag as 'es' | 'fr' | 'de' | 'it' | 'pt' | 'nl');
      await i18n.loadNamespaces(['bar', 'common', 'onboarding', 'settings']);

      try {
        // Use getResource rather than i18n.t() to avoid the typed-i18next
        // overload's strict literal-key inference; we're passing keys
        // dynamically per locale. Equivalent for the smoke test: a
        // missing namespace or stale catalogue returns undefined rather
        // than the raw catalogue path.
        const samples: ReadonlyArray<{ readonly ns: string; readonly key: string }> = [
          nav,
          bar,
          step,
          settings,
          whisper,
        ];
        for (const { ns, key } of samples) {
          const out = i18n.getResource(tag, ns, key);
          expect(
            typeof out,
            `${tag}/${ns}.${key} should resolve to a string`,
          ).toBe('string');
          expect(
            (out as string).length,
            `${tag}/${ns}.${key} should resolve to a non-empty string`,
          ).toBeGreaterThan(0);
          // The raw key MUST NOT leak through. getResource returns the
          // value verbatim, so a leaked key would mean the catalogue
          // literally stored its own path, which the parity test should
          // already catch. Belt-and-braces here.
          expect(
            out,
            `${tag}/${ns}.${key} should not contain the raw catalogue path`,
          ).not.toBe(key);
        }
      } finally {
        applyAppLanguage('en');
      }
    },
  );

  it('every review-batch locale defines a self-name in common.language.<code>', () => {
    // The App-language picker chips render via
    // `tCommon(\`language.${code === 'en' ? 'english' : code}\`)`. So
    // the BCP-47-coded keys must exist in EVERY locale's catalogue (so
    // each chip renders the same self-name regardless of which UI
    // locale is currently active). English uses the legacy
    // `language.english` key; non-English locales use BCP-47 codes.
    const expectedCodes: ReadonlyArray<string> = ['es', 'fr', 'de', 'it', 'pt', 'nl'];
    for (const tag of ['en', ...expectedCodes]) {
      for (const code of expectedCodes) {
        const value = i18n.getResource(tag, 'common', `language.${code}`);
        expect(
          value,
          `${tag}/common.json must expose language.${code} (the BCP-47 self-name)`,
        ).toBeTruthy();
        expect(
          typeof value,
          `${tag}/common.json language.${code} must be a string`,
        ).toBe('string');
      }
    }
  });

  it('only registered locale folders exist on disk', () => {
    // Belt-and-braces: every directory under src/i18n/locales/ MUST be
    // a registered locale, and every registered locale MUST have a
    // matching directory. A stray "tr" folder on disk that hasn't been
    // wired up to i18next + AppLanguage + APP_LANGUAGES is exactly the
    // class of drift this test catches.
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const fs = require('node:fs') as typeof import('node:fs');
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const path = require('node:path') as typeof import('node:path');
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const url = require('node:url') as typeof import('node:url');
    const here = path.dirname(url.fileURLToPath(import.meta.url));
    const localesDir = path.resolve(here, 'locales');
    const onDisk = fs
      .readdirSync(localesDir, { withFileTypes: true })
      .filter((d) => d.isDirectory())
      .map((d) => d.name)
      .sort();
    const registered = ['en', 'es', 'fr', 'de', 'it', 'pt', 'nl'].sort();
    expect(onDisk).toEqual(registered);
  });

  it('every registered locale has all four required namespace files', () => {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const fs = require('node:fs') as typeof import('node:fs');
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const path = require('node:path') as typeof import('node:path');
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const url = require('node:url') as typeof import('node:url');
    const here = path.dirname(url.fileURLToPath(import.meta.url));
    const localesDir = path.resolve(here, 'locales');
    const requiredFiles = ['bar.json', 'common.json', 'onboarding.json', 'settings.json'];
    const tags = ['en', 'es', 'fr', 'de', 'it', 'pt', 'nl'];
    for (const tag of tags) {
      for (const file of requiredFiles) {
        const target = path.resolve(localesDir, tag, file);
        expect(
          fs.existsSync(target),
          `${tag}/${file} must exist`,
        ).toBe(true);
      }
    }
  });

  it('switching between locales does not corrupt the dictation-language registry', () => {
    // The dictation-language picker reads word-name keys
    // (language.spanish, language.french, etc). The App-language
    // picker reads BCP-47-coded keys (language.es, language.fr).
    // These are independent; switching app language must not alias or
    // mutate the word-name keys.
    for (const tag of ['en', 'es', 'fr', 'de', 'it', 'pt', 'nl']) {
      applyAppLanguage(tag as 'en' | 'es' | 'fr' | 'de' | 'it' | 'pt' | 'nl');
      // Word-name keys still resolve in the active locale.
      expect(i18n.getResource(tag, 'common', 'language.english')).toBeTruthy();
      expect(i18n.getResource(tag, 'common', 'language.spanish')).toBeTruthy();
      expect(i18n.getResource(tag, 'common', 'language.french')).toBeTruthy();
      // BCP-47 self-name keys stay in their own language.
      expect(i18n.getResource(tag, 'common', 'language.es')).toBe('Español');
      expect(i18n.getResource(tag, 'common', 'language.fr')).toBe('Français');
      expect(i18n.getResource(tag, 'common', 'language.de')).toBe('Deutsch');
      expect(i18n.getResource(tag, 'common', 'language.it')).toBe('Italiano');
      expect(i18n.getResource(tag, 'common', 'language.pt')).toBe('Português');
      expect(i18n.getResource(tag, 'common', 'language.nl')).toBe('Nederlands');
    }
    applyAppLanguage('en');
  });
});
