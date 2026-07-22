import { useEffect, useMemo, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { X } from 'lucide-react';

import { useDictation } from '@/state/dictation';
import { cn } from '@/lib/cn';

function fmtTime(secs: number): string {
  const mm = Math.floor(secs / 60);
  const ss = secs % 60;
  return `${String(mm).padStart(2, '0')}:${String(ss).padStart(2, '0')}`;
}

/**
 * Live Meeting Mode transcript panel (v0.21.x). While a two-channel
 * meeting records, the Rust worker transcribes each speech span as it
 * closes and streams it here via `meeting:live-segment`; this panel
 * renders the transcript building up speaker-by-speaker in real time.
 * A floating, dismissible overlay so it's visible on any main-window
 * route. Segments are sorted by start time for display.
 */
export function LiveMeetingTranscript() {
  const { t } = useTranslation('common');
  const segments = useDictation((s) => s.liveMeeting);
  const active = useDictation((s) => s.liveMeetingActive);
  const dismiss = useDictation((s) => s.dismissLiveMeeting);
  const scrollRef = useRef<HTMLDivElement>(null);

  const sorted = useMemo(
    () => [...segments].sort((a, b) => a.startSecs - b.startSecs || a.id - b.id),
    [segments],
  );

  useEffect(() => {
    // Keep the newest line in view as the transcript grows.
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [sorted.length]);

  if (!active && segments.length === 0) return null;

  return (
    <div className="pointer-events-auto fixed bottom-4 right-4 z-40 flex max-h-[52vh] w-[380px] flex-col overflow-hidden rounded-xl border border-border-soft/60 bg-bg-1 shadow-lg">
      <div className="flex items-center justify-between border-b border-border-soft/60 px-4 py-2.5">
        <div className="flex items-center gap-2">
          <span className="relative flex h-2 w-2">
            {active && (
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--accent)] opacity-70" />
            )}
            <span className="relative inline-flex h-2 w-2 rounded-full bg-[var(--accent)]" />
          </span>
          <span className="text-[13px] font-medium text-fg-hi">{t('meeting_live.title')}</span>
        </div>
        <button
          type="button"
          onClick={dismiss}
          className="rounded p-1 text-muted transition-colors hover:bg-bg-2 hover:text-fg"
          aria-label="Close"
        >
          <X className="h-3.5 w-3.5" />
        </button>
      </div>
      <div ref={scrollRef} className="min-h-0 flex-1 space-y-2 overflow-y-auto px-4 py-3">
        {sorted.length === 0 ? (
          <p className="text-[12px] italic text-muted">{t('meeting_live.waiting')}</p>
        ) : (
          sorted.map((seg) => (
            <div key={seg.id} className="text-[12.5px] leading-relaxed">
              <span className="mr-1 font-mono text-[11px] text-muted">
                {fmtTime(seg.startSecs)}
              </span>
              <span
                className={cn(
                  'font-semibold',
                  seg.speakerId === 0 ? 'text-[var(--accent)]' : 'text-fg-hi',
                )}
              >
                {seg.speakerId === 0 ? t('meeting_live.you') : t('meeting_live.others')}:
              </span>{' '}
              <span className="text-fg">{seg.text}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
