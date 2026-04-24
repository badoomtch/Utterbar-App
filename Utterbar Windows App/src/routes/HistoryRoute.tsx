import { useEffect, useMemo, useRef, useState } from 'react';
import {
  differenceInCalendarDays,
  format,
  formatDistanceToNowStrict,
  isToday,
  isYesterday,
} from 'date-fns';
import { Check, Copy, FileText, Search, Sparkles, Trash2, X } from 'lucide-react';

import { Button } from '@/components/common/Button';
import { PageHeader } from '@/components/common/PageHeader';
import { ProNudge } from '@/components/common/ProNudge';
import { Select } from '@/components/common/Select';
import { cn } from '@/lib/cn';
import { invoke } from '@/lib/ipc';
import { formatDuration } from '@/lib/format';
import { useLicence, isPro } from '@/state/licence';
import { useModes } from '@/state/modes';

// Mirrors FREE_HISTORY_CAP in src-tauri/src/util/gating.rs. Display-only —
// the backend truncates the list, this is just for the footnote copy.
const FREE_HISTORY_CAP = 100;

type HistoryEntry = {
  id: string;
  created_at: number;
  mode_id: string | null;
  mode_name_snapshot: string;
  raw_text: string;
  processed_text: string;
  audio_duration_ms: number;
  app_exe: string | null;
  app_title: string | null;
  context_payload: string | null;
  status: string;
};

/**
 * History — the permanent record.
 *
 * v3 layout (2026-04-24): single column. The earlier two-column grid with a
 * fixed 540 px detail pane squeezed the list down to ~300 px on a normal
 * window, and then the ProNudge / empty-state cards inside it wrapped to
 * one-word-per-line — genuinely unreadable. v0.6.1's release notes
 * over-claimed a fix here; v0.6.3 actually lands the rewrite.
 *
 *   • ONE full-width list. Each entry is a card: transcript first, metadata
 *     second, inline copy affordance on hover.
 *   • Clicking a row opens the detail in a centre-screen modal. No permanent
 *     side pane that eats layout real estate when nothing is selected.
 *   • The Pro nudge is the first full-width card, not a half-width empty
 *     state — so it reads as a clear pitch rather than a cramped nag.
 *
 * Per DESIGN_SYSTEM.md §4, panels sit on bg-1 with NO border / shadow;
 * separation comes from tone. The modal is a centre overlay on a
 * semi-transparent bg-0 scrim, matching the app's UpgradeModal convention.
 */
