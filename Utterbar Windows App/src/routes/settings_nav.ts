import type { TFunction } from 'i18next';
import {
  Accessibility,
  Info,
  KeyRound,
  Keyboard,
  Mic,
  Minimize2,
  Radio,
  RefreshCw,
  Settings2,
  ShieldCheck,
  SlidersHorizontal,
  type LucideIcon,
} from 'lucide-react';

import type { SettingsSearchMatch } from '@/routes/settings_search';

/**
 * Settings navigation registry. Splits the static parts of the nav data
 * (route, icon, search keywords, per-control aliases) from the
 * translatable parts (section label, match display label) so the
 * `settings_search.ts` helpers stay untouched.
 *
 * Keywords and aliases remain English-only on purpose: they are search
 * haystack terms, not display strings. When a future locale ships its
 * own aliases, they can be appended alongside the English ones rather
 * than replacing them, keeping the multi-token AND search compatible
 * across locales.
 */

type SectionSpec = {
  readonly to: string;
  /** Stable id used as the translation key under `nav.<id>.label`. */
  readonly id: string;
  readonly icon: LucideIcon;
  /** Section-level keywords for the search input. English-only on
   *  purpose (search haystack). */
  readonly keywords: readonly string[];
  /** Map of stable match id (translation key under
   *  `nav.<section>.matches.<id>`) to the search aliases for that
   *  control. An empty array means "no aliases" — equivalent to the
   *  historical bare-string `SettingsSearchMatch` variant. Insertion
   *  order is the surface order, which `settingsSearchMatchSummary`
   *  relies on when it picks up to two hits. */
  readonly matches: Readonly<Record<string, readonly string[]>>;
};

/**
 * Static registry. Order is the visible nav order — keep stable.
 * Aliases mirror the historical `MainShell.tsx` lists verbatim so the
 * existing English search examples (`smart rewrite` → Smart rewrite,
 * `teams mute` → Mute conferencing app output, etc.) continue to land
 * on the same matches. Updating an alias here is the same surface as
 * the old hardcoded array.
 */
