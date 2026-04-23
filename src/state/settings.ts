import { create } from 'zustand';
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

  // UI scale: drive the root `html` font-size, which scales every Tailwind `rem` unit
  // across the app — padding, gaps, font sizes, buttons, everything. The floating bar
  // opts out by carrying a `floating-shell` class that pins its own font-size, so the
  // pill stays the same physical size whatever the user has set here.
  const rawScale = Number(s.general.ui_scale ?? 100);
  const clamped = Number.isFinite(rawScale) ? Math.min(175, Math.max(90, rawScale)) : 100;
  // 16 px is the browser default, our Tailwind `rem` baseline.
  root.style.fontSize = `${(16 * clamped) / 100}px`;

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
