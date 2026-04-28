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
// settings object is updated AND whenever modes are mutated
// (upsert_mode / delete_mode / duplicate_mode all re-emit the
// settings snapshot). We:
//   1. Pull the active id from the payload — cheap, no IPC needed.
//   2. Re-fetch the full modes list — the floating bar's quick-
//      switcher relies on `is_favourite` and on the list shape, so
//      a favourite toggle in one window must propagate.
// `load()` is two IPC calls; we only run it after a settings change
// (not on every render) so the cost is bounded by mutation frequency,
// which is small. Skipping the reload if the activeId-only delta is
// trivially safe is a future optimisation we don't need yet.
void listen<{ active_mode_id?: string | null }>(EVT.settingsChanged, (e) => {
  const next = e.payload?.active_mode_id ?? null;
  useModes.setState((s) => (s.activeId === next ? s : { ...s, activeId: next }));
  void useModes.getState().load();
});