const SECTIONS: readonly SectionSpec[] = [
  {
    to: '/settings/general',
    id: 'general',
    icon: Settings2,
    keywords: [
      'theme',
      'accent',
      'colour',
      'color',
      'minimised',
      'minimized',
      'tray',
      'sound',
      'notification',
      'reduce motion',
    ],
    matches: {
      theme: [],
      accent: ['accent', 'colour', 'color'],
      launch_at_startup: ['launch', 'startup', 'autostart', 'auto start', 'boot'],
      start_minimised: ['minimised', 'minimized'],
      show_in_tray: ['tray'],
      notification_sound: ['sound', 'notification', 'beep'],
      app_language: ['language', 'locale'],
    },
  },
  {
    to: '/settings/bar',
    id: 'bar',
    icon: Minimize2,
    keywords: ['floating bar', 'pill'],
    matches: {
      position: ['position', 'placement', 'bar location'],
      auto_hide: ['auto-hide', 'auto hide', 'hide', 'visibility'],
      background: ['background'],
    },
  },
  {
    to: '/settings/audio',
    id: 'audio',
    icon: Mic,
    keywords: ['noise', 'silence', 'normalisation', 'normalization', 'loopback'],
    matches: {
      microphone: ['mic', 'microphone', 'input device', 'capture'],
      input_device: ['device', 'input'],
      noise_gate: ['gate', 'noise'],
      gain: ['gain', 'volume', 'level'],
      sample_rate: ['sample rate', 'rate', 'hz'],
      system_audio: ['system audio', 'system', 'desktop audio', 'pc audio', 'loopback'],
    },
  },
  {
    to: '/settings/models',
    id: 'models',
    icon: Radio,
    keywords: ['ASR', 'language'],
    matches: {
      default_dictation_model: ['default model', 'dictation model', 'whisper', 'parakeet'],
      meeting_mode_model: ['meeting model', 'meeting mode', 'meeting'],
      model_downloads: ['download', 'install model', 'install'],
      keep_model_warm: ['warm', 'unload', 'keep loaded', 'preload'],
      whisper: ['whisper'],
      parakeet: ['parakeet'],
    },
  },
  {
    to: '/settings/shortcuts',
    id: 'shortcuts',
    icon: Keyboard,
    keywords: ['keybind', 'global shortcut'],
    matches: {
      toggle_recording_hotkey: [
        'hotkey',
        'shortcut',
        'toggle',
        'toggle recording',
        'record',
        'start record',
        'stop record',
      ],
      cancel_recording_hotkey: ['cancel', 'cancel recording', 'abort', 'cancel hotkey'],
    },
  },
  {
    to: '/settings/text-output',
    id: 'text_output',
    icon: SlidersHorizontal,
    keywords: ['punctuation', 'keystroke fallback'],
    matches: {
      paste_automatically: ['paste', 'auto paste', 'clipboard'],
      restore_clipboard: ['restore clipboard', 'clipboard'],
      type_out_if_blocked: [
        'paste blocked',
        'fallback',
        'keystroke',
        'type out',
        'character by character',
      ],
      auto_capitalise: ['capitalise', 'capitalize', 'auto capital', 'sentence case'],
      trim_trailing_whitespace: ['trim', 'whitespace', 'trailing space'],
      auto_space: ['auto space', 'spacing'],
      numbers_as_digits: ['numbers', 'digits', 'numerals'],
      smart_rewrite: [
        'smart rewrite',
        'rewrite',
        'filler',
        'filler words',
        'fillers',
        'um',
        'uh',
        'tidy',
        'cleanup',
        'code dictation',
        'code',
        'polish',
      ],
      hold_shift_auto_send: ['shift', 'auto-send', 'auto send', 'send', 'enter'],
    },
  },
  {
    to: '/settings/accessibility',
    id: 'accessibility',
    icon: Accessibility,
    keywords: ['screen reader', 'a11y'],
    matches: {
      reduce_motion: ['reduce motion', 'motion', 'animation'],
      high_contrast: ['high contrast', 'contrast'],
      text_size: ['text size', 'font size', 'size', 'large text'],
      tooltips: ['tooltips', 'hints', 'tips'],
    },
  },
  {
    to: '/settings/privacy',
    id: 'privacy',
    icon: ShieldCheck,
    keywords: ['retain audio', 'recordings', 'history'],
    matches: {
      save_recordings: ['save recording', 'save audio', 'keep audio', 'retain audio', 'recording'],
      delete_after: ['delete', 'auto delete', 'retention'],
      active_app_context: [
        'active app',
        'active-app',
        'app context',
        'focused app',
        'foreground app',
      ],
      selection_context: ['selection', 'selected text'],
      clipboard_context: ['clipboard', 'clipboard context'],
      pause_music: ['pause music', 'spotify', 'media', 'pause media'],
      mute_conferencing_output: [
        'mute',
        'mute output',
        'conferencing',
        'teams',
        'discord',
        'zoom',
        'slack',
        'webex',
        'call',
        'teams mute',
        'discord mute',
        'zoom mute',
      ],
      apps_to_silence: ['apps to silence', 'silence apps', 'silence list', 'app list'],
      data_locations: ['data location', 'where', 'storage', 'appdata', 'paths'],
    },
  },
  {
    to: '/settings/advanced',
    id: 'advanced',
    icon: SlidersHorizontal,
    keywords: ['experimental'],
    matches: {
      gpu_acceleration: ['gpu', 'acceleration', 'directml', 'graphics card'],
      directml: ['directml', 'dml'],
      cpu_only: ['cpu', 'cpu only', 'no gpu'],
      mute_mic_teams: [
        'teams mute',
        'teams mic',
        'mute teams microphone',
        'experimental teams',
        'mic mute teams',
      ],
      mute_mic_discord: [
        'discord mute',
        'discord mic',
        'mute discord microphone',
        'experimental discord',
        'mic mute discord',
      ],
      diagnostics: ['diagnostics', 'logs', 'debug'],
      export_data: ['export', 'export data', 'export settings', 'backup'],
      import_backup: ['import', 'import backup', 'import settings', 'restore'],
      reset_settings: ['reset', 'reset settings', 'factory reset', 'defaults'],
    },
  },
  {
    to: '/settings/licence',
    id: 'licence',
    icon: KeyRound,
    keywords: ['license', 'tier', 'free'],
    matches: {
      activate_pro: ['activate', 'activation', 'pro', 'upgrade', 'enter key'],
      licence_key: ['licence key', 'license key', 'key'],
      devices: ['devices', 'machines', 'seats', 'device cap'],
      managed_licence_bin: ['managed', 'licence.bin', 'license.bin', 'fleet', 'b2b', 'enterprise'],
      air_gapped_activation: [
        'air-gapped',
        'air gapped',
        'airgapped',
        'offline activation',
        'no internet',
      ],
    },
  },
  {
    to: '/settings/updates',
    id: 'updates',
    icon: RefreshCw,
    keywords: ['auto-check'],
    matches: {
      update_channel: ['channel', 'update channel'],
      check_for_updates: ['check for updates', 'check updates', 'check', 'update'],
      beta_channel: ['beta', 'beta channel'],
      stable_channel: ['stable', 'stable channel'],
      release_notes: ['release notes', 'changelog', 'whats new', "what's new"],
    },
  },
  {
    to: '/settings/about',
    id: 'about',
    icon: Info,
    keywords: [],
    matches: {
      version: ['version', 'build'],
      credits: ['credits', 'authors', 'attribution'],
      website_links: ['website', 'links', 'docs', 'support'],
    },
  },
];

