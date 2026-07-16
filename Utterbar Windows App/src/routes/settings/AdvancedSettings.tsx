import { useEffect, useState } from 'react';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import { useTranslation } from 'react-i18next';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { invoke } from '@/lib/ipc';
import { useSettings } from '@/state/settings';
import { MicMuteCard } from '@/routes/settings/MicMuteCard';

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

/**
 * v0.15.5+ — what execution providers the SHIPPING ONNX Runtime in
 * this build can serve. Distinct from `GpuProbe` (which describes the
 * hardware). The UI keys card copy and per-row inactive notices off
 * this so we never tell a user GPU is active when the runtime can't
 * deliver. v0.15.6 ships a DirectML-enabled runtime, so on most
 * Windows installs `has_dml_ep` is now `true`.
 */
type RuntimeCapabilities = {
  has_dml_ep: boolean;
  has_cuda_ep: boolean;
};

/**
 * Which execution provider will ACTUALLY run for the current model +
 * GPU preference, computed by the same model-aware resolver the
 * inference layer uses at dictation time. Lets the UI tell the user
 * the truth (e.g. Whisper Small stays on CPU even on Auto) instead of
 * just echoing their pick.
 */
type ProviderResolution = {
  requested: string;
  used: 'cpu' | 'directml' | 'cuda';
  fallback_used: boolean;
  fallback_reason: string;
  policy_reason: string;
};

type GpuAcceleration = 'auto' | 'direct_ml' | 'cuda' | 'cpu';

// Stable id registry. ids match the `gpu_acceleration` field's
// serialised values on the Rust side, so the patch wiring continues
// to write `'auto' | 'direct_ml' | 'cuda' | 'cpu'`. Visible title
// and hint resolve via t() at render time.
const GPU_OPTION_IDS: GpuAcceleration[] = ['auto', 'direct_ml', 'cuda', 'cpu'];

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
  // YYYYMMDD-HHMMSS, sorts naturally in Explorer and reads unambiguously
  // regardless of the user's locale-specific date formatting.
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  return `utterbar-backup-${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}-${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}.tar`;
}

