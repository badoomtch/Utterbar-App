import { create } from 'zustand';

import { invoke } from '@/lib/ipc';
import type { PolicySnapshot } from '@/generated/PolicySnapshot';

/**
 * Group-policy snapshot for the settings UI. The desktop reads
 * `HKLM\SOFTWARE\Policies\Utterbar` (with HKCU as a fallback) once at
 * boot; the snapshot is delivered via the `policy_snapshot` Tauri
 * command. The settings UI uses it to render a "Disabled by your
 * administrator" hint next to controls the IT admin has pinned.
 *
 * Restart-to-apply: the snapshot reflects whatever was in the
 * registry at app start. If the IT admin pushes a new policy the
 * user must restart Utterbar before the UI hints update — same
 * contract Windows itself has for most policies.
 *
 * Cosmetic only. The actual enforcement happens in the Rust backend
 * (e.g. `commands::history::export_history` refuses regardless of
 * what the UI shows). This store lets the UI explain the refusal in
 * advance instead of letting a click hit the IPC and bounce back as
 * an error.
 */
type PolicyState = {
  snapshot: PolicySnapshot | null;
  loaded: boolean;
  load: () => Promise<void>;
};

const EMPTY: PolicySnapshot = {
  update_channel_pinned: false,
  check_updates_pinned: false,
  model_downloads_blocked: false,
  system_audio_blocked: false,
  history_blocked: false,
  history_export_blocked: false,
  telemetry_blocked: false,
};

export const usePolicy = create<PolicyState>((set) => ({
  snapshot: null,
  loaded: false,
  load: async () => {
    try {
      const snap = await invoke<PolicySnapshot>('policy_snapshot');
      set({ snapshot: snap, loaded: true });
    } catch {
      // Defensive: if the command is missing for any reason (older
      // build, IPC boot race), fall back to "no policy" so the UI
      // renders unchanged.
      set({ snapshot: EMPTY, loaded: true });
    }
  },
}));

/** True if any policy is set; lets pages render a single page-level hint. */
export function isPolicyActive(p: PolicySnapshot | null): boolean {
  if (!p) return false;
  return (
    p.update_channel_pinned ||
    p.check_updates_pinned ||
    p.model_downloads_blocked ||
    p.system_audio_blocked ||
    p.history_blocked ||
    p.history_export_blocked ||
    p.telemetry_blocked
  );
}
