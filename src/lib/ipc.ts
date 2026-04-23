import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen, type EventCallback, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * Typed wrapper around Tauri invoke. Command names are listed here to centralise the
 * contract between Rust `#[tauri::command]` fns and the frontend.
 */
export const invoke = <T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> =>
  tauriInvoke<T>(cmd, args);

export const listen = <T = unknown>(event: string, handler: EventCallback<T>): Promise<UnlistenFn> =>
  tauriListen<T>(event, handler);
