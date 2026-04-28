import { useEffect, useMemo, useState } from 'react';
import { ArrowLeft, Copy, Download, Trash2, Users, Clock, FileText, Sparkles, Pencil } from 'lucide-react';

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
 * v0.15.0: speaker labels are now real diarised cluster ids
 * (`**Speaker 1:**`, `**Speaker 2:**`, …) produced by sherpa-onnx
 * pyannote segmentation + NVIDIA TitaNet small embedding +
 * Whisper Large per segment. The user can rename them post-hoc:
 * the rename map persists per-row in `history.speaker_names`,
 * substitution happens at render time only — `processed_text`
 * stays canonical so re-rename / clear-rename are non-destructive.
 *
 * Older meeting rows (v0.13.x — v0.14.x) had no `**Speaker N:**`
 * tokens at all — just `[mm:ss] text` paragraphs. They render
 * here unchanged: the substitution helpers are no-ops on text
 * that doesn't contain matching tokens, and the speaker-count
 * helper returns zero for them.
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

  // Refresh when a new meeting transcript is committed.
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
    return (
      <MeetingDetail
        entry={selected}
        onBack={() => setSelectedId(null)}
        onDeleted={() => {
          setSelectedId(null);
          void load();
        }}
        onRenamed={(updated) => {
          setMeetings((prev) => prev.map((m) => (m.id === updated.id ? updated : m)));
        }}
      />
    );
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
        your system audio are captured automatically, with timestamps and speaker
        labels (when the diariser models are present).
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
  const createdAt = new Date(Number(entry.created_at));
  const durationSec = Math.max(1, Math.round(Number(entry.audio_duration_ms) / 1000));
  const mm = Math.floor(durationSec / 60);
  const ss = durationSec % 60;
  const speakerCount = countSpeakers(entry.processed_text);
  const renamed = parseRenameMap(entry.speaker_names ?? null);
  const preview = firstLine(applyRenameMap(entry.processed_text, renamed));

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
            {speakerCount > 0 && (
              <span className="flex items-center gap-1">
                <Users className="h-3 w-3" />
                {speakerCount === 1 ? '1 speaker' : `${speakerCount} speakers`}
              </span>
            )}
          </div>
          <div className="mt-2 truncate text-sm text-fg">{preview || '(empty transcript)'}</div>
        </div>
        <div className="flex shrink-0 items-start gap-2">
          <div className="text-[11px] text-muted">{entry.mode_name_snapshot}</div>
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
  onRenamed,
}: {
  entry: HistoryEntry;
  onBack: () => void;
  onDeleted: () => void;
  onRenamed: (updated: HistoryEntry) => void;
}) {
  const createdAt = new Date(Number(entry.created_at));
  const durationSec = Math.max(1, Math.round(Number(entry.audio_duration_ms) / 1000));
  const mm = Math.floor(durationSec / 60);
  const ss = durationSec % 60;

  // Rename map is local state seeded from the row, mutated by the
  // inline rename surface, persisted via set/clear_meeting_speaker_names.
  const [renameMap, setRenameMap] = useState<Record<string, string>>(() =>
    parseRenameMap(entry.speaker_names ?? null),
  );

  // Distinct speaker ids found in the transcript — drives the rename
  // surface (one inline edit per id) and the count badge.
  const speakerIds = useMemo(() => distinctSpeakerIds(entry.processed_text), [entry.processed_text]);

  const renderedTranscript = useMemo(
    () => applyRenameMap(entry.processed_text, renameMap),
    [entry.processed_text, renameMap],
  );

  const persistRename = async (next: Record<string, string>) => {
    // Optimistic UI update — feels instant on the happy path. On
    // failure (validation error, missing meeting row, DB error,
    // etc.) we roll back to the persisted state from `entry`. Both
    // set and clear paths share the same try/catch contract so a
    // failed clear doesn't leave the UI showing renames-cleared
    // while the DB still has them.
    const isClear = Object.keys(next).length === 0;
    setRenameMap(next);
    try {
      if (isClear) {
        await invoke('clear_meeting_speaker_names', { historyId: entry.id });
        onRenamed({ ...entry, speaker_names: null });
      } else {
        await invoke('set_meeting_speaker_names', {
          historyId: entry.id,
          names: next,
        });
        onRenamed({ ...entry, speaker_names: JSON.stringify(next) });
      }
    } catch (e) {
      // Surface a friendly message when the backend gave us one
      // (Tauri stringifies OrliqError::User as a plain string), and
      // a generic fallback otherwise. The rollback restores the
      // local state from the canonical `entry.speaker_names` so
      // the user sees the persisted rename map again.
      console.error(
        isClear
          ? 'clear_meeting_speaker_names failed'
          : 'set_meeting_speaker_names failed',
        e,
      );
      const fallbackMessage = isClear
        ? 'Could not clear speaker names. The meeting may have been deleted; try refreshing.'
        : 'Speaker name rejected. Names must be 1–64 characters with no control characters.';
      window.alert(typeof e === 'string' ? e : fallbackMessage);
      setRenameMap(parseRenameMap(entry.speaker_names ?? null));
    }
  };

  const [copied, setCopied] = useState(false);
  const onCopy = async () => {
    await navigator.clipboard.writeText(renderedTranscript);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1200);
  };

  const onExportTxt = async () => {
    await downloadAs(`meeting-${formatStamp(createdAt)}.txt`, renderedTranscript, 'text/plain');
  };

  const onExportMd = async () => {
    const md =
      `---\n` +
      `date: ${createdAt.toISOString()}\n` +
      `duration: ${mm}:${ss.toString().padStart(2, '0')}\n` +
      `mode: ${entry.mode_name_snapshot}\n` +
      `---\n\n` +
      `# Meeting transcript\n\n` +
      renderedTranscript +
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
            {speakerIds.length > 0 && (
              <span className="flex items-center gap-1">
                <Users className="h-3 w-3" />
                {speakerIds.length === 1 ? '1 speaker' : `${speakerIds.length} speakers`}
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

      {/* Speaker rename surface — only shown when the transcript
          carries `**Speaker N:**` tokens (i.e. v0.15+ diarised
          rows). Old meeting rows without diarised labels skip this
          block entirely. */}
      {speakerIds.length > 0 && (
        <div className="mt-6 rounded-input bg-bg-2 p-4">
          <div className="mb-3 flex items-center gap-2 text-sm font-medium text-fg">
            <Pencil className="h-4 w-4" />
            Speakers
          </div>
          <div className="space-y-2">
            {speakerIds.map((id) => (
              <SpeakerRenameRow
                key={id}
                speakerId={id}
                value={renameMap[id.toString()] ?? ''}
                onCommit={async (newName) => {
                  const next = { ...renameMap };
                  if (newName.trim().length === 0) {
                    delete next[id.toString()];
                  } else {
                    next[id.toString()] = newName.trim();
                  }
                  await persistRename(next);
                }}
              />
            ))}
          </div>
          {Object.keys(renameMap).length > 0 && (
            <button
              type="button"
              onClick={() => persistRename({})}
              className="mt-3 text-[12px] text-muted hover:text-fg"
            >
              Clear all renames
            </button>
          )}
        </div>
      )}

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

      <div className="mt-6">
        <div className="mb-2 text-sm font-medium text-fg">Transcript</div>
        <pre className="whitespace-pre-wrap rounded-input bg-bg-2 p-4 font-mono text-[13px] leading-relaxed text-fg">
          {renderedTranscript || '(empty)'}
        </pre>
      </div>
    </div>
  );
}

function SpeakerRenameRow({
  speakerId,
  value,
  onCommit,
}: {
  speakerId: number;
  value: string;
  onCommit: (newName: string) => Promise<void>;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  // Keep `draft` in sync if the underlying value updates from
  // outside (e.g. the user opens the same meeting in a second
  // window and renames there — the listener on EVT_DICTATION_RESULT
  // wouldn't catch this today but the prop sync still keeps the row
  // honest).
  useEffect(() => {
    setDraft(value);
  }, [value]);

  if (!editing) {
    return (
      <div className="flex items-center gap-3 text-[13px]">
        <span className="w-24 shrink-0 text-muted">Speaker {speakerId}</span>
        <span className="flex-1 truncate text-fg">{value || `(default — Speaker ${speakerId})`}</span>
        <button
          type="button"
          onClick={() => setEditing(true)}
          className="text-[12px] text-muted hover:text-fg"
        >
          {value ? 'Edit' : 'Rename'}
        </button>
      </div>
    );
  }

  return (
    <form
      onSubmit={async (e) => {
        e.preventDefault();
        await onCommit(draft);
        setEditing(false);
      }}
      className="flex items-center gap-3 text-[13px]"
    >
      <span className="w-24 shrink-0 text-muted">Speaker {speakerId}</span>
      <input
        type="text"
        autoFocus
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        maxLength={64}
        placeholder={`Speaker ${speakerId}`}
        className="flex-1 rounded-input bg-bg-1 px-2 py-1 text-[13px] text-fg outline-none focus:ring-1 focus:ring-accent"
      />
      <button
        type="submit"
        className="rounded-input bg-accent/10 px-2 py-1 text-[12px] text-accent hover:bg-accent/20"
      >
        Save
      </button>
      <button
        type="button"
        onClick={() => {
          setDraft(value);
          setEditing(false);
        }}
        className="rounded-input px-2 py-1 text-[12px] text-muted hover:text-fg"
      >
        Cancel
      </button>
    </form>
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

// ── Helpers ──────────────────────────────────────────────────────────

/**
 * Substitute `**Speaker N:**` literal tokens in the transcript with
 * `**<renamed>:**` per the rename map. Strict regex — only matches
 * the canonical bold-bold-Speaker-N-colon-bold-bold form. Plain
 * "Speaker 1" text in the body (lowercase, no markdown) is never
 * touched.
 *
 * If the speaker id from the regex isn't in the rename map, the
 * substitution leaves the original token unchanged. That means a
 * partial rename map (`{1: "Jay"}` with Speaker 2 + Speaker 3 still
 * present in the body) renders as: `**Jay:**` for id 1, `**Speaker
 * 2:**` for id 2, `**Speaker 3:**` for id 3.
 */
function applyRenameMap(transcript: string, renameMap: Record<string, string>): string {
  if (Object.keys(renameMap).length === 0) return transcript;
  return transcript.replace(/\*\*Speaker (\d+):\*\*/g, (full, idStr: string) => {
    const renamed = renameMap[idStr];
    if (!renamed) return full;
    return `**${renamed}:**`;
  });
}

/**
 * Parse the JSON-encoded `speaker_names` column into a flat
 * { id-as-string → display name } object. Returns `{}` for null,
 * undefined, empty string, or invalid JSON. We never throw — bad
 * data shows as "no rename" rather than breaking the page.
 */
function parseRenameMap(json: string | null): Record<string, string> {
  if (!json) return {};
  try {
    const parsed = JSON.parse(json);
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
      const out: Record<string, string> = {};
      for (const [k, v] of Object.entries(parsed)) {
        if (typeof v === 'string') out[k] = v;
      }
      return out;
    }
  } catch {
    // Malformed — fall through to empty.
  }
  return {};
}

/** Distinct sherpa speaker ids appearing in the transcript, sorted ascending. */
function distinctSpeakerIds(transcript: string): number[] {
  const found = new Set<number>();
  const re = /\*\*Speaker (\d+):\*\*/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(transcript)) !== null) {
    const id = Number(m[1]);
    if (Number.isFinite(id)) found.add(id);
  }
  return [...found].sort((a, b) => a - b);
}

/**
 * How many distinct speakers appear in the transcript. Returns 0
 * for the v0.13.x — v0.14.x format that had no `**Speaker N:**`
 * tokens — those rows just render their `[mm:ss] text` paragraphs
 * unchanged and the speaker badge stays hidden.
 */
function countSpeakers(transcript: string): number {
  return distinctSpeakerIds(transcript).length;
}

/** First non-empty line of body, stripped of `[mm:ss] **Tag**:` prefix. */
function firstLine(transcript: string): string {
  const lines = transcript.split('\n').map((l) => l.trim()).filter(Boolean);
  const first = lines[0] ?? '';
  // Strip "[mm:ss] **Anything**: " prefix if present so the preview
  // shows actual content not metadata. Now matches both
  // **Speaker N:** (v0.15+) and any future label shape, plus the
  // older **You:** / **Other:** if a stale row still has them.
  return first.replace(/^\[\d{2}:\d{2}\]\s+(\*\*[^*]+\*\*:\s+)?/, '').slice(0, 120);
}

/** "2026-04-28T11-30-15" — filename-safe ISO-ish stamp. */
function formatStamp(d: Date): string {
  const pad = (n: number) => n.toString().padStart(2, '0');
  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}` +
    `T${pad(d.getHours())}-${pad(d.getMinutes())}-${pad(d.getSeconds())}`
  );
}

/** Trigger a browser download with `text` as the file body. */
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
    setTimeout(() => URL.revokeObjectURL(url), 0);
  }
}
