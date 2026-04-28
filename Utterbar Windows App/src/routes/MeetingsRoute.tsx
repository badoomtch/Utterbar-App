import { useEffect, useState } from 'react';
import { ArrowLeft, Copy, Download, Trash2, Users, Clock, FileText, Sparkles } from 'lucide-react';

import { PageHeader } from '@/components/common/PageHeader';
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { cn } from '@/lib/cn';
import type { HistoryEntry } from '@/generated/HistoryEntry';

/**
 * Meetings page. Renders the small subset of history rows that came
 * from Meeting Mode (`is_meeting = 1` in the DB) as a card-based list
 * rather than the dense table /history uses for dictations.
 *
 * Two views, controlled by `selectedId` local state:
 *   - List: card per meeting with date, duration, speaker turn breakdown
 *     (derived from the transcript's `**You**:` / `**Other**:` tags),
 *     and a one-line preview pulled from the first chunk's body.
 *   - Detail: full timestamped transcript, copy/export buttons, and a
 *     dormant Summary section. The summary feature is not implemented:
 *     the offline LLM backend it depends on (llama.cpp + a small
 *     instruction-tuned model) has not been wired in this branch. The
 *     section exists in the DOM as a placeholder so the detail layout
 *     doesn't reshape when the feature does land.
 *
 * No reprocess button. The meeting transcribe pipeline isn't a
 * post-processing pass: rerunning it would mean re-chunking the audio,
 * which is an audio-level operation. Audio playback in the detail view
 * is also not wired yet. It would need an asset:// scheme bridge and a
 * player component.
 */
