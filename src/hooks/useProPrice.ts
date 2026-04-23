import { useEffect, useRef, useState } from 'react';
import { invoke } from '@/lib/ipc';

/**
 * Pro-tier pricing info surfaced to React.
 *
 * Mirrors `ProPrice` in `src-tauri/src/commands/pricing.rs`. The backend
 * folds network failures into an offline fallback so this hook never needs
 * to handle an error state — the only loading state callers see is "still
 * fetching" vs "we have a price to show".
 */
export interface ProPrice {
  amount_usd: number;
  currency: string;
  period: string;
  display: string;
  source: 'server' | 'fallback' | string;
}

/**
 * Static offline fallback surfaced during the initial loading tick. Matches
 * the Rust fallback exactly so the modal opens with a price already baked
 * in rather than flashing a "loading" state — a one-frame "$29" that later
 * refines to "$29 / month" (if the server contract changes) reads cleaner
 * than a skeleton → number transition for a single line of copy.
 */
const BOOTSTRAP_FALLBACK: ProPrice = {
  amount_usd: 29,
  currency: 'USD',
  period: 'one_time',
  display: '$29',
  source: 'fallback',
};

/**
 * Fetch Pro pricing once per component lifetime. Kept per-component rather
 * than globally cached because the call is rare (only when an upgrade modal
 * opens) and the server is authoritative — we don't want a stale hour-old
 * price pinned in a React Context outliving a price change. Server response
 * time is capped at 6s inside the Rust client, so worst-case the modal
 * shows the bootstrap fallback for a moment before updating.
 */
export function useProPrice(): { price: ProPrice; loading: boolean } {
  const [price, setPrice] = useState<ProPrice>(BOOTSTRAP_FALLBACK);
  const [loading, setLoading] = useState(true);
  // Strict-mode safety: don't set state after unmount. The fetch is only
  // 6s worst case so unmount-before-resolve is rare, but it's common
  // enough during modal open/close that the warning would fire in dev.
  const aliveRef = useRef(true);

  useEffect(() => {
    aliveRef.current = true;
    let cancelled = false;
    (async () => {
      try {
        const p = await invoke<ProPrice>('get_pro_price');
        if (!cancelled && aliveRef.current) setPrice(p);
      } catch {
        // Unreachable in practice (Rust swallows errors into fallback) but
        // we keep the catch so a future command-signature change can't crash
        // the modal.
      } finally {
        if (!cancelled && aliveRef.current) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
      aliveRef.current = false;
    };
  }, []);

  return { price, loading };
}