export type NavSettingsItem = {
  readonly to: string;
  readonly label: string;
  readonly icon: LucideIcon;
  readonly keywords: readonly string[];
  readonly matches: readonly SettingsSearchMatch[];
};

/**
 * Build the Settings nav registry against the active i18n language.
 *
 * Section labels and per-match display labels resolve through `t()` so a
 * future locale flip swaps them in without restart. Keywords and
 * aliases come from the static `SECTIONS` registry above and remain
 * English; they're search-haystack terms rather than display strings.
 *
 * The shape of the returned array is identical to the historical
 * `NAV_SETTINGS` const, so `filterSettingsByQuery` /
 * `settingsSearchMatchSummary` consume it without modification.
 */
export function buildSettingsNav(t: TFunction<'settings'>): readonly NavSettingsItem[] {
  // Both template-literal keys below are valid entries in settings.json
  // (the i18n.test.ts companion test pins every one), but i18next's
  // typed `t()` only accepts the closed union of literal keys it can
  // statically prove from the JSON resource. Building keys with
  // `${id}` interpolation widens to `string` and trips the type
  // checker. The cast funnels each call through a single helper so
  // the boundary stays explicit and easy to grep.
  const tKey = (key: string): string => (t as unknown as (k: string) => string)(key);

  return SECTIONS.map((section) => {
    const matches: SettingsSearchMatch[] = Object.entries(section.matches).map(
      ([matchId, aliases]) => {
        const label = tKey(`nav.${section.id}.matches.${matchId}`);
        // Preserve the legacy `string` shape when there are no aliases —
        // `settings_search.ts` accepts both `string` and `{label, aliases}`
        // and treats the bare-string variant as "no aliases", which keeps
        // the behaviour identical for matches that only ever hit on their
        // exact label (currently only General → Theme).
        return aliases.length > 0 ? { label, aliases } : label;
      },
    );

    return {
      to: section.to,
      label: tKey(`nav.${section.id}.label`),
      icon: section.icon,
      keywords: section.keywords,
      matches,
    };
  });
}
