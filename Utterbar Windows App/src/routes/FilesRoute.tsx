import { useEffect, useState } from 'react';
import {
  CheckCircle2,
  CircleSlash,
  Copy,
  FileAudio,
  FolderOpen,
  Loader2,
  Mic,
  Podcast,
  Trash2,
  UploadCloud,
  Users,
  X,
  XCircle,
} from 'lucide-react';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

import { Button } from '@/components/common/Button';
import { PageHeader } from '@/components/common/PageHeader';
import { ProNudge } from '@/components/common/ProNudge';
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { cn } from '@/lib/cn';
import { useLicence, isPro } from '@/state/licence';

type FileJob = {
  id: string;
  source_path: string;
  status: 'queued' | 'running' | 'done' | 'error' | 'cancelled';
  progress: number;
  result_path: string | null;
  error_message: string | null;
};

type Progress = { id: string; progress: number; status: FileJob['status'] };

/**
 * Files — offline batch transcription.
 *
 * Per DESIGN_SYSTEM.md §4 the dropzone and job rows are flat surfaces
 * (no shadow, no outside border). The dropzone gets a dashed hairline
 * because the dashed pattern is itself the "drop here" affordance — this
 * is the one exception to the "no borders on cards" rule and only because
 * replacing the dash with surface tone would lose the metaphor.
 */