export function MeetingsRoute() {
  const [meetings, setMeetings] = useState<HistoryEntry[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const load = () => {
    setLoading(true);
    return invoke<HistoryEntry[]>('list_history', {
      filter: {
        is_meeting: true,
        limit: 200,
      },
    })
      .then((rows) => setMeetings(rows ?? []))
      .catch(() => setMeetings([]))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    void load();
  }, []);

  // Refresh when a new meeting transcript is committed. Rust emits
  // `dictation:result` after the history row is inserted; the payload
  // is the full HistoryEntry. We filter on `is_meeting` so dictation
  // entries don't trigger a redundant Meetings reload — they go to
  // /history, not here. Event-driven so the user staying on /meetings
  // sees the new row arrive without clicking around.
  useEffect(() => {
    const p = listen<HistoryEntry>(EVT.dictationResult, (e) => {
      if (e.payload?.is_meeting) {
        void load();
      }
    });
    return () => {
      void p.then((u) => u()).catch(() => undefined);
    };
  }, []);

  const selected = meetings.find((m) => m.id === selectedId) ?? null;

  if (selected) {
    return <MeetingDetail entry={selected} onBack={() => setSelectedId(null)} onDeleted={() => {
      setSelectedId(null);
      void load();
    }} />;
  }

  return (
    <div className="mx-auto w-full max-w-[880px] px-6 py-6">
      <PageHeader
        title="Meetings"
        subtitle="Recordings from Meeting Mode. Use Meeting Mode (Settings → Modes → Meeting) to record a call or long-form session."
      />

      {loading ? (
        <div className="mt-6 text-sm text-muted">Loading…</div>
      ) : meetings.length === 0 ? (
        <EmptyState />
      ) : (
        <div className="mt-6 space-y-3">
          {meetings.map((m) => (
            <MeetingCard
              key={m.id}
              entry={m}
              onClick={() => setSelectedId(m.id)}
              onDelete={async () => {
                if (
                  !window.confirm(
                    'Delete this meeting transcript? This cannot be undone.',
                  )
                ) {
                  return;
                }
                await invoke('delete_history_entry', { id: m.id });
                if (selectedId === m.id) setSelectedId(null);
                void load();
              }}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function EmptyState() {
  return (
    <div className="mt-12 rounded-input bg-bg-2 p-8 text-center">
      <Users className="mx-auto h-10 w-10 text-muted" />
      <h3 className="mt-3 text-base font-medium text-fg">No meetings yet</h3>
      <p className="mt-2 text-sm text-muted">
        Switch to Meeting Mode (Settings → Modes → Meeting), press your hotkey, and
        speak. Meetings save here when you stop recording. Both your microphone and
        your system audio are captured automatically, with timestamps and speaker tags.
      </p>
    </div>
  );
}

function MeetingCard({
  entry,
  onClick,
  onDelete,
}: {
  entry: HistoryEntry;
  onClick: () => void;
  onDelete: () => void;
}) {
  // Number(...) because ts-rs emits bigint for i64 fields.
  const createdAt = new Date(Number(entry.created_at));
  const durationSec = Math.max(1, Math.round(Number(entry.audio_duration_ms) / 1000));
  const mm = Math.floor(durationSec / 60);
  const ss = durationSec % 60;
  const breakdown = countSpeakerTurns(entry.processed_text);
  // First non-empty line of the body, stripped of timestamp + tag, capped
  // at ~120 chars. Gives a glanceable "what was this about" preview.
  const preview = firstLine(entry.processed_text);

  // Outer is a div with role=button rather than a <button> so the inner
  // delete affordance can also be a real button without nesting buttons
  // (invalid HTML, breaks keyboard navigation in unhelpful ways). Keyboard
  // and ARIA semantics preserved by hand: tabIndex + role + Enter/Space
  // handler.
  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick();
        }
      }}
      className="group block w-full cursor-pointer rounded-input bg-bg-2 p-4 text-left transition-colors hover:bg-bg-2/80 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-3 text-[12px] text-muted">
            <span>{createdAt.toLocaleString()}</span>
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {mm}:{ss.toString().padStart(2, '0')}
            </span>
            {(breakdown.you > 0 || breakdown.other > 0) && (
              <span className="flex items-center gap-1">
                <Users className="h-3 w-3" />
                You: {breakdown.you} · Other: {breakdown.other}
              </span>
            )}
          </div>
          <div className="mt-2 truncate text-sm text-fg">{preview || '(empty transcript)'}</div>
        </div>
        <div className="flex shrink-0 items-start gap-2">
          <div className="text-[11px] text-muted">{entry.mode_name_snapshot}</div>
          {/* Delete affordance. Hidden until card hover or focus to keep
              the list calm on first read; revealed when the user is
              already focused on this row. stopPropagation so the click
              doesn't bubble up and open the detail view. */}
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onDelete();
            }}
            aria-label="Delete meeting"
            title="Delete meeting"
            className="shrink-0 rounded p-1 text-muted opacity-0 transition-opacity hover:bg-rose-500/10 hover:text-rose-400 focus-visible:opacity-100 group-hover:opacity-100"
          >
            <Trash2 className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
}

function MeetingDetail({
  entry,
  onBack,
  onDeleted,
}: {
  entry: HistoryEntry;
  onBack: () => void;
  onDeleted: () => void;
}) {
  const createdAt = new Date(Number(entry.created_at));
  const durationSec = Math.max(1, Math.round(Number(entry.audio_duration_ms) / 1000));
  const mm = Math.floor(durationSec / 60);
  const ss = durationSec % 60;
  const breakdown = countSpeakerTurns(entry.processed_text);

  const [copied, setCopied] = useState(false);
  const onCopy = async () => {
    await navigator.clipboard.writeText(entry.processed_text);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1200);
  };

  const onExportTxt = async () => {
    await downloadAs(`meeting-${formatStamp(createdAt)}.txt`, entry.processed_text, 'text/plain');
  };

  const onExportMd = async () => {
    // Markdown export adds a small front-matter block so a future re-import
    // (or an LLM consuming the file) has the metadata it needs without
    // having to re-derive it from the body. Renders cleanly in any
    // markdown viewer that ignores unknown YAML.
    const md =
      `---\n` +
      `date: ${createdAt.toISOString()}\n` +
      `duration: ${mm}:${ss.toString().padStart(2, '0')}\n` +
      `mode: ${entry.mode_name_snapshot}\n` +
      `---\n\n` +
      `# Meeting transcript\n\n` +
      entry.processed_text +
      `\n`;
    await downloadAs(`meeting-${formatStamp(createdAt)}.md`, md, 'text/markdown');
  };

  const onDelete = async () => {
    if (!window.confirm('Delete this meeting transcript? This cannot be undone.')) return;
    await invoke('delete_history_entry', { id: entry.id });
    onDeleted();
  };

  return (
    <div className="mx-auto w-full max-w-[880px] px-6 py-6">
      <button
        onClick={onBack}
        className="mb-4 inline-flex items-center gap-2 text-sm text-muted hover:text-fg"
      >
        <ArrowLeft className="h-4 w-4" />
        Back to meetings
      </button>

      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-xl font-medium text-fg">
            {createdAt.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })}
          </h1>
          <div className="mt-2 flex flex-wrap items-center gap-3 text-[12px] text-muted">
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {mm}:{ss.toString().padStart(2, '0')}
            </span>
            {(breakdown.you > 0 || breakdown.other > 0) && (
              <span className="flex items-center gap-1">
                <Users className="h-3 w-3" />
                You: {breakdown.you} · Other: {breakdown.other}
              </span>
            )}
            <span>Model: {entry.asr_model_id}</span>
            <span>Mode: {entry.mode_name_snapshot}</span>
          </div>
        </div>
        <div className="flex shrink-0 gap-2">
          <ActionButton onClick={onCopy} icon={Copy} label={copied ? 'Copied' : 'Copy'} />
          <ActionButton onClick={onExportTxt} icon={Download} label=".txt" />
          <ActionButton onClick={onExportMd} icon={FileText} label=".md" />
          <ActionButton onClick={onDelete} icon={Trash2} label="Delete" tone="danger" />
        </div>
      </div>

      {/* Summary placeholder. The offline LLM backend (llama.cpp + a
          small instruction-tuned model) is not wired in this branch.
          The section exists in the DOM so the detail layout has the
          right shape when the feature lands; until then the body is
          honest about not being implemented yet. Removing this whole
          block when the feature ships is one small UI change, not a
          layout reshape. */}
      <div className="mt-6 rounded-input bg-bg-2 p-4">
        <div className="flex items-center gap-2 text-sm font-medium text-fg">
          <Sparkles className="h-4 w-4" />
          Summary
        </div>
        <p className="mt-2 text-[13px] text-muted">
          Auto-generated meeting summaries are not available yet. When they ship
          they will run locally on this machine. No cloud round-trip.
        </p>
      </div>

      {/* Transcript. Pre + whitespace-pre-wrap so the [mm:ss] timestamp
          prefixes and `**Tag**:` markers stack cleanly with chunk
          paragraph breaks intact (the chunker already inserts \n\n
          between chunks). */}
      <div className="mt-6">
        <div className="mb-2 text-sm font-medium text-fg">Transcript</div>
        <pre className="whitespace-pre-wrap rounded-input bg-bg-2 p-4 font-mono text-[13px] leading-relaxed text-fg">
          {entry.processed_text || '(empty)'}
        </pre>
      </div>
    </div>
  );
}

