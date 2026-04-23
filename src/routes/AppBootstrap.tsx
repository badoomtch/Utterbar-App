import { useEffect, useRef, type ReactNode } from 'react';
import { MotionConfig } from 'framer-motion';

import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { useDictation, type DictationPhase, type MeterFrame } from '@/state/dictation';
import { useSettings } from '@/state/settings';
import { attachLicenceListener, useLicence } from '@/state/licence';
import { useSoundCues } from '@/design/sounds';
import { useSystemNotifications } from '@/design/notifications';

type ShortcutId =
  | 'toggle_recording'
  | 'push_to_talk'
  | 'hold_to_talk'
  | 'cancel_recording'
  | 'cycle_mode'
  | 'open_settings'
  | 'open_history';

export function AppBootstrap({ children }: { children: ReactNode }) {
  const loadSettings = useSettings((s) => s.load);
  const setPhase = useDictation((s) => s.setPhase);
  const setMeter = useDictation((s) => s.setMeter);
  const loadLicence = useLicence((s) => s.load);
  const reduceMotion = useSettings((s) => s.data?.general.reduce_motion ?? false);

  useSoundCues();
  useSystemNotifications();

  useEffect(() => {
    void loadSettings();
  }, [loadSettings]);

  // Licence: pull the current tier once on mount and subscribe to the
  // `licence_changed` event so the store stays in sync with Rust through
  // activate / deactivate / background heartbeat revocations.
  useEffect(() => {
    void loadLicence();
    const p = attachLicenceListener();
    return () => {
      void p.then((u) => u()).catch(() => undefined);
    };
  }, [loadLicence]);

  // Tag <html> with the reduce-motion flag so globals.css can kill transitions, and feed
  // the same value into Framer Motion's context so its springs respect the user's
  // accessibility preference without each component having to opt in.
  useEffect(() => {
    const root = document.documentElement;
    if (reduceMotion) {
      root.setAttribute('data-reduce-motion', 'true');
    } else {
      root.removeAttribute('data-reduce-motion');
    }
  }, [reduceMotion]);

  // Hotkey debounce + in-flight latch. Fixes two different double-fire sources:
  //   1) Mouse buttons (especially cheap gaming mice / AutoHotKey rebinds) that bounce
  //      the key down event inside ~30ms, producing two global-shortcut pressed events
  //      back-to-back.
  //   2) The genuine race where the user presses a second time before `start_recording`
  //      has finished its async dance and set phase → Listening. Reading
  //      `useDictation.getState().phase` alone isn't enough because the state update is
  //      gated behind the Tauri event round-trip.
  // 180ms is short enough to be imperceptible for a deliberate stop-then-start tap, long
  // enough to swallow any physical or software bounce we've seen in the wild.
  const lastHotkeyAtRef = useRef<number>(0);
  const hotkeyInFlightRef = useRef<boolean>(false);

  useEffect(() => {
    const unlisteners: Array<Promise<() => void>> = [
      listen<DictationPhase>(EVT.dictationPhase, (e) => setPhase(e.payload)),
      listen<MeterFrame>(EVT.dictationMeter, (e) => setMeter(e.payload)),
      listen<ShortcutId>(EVT.shortcutTriggered, async (e) => {
        const id = typeof e.payload === 'string' ? e.payload : undefined;
        switch (id) {
          case 'toggle_recording': {
            const now = performance.now();
            if (hotkeyInFlightRef.current) return;
            if (now - lastHotkeyAtRef.current < 180) return;
            lastHotkeyAtRef.current = now;
            hotkeyInFlightRef.current = true;
            try {
              const current = useDictation.getState().phase;
              if (current.kind === 'listening') {
                await invoke('stop_recording');
              } else {
                await invoke('start_recording', { deviceId: null });
              }
            } finally {
              hotkeyInFlightRef.current = false;
            }
            break;
          }
          case 'cancel_recording':
            await invoke('cancel_recording');
            break;
          case 'open_settings':
            await invoke('show_main_at', { path: '/settings/general' });
            break;
          case 'open_history':
            await invoke('show_main_at', { path: '/history' });
            break;
          default:
            break;
        }
      }),
      listen(EVT.trayStartRecording, () => void invoke('start_recording', { deviceId: null })),
    ];
    return () => {
      for (const p of unlisteners) void p.then((u) => u()).catch(() => undefined);
    };
  }, [setPhase, setMeter]);

  return (
    <MotionConfig reducedMotion={reduceMotion ? 'always' : 'never'}>{children}</MotionConfig>
  );
}
