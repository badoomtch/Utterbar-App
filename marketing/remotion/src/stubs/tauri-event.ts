/**
 * Remotion-side stub for `@tauri-apps/api/event`.
 *
 * `src/state/settings.ts` registers a `listen(EVT.settingsChanged, …)`
 * handler at module scope. In Remotion there's no IPC emitter, so we
 * hand back a noop unlisten and never fire the callback.
 *
 * Wired via webpack `resolve.alias` in `remotion.config.ts`.
 */
type EventCallback<T> = (event: { payload: T }) => void;
type UnlistenFn = () => void;

export async function listen<T>(
  _event: string,
  _handler: EventCallback<T>,
): Promise<UnlistenFn> {
  return () => undefined;
}

export type { EventCallback, UnlistenFn };
