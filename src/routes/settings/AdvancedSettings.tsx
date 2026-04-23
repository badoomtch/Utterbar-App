import { useEffect, useState } from 'react';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { invoke } from '@/lib/ipc';
import { useSettings } from '@/state/settings';

type Diagnostics = {
  version: string;
  os: string;
  cpu_count: number;
  log_path: string;
  db_path: string;
  models_path: string;
  onnx_feature: boolean;
  llama_feature: boolean;
};

type GpuAdapter = {
  name: string;
  dedicated_video_memory_mb: number;
  dedicated_system_memory_mb: number;
  shared_system_memory_mb: number;
  is_software: boolean;
  vendor: 'nvidia' | 'amd' | 'intel' | 'qualcomm' | 'microsoft' | 'other';
};

type GpuProbe = {
  adapters: GpuAdapter[];
  has_hardware_gpu: boolean;
};

type GpuAcceleration = 'auto' | 'direct_ml' | 'cuda' | 'cpu';

const GPU_LABELS: Record<GpuAcceleration, { title: string; hint: string }> = {
  auto: {
    title: 'Auto',
    hint: 'Use your GPU if one is detected, otherwise CPU. Recommended.',
  },
  direct_ml: {
    title: 'DirectML',
    hint: 'Force DirectML. Runs on any DX12 GPU — NVIDIA, AMD, or Intel.',
  },
  cuda: {
    title: 'CUDA',
    hint: 'Force NVIDIA CUDA. Fastest on NVIDIA cards.',
  },
  cpu: {
    title: 'CPU only',
    hint: 'Keep the GPU free for games or video. A bit slower per utterance.',
  },
};

type ExportSummary = {
  path: string;
  bytes: number;
  included: string[];
};

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function defaultExportName(): string {
  // YYYYMMDD-HHMMSS — sorts naturally in Explorer and reads unambiguously
  // regardless of the user's locale-specific date formatting.
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  return `utterbar-backup-${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}-${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}.tar`;
}