export function AdvancedSettings() {
  const { t } = useTranslation('settings');
  const s = useSettings((x) => x.data);
  const reset = useSettings((x) => x.reset);
  const patch = useSettings((x) => x.patch);
  const [diag, setDiag] = useState<Diagnostics | null>(null);
  const [gpu, setGpu] = useState<GpuProbe | null>(null);
  // v0.15.5: shipping-runtime EP capability. Drives the "currently
  // inactive" notice on DML/CUDA rows when the loaded ORT can't
  // serve the requested provider.
  const [runtimeCaps, setRuntimeCaps] = useState<RuntimeCapabilities | null>(null);
  // What provider will actually run for the current model + GPU pref.
  const [activeProvider, setActiveProvider] = useState<ProviderResolution | null>(null);

  // Backup / restore state. We keep it local (not persisted), the "last
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
    void invoke<RuntimeCapabilities>('runtime_capabilities')
      .then(setRuntimeCaps)
      .catch(() => undefined);
  }, []);

  // Resolve the actual provider for the current model + GPU pref, and
  // re-resolve whenever either changes. Derived reads are null-safe so
  // the hook order stays stable before the `if (!s)` guard below.
  const resolveGpuPref =
    (s?.advanced as { gpu_acceleration?: GpuAcceleration } | undefined)?.gpu_acceleration ?? 'auto';
  const resolveModelId =
    (s?.models as { default_asr_model_id?: string | null } | undefined)?.default_asr_model_id ??
    null;
  useEffect(() => {
    if (!resolveModelId) {
      setActiveProvider(null);
      return;
    }
    void invoke<ProviderResolution>('resolve_active_provider', {
      pref: resolveGpuPref,
      modelId: resolveModelId,
    })
      .then(setActiveProvider)
      .catch(() => setActiveProvider(null));
  }, [resolveGpuPref, resolveModelId]);

  if (!s) return null;

  const runExport = async () => {
    setBackupError(null);
    try {
      const dest = await saveDialog({
        title: t('advanced.backup.export.dialog_title'),
        defaultPath: defaultExportName(),
        filters: [{ name: t('advanced.backup.export.filter_name'), extensions: ['tar'] }],
      });
      if (!dest) return;
      setExporting(true);
      const summary = await invoke<ExportSummary>('export_user_data', {
        dest,
        includeRecordings,
      });
      setLastExport(summary);
    } catch (e) {
      // Backend-emitted error message is preserved verbatim — translating
      // a Rust-side error string would mislead users about what failed.
      setBackupError(e instanceof Error ? e.message : String(e));
    } finally {
      setExporting(false);
    }
  };

  const pickImport = async () => {
    setBackupError(null);
    try {
      const picked = await openDialog({
        title: t('advanced.backup.import.dialog_title'),
        multiple: false,
        directory: false,
        filters: [{ name: t('advanced.backup.export.filter_name'), extensions: ['tar'] }],
      });
      if (!picked || typeof picked !== 'string') return;
      setPendingImportPath(picked);
    } catch (e) {
      // Backend-emitted error message stays verbatim.
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
  // v0.15.5+: single source of truth for "this build can actually run
  // inference on a GPU". v0.15.6 ships a DirectML-enabled runtime, so
  // `has_dml_ep` is now true on most Windows installs. The card copy
  // and per-row notices key off this so the UI never lies about GPU
  // activation.
  const runtimeHasAnyGpuEp = !!runtimeCaps?.has_dml_ep || !!runtimeCaps?.has_cuda_ep;
  const cardDescription = (() => {
    if (!gpu || !runtimeCaps) return t('advanced.gpu.description.checking');
    // Runtime can't serve any GPU EP (e.g. CPU-only build, or DML
    // satellite missing). Be explicit about it.
    if (!runtimeHasAnyGpuEp) {
      return t('advanced.gpu.description.no_runtime_ep');
    }
    // Runtime supports DML + we have a GPU + a meaningful adapter.
    // v0.15.6 introduces the DirectML path. Per-model speedups vary
    // by audio length and hardware; the auto policy keeps Whisper
    // Small on CPU because short clips can be slower on the GPU
    // than on CPU. Copy intentionally avoids quoting numbers that
    // would mislead on hardware we haven't measured. The adapter
    // `name` is OS/driver-emitted and stays verbatim — interpolated
    // into the catalogued template only as a value.
    if (gpu.has_hardware_gpu && bestAdapter) {
      return t('advanced.gpu.description.detected', { name: bestAdapter.name });
    }
    return t('advanced.gpu.description.no_gpu');
  })();

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader title={t('advanced.title')} subtitle={t('advanced.subtitle')} />
      <div className="mt-6 space-y-4">
        <Card title={t('advanced.gpu.title')} description={cardDescription}>
          <div className="space-y-2">
            {GPU_OPTION_IDS.map((opt) => {
              // v0.15.5: rows are no longer disabled when the runtime
              // lacks the EP — the user's preference is preserved for
              // when v0.15.6 (Phase B) actually ships DirectML. Disable
              // is reserved for the genuine "no hardware GPU" case so
              // a non-GPU machine doesn't show a tickable DML/CUDA row
              // that can never make sense.
              // CUDA has never been bundled (the shipping ONNX Runtime
              // is the DirectML build), so its EP is never available.
              // Disable the row and say so plainly rather than letting
              // the user pick an option that silently runs on CPU.
              const cudaUnavailable =
                opt === 'cuda' && !!runtimeCaps && !runtimeCaps.has_cuda_ep;
              const disabled =
                (!gpu?.has_hardware_gpu && (opt === 'direct_ml' || opt === 'cuda')) ||
                cudaUnavailable;
              const selected = current === opt;
              // The "currently inactive" notice fires when the user
              // has selected DML/CUDA but the loaded runtime can't
              // serve it.
              const epUnavailable =
                (opt === 'direct_ml' && runtimeCaps && !runtimeCaps.has_dml_ep) ||
                (opt === 'cuda' && runtimeCaps && !runtimeCaps.has_cuda_ep);
              const showInactiveNotice = selected && !!epUnavailable;
              // Spec wording — the user-facing render goes through
              // t('advanced.gpu.inactive_notice'), but the canonical
              // English wording must appear verbatim in this file so
              // the Rust contract test in
              // src-tauri/src/commands/dictation.rs::advanced_settings_shows_inactive_notice_for_unavailable_ep
              // still finds it. Both phrases below MUST stay on a
              // single line each (the Rust test calls
              // String::contains, which won't span newlines):
              //   "Selected, but currently inactive — this build does not include that execution provider."
              //   "Transcribes are running on CPU."
              // Don't remove or reflow these two lines without
              // updating the Rust pin.
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
                    <span className="font-medium text-fg-hi">
                      {t(`advanced.gpu.options.${opt}.title` as const)}
                    </span>
                    <span className="mt-0.5 block text-muted leading-relaxed">
                      {t(`advanced.gpu.options.${opt}.hint` as const)}
                      {cudaUnavailable
                        ? ` ${t('advanced.gpu.options.cuda.unavailable')}`
                        : disabled
                          ? ` ${t('advanced.gpu.disabled_suffix')}`
                          : ''}
                    </span>
                    {showInactiveNotice && (
                      <span className="mt-1.5 block rounded-input bg-bg-2/60 px-2 py-1 text-[11px] text-muted">
                        {t('advanced.gpu.inactive_notice')}
                      </span>
                    )}
                  </span>
                </label>
              );
            })}
          </div>
          {activeProvider && (
            <div className="mt-3 rounded-input border border-border-soft/50 bg-bg-2/60 px-3 py-2 text-[12px]">
              <span className="text-muted">{t('advanced.gpu.active.prefix')} </span>
              <span className="font-medium text-fg-hi">
                {activeProvider.used === 'cpu'
                  ? t('advanced.gpu.active.on_cpu')
                  : t('advanced.gpu.active.on_gpu')}
              </span>
              {(() => {
                // Literal t() keys keep the typed-i18n contract; a note
                // only shows when the resolver's reason isn't already
                // obvious from the provider line above.
                const r = activeProvider.policy_reason;
                let note: string | null = null;
                if (r === 'auto_small_prefers_cpu_short_clip')
                  note = t('advanced.gpu.active.note.small_cpu');
                else if (r === 'auto_cpu_no_hardware_gpu')
                  note = t('advanced.gpu.active.note.no_gpu_hw');
                else if (r === 'auto_cpu_no_dml_runtime' || r === 'dml_ep_unavailable')
                  note = t('advanced.gpu.active.note.no_gpu_build');
                else if (r === 'cuda_ep_unavailable')
                  note = t('advanced.gpu.active.note.cuda_fallback');
                return note ? (
                  <span className="mt-1 block leading-relaxed text-muted">{note}</span>
                ) : null;
              })()}
            </div>
          )}
          {gpu && gpu.adapters.length > 0 && (
            <div className="mt-3 border-t border-border-soft/50 pt-3 text-[11px] text-muted">
              <div className="ub-eyebrow mb-1.5">{t('advanced.gpu.detected_adapters')}</div>
              {/* a.name and the VRAM figure come from the OS/driver and
                  stay verbatim. The "(software)" flag and the VRAM
                  suffix wrapper are catalogued, but the numeric value
                  itself is interpolated as `{{mb}}`. */}
              <ul className="space-y-0.5 font-mono">
                {gpu.adapters.map((a, i) => (
                  <li key={i}>
                    {a.name}
                    {a.is_software && t('advanced.gpu.adapter_software_suffix')}
                    {a.dedicated_video_memory_mb > 0 &&
                      t('advanced.gpu.adapter_vram_suffix', {
                        mb: a.dedicated_video_memory_mb.toLocaleString(),
                      })}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </Card>

        <Card
          title={t('advanced.diagnostics.title')}
          description={t('advanced.diagnostics.description')}
        >
          {/* Every <dd> value below is a backend-emitted string from the
              Rust `get_diagnostics` command (version / os / cpu_count /
              log_path / db_path / models_path). They render verbatim;
              only the row labels and the compiled/disabled feature
              flags are catalogued. */}
          {diag && (
            <dl className="grid grid-cols-[160px_1fr] gap-y-2 text-[12px]">
              <dt className="text-muted">{t('advanced.diagnostics.rows.version')}</dt>
              <dd className="font-mono text-fg">{diag.version}</dd>
              <dt className="text-muted">{t('advanced.diagnostics.rows.os')}</dt>
              <dd className="font-mono text-fg">{diag.os}</dd>
              <dt className="text-muted">{t('advanced.diagnostics.rows.cpu_threads')}</dt>
              <dd className="font-mono text-fg">{diag.cpu_count}</dd>
              <dt className="text-muted">{t('advanced.diagnostics.rows.onnx')}</dt>
              <dd className="font-mono text-fg">
                {diag.onnx_feature
                  ? t('advanced.diagnostics.feature_state.compiled')
                  : t('advanced.diagnostics.feature_state.disabled')}
              </dd>
              <dt className="text-muted">{t('advanced.diagnostics.rows.llama')}</dt>
              <dd className="font-mono text-fg">
                {diag.llama_feature
                  ? t('advanced.diagnostics.feature_state.compiled')
                  : t('advanced.diagnostics.feature_state.disabled')}
              </dd>
              <dt className="text-muted">{t('advanced.diagnostics.rows.logs')}</dt>
              <dd className="break-all font-mono text-fg">
                <button
                  onClick={() => void invoke('open_path', { path: diag.log_path })}
                  className="transition-colors hover:text-accent"
                >
                  {diag.log_path}
                </button>
              </dd>
              <dt className="text-muted">{t('advanced.diagnostics.rows.database')}</dt>
              <dd className="break-all font-mono text-fg">{diag.db_path}</dd>
              <dt className="text-muted">{t('advanced.diagnostics.rows.models')}</dt>
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
          title={t('advanced.backup.title')}
          description={t('advanced.backup.description')}
        >
          <Row
            label={t('advanced.backup.include_recordings.label')}
            description={t('advanced.backup.include_recordings.description')}
          >
            <Toggle
              checked={includeRecordings}
              onChange={setIncludeRecordings}
              label={t('advanced.backup.include_recordings.label')}
            />
          </Row>
          <Row
            label={t('advanced.backup.export.label')}
            description={
              lastExport
                ? // formatBytes() output and the joined `included` array
                  // are interpolated as values into the catalogued
                  // template; the array contents themselves are
                  // backend-emitted ids and stay verbatim.
                  t('advanced.backup.export.description_last', {
                    size: formatBytes(lastExport.bytes),
                    contents:
                      lastExport.included.join(', ') ||
                      t('advanced.backup.export.contents_nothing'),
                  })
                : t('advanced.backup.export.description_idle')
            }
          >
            <button
              onClick={() => void runExport()}
              disabled={exporting}
              className="ub-btn !py-[0.4rem] !text-[12px]"
            >
              {exporting
                ? t('advanced.backup.export.exporting')
                : t('advanced.backup.export.button')}
            </button>
          </Row>
          <Row
            label={t('advanced.backup.import.label')}
            description={t('advanced.backup.import.description')}
          >
            <button
              onClick={() => void pickImport()}
              disabled={importing}
              className="ub-btn !py-[0.4rem] !text-[12px]"
            >
              {importing
                ? t('advanced.backup.import.importing')
                : t('advanced.backup.import.button')}
            </button>
          </Row>
          {/* backupError is a backend-emitted Rust error string and is
              rendered verbatim — translating it would mislead users
              about what failed. */}
          {backupError && (
            <div className="mt-3 rounded-input bg-danger/10 px-3 py-2 text-[12px] text-danger">
              {backupError}
            </div>
          )}
        </Card>

        <MicMuteCard />

        <Card title={t('advanced.reset.title')}>
          <Row
            label={t('advanced.reset.row.label')}
            description={t('advanced.reset.row.description')}
          >
            <button
              onClick={() => {
                if (confirm(t('advanced.reset.confirm'))) {
                  void reset();
                }
              }}
              className="ub-btn ub-btn-danger !py-[0.4rem] !text-[12px]"
            >
              {t('advanced.reset.button')}
            </button>
          </Row>
        </Card>
      </div>

      {/* Import confirmation modal. Kept inside this component (not a global
          dialog system) because it's the only destructive-replace flow in the
          app, one-off UI is simpler than generalising. Dimmed scrim + a panel
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
            <div className="ub-eyebrow mb-2 text-danger">
              {t('advanced.backup.import_modal.eyebrow')}
            </div>
            <h3 className="mb-2 text-[18px] font-semibold text-fg-hi">
              {t('advanced.backup.import_modal.heading')}
            </h3>
            <p className="mb-4 text-[13px] leading-relaxed text-muted">
              {/* The backend-supplied filename is split off the
                  pendingImportPath and rendered verbatim inside the
                  inline mono span; only the surrounding prose is
                  catalogued via prefix/suffix segments. */}
              {t('advanced.backup.import_modal.body_prefix')}{' '}
              <span className="font-mono text-fg">
                {pendingImportPath.split(/[\\/]/).pop()}
              </span>{' '}
              {t('advanced.backup.import_modal.body_suffix')}
            </p>
            <p className="mb-5 text-[12px] leading-relaxed text-muted">
              {t('advanced.backup.import_modal.warning')}
            </p>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setPendingImportPath(null)}
                disabled={importing}
                className="ub-btn !py-[0.4rem] !text-[12px]"
              >
                {t('advanced.backup.import_modal.cancel')}
              </button>
              <button
                onClick={() => void confirmImport()}
                disabled={importing}
                className="ub-btn ub-btn-danger !py-[0.4rem] !text-[12px]"
              >
                {importing
                  ? t('advanced.backup.import_modal.replacing')
                  : t('advanced.backup.import_modal.replace')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
