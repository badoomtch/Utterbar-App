import { useEffect, useMemo, useState } from 'react';
import { Flame, Radio } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { PageHeader } from '@/components/common/PageHeader';
import { Card } from '@/components/common/Card';
import { ProNudge } from '@/components/common/ProNudge';
import { Select, type SelectOption } from '@/components/common/Select';
import { ScoreBars } from '@/components/common/ScoreBars';
import { Toggle } from '@/components/common/Toggle';

import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { formatBytes } from '@/lib/format';
import { cn } from '@/lib/cn';
import { useLicence, isPro } from '@/state/licence';
import { useSettings } from '@/state/settings';
import { DICTATION_LANGUAGES, isMultilingualOnly } from '@/lib/dictationLanguages';
import {
  recommendModelForLanguage,
  type RecommendationModel,
} from '@/routes/onboarding_model_recommendation';

// Sentinel for "keep the model loaded forever", matches the u32::MAX
// special case in `services/inference/mod.rs` (both the inline unload
// and the idle sweeper short-circuit on this value).
const FOREVER = 4_294_967_295; // u32::MAX

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
 *  can see at a glance which family a model comes from. All three
 *  engines (Parakeet-TDT, Whisper, Zipformer) are live as of v0.8.x. */
const ENGINE_LABEL: Record<string, string> = {
  'parakeet-tdt': 'Parakeet · NVIDIA',
  whisper: 'Whisper · OpenAI',
  zipformer: 'Zipformer · k2-fsa',
};

// Whisper-language picker shortlist lives in the shared
// `lib/dictationLanguages` registry alongside the onboarding
// dictation-language step (imported above), so the two surfaces stay
// aligned. `whisper_language` is still written verbatim into
// Settings; labels resolve through i18next at render time. Whisper
// itself supports ~99 languages — power users can edit the field
// directly in the settings JSON if a less common code is needed.

type Progress = {
  model_id: string;
  bytes_done: number;
  bytes_total: number;
  phase: 'starting' | 'downloading' | 'verifying' | 'extracting' | 'done' | 'error';
};