export function HistoryRoute() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [search, setSearch] = useState('');
  const [selected, setSelected] = useState<HistoryEntry | null>(null);
  const modes = useModes((s) => s.list);
  const loadModes = useModes((s) => s.load);
  const [reprocessMode, setReprocessMode] = useState<string>('');
  // Per-row "copied" confirmation flash. We key by entry id rather than using
  // a single toast because the user may copy several rows quickly — each row
  // should get its own ✓ acknowledgement for ~1.2s and then return to the
  // copy icon.
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const copiedTimerRef = useRef<number | undefined>(undefined);

  const reload = (q: string) =>
    invoke<HistoryEntry[]>('list_history', {
      filter: { search: q || null, limit: 200 },
    })
      .then(setEntries)
      .catch(() => setEntries([]));

  useEffect(() => {
    void loadModes();
  }, [loadModes]);

  useEffect(() => {
    const t = setTimeout(() => void reload(search), 150);
    return () => clearTimeout(t);
  }, [search]);

  useEffect(() => {
    if (modes.length > 0 && !reprocessMode) setReprocessMode(modes[0].id);
  }, [modes, reprocessMode]);

  const copy = async (text: string) => {
    await navigator.clipboard.writeText(text);
  };

  // Inline row copy. Copies whichever text the row is actually displaying
  // (processed if it exists, otherwise raw — matches the snippet line below)
  // so the user gets exactly what they see. The event is stopped from
  // propagating so we don't also open the detail modal when someone just
  // wants the text on their clipboard.
  const copyRow = async (e: React.MouseEvent, entry: HistoryEntry) => {
    e.stopPropagation();
    const text = entry.processed_text || entry.raw_text;
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      return;
    }
    setCopiedId(entry.id);
    if (copiedTimerRef.current) window.clearTimeout(copiedTimerRef.current);
    copiedTimerRef.current = window.setTimeout(() => setCopiedId(null), 1200);
  };

  useEffect(() => {
    return () => {
      if (copiedTimerRef.current) window.clearTimeout(copiedTimerRef.current);
    };
  }, []);

  const reprocess = async () => {
    if (!selected || !reprocessMode) return;
    const next = await invoke<HistoryEntry>('reprocess_history', {
      id: selected.id,
      modeId: reprocessMode,
    });
    setSelected(next);
    await reload(search);
  };

  const remove = async (id: string) => {
    await invoke('delete_history_entry', { id });
    if (selected?.id === id) setSelected(null);
    await reload(search);
  };

  // ESC to close the detail modal — standard dismissal convention the user
  // will try first. Scoped to the document so typing in the search field
  // above doesn't get intercepted.
  useEffect(() => {
    if (!selected) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setSelected(null);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [selected]);

  // Group entries by calendar day so we can render sticky date eyebrows.
  // `entries` is already sorted newest-first by the backend, so we just fold
  // in order and push into buckets keyed by the day label.
  const licenceStatus = useLicence((s) => s.status);
  const pro = isPro(licenceStatus);

  const grouped = useMemo(() => {
    const out: { key: string; label: string; items: HistoryEntry[] }[] = [];
    let current: (typeof out)[number] | null = null;
    for (const e of entries) {
      const d = new Date(e.created_at);
      const key = format(d, 'yyyy-MM-dd');
      if (!current || current.key !== key) {
        current = { key, label: dayLabel(d), items: [] };
        out.push(current);
      }
      current.items.push(e);
    }
    return out;
  }, [entries]);

  return (
    // Outer padding steps down on narrower widths so the page doesn't feel
    // like it's been force-fed 40 px of empty gutter when the user has pulled
    // the window in to pair it with a video/editor. The max-w cap prevents
    // sprawl at ultrawide — entries stay readable rather than stretching
    // to a 2000 px single line.
    <div className="mx-auto flex h-full max-w-[1100px] flex-col px-6 pb-8 pt-6 md:px-8 md:pb-10 md:pt-8 lg:px-10">
      <PageHeader
        title="History"
        subtitle="Every dictation, stored locally. Reprocess, export, or wipe at any time."
      />

      {/* Search bar — sits on bg-2 so it reads as a distinct affordance from
          the bg-1 panels below. Larger hit target and a subtle count chip on
          the right side so the user sees scope at a glance. */}
      <div className="mt-6 flex items-center gap-3 rounded-input bg-bg-2 px-4 py-2.5">
        <Search className="h-4 w-4 text-fg-dim" />
        <input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search transcripts, apps, or modes"
          className="flex-1 bg-transparent text-[14px] text-fg-hi outline-none placeholder:text-fg-dim"
        />
        <span className="font-mono text-[11px] text-fg-dim">
          {entries.length} {entries.length === 1 ? 'entry' : 'entries'}
        </span>
      </div>

      {/* The Pro nudge lives above the list as a full-width card when the
          user is Free and has hit the cap. Before v0.6.3 this sat INSIDE
          the list column and, on narrow windows, wrapped one word per line.
          Hoisting it out is the single biggest readability win on the page. */}
      {!pro && entries.length >= FREE_HISTORY_CAP && (
        <div className="mt-6">
          <ProNudge
            feature="Full history"
            description={`Free shows your most recent ${FREE_HISTORY_CAP} dictations. Pro shows everything — the older entries are still on disk and reappear the moment you activate.`}
          />
        </div>
      )}

      {/* List — single column, full width, scrollable. `min-h-0` is the
          Tailwind-flavoured fix for the flex-1 + overflow footgun: without
          it, min-content height would fight `overflow-hidden` and spill
          children out of the viewport. */}
      <div className="mt-6 min-h-0 flex-1 overflow-y-auto rounded-card bg-bg-1">
        {entries.length === 0 && (
          // Empty state differentiates "no results for this search" from
          // "nothing recorded yet" — otherwise a user searching for a term
          // that doesn't exist sees the welcome copy and thinks their
          // history has been wiped. Copy stays short either way; the full
          // "how to dictate" story lives in onboarding.
          <div className="flex h-full flex-col items-center justify-center gap-2 p-12 text-center">
            <FileText className="h-8 w-8 text-fg-dim opacity-60" />
            <div className="text-[14px] text-fg">
              {search ? 'No matches' : 'No dictations yet'}
            </div>
            <div className="max-w-xs text-[12px] text-muted">
              {search
                ? 'No transcripts match your search. Try a shorter query or clear the filter.'
                : 'Press your hotkey (Ctrl+Space by default) to start dictating — transcripts will appear here, stored only on this device.'}
            </div>
          </div>
        )}
        {grouped.map((group) => (
          <section key={group.key} className="pb-2">
            {/* Sticky date eyebrow. `sticky top-0` keeps it pinned while the
                user scrolls through a long day's worth of dictations, and
                `bg-bg-1` matches the panel so it occludes rows below it. */}
            <div className="sticky top-0 z-10 flex items-center gap-3 bg-bg-1/95 px-5 py-2 backdrop-blur-sm">
              <span className="ub-eyebrow">{group.label}</span>
              <span className="font-mono text-[10px] text-fg-dim">
                {group.items.length}
              </span>
            </div>
            <ul>
              {group.items.map((e) => {
                const created = new Date(e.created_at);
                return (
                  <li
                    key={e.id}
                    onClick={() => setSelected(e)}
                    className={cn(
                      'group relative cursor-pointer px-5 py-4 transition-colors',
                      'border-t border-border-soft/40 first:border-t-0',
                      'hover:bg-bg-2/60',
                    )}
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="min-w-0 flex-1">
                        {/* Transcript first — this is what the user is
                            actually scanning for. Line-clamped to 2 so the
                            row height stays predictable. */}
                        <div className="line-clamp-2 text-[14px] leading-[1.45] text-fg">
                          {e.processed_text || e.raw_text || (
                            <span className="italic text-muted">Empty transcript</span>
                          )}
                        </div>
                        {/* Metadata row — time · mode · duration · app. */}
                        <div className="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-[11px] text-fg-dim">
                          <span title={created.toLocaleString()}>
                            {formatDistanceToNowStrict(created, { addSuffix: true })}
                          </span>
                          <span aria-hidden>·</span>
                          <span className="truncate">{e.mode_name_snapshot}</span>
                          <span aria-hidden>·</span>
                          <span>{formatDuration(e.audio_duration_ms)}</span>
                          {e.app_exe && (
                            <>
                              <span aria-hidden>·</span>
                              <span className="truncate">{e.app_exe}</span>
                            </>
                          )}
                        </div>
                      </div>
                      {/* Inline copy affordance. Lives on every row so the
                          user doesn't have to open detail — 80% of the time
                          they just want the text they can see. */}
                      <button
                        onClick={(ev) => void copyRow(ev, e)}
                        onPointerDown={(ev) => ev.stopPropagation()}
                        title="Copy"
                        aria-label="Copy transcript"
                        className={cn(
                          'inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-[8px] transition-all',
                          'opacity-0 group-hover:opacity-100 focus-visible:opacity-100',
                          copiedId === e.id
                            ? 'bg-accent/15 text-accent opacity-100'
                            : 'bg-bg-1 text-muted hover:bg-bg-2 hover:text-fg-hi',
                        )}
                      >
                        {copiedId === e.id ? (
                          <Check className="h-3.5 w-3.5" />
                        ) : (
                          <Copy className="h-3.5 w-3.5" />
                        )}
                      </button>
                    </div>
                  </li>
                );
              })}
            </ul>
          </section>
        ))}
      </div>

      {/* Detail modal. Centre-screen overlay, dismissed by ESC (wired
          in a useEffect above), clicking outside, or the explicit close
          button. Because it's modal and not a side pane, it works at ANY
          window size — no more "list column collapses to 200 px". */}
      {selected && (
        <div
          className="fixed inset-0 z-40 flex items-center justify-center bg-bg-0/80 backdrop-blur-sm"
          onClick={() => setSelected(null)}
          role="dialog"
          aria-modal="true"
        >
          <div
            className="relative max-h-[86vh] w-[min(720px,92vw)] overflow-y-auto rounded-card bg-bg-1 p-7"
            // Stop click-outside-to-close from firing when the click lands
            // inside the modal body.
            onClick={(e) => e.stopPropagation()}
          >
            <button
              onClick={() => setSelected(null)}
              aria-label="Close"
              title="Close"
              className="absolute right-4 top-4 inline-flex h-8 w-8 items-center justify-center rounded-[8px] text-fg-dim transition-colors hover:bg-bg-2 hover:text-fg-hi"
            >
              <X className="h-4 w-4" />
            </button>

            <div className="flex items-start justify-between gap-3 pr-10">
              <div className="min-w-0 flex-1">
                <div className="font-mono text-[11px] text-fg-dim">
                  {format(new Date(selected.created_at), 'EEEE, d MMM yyyy · HH:mm')}
                </div>
                {/* Weight 700 — font-semibold (600) is forbidden per §2.3. */}
                <div className="mt-1.5 truncate text-[16px] font-bold text-fg-hi">
                  {selected.mode_name_snapshot}
                </div>
                {selected.app_exe && (
                  <div className="mt-1 truncate text-[11px] text-muted">
                    {selected.app_exe}
                    {selected.app_title ? ` · ${selected.app_title}` : ''}
                  </div>
                )}
              </div>
              <button
                onClick={() => void remove(selected.id)}
                title="Delete"
                aria-label="Delete entry"
                className="inline-flex h-8 w-8 items-center justify-center rounded-[8px] text-fg-dim transition-colors hover:bg-bg-2 hover:text-rose"
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </div>

            {pro ? (
              <div className="mt-5 flex items-center gap-2 rounded-input bg-bg-2 p-2">
                <Sparkles className="h-3.5 w-3.5 text-accent" />
                <span className="text-[11px] text-muted">Reprocess as</span>
                <div className="flex-1">
                  <Select
                    value={reprocessMode}
                    onChange={setReprocessMode}
                    options={modes.map((m) => ({ value: m.id, label: m.name }))}
                  />
                </div>
                <Button variant="ghost" size="sm" noLed onClick={() => void reprocess()}>
                  Run
                </Button>
              </div>
            ) : (
              <ProNudge
                feature="Re-transcribe"
                description="Run any stored transcript back through a different mode — swap formatting, try a different template — without recording again."
                dense
                className="mt-5"
              />
            )}

            <Block
              label="Raw transcript"
              text={selected.raw_text}
              onCopy={() => void copy(selected.raw_text)}
            />
            <Block
              label="Processed"
              text={selected.processed_text}
              onCopy={() => void copy(selected.processed_text)}
            />

            {parsedContext(selected) && (
              <details className="mt-5 rounded-input bg-bg-2 p-3 text-[12px]">
                <summary className="cursor-pointer font-medium text-fg-hi">
                  Context inspection
                </summary>
                <pre className="mt-2 whitespace-pre-wrap break-words font-mono text-[11px] text-muted">
                  {JSON.stringify(parsedContext(selected), null, 2)}
                </pre>
              </details>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

/** Safely parse the stored context JSON. Kept outside the component so the
 *  modal can call it for both the `if` guard and the rendered body without
 *  re-running the parse twice. */
function parsedContext(e: HistoryEntry): Record<string, string | null> | null {
  if (!e.context_payload) return null;
  try {
    return JSON.parse(e.context_payload) as Record<string, string | null>;
  } catch {
    return null;
  }
}

/** Human-friendly day label. Today / Yesterday / weekday (this week) /
 *  absolute date (further back). Matches Slack + Linear conventions. */
function dayLabel(d: Date): string {
  if (isToday(d)) return 'Today';
  if (isYesterday(d)) return 'Yesterday';
  const diff = differenceInCalendarDays(new Date(), d);
  if (diff < 7) return format(d, 'EEEE'); // Monday, Tuesday, …
  return format(d, 'EEEE, d MMMM yyyy');
}

function Block({
  label,
  text,
  onCopy,
}: {
  label: string;
  text: string;
  onCopy: () => void;
}) {
  return (
    <div className="mt-5">
      <div className="flex items-center justify-between">
        {/* Eyebrow — Instrument Sans 500 11px +0.14em tracking per §2.2 */}
        <span className="ub-eyebrow">{label}</span>
        <button
          onClick={onCopy}
          className="inline-flex items-center gap-1 rounded-[6px] px-1.5 py-0.5 text-[11px] text-muted transition-colors hover:bg-bg-2 hover:text-fg-hi"
        >
          <Copy className="h-3 w-3" /> Copy
        </button>
      </div>
      <div className="mt-2 whitespace-pre-wrap rounded-input bg-bg-2 p-4 text-[13px] leading-[1.55] text-fg">
        {text || <span className="italic text-muted">Empty</span>}
      </div>
    </div>
  );
}