function ActionButton({
  onClick,
  icon: Icon,
  label,
  tone = 'default',
}: {
  onClick: () => void;
  icon: typeof Copy;
  label: string;
  tone?: 'default' | 'danger';
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'flex items-center gap-1.5 rounded-input px-3 py-1.5 text-[12px] transition-colors',
        tone === 'danger'
          ? 'bg-bg-2 text-muted hover:bg-rose-500/10 hover:text-rose-400'
          : 'bg-bg-2 text-fg hover:bg-bg-2/80',
      )}
    >
      <Icon className="h-3.5 w-3.5" />
      {label}
    </button>
  );
}

/**
 * Count `**You**` and `**Other**` speaker turns in the transcript.
 * Imperfect because the markers are part of the transcript body, but
 * the regex below skips matches that aren't at the start of a chunk
 * paragraph — we look for the boundary that always follows a chunk
 * timestamp like `[mm:ss] **You**:`.
 */
function countSpeakerTurns(transcript: string): { you: number; other: number } {
  const youRe = /\[\d{2}:\d{2}\]\s+\*\*You\*\*:/g;
  const otherRe = /\[\d{2}:\d{2}\]\s+\*\*Other\*\*:/g;
  return {
    you: (transcript.match(youRe) ?? []).length,
    other: (transcript.match(otherRe) ?? []).length,
  };
}

/** First non-empty line of body, stripped of `[mm:ss] **Tag**:` prefix. */
function firstLine(transcript: string): string {
  const lines = transcript.split('\n').map((l) => l.trim()).filter(Boolean);
  const first = lines[0] ?? '';
  // Strip "[mm:ss] **Tag**: " prefix if present so the preview shows
  // actual content not metadata.
  return first.replace(/^\[\d{2}:\d{2}\]\s+(\*\*\w+\*\*:\s+)?/, '').slice(0, 120);
}

/** "2026-04-28T11-30-15" — filename-safe ISO-ish stamp. */
function formatStamp(d: Date): string {
  const pad = (n: number) => n.toString().padStart(2, '0');
  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}` +
    `T${pad(d.getHours())}-${pad(d.getMinutes())}-${pad(d.getSeconds())}`
  );
}

/** Trigger a browser download with `text` as the file body. Tauri's
 * webview supports the standard a[download] flow. */
async function downloadAs(filename: string, text: string, mime: string) {
  const blob = new Blob([text], { type: mime });
  const url = URL.createObjectURL(blob);
  try {
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  } finally {
    // Revoke after the click handler has had a chance to start the
    // download. 0-tick timeout is enough — browsers preserve the
    // blob reference for the in-flight download.
    setTimeout(() => URL.revokeObjectURL(url), 0);
  }
}
