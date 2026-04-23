import { useEffect, useMemo, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { Download } from 'lucide-react';

import { Button } from '@/components/common/Button';
import { invoke } from '@/lib/ipc';
import { springs } from '@/design/motion';
import { useSettings } from '@/state/settings';
import type { UpdateInfo } from '@/generated/UpdateInfo';

/**
 * Tiny bottom-right toast that surfaces new versions without nagging.
 *
 * Per DESIGN_SYSTEM.md the toast sits on bg-1 (matching card elevation), has
 * no drop shadow (depth is carried by tonal lift over the app background),
 * and limits its cyan moment to a single icon + the CTA pill — we stay well
 * inside the §1.6 cyan budget.
 *
 * CTA copy reads "Install & restart" because the real flow (see
 * `commands/updater.rs`) is: download → SHA-256 verify → spawn silent NSIS →
 * Tauri exits → installer's post-install hook re-launches Utterbar. The user
 * sees the app close and re-open once; no manual restart required.
 */
export function UpdateToast() {
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const autoCheck = useSettings((x) => x.data?.general.check_updates_automatically ?? true);

  useEffect(() => {
    if (!autoCheck) return;
    void invoke<UpdateInfo>('check_update')
      .then((u) => {
        if (u.available) setInfo(u);
      })
      .catch(() => undefined);
  }, [autoCheck]);

  const install = async () => {
    setInstalling(true);
    setError(null);
    try {
      await invoke('install_update');
      // If we get here the app is about to exit; keep the spinner up so the
      // user isn't looking at a re-enabled button for the ~400ms before exit.
    } catch (e) {
      setInstalling(false);
      setError(extractErrorMessage(e));
    }
  };

  // First non-empty bullet / line of the release notes, clipped so the toast
  // doesn't balloon on verbose changelogs. Everything lives on About → "What's
  // new in this version" for the full story.
  const notesPreview = useMemo(() => firstNotesLine(info?.notes), [info?.notes]);

  const show = info && info.available && !dismissed;

  return (
    <AnimatePresence>
      {show && (
        <motion.div
          key="update-toast"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 10 }}
          transition={springs.glide}
          className="fixed bottom-5 right-5 flex w-80 items-start gap-3 rounded-card bg-bg-1 px-4 py-3"
        >
          <Download className="mt-0.5 h-4 w-4 text-accent" />
          <div className="flex-1">
            <div className="text-[13px] font-medium text-fg-hi">Update available</div>
            <div className="mt-0.5 font-mono text-[11px] text-muted">
              {info!.current} → {info!.available}
            </div>
            {notesPreview && (
              <div className="mt-1.5 line-clamp-2 text-[12px] text-fg-lo">{notesPreview}</div>
            )}
            {error && (
              <div className="mt-1.5 text-[12px] text-[color:var(--danger,#f87171)]">{error}</div>
            )}
            <div className="mt-2 flex items-center gap-2">
              <Button
                variant="cta"
                size="sm"
                onClick={() => void install()}
                disabled={installing}
              >
                {installing ? 'Installing…' : 'Install & restart'}
              </Button>
              <button
                onClick={() => setDismissed(true)}
                className="text-[12px] text-muted transition-colors hover:text-fg-hi"
              >
                Later
              </button>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

/**
 * Pull the first meaningful line of the release notes for the preview. We
 * strip leading list markers (`- `, `* `, `• `) so the toast reads as a
 * sentence rather than a floating bullet, and trim markdown heading hashes.
 */
function firstNotesLine(notes: string | null | undefined): string | null {
  if (!notes) return null;
  for (const raw of notes.split('\n')) {
    const line = raw.trim();
    if (!line) continue;
    const cleaned = line
      .replace(/^#+\s+/, '')
      .replace(/^[-*•]\s+/, '')
      .trim();
    if (cleaned) return cleaned;
  }
  return null;
}

/**
 * OrliqError-shaped objects come through with `{ code, message }`; bare
 * strings and anything else fall back to a friendly generic. We never let a
 * raw JS Error toString() bleed into the toast — "TypeError: Failed to fetch"
 * on the user's screen reads as a bug even when it's just flaky wifi.
 */
function extractErrorMessage(e: unknown): string {
  if (e && typeof e === 'object') {
    const anyE = e as { code?: unknown; message?: unknown };
    if (typeof anyE.code === 'string' && typeof anyE.message === 'string') {
      return anyE.message;
    }
  }
  if (typeof e === 'string' && e.length < 160) return e;
  return 'Update failed. Try again in a moment.';
}
