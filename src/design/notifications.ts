import { useEffect } from 'react';

import { listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { useSettings } from '@/state/settings';

/**
 * System-level notifications for events the user should know about even when the main
 * window is minimised — dictation errors, failed model installs, and available updates.
 * We use the web `Notification` API rather than the Tauri plugin because it's already
 * available in the webview, doesn't need an extra capability entry, and gives us the
 * same native toast on Windows. Silent failure if the user has denied permission.
 */
export function useSystemNotifications() {
  const enabled = useSettings((s) => s.data?.general.notifications_enabled ?? true);

  useEffect(() => {
    if (!enabled) return;
    // Request permission once on mount; the browser remembers the answer so subsequent
    // calls are effectively free. Silent failure if the permission dialog is blocked.
    if ('Notification' in window && Notification.permission === 'default') {
      void Notification.requestPermission().catch(() => undefined);
    }

    const show = (title: string, body: string) => {
      if (!('Notification' in window)) return;
      if (Notification.permission !== 'granted') return;
      try {
        new Notification(title, { body, silent: true });
      } catch {
        // iced out by the OS (DND, focus assist) — nothing to do.
      }
    };

    const errP = listen<string>(EVT.dictationError, (e) => {
      show('Dictation error', typeof e.payload === 'string' ? e.payload : 'Something went wrong');
    });
    const progP = listen<{ phase: string; model_id: string }>(EVT.modelProgress, (e) => {
      if (e.payload.phase === 'error') {
        show('Model install failed', `Couldn't finish installing ${e.payload.model_id}. Check the Models page for details.`);
      } else if (e.payload.phase === 'done') {
        show('Model ready', `${e.payload.model_id} is installed and ready to use.`);
      }
    });

    return () => {
      void errP.then((u) => u()).catch(() => undefined);
      void progP.then((u) => u()).catch(() => undefined);
    };
  }, [enabled]);
}
