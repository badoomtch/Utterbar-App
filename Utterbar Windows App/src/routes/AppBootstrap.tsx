import { useEffect, useRef, type ReactNode } from 'react';
import { MotionConfig } from 'framer-motion';

import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { useDictation, type DictationPhase, type MeterFrame } from '@/state/dictation';
import { useModes } from '@/state/modes';
import { useSettings } from '@/state/settings';
import { attachLicenceListener, useLicence } from '@/state/licence';
import { usePolicy } from '@/state/policy';
import { useSoundCues } from '@/design/sounds';
import { useSystemNotifications } from '@/design/notifications';
import { applyAppLanguage } from '@/i18n';

import type { ShortcutId } from '@/generated/ShortcutId';

/**
 * Identify the current webview by hash. Each of `main`/`floating`/`onboarding`
 * loads `main.tsx`, so every hook and listener in `AppBootstrap` would otherwise
 * run three times. Gate anything that must only happen once (sound cues, system
 * notifications, shortcut-dispatching `invoke()` calls) on `isMainWindow()`.
 */
function isMainWindow(): boolean {
  const h = typeof window !== 'undefined' ? window.location.hash : '';
  return (
    !h.startsWith('#/floating') &&
    !h.startsWith('#/onboarding') &&
    !h.startsWith('#/vocab-add')
  );
}