export function FilesRoute() {
  const [jobs, setJobs] = useState<FileJob[]>([]);
  const [dragActive, setDragActive] = useState(false);
  const [openJob, setOpenJob] = useState<FileJob | null>(null);
  // Brief inline toast: "Queued 14 files." Cleared after a few seconds.
  // Lives at the top of the dropzone so users immediately see that a
  // batch enqueue actually did something — the job rows arrive on the
  // next reload tick which feels slow for hundreds of files.
  const [batchToast, setBatchToast] = useState<string | null>(null);

  const reload = () =>
    invoke<FileJob[]>('list_jobs').then(setJobs).catch(() => setJobs([]));

  useEffect(() => {
    void reload();
  }, []);

  useEffect(() => {
    const p = listen<Progress>(EVT.fileJobProgress, (e) => {
      setJobs((prev) =>
        prev.map((j) =>
          j.id === e.payload.id
            ? { ...j, progress: e.payload.progress, status: e.payload.status }
            : j,
        ),
      );
      if (e.payload.status === 'done' || e.payload.status === 'error') void reload();
    });
    return () => void p.then((u) => u());
  }, []);

  // Tauri-native drag-drop. The HTML5 drop event in a Tauri webview
  // never sees real filesystem paths (the `File.path` property is a
  // browser-sandbox illusion that doesn't exist in webviews); previous
  // builds fell back to `f.name` and the Rust side then got "test.wav"
  // and tried to open a relative path → "file not found". Tauri's
  // window-level drag-drop event delivers full absolute paths.
  useEffect(() => {
    const win = getCurrentWebviewWindow();
    let unlisten: (() => void) | null = null;
    void win
      .onDragDropEvent((event) => {
        if (event.payload.type === 'enter' || event.payload.type === 'over') {
          setDragActive(true);
          return;
        }
        if (event.payload.type === 'leave') {
          setDragActive(false);
          return;
        }
        if (event.payload.type === 'drop') {
          setDragActive(false);
          // Each dropped path could be a file OR a folder. The Rust
          // side decides — `enqueue_path` checks `is_dir()` and routes
          // accordingly. Returns a count so we can show a "Queued N"
          // toast for the whole batch.
          void enqueueBatch(event.payload.paths);
        }
      })
      .then((u) => {
        unlisten = u;
      })
      .catch(() => undefined);
    return () => {
      if (unlisten) unlisten();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Single-path enqueue — can be a file OR folder. Backend routes via
  // `enqueue_path`. Returns the number of jobs created so the caller
  // can sum across a batch.
  const enqueuePath = async (path: string): Promise<number> => {
    try {
      const n = await invoke<number>('enqueue_path', {
        path,
        modeId: null,
        modelId: null,
      });
      return typeof n === 'number' ? n : 0;
    } catch (e) {
      console.warn('[utterbar] enqueue_path failed', path, e);
      return 0;
    }
  };

  // Multi-path orchestrator. Sums the per-path counts, surfaces a
  // single "Queued N file(s)" toast at the end, and reloads once.
  // Reloading per-path would hammer the IPC bridge for big batches.
  const enqueueBatch = async (paths: string[]) => {
    if (paths.length === 0) return;
    let total = 0;
    for (const p of paths) {
      total += await enqueuePath(p);
    }
    await reload();
    if (total > 0) {
      setBatchToast(`Queued ${total} ${total === 1 ? 'file' : 'files'}.`);
      window.setTimeout(() => setBatchToast(null), 4000);
    } else {
      setBatchToast('No supported audio or video files found.');
      window.setTimeout(() => setBatchToast(null), 4000);
    }
  };

  // Click-to-pick — files. Uses Tauri's native dialog so we get
  // absolute paths (HTML5 `<input type="file">` doesn't expose them
  // inside a webview).
  const pickFiles = async () => {
    try {
      const selected = await openDialog({
        multiple: true,
        filters: [
          {
            name: 'Audio / video',
            extensions: ['mp3', 'wav', 'm4a', 'flac', 'ogg', 'mp4', 'mov', 'mkv'],
          },
        ],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      void enqueueBatch(paths.filter((p): p is string => typeof p === 'string'));
    } catch (e) {
      console.warn('[utterbar] file picker failed', e);
    }
  };

  // Click-to-pick — a whole folder. The Rust side walks the tree
  // recursively, filters to supported extensions, and enqueues up to
  // 1,000 files in one go. Bigger trees get the first 1k and the user
  // can re-drop the rest after the queue drains.
  const pickFolder = async () => {
    try {
      const selected = await openDialog({ directory: true, multiple: false });
      if (!selected || typeof selected !== 'string') return;
      void enqueueBatch([selected]);
    } catch (e) {
      console.warn('[utterbar] folder picker failed', e);
    }
  };

  const licenceStatus = useLicence((s) => s.status);
  const pro = isPro(licenceStatus);

  // Free-tier users see only a ProNudge — the dropzone and job list are
  // Pro-only surfaces. The backend also rejects `enqueue_file`, so the
  // worst case is a stale job list from a previous Pro session (harmless).
  if (!pro) {
    return (
      <div className="mx-auto max-w-4xl p-8">
        <PageHeader
          title="File transcription"
          subtitle="Turn any recording into searchable text — locally, with no upload, no token cost, no cloud queue."
        />
        <UseCaseStrip />
        <ProNudge
          feature="File transcription"
          description="Drop a meeting recording, a podcast episode, or a voice memo and get a plain-text transcript next to the source file. Same Parakeet model that powers your dictation; nothing leaves your machine."
          className="mt-6"
        />
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-4xl p-8">
      <PageHeader
        title="File transcription"
        subtitle="Drop audio or video files to transcribe locally. Results land next to the source file as .utterbar.txt."
      />

      {jobs.length === 0 && <UseCaseStrip />}

      {/* Dropzone. Accepts files AND folders. Drag-drop via Tauri's
          window-level event (absolute paths only — HTML5 drop is
          suppressed in webviews by default). Click goes to the file
          picker; the "Pick folder" link below opens the directory
          picker for batch mode. */}
      <button
        type="button"
        onClick={() => void pickFiles()}
        className={cn(
          'mt-6 flex w-full cursor-pointer flex-col items-center justify-center gap-2 rounded-block p-12 text-center transition-colors',
          'border border-dashed',
          dragActive
            ? 'border-accent/60 bg-bg-2'
            : 'border-border-soft bg-bg-1 hover:bg-bg-2/60',
        )}
      >
        <UploadCloud className="h-8 w-8 text-fg-dim" />
        <div className="text-[14px] font-medium text-fg-hi">
          Drop files or a whole folder here
        </div>
        <div className="text-[12px] text-muted">
          Or click to pick files. Transcripts land next to each source file as
          <code className="mx-1 rounded bg-bg-2 px-1.5 py-0.5 font-mono text-[11px]">
            .utterbar.txt
          </code>
        </div>
        <div className="font-mono text-[11px] text-fg-dim">
          .mp3 · .wav · .m4a · .flac · .ogg · .mp4 · .mov · .mkv
        </div>
      </button>

      {/* Batch helpers. "Pick folder" lives below the dropzone rather
          than as a second giant button — most people will drag, this
          is a fallback for keyboard-only / picker-preferring users. */}
      <div className="mt-3 flex items-center justify-between gap-3 px-1 text-[11px] text-muted">
        <button
          type="button"
          onClick={() => void pickFolder()}
          className="inline-flex items-center gap-1.5 text-fg-dim transition-colors hover:text-accent"
        >
          <FolderOpen className="h-3 w-3" />
          Pick a folder instead — batch transcribe up to 1,000 files
        </button>
        {batchToast && (
          <span className="text-accent" role="status" aria-live="polite">
            {batchToast}
          </span>
        )}
      </div>

      <ul className="mt-6 space-y-2">
        {jobs.map((j) => {
          const done = j.status === 'done';
          // Done rows are clickable — opens the transcript modal.
          // Anything else is read-only with a Cancel option for in-flight jobs.
          const Row = done ? 'button' : 'div';
          return (
            <li key={j.id}>
              <Row
                {...(done
                  ? {
                      type: 'button' as const,
                      onClick: () => setOpenJob(j),
                    }
                  : {})}
                className={cn(
                  'flex w-full items-center gap-4 rounded-card bg-bg-1 px-4 py-3 text-left transition-colors',
                  done && 'cursor-pointer hover:bg-bg-2/60',
                )}
              >
                <FileAudio className="h-4 w-4 shrink-0 text-fg-dim" />
                <div className="flex-1 min-w-0">
                  <div className="truncate text-[13px] text-fg">{j.source_path}</div>
                  {j.error_message ? (
                    <div className="mt-1 text-[11px] text-rose">{j.error_message}</div>
                  ) : done ? (
                    <div className="mt-1 text-[11px] text-muted">
                      Click to read · copy · open in folder
                    </div>
                  ) : (
                    <div className="mt-1 h-1 w-full overflow-hidden rounded-full bg-bg-2">
                      <div
                        className={cn(
                          'h-full transition-[width] duration-300',
                          j.status === 'error' ? 'bg-rose' : 'bg-accent',
                        )}
                        style={{ width: `${Math.round((j.progress ?? 0) * 100)}%` }}
                      />
                    </div>
                  )}
                </div>
                <StatusPill status={j.status} />
                {j.status === 'running' || j.status === 'queued' ? (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      void invoke('cancel_job', { id: j.id }).then(reload);
                    }}
                    className="text-[11px] text-fg-dim transition-colors hover:text-rose"
                  >
                    Cancel
                  </button>
                ) : (
                  // Inline row delete. Available on done / error / cancelled
                  // rows so users can clear out finished or failed jobs
                  // without opening anything. Drops the row only — the
                  // .utterbar.txt file (when present) stays on disk; the
                  // modal's "Row + file" option is the way to remove that
                  // too. Errored rows have no file so this is the only
                  // path for clearing them.
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      void invoke('delete_job', { id: j.id, alsoDeleteFile: false }).then(reload);
                    }}
                    aria-label="Delete from list"
                    title="Delete from list (transcript file stays on disk)"
                    className="text-fg-dim transition-colors hover:text-rose"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                )}
              </Row>
            </li>
          );
        })}
      </ul>

      <TranscriptModal
        job={openJob}
        onClose={() => setOpenJob(null)}
        onDeleted={() => {
          setOpenJob(null);
          void reload();
        }}
      />
    </div>
  );
}

/**
 * Modal showing the finished transcript with copy / show-in-folder /
 * delete actions. Lightweight — no router state, no portal library;
 * just a fixed overlay that takes over the viewport when `job` is set.
 *
 * The transcript is loaded lazily on open so the row click stays fast
 * even for very long transcripts. Failed reads (file moved, deleted)
 * surface inline so the user understands why.
 */
function TranscriptModal({
  job,
  onClose,
  onDeleted,
}: {
  job: FileJob | null;
  onClose: () => void;
  onDeleted: () => void;
}) {
  const [text, setText] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  useEffect(() => {
    if (!job) {
      setText(null);
      setError(null);
      setCopied(false);
      setConfirmingDelete(false);
      return;
    }
    setLoading(true);
    setError(null);
    invoke<string>('read_job_transcript', { id: job.id })
      .then((t) => {
        setText(t);
        setLoading(false);
      })
      .catch((e) => {
        const msg = (e as { message?: string } | null)?.message;
        setError(typeof msg === 'string' && msg ? msg : 'Could not read the transcript.');
        setLoading(false);
      });
  }, [job]);

  // Esc closes.
  useEffect(() => {
    if (!job) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [job, onClose]);

  if (!job) return null;

  const copy = async () => {
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1600);
    } catch {
      setError('Could not copy to clipboard.');
    }
  };

  const reveal = () => {
    void invoke('reveal_job_in_folder', { id: job.id }).catch((e) => {
      const msg = (e as { message?: string } | null)?.message;
      setError(typeof msg === 'string' && msg ? msg : 'Could not open the folder.');
    });
  };

  const remove = async (alsoFile: boolean) => {
    try {
      await invoke('delete_job', { id: job.id, alsoDeleteFile: alsoFile });
      onDeleted();
    } catch (e) {
      const msg = (e as { message?: string } | null)?.message;
      setError(typeof msg === 'string' && msg ? msg : 'Could not delete.');
    }
  };

  // Filename only for the header — full path in a small line below.
  const filename =
    job.source_path.split(/[\\/]/).pop() || job.source_path;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 px-6 py-10"
      onClick={onClose}
    >
      <div
        className="flex max-h-[80vh] w-full max-w-2xl flex-col rounded-card bg-bg-1"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="flex items-start justify-between gap-4 px-5 pt-4 pb-3">
          <div className="min-w-0 flex-1">
            <div className="truncate text-[14px] font-medium text-fg-hi">{filename}</div>
            <div className="mt-0.5 truncate font-mono text-[11px] text-muted">
              {job.source_path}
            </div>
          </div>
          <button
            onClick={onClose}
            aria-label="Close"
            className="text-muted transition-colors hover:text-fg-hi"
          >
            <X className="h-4 w-4" />
          </button>
        </header>

        <div className="flex-1 overflow-y-auto border-y border-border-soft/40 bg-bg-0 px-5 py-4">
          {loading ? (
            <div className="flex items-center gap-2 text-[12px] text-muted">
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
              Loading transcript…
            </div>
          ) : error ? (
            <div className="text-[13px] text-rose">{error}</div>
          ) : (
            <pre className="whitespace-pre-wrap font-sans text-[13px] leading-relaxed text-fg">
              {text}
            </pre>
          )}
        </div>

        <footer className="flex flex-wrap items-center justify-between gap-2 px-5 py-3">
          <div className="flex flex-wrap items-center gap-2">
            <Button variant="cta" size="sm" onClick={() => void copy()} disabled={!text}>
              <Copy className="h-3 w-3" />
              {copied ? 'Copied' : 'Copy'}
            </Button>
            <Button variant="ghost" size="sm" noLed onClick={reveal}>
              <FolderOpen className="h-3 w-3" />
              Show in folder
            </Button>
          </div>
          {confirmingDelete ? (
            <div className="flex items-center gap-2 text-[12px]">
              <span className="text-muted">Delete the row, the .txt file too?</span>
              <button
                onClick={() => void remove(false)}
                className="rounded-input bg-bg-2 px-2 py-1 text-[11px] text-fg-hi hover:bg-bg-2/70"
              >
                Row only
              </button>
              <button
                onClick={() => void remove(true)}
                className="rounded-input bg-rose/20 px-2 py-1 text-[11px] text-rose hover:bg-rose/30"
              >
                Row + file
              </button>
              <button
                onClick={() => setConfirmingDelete(false)}
                className="text-[11px] text-muted hover:text-fg-hi"
              >
                Cancel
              </button>
            </div>
          ) : (
            <button
              onClick={() => setConfirmingDelete(true)}
              className="inline-flex items-center gap-1 text-[12px] text-muted transition-colors hover:text-rose"
            >
              <Trash2 className="h-3 w-3" />
              Delete
            </button>
          )}
        </footer>
      </div>
    </div>
  );
}

