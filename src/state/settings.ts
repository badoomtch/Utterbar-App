import { create } from 'zustand';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import type { Settings } from '@/generated/Settings';

// Re-export so the rest of the app can reach the type via the zustand module it's already
// importing. `Settings` itself is now fully generated from the Rust side by `ts-rs` into
// `src/generated/` — never hand-edit it here.
export type { Settings };

type SettingsStore = {
  data: Settings | null;
  load(): Promise<void>;
  patch(partial: Partial<Settings> | Record<string, unknown>): Promise<Settings>;
  reset(): Promise<Settings>;
};

export const useSettings = create<SettingsStore>((set) => ({
  data: null,
  async load() {
    const s = await invoke<Settings>('get_settings');
    set({ data: s });
    applyThemeClasses(s);
  },
  async patch(partial) {
    const updated = await invoke<Settings>('update_settings', { patch: partial });
    set({ data: updated });
    applyThemeClasses(updated);
    return updated;
  },
  async reset() {
    const s = await invoke<Settings>('reset_settings');
    set({ data: s });
    applyThemeClasses(s);
    return s;
  },
}));

export function applyThemeClasses(s: Settings) {
  const root = document.documentElement;
  root.classList.remove('theme-obsidian', 'theme-graphite', 'theme-paper');
  root.classList.add(`theme-${s.general.theme}`);
  root.classList.remove(
    'accent-teal',
    'accent-amber',
    'accent-violet',
    'accent-rose',
    'accent-cyan',
    'accent-lime',
    'accent-custom',
  );
  root.classList.add(`accent-${s.general.accent}`);

  // Custom accent (Pro). When `accent === 'custom'` we source the hex from
  // `general.custom_accent_hex` and set the CSS vars inline. We derive
  // `--accent-2` and `--accent-3` by tint/shade math so surfaces that use
  // the gradient (wordmark, pill glow, CTA buttons) still have depth
  // instead of reading as a single flat colour. Any non-`#RRGGBB` input
  // falls back to cyan so a corrupted settings.json can't brick the theme.
  if (s.general.accent === 'custom') {
    const hex =
      typeof s.general.custom_accent_hex === 'string'
        ? s.general.custom_accent_hex
        : '';
    const valid = /^#[0-9a-fA-F]{6}$/.test(hex) ? hex : '#22D3EE';
    root.style.setProperty('--accent', valid);
    root.style.setProperty('--accent-2', tintHex(valid, 0.35));
    root.style.setProperty('--accent-3', shadeHex(valid, 0.3));
    root.style.setProperty(
      '--gradient-wordmark',
      `linear-gradient(90deg, ${tintHex(valid, 0.25)} 0%, ${shadeHex(valid, 0.2)} 100%)`,
    );
  } else {
    // Clear the inline overrides when switching back to a preset so the
    // class-based cascade wins again. Without this, the last custom hex
    // would persist on top of e.g. `.accent-violet` and wash out the
    // preset colour.
    root.style.removeProperty('--accent');
    root.style.removeProperty('--accent-2');
    root.style.removeProperty('--accent-3');
    root.style.removeProperty('--gradient-wordmark');
  }
  if (s.general.theme === 'paper') root.classList.remove('dark');
  else root.classList.add('dark');

  // Drive the OS window chrome (native Windows title bar on main + onboarding)
  // off the app theme so Paper gets a light title bar and Obsidian/Graphite
  // keep the dark one. Without this, `tauri.conf.json`'s hardcoded
  // `"theme": "Dark"` pinned every window's caption to a charcoal strip that
  // read as "just greyish" on the cream Paper theme — the top bar looked
  // completely detached from the app's content.
  //
  // This has to run via the Tauri API at runtime because the config value is
  // only consulted at window-create time. Failures are swallowed because the
  // call is a no-op in the onboarding webview until the plugin has loaded,
  // and a best-effort theme sync should never block settings persistence.
  void getCurrentWindow()
    .setTheme(s.general.theme === 'paper' ? 'light' : 'dark')
    .catch(() => undefined);

  // UI scale: drive it through CSS `zoom` on <body>, not via root font-size.
  //
  // The earlier implementation set `root.style.fontSize = 16 * scale / 100`.
  // That scaled every Tailwind `rem` (padding, gap, width) but left absolute
  // px values (the `text-[14px]` form used throughout the app) untouched —
  // so at 175% the padding doubled while the text stayed the same size and
  // the layout read as broken, exactly as users reported ("buttons grow but
  // the text doesn't, it's all off"). Switching to `zoom` scales every
  // dimension — text, padding, borders, icons — uniformly, which is what
  // OS display-scale does and what the user expects.
  //
  // The floating pill opts out by carrying the `floating-shell` class on
  // <html>; we keep its zoom pinned to 1 so the bar stays the same physical
  // size regardless of this setting.
  const rawScale = Number(s.general.ui_scale ?? 100);
  const clamped = Number.isFinite(rawScale) ? Math.min(175, Math.max(90, rawScale)) : 100;
  // Identify the floating window via the URL route rather than the
  // `floating-shell` class, because `applyThemeClasses` can run during
  // settings hydration before FloatingBar's mount effect has had a chance
  // to attach that class. URL hash is set at window-open time by
  // tauri.conf.json's `url: "index.html#/floating"`, so it's always right.
  const isFloating = window.location.hash.startsWith('#/floating');
  // Clear the legacy font-size override in case an older build left one on
  // the root element — without this it would compound with the new zoom.
  root.style.fontSize = '';
  document.body.style.zoom = isFloating ? '1' : String(clamped / 100);

  // Accessibility flags — each is a body-level class hook that `globals.css` attaches
  // behaviour to. Toggled here so every window (main / floating / onboarding) picks them
  // up via the shared settings listener below.
  root.classList.toggle('a11y-high-contrast', !!s.general.high_contrast);
  root.classList.toggle('a11y-large-focus', !!s.general.larger_focus_rings);
  root.classList.toggle('a11y-no-transparency', !!s.general.reduce_transparency);
  root.classList.toggle('a11y-reduce-motion', !!s.general.reduce_motion);
}

