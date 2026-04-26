import { create } from 'zustand';

import { invoke } from '@/lib/ipc';
import type { UpdateInfo } from '@/generated/UpdateInfo';

/**
 * Shared update state. The toast and the Settings → General "Check for
 * updates now" button both read/write this store so a manual click and an
 * automatic on-focus check use the same in-flight gate, the same dismissal,
 * and the same "checked at" timestamp.
 */
type UpdateState = {
  info: UpdateInfo | null;
  dismissed: boolean;
  checking: boolean;
  /** ms since epoch of the last completed check, regardless of outcome. */
  lastCheckedAt: number | null;
  /** Outcome of the last manual check, used by the Settings button to
   *  display "you're up to date" / "update available" inline feedback. The
   *  background-on-mount checks don't set this — only explicit user clicks
   *  do, so the inline status reflects user intent rather than ambient. */
  lastManualOutcome: 'idle' | 'available' | 'up-to-date' | 'failed';
  /** When this launch is the first run after a successful in-app update,
   *  the Rust side returns the new version via `consume_post_update_marker`.
   *  We hold it here so the post-update toast can render "Updated to vX.Y.Z"
   *  with a "See what's new" link. Cleared on dismiss. Read-and-delete on
   *  the Rust side, so this only ever gets set once per update. */
  justUpdatedTo: string | null;
  check: (opts?: { manual?: boolean }) => Promise<void>;
  dismiss: () => void;
  consumePostUpdateMarker: () => Promise<void>;
  dismissPostUpdate: () => void;
};

export const useUpdate = create<UpdateState>((set, get) => ({
  info: null,
  dismissed: false,
  checking: false,
  lastCheckedAt: null,
  lastManualOutcome: 'idle',
  justUpdatedTo: null,

  check: async ({ manual = false } = {}) => {
    if (get().checking) return;
    set({
      checking: true,
      ...(manual ? { lastManualOutcome: 'idle' as const } : {}),
    });
    try {
      const u = await invoke<UpdateInfo>('check_update');
      const now = Date.now();
      if (u.available) {
        set({
          info: u,
          // A new version detected resurrects the toast even if the user
          // had dismissed an earlier one — the available version string
          // changing is the signal there's something new to show.
          dismissed:
            get().info?.available === u.available ? get().dismissed : false,
          checking: false,
          lastCheckedAt: now,
          lastManualOutcome: manual ? 'available' : get().lastManualOutcome,
        });
      } else {
        set({
          info: null,
          dismissed: false,
          checking: false,
          lastCheckedAt: now,
          lastManualOutcome: manual ? 'up-to-date' : get().lastManualOutcome,
        });
      }
    } catch {
      set({
        checking: false,
        lastCheckedAt: Date.now(),
        lastManualOutcome: manual ? 'failed' : get().lastManualOutcome,
      });
    }
  },

  dismiss: () => set({ dismissed: true }),

  // One-shot read of the Rust-side pending_relaunch.json marker. The Rust
  // side deletes the marker as part of the read, so calling this twice is
  // safe — the second call returns null. We still guard against double-
  // setting by only writing when the response is non-null.
  consumePostUpdateMarker: async () => {
    try {
      const v = await invoke<string | null>('consume_post_update_marker');
      if (v) set({ justUpdatedTo: v });
    } catch {
      // The IPC bridge is the only thing that can fail here; silently
      // ignore. A missing post-update toast is not worth a console error.
    }
  },

  dismissPostUpdate: () => set({ justUpdatedTo: null }),
}));