export function AdvancedSettings() {
  const s = useSettings((x) => x.data);
  const reset = useSettings((x) => x.reset);
  const patch = useSettings((x) => x.patch);
  const [diag, setDiag] = useState<Diagnostics | null>(null);
  const [gpu, setGpu] = useState<GpuProbe | null>(null);

  // Backup / restore state. We keep it local (not persisted) — the "last
  // export" hint is a session courtesy, not a durable record, and resetting
  // it on navigation is the right default.
  const [includeRecordings, setIncludeRecordings] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);
  const [lastExport, setLastExport] = useState<ExportSummary | null>(null);
  const [backupError, setBackupError] = useState<string | null>(null);
  const [pendingImportPath, setPendingImportPath] = useState<string | null>(null);

  useEffect(() => {
    void invoke<Diagnostics>('get_diagnostics').then(setDiag).catch(() => undefined);
    void invoke<GpuProbe>('probe_gpu').then(setGpu).catch(() => undefined);
  }, []);

  if (!s) return null;

  const runExport = async () => {
    setBackupError(null);
    try {
      const dest = await saveDialog({
        title: 'Export Utterbar data',
        defaultPath: defaultExportName(),
        filters: [{ name: 'Utterbar backup', extensions: ['tar'] }],
      });
      if (!dest) return;
      setExporting(true);
      const summary = await invoke<ExportSummary>('export_user_data', {
        dest,
        includeRecordings,
      });
      setLastExport(summary);
    } catch (e) {
      setBackupError(e instanceof Error ? e.message : String(e));
    } finally {
      setExporting(false);
    }
  };

  const pickImport = async () => {
    setBackupError(null);
    try {
      const picked = await openDialog({
        title: 'Import Utterbar backup',
        multiple: false,
        directory: false,
        filters: [{ name: 'Utterbar backup', extensions: ['tar'] }],
      });
      if (!picked || typeof picked !== 'string') return;
      setPendingImportPath(picked);
    } catch (e) {
      setBackupError(e instanceof Error ? e.message : String(e));
    }
  };

  const confirmImport = async () => {
    if (!pendingImportPath) return;
    setImporting(true);
    setBackupError(null);
    try {
      await invoke('import_user_data', { src: pendingImportPath });
      // The Rust side schedules a restart 300ms after returning; leave the
      // modal in "importing" state so the UI reads as "this is happening"
      // until the process tears down.
    } catch (e) {
      setBackupError(e instanceof Error ? e.message : String(e));
      setImporting(false);
      setPendingImportPath(null);
    }
  };

  const setGpuPref = (value: GpuAcceleration) =>
    patch({ advanced: { ...s.advanced, gpu_acceleration: value } });

  const current = (s.advanced as { gpu_acceleration?: GpuAcceleration }).gpu_acceleration ?? 'auto';
  const bestAdapter = gpu?.adapters.find((a) => !a.is_software);

  return (
    <div className="mx-auto max-w-3xl p-8">
      <PageHeader title="Advanced" subtitle="Diagnostics, hardware preferences, and experimental toggles." />
      <div className="mt-6 space-y-4">
        <Card
          title="GPU acceleration"
          description={
            gpu
              ? gpu.has_hardware_gpu && bestAdapter
                ? `Detected ${bestAdapter.name}. Running inference on the GPU is typically 3–10× faster than CPU, at the cost of briefly competing with other GPU apps (games, 3D tools, heavy browser pages).`
                : 'No dedicated or integrated GPU detected. Utterbar will run on CPU. This is fine for short dictation but longer recordings will feel sluggish.'
              : 'Checking what hardware is available…'
          }
        >
          <div className="space-y-2">
            {(['auto', 'direct_ml', 'cuda', 'cpu'] as GpuAcceleration[]).map((opt) => {
              const disabled =
                !gpu?.has_hardware_gpu && (opt === 'direct_ml' || opt === 'cuda');
              const selected = current === opt;
              return (
                <label
                  key={opt}
                  className={`group relative flex items-start gap-3 rounded-input p-3 text-[12px] transition-colors ${
                    selected ? 'bg-bg-2' : 'bg-bg-2/40 hover:bg-bg-2/70'
                  } ${disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'}`}
                >
                  <span
                    className={`absolute left-0 top-2 bottom-2 w-[2px] rounded-r-full transition-opacity ${
                      selected ? 'bg-accent opacity-100' : 'opacity-0'
                    }`}
                    aria-hidden
                  />
                  <input
                    type="radio"
                    name="gpu_acceleration"
                    value={opt}
                    checked={selected}
                    disabled={disabled}
                    onChange={() => void setGpuPref(opt)}
                    className="mt-0.5 accent-[var(--accent)]"
                  />
                  <span className="flex-1 pl-1">
                    <span className="font-medium text-fg-hi">{GPU_LABELS[opt].title}</span>
                    <span className="mt-0.5 block text-muted leading-relaxed">
                      {GPU_LABELS[opt].hint}
                      {disabled && ' — not available on this machine.'}
                    </span>
                  </span>
                </label>
              );
            })}
          </div>
          {gpu && gpu.adapters.length > 0 && (
            <div className="mt-3 border-t border-border-soft/50 pt-3 text-[11px] text-muted">
              <div className="ub-eyebrow mb-1.5">Detected adapters</div>
              <ul className="space-y-0.5 font-mono">
                {gpu.adapters.map((a, i) => (
                  <li key={i}>
                    {a.name}
                    {a.is_software && ' (software)'}
                    {a.dedicated_video_memory_mb > 0 &&
                      ` — ${a.dedicated_video_memory_mb.toLocaleString()} MB VRAM`}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </Card>

        <Card title="Diagnostics" description="Everything below is read from this machine. Nothing is sent anywhere.">
          {diag && (
            <dl className="grid grid-cols-[160px_1fr] gap-y-2 text-[12px]">
              <dt className="text-muted">Version</dt>
              <dd className="font-mono text-fg">{diag.version}</dd>
              <dt className="text-muted">OS</dt>
              <dd className="font-mono text-fg">{diag.os}</dd>
              <dt className="text-muted">CPU threads</dt>
              <dd className="font-mono text-fg">{diag.cpu_count}</dd>
              <dt className="text-muted">ONNX backend</dt>
              <dd className="font-mono text-fg">{diag.onnx_feature ? 'compiled' : 'disabled'}</dd>
              <dt className="text-muted">llama.cpp backend</dt>
              <dd className="font-mono text-fg">{diag.llama_feature ? 'compiled' : 'disabled'}</dd>
              <dt className="text-muted">Logs</dt>
              <dd className="break-all font-mono text-fg">
                <button
                  onClick={() => void invoke('open_path', { path: diag.log_path })}
                  className="transition-colors hover:text-accent"
                >
                  {diag.log_path}
                </button>
              </dd>
              <dt className="text-muted">Database</dt>
              <dd className="break-all font-mono text-fg">{diag.db_path}</dd>
              <dt className="text-muted">Models</dt>
              <dd className="break-all font-mono text-fg">
                <button
                  onClick={() => void invoke('open_path', { path: diag.models_path })}
                  className="transition-colors hover:text-accent"
                >
                  {diag.models_path}
                </button>
              </dd>
            </dl>
          )}
        </Card>

        <Card
          title="Backup & restore"
          description="Export your settings, modes, vocabulary, and history to a single file you can move between machines or keep as a safety net. Models are excluded — they're re-downloadable from the catalogue."
        >
          <Row
            label="Include recordings in export"
            description="Audio files from history. Usually the bulk of the archive — leave off when you just want to move preferences."
          >
            <Toggle
              checked={includeRecordings}
              onChange={setIncludeRecordings}
              label="Include recordings in export"
            />
          </Row>
          <Row
            label="Export data"
            description={
              lastExport
                ? `Last export: ${formatBytes(lastExport.bytes)} — included ${lastExport.included.join(', ') || 'nothing'}.`
                : 'Writes a .tar file to the location you choose.'
            }
          >
            <button
              onClick={() => void runExport()}
              disabled={exporting}
              className="ub-btn !py-[0.4rem] !text-[12px]"
            >
              {exporting ? 'Exporting…' : 'Export…'}
            </button>
          </Row>
          <Row
            label="Import data"
            description="Replaces current settings, modes, vocabulary, and history with the backup's contents. Utterbar restarts immediately after."
          >
            <button
              onClick={() => void pickImport()}
              disabled={importing}
              className="ub-btn !py-[0.4rem] !text-[12px]"
            >
              {importing ? 'Importing…' : 'Import…'}
            </button>
          </Row>
          {backupError && (
            <div className="mt-3 rounded-input bg-danger/10 px-3 py-2 text-[12px] text-danger">
              {backupError}
            </div>
          )}
        </Card>

        <Card title="Reset">
          <Row
            label="Reset all settings to defaults"
            description="Modes, vocabulary, and history are preserved."
          >
            <button
              onClick={() => {
                if (
                  confirm(
                    'Reset all Utterbar settings? This does not delete recordings or history.',
                  )
                ) {
                  void reset();
                }
              }}
              className="ub-btn ub-btn-danger !py-[0.4rem] !text-[12px]"
            >
              Reset
            </button>
          </Row>
        </Card>
      </div>

      {/* Import confirmation modal. Kept inside this component (not a global
          dialog system) because it's the only destructive-replace flow in the
          app — one-off UI is simpler than generalising. Dimmed scrim + a panel
          on bg-1 to match Settings cards so it reads as "part of the page" not
          an OS alert. */}
      {pendingImportPath && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
          role="dialog"
          aria-modal="true"
          onClick={() => {
            if (!importing) setPendingImportPath(null);
          }}
        >
          <div
            className="w-[440px] rounded-card bg-bg-1 p-6 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="ub-eyebrow mb-2 text-danger">Destructive action</div>
            <h3 className="mb-2 text-[18px] font-semibold text-fg-hi">Replace all Utterbar data?</h3>
            <p className="mb-4 text-[13px] leading-relaxed text-muted">
              Importing <span className="font-mono text-fg">{pendingImportPath.split(/[\\/]/).pop()}</span>{' '}
              will overwrite your current settings, modes, vocabulary, history, and any recordings
              included in the archive. Utterbar will restart immediately afterwards.
            </p>
            <p className="mb-5 text-[12px] leading-relaxed text-muted">
              This cannot be undone. If you're not sure, cancel and export your current data first.
            </p>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setPendingImportPath(null)}
                disabled={importing}
                className="ub-btn !py-[0.4rem] !text-[12px]"
              >
                Cancel
              </button>
              <button
                onClick={() => void confirmImport()}
                disabled={importing}
                className="ub-btn ub-btn-danger !py-[0.4rem] !text-[12px]"
              >
                {importing ? 'Importing & restarting…' : 'Replace and restart'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
