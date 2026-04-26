import { useEffect, useMemo, useState } from 'react';
import { Flame, Radio } from 'lucide-react';

import { PageHeader } from '@/components/common/PageHeader';
import { Card } from '@/components/common/Card';
import { ProNudge } from '@/components/common/ProNudge';
import { Select, type SelectOption } from '@/components/common/Select';
import { ScoreBars } from '@/components/common/ScoreBars';
import { Toggle } from '@/components/common/Toggle';

// Sentinel for "keep the model loaded forever" — matches the u32::MAX
// special case in `services/inference/mod.rs` (both the inline unload
// and the idle sweeper short-circuit on this value).
const FOREVER = 4_294_967_295; // u32::MAX
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { formatBytes } from '@/lib/format';
import { cn } from '@/lib/cn';
import { useLicence, isPro } from '@/state/licence';
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
  accuracy_score: number;
  speed_score: number;
  requires_pro: boolean;
  engine?: 'parakeet-tdt' | 'whisper' | 'zipformer';
};

/** Human label for the engine field. Used on each model card so users
 *  can see at a glance which family a model comes from. Parakeet-TDT
 *  + Whisper are live; Zipformer lands in a later release. */
const ENGINE_LABEL: Record<string, string> = {
  'parakeet-tdt': 'Parakeet · NVIDIA',
  whisper: 'Whisper · OpenAI',
  zipformer: 'Zipformer · k2-fsa',
};

/** Whisper language picker shortlist. ~20 most common languages plus
 *  Auto-detect at the top. Whisper itself supports ~99 languages — if
 *  someone needs a less common one, they can edit `whisper_language`
 *  in the settings JSON directly (advanced path). The shortlist keeps
 *  the visible UI manageable and covers the realistic 95% of users. */
const WHISPER_LANGUAGE_OPTIONS: SelectOption<string>[] = [
  { value: 'auto', label: 'Auto-detect (recommended)' },
  { value: 'en', label: 'English' },
  { value: 'es', label: 'Spanish' },
  { value: 'fr', label: 'French' },
  { value: 'de', label: 'German' },
  { value: 'it', label: 'Italian' },
  { value: 'pt', label: 'Portuguese' },
  { value: 'nl', label: 'Dutch' },
  { value: 'pl', label: 'Polish' },
  { value: 'tr', label: 'Turkish' },
  { value: 'ru', label: 'Russian' },
  { value: 'ar', label: 'Arabic' },
  { value: 'hi', label: 'Hindi' },
  { value: 'ja', label: 'Japanese' },
  { value: 'ko', label: 'Korean' },
  { value: 'zh', label: 'Chinese' },
  { value: 'id', label: 'Indonesian' },
  { value: 'th', label: 'Thai' },
  { value: 'vi', label: 'Vietnamese' },
  { value: 'sv', label: 'Swedish' },
  { value: 'no', label: 'Norwegian' },
  { value: 'da', label: 'Danish' },
  { value: 'fi', label: 'Finnish' },
  { value: 'cs', label: 'Czech' },
  { value: 'el', label: 'Greek' },
  { value: 'he', label: 'Hebrew' },
];

type Progress = {
  model_id: string;
  bytes_done: number;
  bytes_total: number;
  phase: 'starting' | 'downloading' | 'verifying' | 'extracting' | 'done' | 'error';
};

