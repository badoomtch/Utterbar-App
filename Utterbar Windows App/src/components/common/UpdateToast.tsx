import { useEffect, useMemo, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { CheckCircle2, Download, X } from 'lucide-react';
import { open as openShell } from '@tauri-apps/plugin-shell';
import { useTranslation } from 'react-i18next';

import { Button } from '@/components/common/Button';
import { invoke } from '@/lib/ipc';
import { springs } from '@/design/motion';
import { useSettings } from '@/state/settings';
import { useUpdate } from '@/state/update';

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
 *
 * The actual fetch + state lives in `useUpdate` (state/update.ts) so this
 * component, the Settings → General "Check for updates now" button, and any
 * future surface (tray menu, etc.) all share a single in-flight gate and a
 * single dismissal record.
 *
 * Schedule: ONE check per app launch (when this component mounts), gated
 * on the user's `check_updates_automatically` preference. We deliberately
 * do NOT poll, do NOT re-check on window focus, do NOT re-check whenever
 * Settings is opened. The user only restarts the app every few days at
 * most, and a release every few weeks; one outbound HTTP request per
 * session is the lightweight ceiling. Anything more is system-usage
 * spend without user value.
 *
 * If a user wants a fresh check between launches, the manual "Check now"
 * button in Settings → General does the same thing on demand.
 */
export function UpdateToast() {
  const { t } = useTranslation('common');
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const autoCheck = useSettings((x) => x.data?.general.check_updates_automatically ?? true);
  const settingsLoaded = useSettings((x) => x.data !== null);

  const info = useUpdate((s) => s.info);
  const dismissed = useUpdate((s) => s.dismissed);
  const check = useUpdate((s) => s.check);
  const dismiss = useUpdate((s) => s.dismiss);

  // Single check per app launch. Wait for settings to hydrate before
  // firing — calling pre-hydrate would read the default `true` and run a
  // check the user may have explicitly disabled in a previous session,
  // which would be a privacy violation. Toggling the setting back on
  // mid-session re-runs this effect and triggers another check; that's
  // the user's deliberate action so it's fair game.
  useEffect(() => {
    if (!settingsLoaded) return;
    if (!autoCheck) return;
    void check();
  }, [settingsLoaded, autoCheck, check]);

  const install = async () => {
    setInstalling(true);
    setError(null);
    try {
      await invoke('install_update');
      // If we get here the app is about to exit; keep the spinner up so the
      // user isn't looking at a re-enabled button for the ~400ms before exit.
    } catch (e) {
      setInstalling(false);
      setError(extractErrorMessage(e, t));
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
            <div className="text-[13px] font-medium text-fg-hi">
              {t('chrome.update_toast.title')}
            </div>
            <div className="mt-0.5 font-mono text-[11px] text-muted">
              {info!.current} → {info!.available}
            </div>
            {/*
              `notesPreview` is the first line of `UpdateInfo.notes`,
              which is Rust-emitted release-note prose served by the
              admin updater feed. Backend prose stays verbatim here;
              localising it would require the server to emit a code +
              args contract, which is the future error/event-codes
              branch. Same deferral noted in StatePill.tsx,
              MicMuteBanner.tsx, and Onboarding.tsx.
            */}
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
                {installing
                  ? t('chrome.update_toast.installing')
                  : t('chrome.update_toast.install')}
              </Button>
              <button
                onClick={() => dismiss()}
                className="text-[12px] text-muted transition-colors hover:text-fg-hi"
              >
                {t('chrome.update_toast.later')}
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
 *
 * `t` is passed in from the caller so the frontend fallback resolves
 * through the active locale. Rust-emitted `OrliqError.message` values
 * still pass through verbatim; localising backend prose is deferred to
 * a future error/event-codes branch.
 */
function extractErrorMessage(
  e: unknown,
  t: (key: 'chrome.update_toast.error_generic') => string,
): string {
  if (e && typeof e === 'object') {
    const anyE = e as { code?: unknown; message?: unknown };
    if (typeof anyE.code === 'string' && typeof anyE.message === 'string') {
      return anyE.message;
    }
  }
  if (typeof e === 'string' && e.length < 160) return e;
  return t('chrome.update_toast.error_generic');
}

/**
 * Post-update toast — fires once on the first launch after a successful
 * in-app update. Reads the Rust-side `pending_relaunch.json` marker via
 * `consume_post_update_marker`, which the Rust side deletes on read so
 * this toast shows exactly once per update.
 *
 * Self-dismisses after 12 seconds (long enough to read the version + click
 * the link, short enough not to linger). User can dismiss manually too.
 *
 * "See what's new" opens the live changelog at utterbar.com/changelog
 * rather than trying to render the local release-notes file — the
 * website page is the single source of truth and matches what users see
 * on the marketing site.
 */
export function PostUpdateToast() {
  const { t } = useTranslation('common');
  const justUpdatedTo = useUpdate((s) => s.justUpdatedTo);
  const consume = useUpdate((s) => s.consumePostUpdateMarker);
  const dismiss = useUpdate((s) => s.dismissPostUpdate);

  // One-shot consume on mount. The Rust side handles the read-and-delete
  // so re-mounts (StrictMode dev double-mount, route changes) don't keep
  // re-firing the toast — the second consume call returns null.
  useEffect(() => {
    void consume();
  }, [consume]);

  // Auto-dismiss after 12 s. Cleared if the user dismisses manually.
  useEffect(() => {
    if (!justUpdatedTo) return;
    const t = window.setTimeout(() => dismiss(), 12_000);
    return () => window.clearTimeout(t);
  }, [justUpdatedTo, dismiss]);

  return (
    <AnimatePresence>
      {justUpdatedTo && (
        <motion.div
          key="post-update-toast"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 10 }}
          transition={springs.glide}
          className="fixed bottom-5 right-5 flex w-80 items-start gap-3 rounded-card bg-bg-1 px-4 py-3"
        >
          <CheckCircle2 className="mt-0.5 h-4 w-4 text-accent" />
          <div className="flex-1">
            <div className="text-[13px] font-medium text-fg-hi">
              {t('chrome.update_toast.post_title', { version: justUpdatedTo })}
            </div>
            <div className="mt-1 text-[12px] leading-relaxed text-muted">
              {t('chrome.update_toast.post_body')}
            </div>
            <div className="mt-2 flex items-center gap-2">
              <button
                onClick={() =>
                  void openShell('https://utterbar.com/changelog').catch(
                    () => undefined,
                  )
                }
                className="text-[12px] font-medium text-accent transition-opacity hover:opacity-80"
              >
                {t('chrome.update_toast.post_link')}
              </button>
            </div>
          </div>
          <button
            onClick={() => dismiss()}
            aria-label={t('chrome.update_toast.dismiss_aria')}
            className="ml-1 text-muted transition-colors hover:text-fg-hi"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
