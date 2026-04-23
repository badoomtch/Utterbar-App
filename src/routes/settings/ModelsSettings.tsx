import { useEffect, useMemo, useState } from 'react';
import { Flame, Radio } from 'lucide-react';

import { PageHeader } from '@/components/common/PageHeader';
import { Card } from '@/components/common/Card';
import { Select, type SelectOption } from '@/components/common/Select';
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { formatBytes } from '@/lib/format';
import { useSettings } from '@/state/settings';

type ModelInfo = {
  id: string;
  display_name: string;
  description: string;
  version: string;
  size_bytes: number;
  languages: string[];
  installed: boolean;
  recommended: boolean;
};

type Progress = {
  model_id: string;
  bytes_done: number;
  bytes_total: number;
  phase: 'starting' | 'downloading' | 'verifying' | 'extracting' | 'done' | 'error';
};

export function ModelsSettings() {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [progress, setProgress] = useState<Record<string, Progress>>({});

  const reload = () =>
    invoke<ModelInfo[]>('list_available_models').then(setModels).catch(() => setModels([]));
  useEffect(() => {
    void reload();
  }, []);

  useEffect(() => {
    const p = listen<Progress>(EVT.modelProgress, (e) => {
      setProgress((prev) => ({ ...prev, [e.payload.model_id]: e.payload }));
      if (e.payload.phase === 'done') void reload();
    });
    const c = listen<string>(EVT.modelChanged, () => void reload());
    return () => {
      void p.then((u) => u());
      void c.then((u) => u());
    };
  }, []);

  const items = useMemo(() => models.sort((a, b) => Number(b.recommended) - Number(a.recommended)), [models]);

  const settings = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  const warmSecs =
    (settings?.models as { warm_duration_secs?: number } | undefined)?.warm_duration_secs ?? 120;
  // `default_asr_model_id` is the fallback a dictation uses when the active mode
  // hasn't pinned its own model. Before this was only editable per-mode, which
  // meant the natural "set my default model" mental model had no home. Now it
  // lives on the Models page as the top-level choice and modes opt into
  // overriding it.
  const defaultModelId =
    (settings?.models as { default_asr_model_id?: string | null } | undefined)?.default_asr_model_id ?? null;

  const updateWarm = (value: number) => {
    if (!settings) return;
    void patch({
      models: { ...(settings.models as Record<string, unknown>), warm_duration_secs: value },
    });
  };

  const updateDefaultModel = (id: string | null) => {
    if (!settings) return;
    void patch({
      models: { ...(settings.models as Record<string, unknown>), default_asr_model_id: id },
    });
  };

  const installedModels = useMemo(() => items.filter((m) => m.installed), [items]);

  return (
    <div className="mx-auto max-w-3xl p-8">
      <PageHeader
        title="Models"
        subtitle="Parakeet ASR models. Downloaded once, stored locally, never uploaded anywhere."
      />

      <Card
        className="mt-6"
        title="Default model"
        description="Used by any mode that hasn't pinned a specific model. Modes can still override this — the setting here is the fallback your whole app uses by default."
      >
        {installedModels.length === 0 ? (
          <div className="flex items-center gap-2 rounded-input border border-dashed border-border-soft bg-bg-2/50 px-3 py-2.5 text-[12px] text-muted">
            <Radio className="h-4 w-4 opacity-70" />
            Install a model below to pick a default.
          </div>
        ) : (
          <div className="flex items-center gap-3">
            <Radio className="h-4 w-4 text-accent" />
            <Select
              value={defaultModelId ?? ''}
              onChange={(v) => updateDefaultModel(v === '' ? null : v)}
              minWidth="min-w-[20rem]"
              options={[
                { value: '', label: 'Auto (first installed)' },
                ...installedModels.map<SelectOption<string>>((m) => ({
                  value: m.id,
                  label: m.display_name,
                  caption: `${formatBytes(m.size_bytes)} · ${m.languages.join(', ')}`,
                })),
              ]}
            />
          </div>
        )}
      </Card>

      <Card
        className="mt-4"
        title="Keep-warm duration"
        description="The first dictation after a cold start loads the model into memory — that one takes a few seconds. Subsequent dictations while the model is warm feel near-instant. We keep the model loaded for a while after each use so you don't pay the cold-start cost on every burst of speech."
      >
        <div className="flex items-center gap-3 text-[12px] text-muted">
          <Flame className="h-4 w-4 text-accent" />
          Model stays warm for{' '}
          <span className="font-mono text-[13px] text-fg-hi">{formatWarm(warmSecs)}</span>{' '}
          after a dictation.
        </div>
        <div className="mt-4 flex items-center gap-4">
          <input
            type="range"
            min={0}
            max={600}
            step={10}
            value={warmSecs}
            onChange={(e) => updateWarm(Number(e.target.value))}
            className="flex-1"
          />
          <span className="w-20 text-right font-mono text-[12px] tabular-nums text-fg-hi">{formatWarm(warmSecs)}</span>
        </div>
        <p className="mt-3 text-[11px] text-muted leading-relaxed">
          Longer durations give you the instant-feeling second-speech at the cost of RAM staying
          reserved; shorter ones free memory sooner but you'll feel the load time again after the
          timeout. 0 means unload immediately after each dictation.
        </p>
      </Card>

      <Card className="mt-4">
        <ul className="divide-y divide-border-soft/50">
          {items.map((m) => {
            const prog = progress[m.id];
            const downloading = prog && prog.phase !== 'done' && prog.phase !== 'error';
            const pct = prog && prog.bytes_total > 0 ? Math.round((prog.bytes_done / prog.bytes_total) * 100) : 0;
            return (
              <li key={m.id} className="py-4">
                <div className="flex items-start gap-4">
                  <div className="flex-1">
                    {/* Weight 700 — font-semibold (600) forbidden per §2.3. */}
                    <div className="flex items-center gap-2 text-[14px] font-bold text-fg-hi">
                      {m.display_name}
                      {m.recommended && (
                        /* Live chip reserved for the single "Recommended" model —
                           this is the cyan budget for the Models route. */
                        <span className="ub-chip ub-chip--live">
                          <span className="ub-led" aria-hidden />
                          Recommended
                        </span>
                      )}
                    </div>
                    <div className="mt-0.5 font-mono text-[11px] text-fg-dim">
                      v{m.version} · {formatBytes(m.size_bytes)} · {m.languages.join(', ')}
                    </div>
                    {m.description && (
                      <p className="mt-2 max-w-xl text-[12px] leading-relaxed text-muted">
                        {m.description}
                      </p>
                    )}
                  </div>
                  {m.installed ? (
                    <button
                      onClick={() => invoke('remove_model', { id: m.id }).then(reload)}
                      className="ub-btn ub-btn-plain !py-[0.4rem] !text-[12px]"
                    >
                      Remove
                    </button>
                  ) : downloading ? (
                    <span className="font-mono text-[11px] text-muted">{capitalise(prog!.phase)}…</span>
                  ) : (
                    <button
                      onClick={() => invoke('install_model', { id: m.id }).then(reload).catch(() => undefined)}
                      className="ub-btn ub-btn-cta !py-[0.4rem] !text-[12px]"
                    >
                      Install
                    </button>
                  )}
                </div>
                {downloading && (
                  <div className="mt-3 flex items-center gap-3">
                    <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-bg-2">
                      <div
                        className="h-full bg-accent transition-[width] duration-200"
                        style={{ width: `${pct}%` }}
                      />
                    </div>
                    <span className="w-28 text-right font-mono text-[11px] tabular-nums text-muted">
                      {formatBytes(prog.bytes_done)} / {formatBytes(prog.bytes_total)}
                    </span>
                  </div>
                )}
              </li>
            );
          })}
        </ul>
      </Card>
    </div>
  );
}

function capitalise(s: string) {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function formatWarm(secs: number): string {
  if (secs === 0) return 'off';
  if (secs < 60) return `${secs}s`;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return s === 0 ? `${m}m` : `${m}m ${s}s`;
}
