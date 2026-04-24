import { useEffect, useRef, useState } from 'react';
import { CheckCircle2, CircleSlash, FileAudio, Loader2, UploadCloud, XCircle } from 'lucide-react';

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
  const inputRef = useRef<HTMLInputElement>(null);

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

  const enqueue = async (path: string) => {
    await invoke('enqueue_file', { sourcePath: path, modeId: null, modelId: null });
    await reload();
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
          subtitle="Drop audio or video files to transcribe locally. Results land next to the source file as .utterbar.txt."
        />
        <ProNudge
          feature="File transcription"
          description="Transcribe any audio or video file from disk — meetings, voice memos, lectures — with the same local-only Parakeet model that powers dictation. Nothing leaves your machine."
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

      <label
        onDragOver={(e) => {
          e.preventDefault();
          setDragActive(true);
        }}
        onDragLeave={() => setDragActive(false)}
        onDrop={(e) => {
          e.preventDefault();
          setDragActive(false);
          const files = Array.from(e.dataTransfer.files);
          files.forEach((f) => {
            void enqueue((f as unknown as { path?: string }).path ?? f.name);
          });
        }}
        className={cn(
          'mt-6 flex cursor-pointer flex-col items-center justify-center gap-2 rounded-block p-12 text-center transition-colors',
          'border border-dashed',
          dragActive
            ? 'border-accent/60 bg-bg-2'
            : 'border-border-soft bg-bg-1 hover:bg-bg-2/60',
        )}
      >
        <UploadCloud className="h-8 w-8 text-fg-dim" />
        <div className="text-[14px] font-medium text-fg-hi">Drop audio or video here</div>
        <div className="font-mono text-[11px] text-fg-dim">
          .mp3 · .wav · .m4a · .flac · .ogg · .mp4 · .mov · .mkv
        </div>
        <input
          ref={inputRef}
          type="file"
          multiple
          className="hidden"
          onChange={(e) => {
            const files = Array.from(e.target.files ?? []);
            files.forEach((f) =>
              void enqueue((f as unknown as { path?: string }).path ?? f.name),
            );
          }}
        />
      </label>

      <ul className="mt-6 space-y-2">
        {jobs.map((j) => (
          <li
            key={j.id}
            className="flex items-center gap-4 rounded-card bg-bg-1 px-4 py-3"
          >
            <FileAudio className="h-4 w-4 text-fg-dim" />
            <div className="flex-1 min-w-0">
              <div className="truncate text-[13px] text-fg">{j.source_path}</div>
              {j.error_message ? (
                <div className="mt-1 text-[11px] text-rose">{j.error_message}</div>
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
                onClick={() => invoke('cancel_job', { id: j.id }).then(reload)}
                className="text-[11px] text-fg-dim transition-colors hover:text-rose"
              >
                Cancel
              </button>
            ) : null}
          </li>
        ))}
      </ul>
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
