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
  );
  root.classList.add(`accent-${s.general.accent}`);
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