// Keep the store in sync when the backend pushes changes.
void listen<Settings>(EVT.settingsChanged, (e) => {
  useSettings.setState({ data: e.payload });
  applyThemeClasses(e.payload);
});

// ----- Custom-accent colour math ---------------------------------------------
//
// The six preset accents each ship a hand-tuned `--accent` / `--accent-2` /
// `--accent-3` triple in tokens.css. A user-picked custom colour gives us
// one value — so we derive the other two by tinting (mix toward white) and
// shading (mix toward black). That's not as nice as a designer picking the
// stops, but it gives every custom pick a passable gradient instead of
// painting the whole accent family in one flat colour (which reads as
// "broken"). If we ever want to get fancy, swap this for a proper LCH
// rotation — oklab's tonal shifts would be noticeably better for reds /
// yellows where sRGB linear mixes muddy fast.

function clampByte(n: number): number {
  return Math.max(0, Math.min(255, Math.round(n)));
}

function parseHex(hex: string): [number, number, number] | null {
  if (!/^#[0-9a-fA-F]{6}$/.test(hex)) return null;
  return [
    parseInt(hex.slice(1, 3), 16),
    parseInt(hex.slice(3, 5), 16),
    parseInt(hex.slice(5, 7), 16),
  ];
}

function toHex(r: number, g: number, b: number): string {
  const h = (n: number) => clampByte(n).toString(16).padStart(2, '0');
  return `#${h(r)}${h(g)}${h(b)}`.toUpperCase();
}

/** Mix `hex` toward white by `ratio` (0..1). `ratio=0` returns the input. */
function tintHex(hex: string, ratio: number): string {
  const rgb = parseHex(hex);
  if (!rgb) return hex;
  const t = Math.max(0, Math.min(1, ratio));
  return toHex(
    rgb[0] + (255 - rgb[0]) * t,
    rgb[1] + (255 - rgb[1]) * t,
    rgb[2] + (255 - rgb[2]) * t,
  );
}

/** Mix `hex` toward black by `ratio` (0..1). `ratio=0` returns the input. */
function shadeHex(hex: string, ratio: number): string {
  const rgb = parseHex(hex);
  if (!rgb) return hex;
  const t = Math.max(0, Math.min(1, ratio));
  return toHex(rgb[0] * (1 - t), rgb[1] * (1 - t), rgb[2] * (1 - t));
}