/**
 * Status pill — inline state read next to each job row. Uses the same
 * palette discipline as StatePill: moss for done, rose for error, accent
 * (spinner) for running, muted for idle. Never stack more than ~3 of
 * these coloured states at once — the rail + progress bar already eat
 * into the cyan budget on this page.
 */
function StatusPill({ status }: { status: FileJob['status'] }) {
  const map = {
    queued:    { icon: Loader2,      label: 'Queued',    cls: 'text-fg-dim' },
    running:   { icon: Loader2,      label: 'Running',   cls: 'text-accent animate-spin' },
    done:      { icon: CheckCircle2, label: 'Done',      cls: 'text-moss' },
    error:     { icon: XCircle,      label: 'Error',     cls: 'text-rose' },
    cancelled: { icon: CircleSlash,  label: 'Cancelled', cls: 'text-fg-dim' },
  } as const;
  const { icon: Icon, label, cls } = map[status];
  return (
    <span className="inline-flex items-center gap-1 font-mono text-[11px] text-muted">
      <Icon className={cn('h-3.5 w-3.5', cls)} />
      <span className="w-20 text-right">{label}</span>
    </span>
  );
}

/**
 * Three concrete use-case chips shown above the dropzone when there are
 * no jobs in flight (and on the Free state). Sets a clear "this is what
 * the feature is actually for" expectation up front, instead of leaving
 * the user with a generic dropzone and no hint about why they'd use it.
 *
 * Kept tight — three real workflows, one short sentence each. Anything
 * longer crowds the page; anything shorter is decorative.
 */
function UseCaseStrip() {
  const items = [
    {
      icon: Users,
      title: 'Meeting recordings',
      body: 'Drop a Teams or Zoom recording — get a searchable transcript next to the file, with no upload to a meeting-notes service.',
    },
    {
      icon: Podcast,
      title: 'Podcast & video',
      body: 'Transcribe an episode or video draft locally for show notes, captioning, or quote-mining.',
    },
    {
      icon: Mic,
      title: 'Voice memos',
      body: 'Long-form thinking captured on phone or recorder. Drop the file here and read it back as text.',
    },
  ];
  return (
    <div className="mt-6 grid grid-cols-1 gap-3 sm:grid-cols-3">
      {items.map((it) => (
        <div
          key={it.title}
          className="rounded-card bg-bg-1 p-4"
        >
          <div className="flex items-center gap-2">
            <it.icon className="h-3.5 w-3.5 text-accent" />
            <div className="text-[13px] font-medium text-fg-hi">{it.title}</div>
          </div>
          <p className="mt-1.5 text-[12px] leading-relaxed text-muted">{it.body}</p>
        </div>
      ))}
    </div>
  );
}
