import { create } from 'zustand';
import { invoke } from '@/lib/ipc';

export type Mode = {
  id: string;
  name: string;
  icon: string;
  is_builtin: boolean;
  is_favourite: boolean;
  sort_order: number;
  asr_model_id: string | null;
  target_language: string | null;
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