export function ModelsSettings() {
  const { t } = useTranslation('settings');
  // Language labels live in the `common` namespace because the same
  // strings are reused by the onboarding dictation-language step.
  // Resolved per-render so labels stay reactive to a future locale
  // swap; the values written back to `models.whisper_language` are
  // the structural ISO codes from the registry, not the display
  // labels.
  const { t: tCommon } = useTranslation('common');
  // Cast bypasses i18next's closed-union typed key signature for the
  // dynamic registry lookup. The registry guarantees every entry's
  // labelKey resolves; this cast is the standard shape used elsewhere
  // when a per-id key is built at render time (see settings_nav.ts).
  const tCommonLoose = tCommon as unknown as (k: string) => string;
  const whisperLanguageOptions: SelectOption<string>[] = DICTATION_LANGUAGES.map((lang) => ({
    value: lang.code,
    label: tCommonLoose(lang.labelKey.replace(/^common\./, '')),
  }));
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [progress, setProgress] = useState<Record<string, Progress>>({});
  // `loadedId` is the id of the model currently resident in memory, null
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
    (settings?.models as { warm_duration_secs?: number } | undefined)?.warm_duration_secs ?? FOREVER;
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

  // v0.15.3: Meeting Mode model preference. Pre-v0.15.3 settings.json
  // files miss this field; serde defaults to None on the Rust side
  // and the picker shows "no preference yet" until the user picks.
  // The picker only writes the id; install state is enforced by the
  // existing model-row UI (the Install link in this same page) and
  // by the start_recording preflight (which refuses with a clear
  // "open Settings → Models, then click Install" error).
  const meetingAsrModelId =
    (settings?.models as { meeting_asr_model_id?: string | null } | undefined)
      ?.meeting_asr_model_id ?? null;
  const updateMeetingAsrModelId = (id: string | null) => {
    if (!settings) return;
    void patch({
      models: { ...(settings.models as Record<string, unknown>), meeting_asr_model_id: id },
    });
  };
  // The three meeting-mode picker options + their catalogue ids.
  // Order is "Fast (recommended) → Balanced → Best accuracy" — the
  // recommended option sits at the top, matching how the user
  // reads the trade-off ladder. v0.15.3 had the Recommended chip
  // on Balanced (Medium); v0.15.4 moves it to Fast (Small) because
  // Step 0 measurements showed Medium was too slow on laptop CPUs
  // to be a sane no-preference default.
  const meetingPickerOptions: Array<{
    id: string;
    label: string;
    subtitle: string;
  }> = [
    {
      id: 'sherpa-whisper-small',
      label: 'Fast: Whisper Small',
      subtitle:
        'Fastest Meeting Mode option. Acceptable for clear speech; proper nouns and unusual words are weaker than Medium or Large.',
    },
    {
      id: 'sherpa-whisper-medium',
      label: 'Balanced: Whisper Medium',
      subtitle:
        'Slower than Small with slightly higher quality, especially on proper nouns and unusual words.',
    },
    {
      id: 'sherpa-whisper-large-v3',
      label: 'Best accuracy: Whisper Large v3',
      subtitle:
        "Highest accuracy, slowest. Best for very noisy or multi-speaker audio when you don't mind a long wait.",
    },
  ];

  const installedModels = useMemo(() => items.filter((m) => m.installed), [items]);
  const licenceStatus = useLicence((s) => s.status);
  const pro = isPro(licenceStatus);

  // Reuse the onboarding recommendation helper against the *installed*
  // model list so Settings → Models recommends a switch to a model the
  // user already has, not one they need to download. The helper's
  // `selection_english_only` reason keys the warning + one-click switch;
  // `no_multilingual_available` keys the install hint pointing at the
  // model list further down. The helper takes only id/languages/
  // installed/recommended — `m.display_name`/`m.description`/`m.version`
  // stay verbatim Rust-emitted prose and are not consulted here.
  const settingsRecommendation = recommendModelForLanguage({
    models: installedModels as readonly RecommendationModel[],
    currentSelection: defaultModelId,
    dictationLanguage: whisperLanguage,
  });
  const selectedDefaultModel =
    defaultModelId !== null
      ? installedModels.find((m) => m.id === defaultModelId) ?? null
      : null;
  // Warn only when there is an explicit default (the implicit "first
  // installed" fallback shouldn't trip the warning unless the first
  // installed model is itself English-only). isMultilingualOnly() gates
  // the entire warning so English / Auto users never see anything.
  const showEnglishOnlyWarning =
    isMultilingualOnly(whisperLanguage) &&
    settingsRecommendation.reason === 'selection_english_only' &&
    selectedDefaultModel !== null &&
    !selectedDefaultModel.languages.includes('multi');
  const showNoMultilingualWarning =
    isMultilingualOnly(whisperLanguage) &&
    settingsRecommendation.reason === 'no_multilingual_available';

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader title={t('models.title')} subtitle={t('models.subtitle')} />

      {/* Single source of truth for card spacing. Matches the convention
          used in GeneralSettings, AdvancedSettings, etc. Per-card `mt-*`
          margins were inconsistent (one card had none) and produced
          flush-touching cards. Don't reintroduce them. */}
      <div className="mt-6 space-y-4">

      <Card
        title="Default model"
        description="Used by any mode that hasn't pinned a specific model. Modes can still override this, the setting here is the fallback your whole app uses by default."
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

      {/* v0.15.3: Meeting Mode model picker. Always visible (even if
          no Whisper model is installed yet) so a user with zero
          Whispers can find their way in — the row's Install link
          handles install-state. Picker writes `meeting_asr_model_id`
          on the settings store; start_recording's preflight is the
          enforcement point. Pro-gating mirrors the existing model-row
          treatment below: locked rows show ProNudge + disabled Install. */}
      <Card
        title="Meeting mode model"
        description="Meeting Mode uses a Whisper model to transcribe full meetings. Pick the trade-off that fits how you'll use it."
      >
        <ul className="divide-y divide-border-soft/50">
          {meetingPickerOptions.map((opt) => {
            const entry = items.find((m) => m.id === opt.id);
            const installed = !!entry?.installed;
            const requiresPro = !!entry?.requires_pro;
            const locked = requiresPro && !pro;
            const checked = meetingAsrModelId === opt.id;
            const sizeLabel = entry ? formatBytes(entry.size_bytes) : '-';
            // v0.15.3 round-3 fix: read the same progress map the main
            // model list uses, so an install kicked off from the
            // Meeting card shows phase + bytes + bar in this row, the
            // Install button disables while the download is in flight,
            // and a second click can't re-start install_model for the
            // same id from this card.
            const prog = progress[opt.id];
            const downloading = !!prog && prog.phase !== 'done' && prog.phase !== 'error';
            const pct =
              prog && prog.bytes_total > 0
                ? Math.round((prog.bytes_done / prog.bytes_total) * 100)
                : 0;
            return (
              <li key={opt.id} className="py-3">
                <label className="flex cursor-pointer items-start gap-3">
                  <input
                    type="radio"
                    name="meeting-asr-model-id"
                    className="mt-1 h-4 w-4 cursor-pointer accent-accent"
                    checked={checked}
                    onChange={() => updateMeetingAsrModelId(opt.id)}
                  />
                  <div className="flex-1">
                    <div className="flex items-center gap-2 text-[13px] font-bold text-fg-hi">
                      {opt.label}
                      {opt.id === 'sherpa-whisper-small' && (
                        // v0.15.4: Recommended chip moved from
                        // Medium to Small. Keep this in lockstep
                        // with `recommended: true` in the
                        // services/models.rs catalogue entry —
                        // both surfaces describe the same default.
                        <span className="ub-chip ub-chip--live">
                          <span className="ub-led" aria-hidden />
                          Recommended
                        </span>
                      )}
                    </div>
                    <div className="mt-0.5 font-mono text-[11px] text-fg-dim">
                      {sizeLabel} · Multilingual{requiresPro ? ' · Pro' : ''}
                    </div>
                    <p className="mt-1 max-w-xl text-[12px] leading-relaxed text-muted">
                      {opt.subtitle}
                    </p>
                    {locked && (
                      <ProNudge
                        feature={entry?.display_name ?? opt.label}
                        description="This Whisper checkpoint is part of Pro. Upgrade to install and use it for Meeting Mode."
                        dense
                        className="mt-2 max-w-xl"
                      />
                    )}
                    {/* In-progress install: same shape as the main
                        model list further down — bar + bytes counter.
                        Visible inside the meeting card so the user
                        doesn't have to scroll to find their download. */}
                    {downloading &&
                      (prog!.phase === 'downloading' || prog!.phase === 'starting' ? (
                        <div className="mt-3 flex items-center gap-3">
                          <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-bg-2">
                            <div
                              className="h-full bg-accent transition-[width] duration-200"
                              style={{ width: `${pct}%` }}
                            />
                          </div>
                          <span className="w-28 text-right font-mono text-[11px] tabular-nums text-muted">
                            {formatBytes(prog!.bytes_done)} / {formatBytes(prog!.bytes_total)}
                          </span>
                        </div>
                      ) : (
                        // Post-download: honest label instead of a frozen counter.
                        <div className="mt-3 flex items-center gap-3">
                          <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-bg-2">
                            <div className="h-full w-full animate-pulse rounded-full bg-accent/70" />
                          </div>
                          <span className="whitespace-nowrap text-[11px] text-muted">
                            {prog!.phase === 'verifying'
                              ? 'Checking the download…'
                              : 'Unzipping, this can take a minute…'}
                          </span>
                        </div>
                      ))}
                  </div>
                  <div className="ml-2 shrink-0 self-center">
                    {installed ? (
                      <span className="font-mono text-[11px] text-success">Installed</span>
                    ) : downloading ? (
                      <span className="font-mono text-[11px] text-muted">
                        {capitalise(prog!.phase)}…
                      </span>
                    ) : (
                      <button
                        type="button"
                        onClick={(e) => {
                          e.preventDefault();
                          // Defence-in-depth: the disabled prop already
                          // gates this for `locked || !entry || downloading`,
                          // but a second click during the brief render
                          // cycle before the prog event arrives could
                          // race a duplicate install_model call. Bail
                          // here too.
                          if (locked || !entry || downloading) return;
                          void invoke('install_model', { id: opt.id })
                            .then(reload)
                            .catch(() => undefined);
                        }}
                        disabled={locked || !entry || downloading}
                        className="ub-btn ub-btn-cta !py-[0.4rem] !text-[12px] disabled:opacity-40 disabled:cursor-not-allowed"
                      >
                        Install
                      </button>
                    )}
                  </div>
                </label>
              </li>
            );
          })}
        </ul>
        <p className="mt-3 text-[11px] leading-relaxed text-muted">
          For English meetings, also set Whisper language to English in the section below for best
          accuracy.
        </p>
      </Card>

      {/* Whisper language hint. Only relevant when a Whisper model is
          installed, the picker hides otherwise so the page doesn't
          surface controls that don't apply to the user's setup.
          "Auto-detect" is the default and best for users who dictate in
          more than one language; pinning a specific language is a
          minor speed win when always dictating in the same one. */}
      {hasWhisperInstalled && (
        <Card
          title={t('models.whisper_language.title')}
          description={t('models.whisper_language.description')}
        >
          <div className="flex items-center gap-3">
            <Radio className="h-4 w-4 text-accent" />
            <Select
              value={whisperLanguage}
              onChange={(v) => updateWhisperLanguage(v)}
              minWidth="min-w-[20rem]"
              options={whisperLanguageOptions}
            />
          </div>
        </Card>
      )}

      {/* Language-compatibility warning. Placed directly below the
          Whisper language picker so the warning sits next to the
          control that triggered it; previously rendered between the
          Default model card and the Meeting Mode card, where the link
          back to the language pick wasn't visually obvious. The
          warning's own gating is independent of the Whisper-language
          card's `hasWhisperInstalled` precondition: a user can hold a
          non-English `whisper_language` value with zero Whisper models
          installed (the Whisper-language card hides in that case but
          the no-multilingual-installed warning still fires below).
          When the user has set Whisper language to a non-English
          language but the selected default model is English-only
          (Parakeet, English-only Whisper variants, Zipformer), warn
          and offer a one-click switch to a multilingual model among
          the *installed* set. When no installed model is multilingual,
          point the user at the install list further down rather than
          auto-installing. Meeting Mode below has no equivalent banner
          because every option in its picker is already multilingual
          Whisper. */}
      {showEnglishOnlyWarning && settingsRecommendation.recommendedId && (
        <div className="rounded-input border border-rose-500/40 bg-rose-500/10 p-4">
          <div className="text-[13px] font-medium text-rose-300">
            {t('models.language_warning.english_only_title')}
          </div>
          <div className="mt-1 text-[12px] leading-relaxed text-muted">
            {/* selectedDefaultModel.display_name is Rust-emitted and
                stays verbatim; the surrounding catalogue template
                interpolates it as `{{model}}`. */}
            {t('models.language_warning.english_only_body', {
              model: selectedDefaultModel?.display_name ?? defaultModelId ?? '',
            })}
          </div>
          <button
            type="button"
            onClick={() =>
              updateDefaultModel(settingsRecommendation.recommendedId)
            }
            className="mt-3 ub-btn ub-btn-cta !py-[0.4rem] !text-[12px]"
          >
            {t('models.language_warning.switch_to_multilingual')}
          </button>
        </div>
      )}
      {showNoMultilingualWarning && (
        <div className="rounded-input border border-amber-500/40 bg-amber-500/10 p-4">
          <div className="text-[13px] font-medium text-amber-300">
            {t('models.language_warning.no_multilingual_title')}
          </div>
          <div className="mt-1 text-[12px] leading-relaxed text-muted">
            {t('models.language_warning.no_multilingual_body')}
          </div>
          <div className="mt-2 text-[11.5px] text-muted">
            {t('models.language_warning.install_multilingual_hint')}
          </div>
        </div>
      )}

      <Card
        title="Keep-warm duration"
        description="The first dictation after a cold start loads the model into memory, that one takes a few seconds. Subsequent dictations while the model is warm feel near-instant. We keep the model loaded for a while after each use so you don't pay the cold-start cost on every burst of speech."
      >
        {/* Keep-loaded-forever sentinel. Backed in services/inference: when
            warm_duration_secs is u32::MAX the inline-unload skips and the
            idle sweeper short-circuits, the model stays resident until
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

      <Card>
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
                    {/* Weight 700, font-semibold (600) forbidden per §2.3. */}
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
                    {/* Paired comparison bars, makes the trade-offs between
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
                        description="The higher-precision fp16 model is part of Pro. It's the accuracy ceiling of the Parakeet family and benefits from a real GPU, same privacy guarantees as every other model (never leaves your machine)."
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
                {downloading &&
                  (prog!.phase === 'downloading' || prog!.phase === 'starting' ? (
                    // While bytes are arriving, show real progress + counter.
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
                  ) : (
                    // After the download, the byte counter is meaningless and
                    // stays frozen at 100%, which reads as a hang. Show an
                    // animated bar + an honest label for what's actually
                    // happening (verify / unzip), which can take a while on
                    // large models.
                    <div className="mt-3 flex items-center gap-3">
                      <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-bg-2">
                        <div className="h-full w-full animate-pulse rounded-full bg-accent/70" />
                      </div>
                      <span className="whitespace-nowrap text-[11px] text-muted">
                        {prog!.phase === 'verifying'
                          ? 'Checking the download…'
                          : 'Unzipping, this can take a minute…'}
                      </span>
                    </div>
                  ))}
              </li>
            );
          })}
        </ul>
      </Card>

      </div>
    </div>
  );
}

/**
 * Subtle runtime-state badge shown next to "Default model".
 *
 * ASR models are lazily loaded, the first dictation after launch pays a
 * 1–3 s load cost (ONNX session init, GPU upload) and subsequent dictations
 * land near-instant while the model stays warm. Users who don't know this
 * experience the first-dictation lag as "the app is slow" with no visible
 * reason. This badge makes the cold/warm state legible:
 *
 *   • Ready , model is resident in memory, next dictation is instant.
 *              Cyan LED + `text-accent`, single beat of the cyan budget.
 *   • Idle  , orchestrator is cold, next dictation will take a moment.
 *              Muted dot + `text-muted`, sits back into the card chrome.
 *
 * Deliberately small and quiet, this is meta, not a control. Don't give
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
