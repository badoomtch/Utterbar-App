import { create } from 'zustand';
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';

export type Mode = {
  id: string;
  name: string;
  icon: string;
  is_builtin: boolean;
  is_favourite: boolean;
  sort_order: number;
  asr_model_id: string | null;
  target_language: string | null;
  is_meeting_mode?: boolean;
};

type ModesStore = {
  list: Mode[];
  activeId: string | null;
  load(): Promise<void>;
  setActive(id: string): Promise<void>;
};

export const useModes = create<ModesStore>((set) => ({
  list: [],
  activeId: null,
  async load() {
    const [list, active] = await Promise.all([
      invoke<Mode[]>('list_modes'),
      invoke<Mode | null>('get_active_mode').catch(() => null),
    ]);
    set({ list, activeId: active?.id ?? null });
  },
  async setActive(id) {
    await invoke('set_active_mode', { id });
    set({ activeId: id });
  },
}));

// Cross-window sync. Settings emits `settings:changed` whenever the
// settings object is updated, including via `set_active_mode`. We
// re-pull the active id from the payload so a mode switch in one
// window (e.g. the floating bar's quick switcher) is reflected in
// every other open window (Modes route, etc.) without each one having
// to subscribe individually. The full mode list is loaded lazily; if
// it hasn't been loaded yet, the snapshot here just updates activeId.
void listen<{ active_mode_id?: string | null }>(EVT.settingsChanged, (e) => {
  const next = e.payload?.active_mode_id ?? null;
  useModes.setState((s) => (s.activeId === next ? s : { ...s, activeId: next }));
});
