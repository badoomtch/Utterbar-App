/**
 * Remotion-side stub for `@tauri-apps/api/core`.
 *
 * The real app's components (most importantly `src/routes/FloatingBar.tsx`,
 * `src/state/settings.ts`) call `invoke()` on mount to talk to the Rust
 * backend (window-position polls, clipboard, clamp-floating-position, etc.).
 * In Remotion we have no Tauri runtime — the component still tries, we give
 * back a silent rejection so the caller's `try/catch` / `.catch(() => …)`
 * swallows it. Every callsite in FloatingBar is defensive; none of the
 * visual state depends on the return value.
 *
 * Wired through the webpack `resolve.alias` in `remotion.config.ts` so any
 * `import { invoke } from '@tauri-apps/api/core'` inside the shared app
 * source tree lands here during bundling.
 */
export async function invoke<T = unknown>(
  _cmd: string,
  _args?: Record<string, unknown>,
): Promise<T> {
  // Reject rather than resolve — the shipping app's callers reach for
  // `.catch(() => undefined)` on the results they don't need, and a
  // deliberate rejection keeps pathways like "fell through to fallback"
  // aligned with real-world failures.
  throw new Error('tauri-invoke-stub: not available in Remotion render');
}
