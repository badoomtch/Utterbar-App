import { create } from 'zustand';

import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import type { LicenceStatus } from '@/generated/LicenceStatus';

/**
 * Error slugs surfaced by `OrliqError::Licence { code, message }` on the Rust
 * side. Kept in lockstep with `src-tauri/src/util/result.rs` — the UI matches
 * on these to render the right inline message per the UX brief:
 *
 *   malformed            → inline under the paste field
 *   not_found            → "couldn't find that licence"
 *   revoked              → banner (coloured) with a mailto: to support
 *   device_limit         → show device list + link to utterbar.com/account
 *   network              → transient — "try again"
 *   verification_failed  → server returned a blob we couldn't verify. Fail closed.
 *   internal             → bucket for anything not-user-facing (keyring down, fingerprint missing)
 */
export type LicenceErrorCode =
  | 'malformed'
  | 'not_found'
  | 'revoked'
  | 'device_limit'
  | 'network'
  | 'verification_failed'
  | 'already_activated'
  | 'rate_limited'
  | 'pro_required'
  | 'internal';

export interface LicenceError {
  code: LicenceErrorCode;
  message: string;
}

/**
 * Normalise whatever `invoke()` rejects with into a `LicenceError`. Tauri
 * serialises `OrliqError` as `{ kind: "licence", code, message }`; we also
 * catch the edge case where a plain string or bare Error leaked through.
 */
export function toLicenceError(e: unknown): LicenceError {
  if (e && typeof e === 'object' && 'kind' in e) {
    const env = e as { kind?: string; code?: string; message?: string };
    if (env.kind === 'licence' && typeof env.code === 'string') {
      return {
        code: env.code as LicenceErrorCode,
        message: env.message ?? 'Something went wrong.',
      };
    }
  }
  return {
    code: 'internal',
    message: e instanceof Error ? e.message : String(e ?? 'Unknown error'),
  };
}

export type LicenceState = {
  status: LicenceStatus | null;
  /** `true` once we've at least attempted to read status from Rust. Lets the
   *  Licence panel skip rendering "Free" for a single frame on mount. */
  loaded: boolean;
  setStatus(next: LicenceStatus): void;
  load(): Promise<void>;
};

export const useLicence = create<LicenceState>((set) => ({
  status: null,
  loaded: false,
  setStatus(next) {
    set({ status: next, loaded: true });
  },
  async load() {
    try {
      const s = await invoke<LicenceStatus>('licence_status');
      set({ status: s, loaded: true });
    } catch {
      // If the status call fails we still want the UI to proceed — treat it
      // as Free. The Rust side returns status without network access, so the
      // only way we land here is a catastrophic IPC failure.
      set({ status: { tier: 'free' }, loaded: true });
    }
  },
}));

/**
 * Attach the backend → store sync. Returns an unlisten function. Call once
 * from `AppBootstrap.tsx`, alongside the other `listen()` subscriptions.
 */
export async function attachLicenceListener(): Promise<() => void> {
  return listen<LicenceStatus>(EVT.licenceChanged, (e) => {
    useLicence.getState().setStatus(e.payload);
  });
}

/** Is the current status Pro? Narrow helper so gating code stays readable. */
export function isPro(status: LicenceStatus | null): boolean {
  return status?.tier === 'pro';
}