export function ModelsSettings() {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [progress, setProgress] = useState<Record<string, Progress>>({});
  // `loadedId` is the id of the model currently resident in memory — null
  // when the orchestrator is cold. We poll every 2s while this page is
  // mounted so the "Ready / Idle" badge below stays accurate without
  // wiring AppHandle into the inference orchestrator just for an event.
  // 2s is a deliberate pick: faster feels jittery on a static surface,
  // slower and the state lags noticeably after "stop dictation".
  const [loadedId, setLoadedId] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    const poll = () =>
      invoke<string | null>('asr_runtime_status')
        .then((id) => {
          if (!cancelled) setLoadedId(id ?? null);
        })
        .catch(() => undefined);
    void poll();
    const t = window.setInterval(poll, 2000);
    return () => {
      cancelled = true;
      window.clearInterval(t);
    };
  }, []);

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

  // Whisper language hint. `auto` means "detect on each utterance" —
  // accurate but slightly slower per dictation. Pinning a specific
  // language locks Whisper to that language (no detection cost) but
  // loses the multilingual ability. Only meaningful when a Whisper
  // model is installed; the section auto-hides otherwise.
  const whisperLanguage =
    (settings?.models as { whisper_language?: string } | undefined)?.whisper_language ?? 'auto';
  const hasWhisperInstalled = items.some(
    (m) => m.engine === 'whisper' && m.installed,
  );
  const updateWhisperLanguage = (lang: string) => {
    if (!settings) return;
    void patch({
      models: { ...(settings.models as Record<string, unknown>), whisper_language: lang },
    });
  };

  const installedModels = useMemo(() => items.filter((m) => m.installed), [items]);
  const licenceStatus = useLicence((s) => s.status);
  const pro = isPro(licenceStatus);

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader
        title="Models"
        subtitle="Parakeet ASR models. Downloaded once, stored locally, never uploaded anywhere."
      />

      <Card
        className="mt-6"
        title="Default model"
        description="Used by any mode that hasn't pinned a specific model. Modes can still override this — the setting here is the fallback your whole app uses by default."
        aside={<ModelRuntimeBadge loadedId={loadedId} />}
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

      {/* Whisper language hint. Only relevant when a Whisper model is
          installed — the picker hides otherwise so the page doesn't
          surface controls that don't apply to the user's setup.
          "Auto-detect" is the default and best for users who dictate in
          more than one language; pinning a specific language is a
          minor speed win when always dictating in the same one. */}
      {hasWhisperInstalled && (
        <Card
          title="Whisper language"
          description="Used by Whisper models only. Auto-detect picks the language at the start of each dictation; pinning a specific one is slightly faster but locks you to that language."
        >
          <div className="flex items-center gap-3">
            <Radio className="h-4 w-4 text-accent" />
            <Select
              value={whisperLanguage}
              onChange={(v) => updateWhisperLanguage(v)}
              minWidth="min-w-[20rem]"
              options={WHISPER_LANGUAGE_OPTIONS}
            />
          </div>
        </Card>
      )}

      <Card
        className="mt-4"
        title="Keep-warm duration"
        description="The first dictation after a cold start loads the model into memory — that one takes a few seconds. Subsequent dictations while the model is warm feel near-instant. We keep the model loaded for a while after each use so you don't pay the cold-start cost on every burst of speech."
      >
        {/* Keep-loaded-forever sentinel. Backed in services/inference: when
            warm_duration_secs is u32::MAX the inline-unload skips and the
            idle sweeper short-circuits — the model stays resident until
            the app exits or an explicit reload (e.g. default-model switch). */}
        <div className="flex items-center gap-3 text-[12px] text-muted">
          <Flame className="h-4 w-4 text-accent" />
          {warmSecs === FOREVER ? (
            <>
              Model stays loaded{' '}
              <span className="font-mono text-[13px] text-accent">forever</span>{' '}
              (until you quit Utterbar).
            </>
          ) : (
            <>
              Model stays warm for{' '}
              <span className="font-mono text-[13px] text-fg-hi">{formatWarm(warmSecs)}</span>{' '}
              after a dictation.
            </>
          )}
        </div>
        <div className="mt-4 flex items-center gap-4">
          <input
            type="range"
            min={0}
            max={600}
            step={10}
            value={warmSecs === FOREVER ? 600 : warmSecs}
            onChange={(e) => updateWarm(Number(e.target.value))}
            disabled={warmSecs === FOREVER}
            className="flex-1"
            style={warmSecs === FOREVER ? { opacity: 0.4 } : undefined}
          />
          <span className="w-20 text-right font-mono text-[12px] tabular-nums text-fg-hi">
            {warmSecs === FOREVER ? '∞' : formatWarm(warmSecs)}
          </span>
        </div>
        <div className="mt-4 flex items-center gap-3">
          <Toggle
            checked={warmSecs === FOREVER}
            onChange={(v) => updateWarm(v ? FOREVER : 120)}
          />
          <div>
            <div className="text-[13px] text-fg-hi">Keep model loaded forever</div>
            <div className="text-[11px] text-muted">
              Model stays resident in memory for the whole session. Best latency,
              but the RAM (and VRAM, if you're on GPU) stays reserved while
              Utterbar is running.
            </div>
          </div>
        </div>
        <p className="mt-3 text-[11px] text-muted leading-relaxed">
          Longer durations give you instant-feeling subsequent dictations at the cost of RAM staying
          reserved; shorter ones free memory sooner but you&rsquo;ll feel the load time again after the
          timeout. 0 means unload immediately after each dictation. Forever keeps it loaded for the whole session.
        </p>
      </Card>

      <Card className="mt-4">
        <ul className="divide-y divide-border-soft/50">
          {items.map((m) => {
            const prog = progress[m.id];
            const downloading = prog && prog.phase !== 'done' && prog.phase !== 'error';
            const pct = prog && prog.bytes_total > 0 ? Math.round((prog.bytes_done / prog.bytes_total) * 100) : 0;
            const locked = m.requires_pro && !pro;
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
                      {m.engine && ENGINE_LABEL[m.engine] && (
                        <> · {ENGINE_LABEL[m.engine]}</>
                      )}
                    </div>
                    {m.description && (
                      <p className="mt-2 max-w-xl text-[12px] leading-relaxed text-muted">
                        {m.description}
                      </p>
                    )}
                    {/* Paired comparison bars — makes the trade-offs between
                        the int8 / v3 / fp16 variants legible without the
                        user having to parse the descriptions against each
                        other. Hidden automatically when both scores are 0
                        (DB-only installed rows, unknown models). */}
                    <ScoreBars
                      accuracy={m.accuracy_score}
                      speed={m.speed_score}
                      className="mt-3 max-w-[280px]"
                    />
                    {locked && (
                      <ProNudge
                        feature={m.display_name}
                        description="The higher-precision fp16 model is part of Pro. It's the accuracy ceiling of the Parakeet family and benefits from a real GPU — same privacy guarantees as every other model (never leaves your machine)."
                        dense
                        className="mt-3 max-w-xl"
                      />
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
                      disabled={locked}
                      className="ub-btn ub-btn-cta !py-[0.4rem] !text-[12px] disabled:opacity-40 disabled:cursor-not-allowed"
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

/**
 * Subtle runtime-state badge shown next to "Default model".
 *
 * ASR models are lazily loaded — the first dictation after launch pays a
 * 1–3 s load cost (ONNX session init, GPU upload) and subsequent dictations
 * land near-instant while the model stays warm. Users who don't know this
 * experience the first-dictation lag as "the app is slow" with no visible
 * reason. This badge makes the cold/warm state legible:
 *
 *   • Ready  — model is resident in memory, next dictation is instant.
 *              Cyan LED + `text-accent`, single beat of the cyan budget.
 *   • Idle   — orchestrator is cold, next dictation will take a moment.
 *              Muted dot + `text-muted`, sits back into the card chrome.
 *
 * Deliberately small and quiet — this is meta, not a control. Don't give
 * it the ub-chip active treatment.
 */
function ModelRuntimeBadge({ loadedId }: { loadedId: string | null }) {
  const isReady = loadedId !== null;
  return (
    <span
      className={cn(
        'inline-flex cursor-default select-none items-center gap-1.5 rounded-full px-2.5 py-0.5',
        'font-mono text-[10px] font-medium uppercase tracking-[0.1em]',
        isReady ? 'text-accent' : 'text-muted',
      )}
      title={
        isReady
          ? 'The ASR model is loaded in memory. Next dictation will start instantly.'
          : 'No ASR model is currently loaded. The next dictation will pay a one-off 1–3 s load cost.'
      }
    >
      <span
        aria-hidden
        className={cn(
          'h-1.5 w-1.5 rounded-full',
          isReady ? 'bg-accent shadow-[0_0_8px_var(--accent)]' : 'bg-fg-dim/70',
        )}
      />
      {isReady ? 'Ready' : 'Idle'}
    </span>
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
