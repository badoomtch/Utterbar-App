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
    expect(Object.keys(onboardingEn.welcome).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.model).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.model.download).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.gpu).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.baseline).length).toBeGreaterThan(0);
    expect(Object.keys(onboardingEn.ready).length).toBeGreaterThan(0);
    expect(onboardingEn.baseline.sentences.length).toBeGreaterThan(0);
  });
});
