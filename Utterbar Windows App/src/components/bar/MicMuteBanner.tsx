import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import type { MicMuteStatus } from '@/generated/MicMuteStatus';
import type { MicMutePhase } from '@/generated/MicMutePhase';

/**
 * Banner that renders above the floating bar when the experimental
 * Teams + Discord mic-mute feature has produced a status event. Three
 * shapes:
 *
 * - `recording_started`: subtle sub-line beneath the bar with the
 *   "Tried to mute X while recording. Confirm in X." copy. Sticks
 *   for the duration of the recording (until a stop / cancel /
 *   restore event arrives).
 * - `restore_succeeded`: success copy, auto-dismisses after 5s.
 * - `restore_failed` / `restore_on_launch_failed`: sticky failure
 *   banner with a click-to-dismiss "Confirmed" button. The banner
 *   does NOT auto-dismiss because the user must look at the call
 *   app and verify; a 5-second auto-dismiss would let them dismiss
 *   on autopilot without doing the check.
 *
 * Layout: renders absolutely-positioned ABOVE the bar so it never
 * compresses the existing chrome (timer, mode chip, buttons,
 * waveform). On reduced-motion preferences the fade-in transition
 * is skipped; the banner appears instantly.
 */
export function MicMuteBanner() {
  const { t } = useTranslation('bar');
  const [status, setStatus] = useState<MicMuteStatus | null>(null);
  const [reducedMotion, setReducedMotion] = useState(false);

  useEffect(() => {
    setReducedMotion(
      window.matchMedia?.('(prefers-reduced-motion: reduce)').matches === true,
    );
    const p = listen<MicMuteStatus>(EVT.micMuteStatus, (e) => {
      setStatus(e.payload);
    });
    return () => {
      void p.then((u) => u()).catch(() => undefined);
    };
  }, []);

  // Auto-dismiss for the success path. The active "recording_started"
  // sub-line is dismissed by the next status event (succeeded or
  // failed); the failure paths stay sticky until the user clicks.
  useEffect(() => {
    if (!status) return;
    if (status.phase !== 'restore_succeeded') return;
    const id = window.setTimeout(() => {
      setStatus((cur) =>
        cur && cur.phase === 'restore_succeeded' ? null : cur,
      );
    }, 5000);
    return () => window.clearTimeout(id);
  }, [status]);

  if (!status) return null;

  const phase = status.phase as MicMutePhase;
  const isFailure =
    phase === 'restore_failed' || phase === 'restore_on_launch_failed';

  return (
    <div
      role="status"
      aria-live="polite"
      // pointer-events-auto on the banner element only (the wrapper
      // in FloatingBar.tsx is pointer-events-none so the user can
      // drag the pill through empty banner space).
      className={`pointer-events-auto select-none rounded-lg px-3 py-2 text-[12px] leading-relaxed shadow-lg ${
        reducedMotion ? '' : 'transition-opacity duration-200'
      } ${
        isFailure
          ? 'bg-accent/15 text-fg-hi ring-1 ring-accent/40'
          : phase === 'restore_succeeded'
            ? 'bg-bg-2 text-fg-hi'
            : 'bg-bg-2/85 text-muted'
      }`}
    >
      <div className="flex items-start gap-2">
        {/*
          `status.message` is the Rust-emitted prose for the current
          mic-mute phase (e.g. "Tried to mute Discord while recording.
          Confirm in Discord."). Backend event-message localisation is
          deferred to the later error-codes branch — when that lands,
          the Rust side will emit a stable code + args and this span
          will resolve through `t(...)` like the buttons below.
        */}
        <span className="flex-1">{status.message}</span>
        {isFailure ? (
          <button
            type="button"
            onClick={() => setStatus(null)}
            className="shrink-0 rounded px-2 py-0.5 text-[11px] font-medium text-fg-hi hover:bg-bg-2"
          >
            {t('mic_mute.confirmed')}
          </button>
        ) : null}
      </div>
    </div>
  );
}