export function AppBootstrap({ children }: { children: ReactNode }) {
  const loadSettings = useSettings((s) => s.load);
  const setPhase = useDictation((s) => s.setPhase);
  const setMeter = useDictation((s) => s.setMeter);
  const setMeetingProgress = useDictation((s) => s.setMeetingProgress);
  const setLastText = useDictation((s) => s.setLastText);
  const loadLicence = useLicence((s) => s.load);
  const loadPolicy = usePolicy((s) => s.load);
  const reduceMotion = useSettings((s) => s.data?.general.reduce_motion ?? false);
  const appLanguage = useSettings((s) => s.data?.general.app_language);
  const isMain = isMainWindow();

  // Sound cues and system notifications are single-source by design (no amount of
  // per-hook `hotkeyInFlightRef` latching fixes the multi-window fan-out because
  // each window has its own React state). These hooks early-return in non-main
  // windows, so the calls here are safe no-ops there.
  useSoundCues();
  useSystemNotifications();

  useEffect(() => {
    void loadSettings();
  }, [loadSettings]);

  // Group-policy snapshot is a one-shot read at boot: the registry is
  // resolved once on the Rust side at app start, so refetching across
  // a session never returns new data. Loading it here means every
  // settings page can read `usePolicy().snapshot` synchronously
  // without each one re-invoking.
  useEffect(() => {
    void loadPolicy();
  }, [loadPolicy]);

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

  // Keep i18next's active language in sync with the user's saved
  // preference. Runs once on hydration (settings load is asynchronous, so
  // `appLanguage` flips from `undefined` to the persisted value) and again
  // on every subsequent change. `applyAppLanguage` is a no-op when the
  // language hasn't actually changed.
  useEffect(() => {
    applyAppLanguage(appLanguage);
  }, [appLanguage]);

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
  // Hold-to-record (push/hold-to-talk) state, tracked independently of
  // `phase` so a release that arrives while the cold first `start_recording`
  // is still arming (phase not yet 'listening') still stops the recording.
  const holdActiveRef = useRef<boolean>(false);
  const holdStartingRef = useRef<boolean>(false);
  const releaseRequestedRef = useRef<boolean>(false);

  useEffect(() => {
    const unlisteners: Array<Promise<() => void>> = [
      listen<DictationPhase>(EVT.dictationPhase, (e) => {
        // v0.6.6 diagnostics. Log every phase event as received. When the
        // frontend store desyncs from Rust (stuck-hotkey bug), the log
        // previously went silent on the JS side and we couldn't tell
        // whether the event reached the webview at all or was dropped by
        // the zustand store. The console message is the missing link.
        // Low volume — one line per phase change is fine.
        console.info('[utterbar] phase <-', e.payload.kind);
        setPhase(e.payload);
      }),
      listen<MeterFrame>(EVT.dictationMeter, (e) => setMeter(e.payload)),
      // Meeting Mode chunk progress. Payload shape matches the Rust
      // `MeetingChunkProgress` struct in domain/events.rs. Coarse —
      // one event per chunk boundary, so a 5-minute meeting at the
      // 27 s stride emits ~11 events total.
      listen<{ chunk_index: number; total_chunks: number }>(
        EVT.dictationChunkProgress,
        (e) => {
          setMeetingProgress({
            chunkIndex: e.payload.chunk_index,
            totalChunks: e.payload.total_chunks,
          });
        },
      ),
      // Keep `lastText` in sync with the most recent successful dictation.
      // Powers the floating bar's "Copy last message" button — the button
      // only needs a store read, no SQLite round-trip.
      listen<{ processed_text?: string; raw_text?: string }>(EVT.dictationResult, (e) => {
        const txt = e.payload.processed_text || e.payload.raw_text || '';
        if (txt) setLastText(txt);
      }),
      listen<ShortcutId>(EVT.shortcutTriggered, async (e) => {
        // Shortcut dispatch must only run in the main window. Tauri `emit`
        // broadcasts to every webview, so without this guard each of the 3
        // windows would call `start_recording`/`stop_recording` in parallel.
        if (!isMain) return;
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
              // v0.6.6 diagnostics: log the decision the hotkey made so a
              // stuck-hotkey trace includes whether the store's phase was
              // telling us to start or stop. Cheap — one line per press.
              console.info(
                '[utterbar] hotkey toggle_recording: phase.kind =',
                current.kind,
                '-> will',
                current.kind === 'listening' ? 'STOP' : 'START',
              );
              if (current.kind === 'listening') {
                await invoke('stop_recording');
              } else {
                // v0.6.6: stuck-state recovery.
                //
                // Symptom observed 2026-04-24T17:59: the frontend store's
                // `phase` falls out of sync with the Rust side — phase is
                // e.g. `idle`/`success`/`processing` while Rust still has
                // `recording_in_progress=true` from a previous start whose
                // stop_recording event the frontend missed. Every hotkey
                // press routes through the `else` branch (not listening →
                // start), which Rust then rejects with
                // "recording already in progress". The user sees nothing.
                //
                // Before v0.6.6, there was no client-side recovery: the
                // user had to quit and relaunch. Now: if start_recording
                // rejects with that specific message, force a
                // stop_recording to clear Rust's live capture, nudge the
                // phase back to idle locally, and log the event. Next
                // press lands clean. A real "recording already in
                // progress" (legitimate rapid second press against an
                // ongoing capture) is indistinguishable at this layer —
                // but the 180ms debounce above already blocks those, so
                // in practice this path only fires on genuine desync.
                try {
                  await invoke('start_recording', { deviceId: null });
                } catch (err) {
                  const msg = String(
                    (err as { message?: string } | null)?.message ?? err ?? '',
                  );
                  if (msg.toLowerCase().includes('already in progress')) {
                    console.warn(
                      '[utterbar] start_recording rejected as busy but frontend phase is not Listening — desync detected, running recovery stop_recording',
                      { phase: current.kind, error: msg },
                    );
                    try {
                      await invoke('stop_recording');
                    } catch (stopErr) {
                      console.warn(
                        '[utterbar] recovery stop_recording also failed',
                        stopErr,
                      );
                    }
                    // Force the store back to Idle so the NEXT hotkey
                    // press is guaranteed to hit the start branch.
                    useDictation.getState().setPhase({ kind: 'idle' });
                  } else {
                    // Any other error from start_recording — rethrow so
                    // the outer catch surfaces it in the console, phase
                    // stays as-is.
                    throw err;
                  }
                }
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
          case 'add_to_vocabulary':
            // Capture the foreground app's selection (synthesise Ctrl+C,
            // read clipboard, restore) and open the small quick-add window.
            // The Rust side handles the entire workflow; we just kick it.
            await invoke('open_vocab_quick_add').catch((err) => {
              console.warn('[utterbar] vocab quick-add failed', err);
            });
            break;
          case 'push_to_talk':
          case 'hold_to_talk': {
            // Hold-to-record: key-down starts, key-up (shortcut:released)
            // stops. Track the hold explicitly so a release that lands
            // while the cold first start is still arming still stops it.
            if (holdActiveRef.current) break;
            if (useDictation.getState().phase.kind === 'listening') break;
            holdActiveRef.current = true;
            holdStartingRef.current = true;
            releaseRequestedRef.current = false;
            try {
              await invoke('start_recording', { deviceId: null });
            } catch (err) {
              console.warn('[utterbar] push/hold-to-talk start failed', err);
              holdActiveRef.current = false;
              holdStartingRef.current = false;
              releaseRequestedRef.current = false;
              break;
            }
            holdStartingRef.current = false;
            // The key may have been released while we were still starting;
            // honour that deferred release now.
            if (releaseRequestedRef.current) {
              releaseRequestedRef.current = false;
              holdActiveRef.current = false;
              await invoke('stop_recording').catch((err) =>
                console.warn('[utterbar] push/hold-to-talk deferred stop failed', err),
              );
            }
            break;
          }
          case 'cycle_mode': {
            // Switch to the next mode by sort order, wrapping around.
            const { list, activeId, setActive } = useModes.getState();
            if (list.length === 0) break;
            const sorted = [...list].sort((a, b) => a.sort_order - b.sort_order);
            const idx = sorted.findIndex((m) => m.id === activeId);
            const next = sorted[(idx + 1) % sorted.length];
            if (next && next.id !== activeId) {
              await setActive(next.id).catch((err) =>
                console.warn('[utterbar] cycle_mode failed', err),
              );
            }
            break;
          }
          default:
            break;
        }
      }),
      // Key-up for the hold-to-record styles. Only push/hold-to-talk act
      // on release; every other shortcut is a one-shot on key-down.
      listen<ShortcutId>(EVT.shortcutReleased, async (e) => {
        if (!isMain) return;
        const id = typeof e.payload === 'string' ? e.payload : undefined;
        if (id !== 'push_to_talk' && id !== 'hold_to_talk') return;
        if (!holdActiveRef.current) return;
        // If the start is still arming (cold first press), defer the stop to
        // when start_recording resolves — otherwise we'd try to stop a
        // recording that hasn't finished starting, and the release would be
        // dropped (the bug this fixes).
        if (holdStartingRef.current) {
          releaseRequestedRef.current = true;
          return;
        }
        holdActiveRef.current = false;
        try {
          await invoke('stop_recording');
        } catch (err) {
          console.warn('[utterbar] push/hold-to-talk stop failed', err);
        }
      }),
      listen(EVT.trayStartRecording, () => void invoke('start_recording', { deviceId: null })),
    ];
    return () => {
      for (const p of unlisteners) void p.then((u) => u()).catch(() => undefined);
    };
  }, [setPhase, setMeter, setMeetingProgress, setLastText, isMain]);

  return (
    <MotionConfig reducedMotion={reduceMotion ? 'always' : 'never'}>{children}</MotionConfig>
  );
}
