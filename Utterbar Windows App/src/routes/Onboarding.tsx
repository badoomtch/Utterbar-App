import { AnimatePresence, motion } from 'framer-motion';
import { useEffect, useRef, useState } from 'react';
import {
  ArrowRight,
  CheckCircle2,
  Check,
  ExternalLink,
  Lock,
  Mic,
  Sparkles,
  SlidersHorizontal,
  Keyboard,
  Palette,
  Square,
  Zap,
} from 'lucide-react';
import { open as openShell } from '@tauri-apps/plugin-shell';

import { UtterbarLogoMark } from '@/components/bar/UtterbarLogoMark';
import { UtterbarMiniMark } from '@/components/bar/UtterbarMiniMark';
import { UtterbarWordmark } from '@/components/bar/UtterbarWordmark';
import { Waveform } from '@/components/bar/Waveform';
import { Button } from '@/components/common/Button';
import { Select } from '@/components/common/Select';
import { ScoreBars } from '@/components/common/ScoreBars';
import { Textarea } from '@/components/common/Input';
import { Kbd } from '@/components/common/Kbd';
import { useDictation } from '@/state/dictation';
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { useSettings } from '@/state/settings';
import { springs } from '@/design/motion';
import { cn } from '@/lib/cn';

// 9 steps — down from 10. The separate "Record a quick test" step is folded into the
// mic step now, since having both a mic-test bar and a subsequent record-test page
// on back-to-back screens was confusing.
// Order rationale: narrative first (welcome + privacy), then a single cosmetic pick
// (theme) to warm the user up before any technical commitment. Then the functional
// chain — install model → mic → typing baseline → trigger style → shortcut — flows
// uninterrupted into the payoff (ready).
const ALL_STEPS = [
  { id: 'welcome', title: 'Welcome to Utterbar' },
  { id: 'privacy', title: 'Everything local, always' },
  { id: 'theme', title: 'Pick a theme' },
  { id: 'model', title: 'Choose your ASR model' },
  // `gpu` is conditionally included — only shown when a hardware GPU is detected.
  { id: 'gpu', title: 'Use your GPU?' },
  { id: 'mic', title: 'Pick and test your microphone' },
  { id: 'baseline', title: 'Your typing speed' },
  { id: 'trigger', title: 'How would you like to trigger recording?' },
  { id: 'shortcut', title: 'Bind a recording shortcut' },
  { id: 'bar', title: 'Where Utterbar lives' },
  { id: 'ready', title: 'Ready to dictate' },
];

type GpuProbeResult = {
  adapters: Array<{ name: string; is_software: boolean; dedicated_video_memory_mb: number }>;
  has_hardware_gpu: boolean;
};

export function Onboarding() {
  const [idx, setIdx] = useState(0);
  const [gpu, setGpu] = useState<GpuProbeResult | null>(null);

  useEffect(() => {
    void invoke<GpuProbeResult>('probe_gpu')
      .then(setGpu)
      .catch(() => setGpu({ adapters: [], has_hardware_gpu: false }));
  }, []);

  const STEPS = gpu?.has_hardware_gpu ? ALL_STEPS : ALL_STEPS.filter((s) => s.id !== 'gpu');

  const step = STEPS[idx];
  const back = () => setIdx((i) => Math.max(0, i - 1));
  const next = () => setIdx((i) => Math.min(STEPS.length - 1, i + 1));

  const settings = useSettings((x) => x.data);
  const hasModelChoice = Boolean(settings?.models?.default_asr_model_id);
  const canAdvance = step.id === 'model' ? hasModelChoice : true;

  const finish = async () => {
    await invoke('finish_onboarding').catch((e) => console.error('finish_onboarding', e));
  };

  return (
    <div className="flex h-screen w-screen flex-col bg-bg-0">
      <header className="flex items-center justify-between px-8 pt-6">
        <div className="flex items-center gap-2">
          <UtterbarLogoMark size={26} />
          <UtterbarWordmark size={18} />
        </div>
        <div className="flex items-center gap-1">
          {STEPS.map((_, i) => (
            <span
              key={i}
              className={cn(
                'h-1 rounded-full transition-all',
                i <= idx ? 'w-6 bg-accent' : 'w-3 bg-border-soft',
              )}
            />
          ))}
        </div>
      </header>

      <main className="flex flex-1 items-center justify-center px-8">
        <AnimatePresence mode="wait">
          <motion.div
            key={step.id}
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={springs.glide}
            className="w-full max-w-xl"
          >
            <StepContent id={step.id} onNext={next} onFinish={finish} gpu={gpu} />
          </motion.div>
        </AnimatePresence>
      </main>

      <footer className="flex items-center justify-between border-t border-border-soft/60 px-8 py-4">
        <div className="flex items-center gap-4">
          <button
            onClick={back}
            disabled={idx === 0}
            className="text-[13px] text-muted transition-colors hover:text-fg-hi disabled:opacity-30 disabled:hover:text-muted"
          >
            Back
          </button>
          <CreditLine />
        </div>
        {idx < STEPS.length - 1 ? (
          <div className="flex flex-col items-end gap-1">
            <Button
              variant="cta"
              size="sm"
              onClick={next}
              disabled={!canAdvance}
              className="!py-[0.45rem]"
            >
              Continue <ArrowRight className="h-3.5 w-3.5" />
            </Button>
            {step.id === 'model' && !hasModelChoice && (
              <span className="text-[10px] text-muted">
                Install a model or pick <span className="text-fg-hi">Skip for now</span> to continue.
              </span>
            )}
          </div>
        ) : (
          <Button variant="cta" size="sm" onClick={finish} className="!py-[0.45rem]">
            Start dictating <Check className="h-3.5 w-3.5" />
          </Button>
        )}
      </footer>
    </div>
  );
}

function CreditLine() {
  return (
    <button
      onClick={() => void openShell('https://jayparry.me').catch(() => undefined)}
      className="group inline-flex items-center gap-1 text-[11px] text-muted transition-colors hover:text-fg-hi"
    >
      App by <span className="text-fg-hi group-hover:text-accent">Jay Parry</span>
      <ExternalLink className="h-2.5 w-2.5 opacity-60 group-hover:opacity-100" />
    </button>
  );
}

function StepContent({
  id,
  onNext: _onNext,
  onFinish: _onFinish,
  gpu,
}: {
  id: string;
  onNext: () => void;
  onFinish: () => void;
  gpu: GpuProbeResult | null;
}) {
  switch (id) {
    case 'welcome':
      return <WelcomeStep />;
    case 'privacy':
      return (
        <Hero
          icon={<Lock className="h-9 w-9 text-accent" />}
          title="Everything stays on your machine"
          body="No cloud transcription. No cloud post-processing. No telemetry by default. Your recordings, transcripts, and preferences are stored only on this PC."
        />
      );
    case 'model':
      return <ModelPickStep />;
    case 'gpu':
      return <GpuStep gpu={gpu} />;
    case 'mic':
      return <MicAndTestStep />;
    case 'baseline':
      return <BaselineStep />;
    case 'trigger':
      return <TriggerStep />;
    case 'shortcut':
      return (
        <Hero
          icon={<Keyboard className="h-9 w-9 text-accent" />}
          title="Bind a recording shortcut"
          body="You can bind a global shortcut from Settings → Shortcuts once onboarding finishes. Any key combo works in any app. Ctrl+Space is set by default."
        />
      );
    case 'theme':
      return <ThemeStep />;
    case 'bar':
      return <BarStep />;
    case 'ready':
      return <ReadyStep />;
  }
  return null;
}

function WelcomeStep() {
  // Big logo + wordmark on the page itself — no framed wrapper. First impression
  // of the brand, so we let it breathe against bg-0.
  return (
    <div className="flex flex-col items-center text-center">
      <motion.div
        initial={{ scale: 0.6, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.32, ease: [0.32, 0.72, 0, 1], delay: 0.1 }}
        className="flex items-center gap-5"
      >
        <UtterbarLogoMark size={96} active />
        <UtterbarWordmark size={64} />
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.35, duration: 0.4 }}
        className="mt-10"
      >
        <h1 className="text-3xl font-bold tracking-[-0.02em] text-fg-hi">Welcome</h1>
        <p className="mx-auto mt-3 max-w-md text-[14px] leading-relaxed text-muted">
          A premium dictation app for Windows that runs entirely on your machine. We'll get
          you set up in under two minutes.
        </p>
      </motion.div>
    </div>
  );
}

function Hero({ icon, title, body }: { icon: React.ReactNode; title: string; body: string }) {
  return (
    <div className="rounded-block bg-bg-1 p-10 text-center">
      <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-card bg-[color-mix(in_oklab,var(--accent)_14%,var(--bg-2))]">
        {icon}
      </div>
      <h1 className="mt-6 text-[26px] font-bold tracking-[-0.02em] text-fg-hi">{title}</h1>
      <p className="mx-auto mt-3 max-w-md text-[14px] leading-relaxed text-muted">{body}</p>
    </div>
  );
}

/**
 * RadioTile — the universal "pick one of these" surface used by the model, gpu,
 * and trigger steps. Replaces the legacy `border-accent/60 bg-accent/10` pattern
 * with the §1.3 active-state vocabulary: bg-bg-2 lift + 2px cyan left rail.
 */
function RadioTile({
  selected,
  onSelect,
  disabled,
  name,
  value,
  children,
}: {
  selected: boolean;
  onSelect: () => void;
  disabled?: boolean;
  name: string;
  value: string;
  children: React.ReactNode;
}) {
  return (
    <label
      className={cn(
        'group relative flex items-start gap-3 rounded-input p-4 text-[13px] transition-colors',
        selected ? 'bg-bg-2' : 'bg-bg-2/40 hover:bg-bg-2/70',
        disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer',
      )}
    >
      <span
        className={cn(
          'absolute left-0 top-2 bottom-2 w-[2px] rounded-r-full transition-opacity',
          selected ? 'bg-accent opacity-100' : 'opacity-0',
        )}
        aria-hidden
      />
      <input
        type="radio"
        name={name}
        value={value}
        checked={selected}
        disabled={disabled}
        onChange={onSelect}
        className="mt-0.5 accent-[var(--accent)]"
      />
      <span className="flex-1 pl-1">{children}</span>
    </label>
  );
}

type ModelCatEntry = {
  id: string;
  display_name: string;
  description: string;
  size_bytes: number;
  languages: string[];
  installed: boolean;
  recommended: boolean;
  accuracy_score: number;
  speed_score: number;
};

type ModelProgress = {
  model_id: string;
  bytes_done: number;
  bytes_total: number;
  phase: 'starting' | 'downloading' | 'verifying' | 'extracting' | 'done' | 'error';
};

const EXTRACT_HINT_SECS: Record<string, number> = {
  'sherpa-parakeet-tdt-0.6b-v2-int8': 45,
  'sherpa-parakeet-tdt-0.6b-v2-fp16': 90,
};

function ModelPickStep() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  const [models, setModels] = useState<ModelCatEntry[]>([]);
  const [pick, setPick] = useState<string>('');
  const [progress, setProgress] = useState<ModelProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const downloadStartRef = useRef<number>(0);
  const extractStartRef = useRef<number>(0);
  const [nowTick, setNowTick] = useState(0);

  const reload = () =>
    invoke<ModelCatEntry[]>('list_available_models')
      .then((list) => {
        setModels(list);
        if (!pick) {
          const installed = list.find((m) => m.installed);
          const recommended = list.find((m) => m.recommended);
          setPick(installed?.id ?? recommended?.id ?? list[0]?.id ?? '');
        }
      })
      .catch(() => undefined);

  // Initial load runs once per mount. `reload` is intentionally a closure
  // that captures state setters (all stable), so there's no benefit to
  // re-subscribing on every change — but we still want the lint rule to
  // recognise this as deliberate, hence the eslint-disable.
  useEffect(() => {
    void reload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const p = listen<ModelProgress>(EVT.modelProgress, (e) => {
      setProgress((prev) => {
        if (e.payload.phase === 'downloading' && prev?.phase !== 'downloading') {
          downloadStartRef.current = Date.now();
        }
        if (e.payload.phase === 'extracting' && prev?.phase !== 'extracting') {
          extractStartRef.current = Date.now();
        }
        return e.payload;
      });
      if (e.payload.phase === 'done') void reload();
      if (e.payload.phase === 'error') setError('Download failed. Check your connection and try again.');
    });
    return () => void p.then((u) => u());
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // The ticker only cares about the current `phase`, not the full `progress`
  // object — adding `progress` as a dep would reinstate the interval on
  // every byte-progress event (many times per second during a big download).
  useEffect(() => {
    if (!progress) return;
    if (progress.phase !== 'downloading' && progress.phase !== 'extracting') return;
    const t = window.setInterval(() => setNowTick(Date.now()), 500);
    return () => window.clearInterval(t);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [progress?.phase]);

  const install = async () => {
    if (!pick) return;
    if (pick === 'stub') {
      if (s) await patch({ models: { ...s.models, default_asr_model_id: 'stub' } });
      return;
    }
    setError(null);
    setProgress({ model_id: pick, bytes_done: 0, bytes_total: 0, phase: 'starting' });
    try {
      await invoke('install_model', { id: pick });
      if (s) await patch({ models: { ...s.models, default_asr_model_id: pick } });
    } catch (e) {
      setError(formatInvokeError(e));
    }
  };

  const active = models.find((m) => m.id === pick);
  const isInstalling =
    progress && progress.model_id === pick && progress.phase !== 'done' && progress.phase !== 'error';
  const isInstalled = active?.installed;
  const pct =
    progress && progress.bytes_total > 0
      ? Math.round((progress.bytes_done / progress.bytes_total) * 100)
      : 0;

  const statusLine = (() => {
    if (!progress) return null;
    const elapsedDownloadSecs = Math.max(0, Math.floor((nowTick - downloadStartRef.current) / 1000));
    const elapsedExtractSecs = Math.max(0, Math.floor((nowTick - extractStartRef.current) / 1000));
    switch (progress.phase) {
      case 'starting':
        return 'Connecting…';
      case 'downloading': {
        const rate = elapsedDownloadSecs > 0 ? progress.bytes_done / elapsedDownloadSecs : 0;
        const remaining = Math.max(0, progress.bytes_total - progress.bytes_done);
        const etaSecs = rate > 0 ? Math.ceil(remaining / rate) : null;
        const rateLabel = rate > 0 ? `${formatSize(rate)}/s` : '—';
        const etaLabel = etaSecs != null ? `${formatSecs(etaSecs)} left` : 'calculating…';
        return `Downloading · ${rateLabel} · ${etaLabel}`;
      }
      case 'verifying':
        return 'Verifying checksum…';
      case 'extracting': {
        const typical = EXTRACT_HINT_SECS[progress.model_id] ?? 60;
        return `Extracting · ${formatSecs(elapsedExtractSecs)} / ~${formatSecs(typical)}`;
      }
      case 'done':
        return 'Done.';
      case 'error':
        return 'Error — see below.';
    }
  })();

  return (
    <div className="rounded-block bg-bg-1 p-6">
      <div className="flex items-center gap-2">
        <SlidersHorizontal className="h-5 w-5 text-accent" />
        <h1 className="text-[22px] font-bold tracking-[-0.02em] text-fg-hi">Choose your ASR model</h1>
      </div>
      <p className="mt-2 text-[13px] leading-relaxed text-muted">
        Models run entirely on this machine. Change it later in Settings → Models.
      </p>

      <ul className="mt-5 space-y-2">
        {models.map((m) => (
          <li key={m.id}>
            <RadioTile
              name="onboarding_model"
              value={m.id}
              selected={pick === m.id}
              onSelect={() => setPick(m.id)}
              disabled={Boolean(isInstalling)}
            >
              <div className="flex flex-wrap items-center gap-2">
                <span className="font-medium text-fg-hi">{m.display_name}</span>
                <span className="font-mono text-[11px] text-fg-dim">
                  {formatSize(m.size_bytes)}
                </span>
                {m.recommended && (
                  <span className="ub-chip ub-chip--live">Recommended</span>
                )}
                {m.installed && (
                  <span className="ub-chip text-moss">Installed</span>
                )}
              </div>
              {m.description && (
                <p className="mt-1 text-[12px] leading-snug text-muted">
                  {shortDescription(m.id, m.description)}
                </p>
              )}
              {/* Same comparison bars the Settings → Models page shows, so
                  the onboarding picker can be made on the same criteria
                  the user will see later if they change their mind. Kept
                  compact to preserve the tight tile rhythm. */}
              <ScoreBars
                accuracy={m.accuracy_score}
                speed={m.speed_score}
                compact
                className="mt-2 max-w-[260px]"
              />
            </RadioTile>
          </li>
        ))}
        <li>
          <RadioTile
            name="onboarding_model"
            value="stub"
            selected={pick === 'stub'}
            onSelect={() => setPick('stub')}
            disabled={Boolean(isInstalling)}
          >
            <div className="font-medium text-fg-hi">Skip for now</div>
            <p className="mt-1 text-[12px] leading-snug text-muted">
              Placeholder transcriber. You can install a real model from Settings → Models later.
            </p>
          </RadioTile>
        </li>
      </ul>

      <div className="mt-5 space-y-2">
        <div className="flex items-center gap-3">
          <Button
            variant="cta"
            size="sm"
            onClick={() => void install()}
            disabled={!pick || Boolean(isInstalling) || (isInstalled && pick !== 'stub')}
          >
            {isInstalled && pick !== 'stub'
              ? 'Installed'
              : pick === 'stub'
                ? 'Use placeholder'
                : isInstalling
                  ? `${capitalise(progress!.phase)}…`
                  : 'Download & install'}
          </Button>

          {isInstalling && (
            <div className="flex flex-1 items-center gap-2">
              <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-bg-2">
                <div
                  className={cn(
                    'h-full transition-[width] duration-200',
                    progress?.phase === 'extracting'
                      ? 'bg-accent/70 animate-pulse'
                      : 'bg-accent',
                  )}
                  style={{ width: progress?.phase === 'extracting' ? '100%' : `${pct}%` }}
                />
              </div>
            </div>
          )}
        </div>

        {isInstalling && statusLine && (
          <div className="font-mono text-[10px] text-muted">{statusLine}</div>
        )}
      </div>

      {error && (
        <div className="mt-3 rounded-input bg-[color-mix(in_oklab,var(--rose)_10%,var(--bg-1))] p-2.5 text-[12px] text-rose">
          {error}
        </div>
      )}
    </div>
  );
}

/**
 * First sentence of the description — the card shows a punchy single line, the
 * Settings → Models page keeps the full version.
 */
function shortDescription(_id: string, desc: string): string {
  const firstStop = desc.indexOf('.');
  if (firstStop === -1) return desc;
  return desc.slice(0, firstStop + 1);
}

type InputDevice = {
  id: string;
  name: string;
  is_default: boolean;
};

/** Single step combining mic selection, live meter, and a real record+transcribe test. */
function MicAndTestStep() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  const [devices, setDevices] = useState<InputDevice[]>([]);
  const [meter, setMeter] = useState(0);
  const [metering, setMetering] = useState(false);

  type Phase = 'idle' | 'recording' | 'processing' | 'done' | 'error';
  const [phase, setPhase] = useState<Phase>('idle');
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const startedAt = useRef<number>(0);
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    void invoke<InputDevice[]>('list_input_devices')
      .then(setDevices)
      .catch(() => setDevices([]));
  }, []);

  useEffect(() => {
    const p = listen<{ rms: number }>(EVT.dictationMeter, (e) => setMeter(e.payload.rms));
    return () => void p.then((u) => u());
  }, []);

  useEffect(() => {
    return () => void invoke('stop_meter').catch(() => undefined);
  }, []);

  useEffect(() => {
    if (phase !== 'recording') return;
    const t = window.setInterval(() => setElapsed(Date.now() - startedAt.current), 100);
    return () => window.clearInterval(t);
  }, [phase]);

  const selectedId =
    (s?.audio as { input_device_id?: string | null } | undefined)?.input_device_id ?? '';

  const setDevice = async (id: string) => {
    if (!s) return;
    await patch({ audio: { ...s.audio, input_device_id: id || null } });
  };

  const startMeter = async () => {
    setMetering(true);
    await invoke('start_meter', { deviceId: selectedId || null });
  };
  const stopMeter = async () => {
    setMetering(false);
    await invoke('stop_meter');
    setMeter(0);
  };

  const startRecord = async () => {
    setError(null);
    setResult(null);
    if (metering) await stopMeter();
    try {
      await invoke('start_recording', { deviceId: selectedId || null });
      startedAt.current = Date.now();
      setElapsed(0);
      setPhase('recording');
    } catch (e) {
      setError(formatInvokeError(e));
      setPhase('error');
    }
  };

  const stopRecord = async () => {
    setPhase('processing');
    try {
      const entry = await invoke<{ processed_text: string; raw_text: string }>('stop_recording');
      setResult(entry.processed_text || entry.raw_text || '(empty)');
      setPhase('done');
    } catch (e) {
      setError(formatInvokeError(e));
      setPhase('error');
    }
  };

  const deviceOptions = [
    { value: '', label: 'System default' },
    ...devices.map((d) => ({
      value: d.id,
      label: d.name + (d.is_default ? ' · default' : ''),
    })),
  ];

  return (
    <div className="rounded-block bg-bg-1 p-8">
      <div className="flex items-center gap-2">
        <Mic className="h-5 w-5 text-accent" />
        <h1 className="text-[22px] font-bold tracking-[-0.02em] text-fg-hi">
          Pick and test your microphone
        </h1>
      </div>
      <p className="mt-3 text-[14px] leading-relaxed text-muted">
        Choose your input device, then record a short test to confirm it's working end-to-end.
      </p>

      <div className="mt-5 space-y-4">
        <Select
          value={selectedId}
          options={deviceOptions}
          onChange={(v) => void setDevice(v)}
          disabled={phase === 'recording' || phase === 'processing'}
          className="w-full"
        />

        <div className="flex items-center gap-3">
          <div className="h-2 flex-1 overflow-hidden rounded-full bg-bg-2">
            <div
              className="h-full bg-accent transition-[width] duration-75"
              style={{ width: `${Math.min(100, meter * 220)}%` }}
            />
          </div>
          {phase === 'idle' || phase === 'done' || phase === 'error' ? (
            <>
              <button
                onClick={() => (metering ? void stopMeter() : void startMeter())}
                className={cn(
                  'rounded-full px-3 py-1.5 text-[12px] transition-colors',
                  metering
                    ? 'bg-[color-mix(in_oklab,var(--accent)_14%,var(--bg-2))] text-accent'
                    : 'bg-bg-2 text-fg hover:text-fg-hi',
                )}
              >
                {metering ? 'Stop meter' : 'Live meter'}
              </button>
              <Button variant="ghost" size="sm" noLed onClick={() => void startRecord()}>
                <Mic className="h-3 w-3" /> Record test
              </Button>
            </>
          ) : phase === 'recording' ? (
            <>
              <span className="font-mono text-[12px] tabular-nums text-muted">
                {Math.floor(elapsed / 1000)
                  .toString()
                  .padStart(2, '0')}
                :
                {Math.floor((elapsed % 1000) / 10)
                  .toString()
                  .padStart(2, '0')}
              </span>
              <Button variant="danger" size="sm" onClick={() => void stopRecord()}>
                <Square className="h-3 w-3" /> Stop
              </Button>
            </>
          ) : (
            <span className="text-[12px] text-muted">Processing…</span>
          )}
        </div>

        {phase === 'done' && result && (
          <div className="rounded-input bg-[color-mix(in_oklab,var(--accent)_8%,var(--bg-1))] p-3">
            <div className="ub-eyebrow flex items-center gap-1.5 text-accent">
              <CheckCircle2 className="h-3.5 w-3.5" /> Transcript
            </div>
            <div className="mt-1.5 whitespace-pre-wrap text-[13px] text-fg">{result}</div>
          </div>
        )}

        {error && (
          <div className="rounded-input bg-[color-mix(in_oklab,var(--rose)_10%,var(--bg-1))] p-2.5 text-[12px] text-rose">
            {error}
          </div>
        )}

        {devices.length === 0 && (
          <div className="text-[12px] text-muted">
            No input devices detected. Plug in a microphone or check Windows privacy settings.
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Sample sentences for the typing test. Each is ~15-20 words of plain English
 * with a mix of common letter patterns — no pangrams, no dense punctuation,
 * no numbers. A test that feels like "normal typing" gives a WPM closer to
 * the user's real baseline than a quote-with-em-dashes benchmark would.
 *
 * We pick randomly rather than showing the same sentence every time because
 * a user who retests three times in a row would otherwise memorise it and
 * the second attempt would overshoot their true speed.
 */
const TYPING_TEST_SENTENCES = [
  "The quiet morning light crept across the floor as the kettle started to whistle softly in the kitchen.",
  "Most people only realise how much they type in a day when they stop to watch their own hands move.",
  "A good dictation tool should feel less like a tool and more like a second, patient pair of ears listening carefully.",
  "She picked up the notebook, turned to a fresh page, and wrote the first sentence that came into her head.",
  "Working from home for years had taught him to value the small rituals more than the long meetings ever could.",
] as const;

/**
 * Three-mode baseline step: measure, manual, or skip.
 *
 *   • measure — user types a provided sentence, WPM is calculated from the
 *     elapsed time between the first keystroke and the sentence being
 *     correctly matched. A retry button throws away the current run.
 *   • manual  — the original slider (preserved so users who know their WPM
 *     can still just punch in a number).
 *   • skip    — keeps the default 55 WPM and moves on.
 *
 * The measured WPM is rounded to the nearest 5 because anything finer is
 * noise for a single-sentence test — 47.3 WPM reads as false precision, 45
 * reads as an honest measurement.
 */
function BaselineStep() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  const current = s?.typing_baseline_wpm ?? 55;

  type Mode = 'choose' | 'measure' | 'manual' | 'skipped' | 'measured';
  const [mode, setMode] = useState<Mode>('choose');
  const [sentence, setSentence] = useState<string>(TYPING_TEST_SENTENCES[0]);
  const [typed, setTyped] = useState('');
  const [startAt, setStartAt] = useState<number | null>(null);
  const [measuredWpm, setMeasuredWpm] = useState<number | null>(null);

  const beginMeasure = () => {
    const pick = TYPING_TEST_SENTENCES[
      Math.floor(Math.random() * TYPING_TEST_SENTENCES.length)
    ];
    setSentence(pick);
    setTyped('');
    setStartAt(null);
    setMeasuredWpm(null);
    setMode('measure');
  };

  const onType = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const v = e.target.value;
    if (startAt === null && v.length > 0) setStartAt(Date.now());
    setTyped(v);
    // Finish the moment the user's typed text matches the sentence (trim
    // so a single trailing space doesn't gate them out). Using exact match
    // keeps the test honest — if a user types the sentence wrong the run
    // never auto-completes.
    if (v.trim() === sentence.trim() && startAt !== null) {
      const elapsedSec = (Date.now() - startAt) / 1000;
      const words = sentence.trim().split(/\s+/).length;
      const wpm = Math.round((words / elapsedSec) * 60 / 5) * 5;
      const clamped = Math.min(150, Math.max(15, wpm));
      setMeasuredWpm(clamped);
      void patch({ typing_baseline_wpm: clamped });
      setMode('measured');
    }
  };

  // Per-character correctness for the rendered sentence, so the user can
  // see typos in red and unfinished-text in muted grey. This makes the
  // test feel like monkeytype.com rather than a compliance checkbox.
  const renderedSentence = sentence.split('').map((ch, i) => {
    const tch = typed[i];
    if (tch === undefined) {
      return (
        <span key={i} className="text-muted/70">
          {ch}
        </span>
      );
    }
    if (tch === ch) {
      return (
        <span key={i} className="text-fg-hi">
          {ch}
        </span>
      );
    }
    return (
      <span key={i} className="bg-rose/20 text-rose">
        {ch}
      </span>
    );
  });

  return (
    <div className="rounded-block bg-bg-1 p-8">
      <h1 className="text-[22px] font-bold tracking-[-0.02em] text-fg-hi">
        Your typing speed baseline
      </h1>
      <p className="mt-3 text-[14px] leading-relaxed text-muted">
        We use this to estimate how much time you save by dictating. You can take a quick
        typing test, enter a number you already know, or skip for now and tune it later.
      </p>

      {mode === 'choose' && (
        <div className="mt-6 grid grid-cols-1 gap-2 sm:grid-cols-3">
          <BaselineChoice
            title="Take the test"
            sub="~20 seconds · most accurate"
            onClick={beginMeasure}
            primary
          />
          <BaselineChoice
            title="Enter manually"
            sub="Type in your known WPM"
            onClick={() => setMode('manual')}
          />
          <BaselineChoice
            title="Skip"
            sub={`Use the default ${current} WPM`}
            onClick={() => setMode('skipped')}
          />
        </div>
      )}

      {mode === 'measure' && (
        <div className="mt-6 space-y-4">
          {/* Sentence rendered with live correctness styling above the
              textarea. Kept at 16px with generous line-height so it reads
              as the thing to focus on. */}
          <div className="rounded-input bg-bg-0 p-4 font-mono text-[15px] leading-[1.75]">
            {renderedSentence}
          </div>
          <Textarea
            autoFocus
            value={typed}
            onChange={onType}
            placeholder="Start typing the sentence above…"
            rows={3}
            className="w-full"
          />
          <div className="flex items-center justify-between text-[12px] text-muted">
            <span>
              {startAt === null
                ? 'Timer starts on your first keystroke.'
                : `${Math.floor((Date.now() - startAt) / 1000)}s elapsed…`}
            </span>
            <button
              onClick={beginMeasure}
              className="text-accent underline-offset-2 hover:underline"
            >
              New sentence
            </button>
          </div>
        </div>
      )}

      {mode === 'measured' && measuredWpm != null && (
        <div className="mt-6 rounded-input bg-[color-mix(in_oklab,var(--accent)_8%,var(--bg-1))] p-4">
          <div className="ub-eyebrow text-accent">Result</div>
          <div className="mt-1 text-[18px] text-fg-hi">
            <span className="font-mono text-[32px] tabular-nums font-semibold">
              {measuredWpm}
            </span>{' '}
            words per minute
          </div>
          <div className="mt-3 flex items-center gap-3 text-[12px]">
            <button
              onClick={beginMeasure}
              className="rounded-full bg-bg-2 px-3 py-1.5 text-fg hover:text-fg-hi"
            >
              Try again
            </button>
            <button
              onClick={() => setMode('manual')}
              className="text-muted hover:text-fg-hi"
            >
              Enter manually instead
            </button>
          </div>
        </div>
      )}

      {mode === 'manual' && (
        <div className="mt-6">
          <div className="flex items-center gap-4">
            <input
              type="range"
              min={20}
              max={120}
              value={current}
              onChange={(e) => void patch({ typing_baseline_wpm: Number(e.target.value) })}
              className="flex-1"
            />
            <span className="w-20 text-right font-mono text-[16px] tabular-nums text-fg-hi">
              {current} wpm
            </span>
          </div>
          <div className="mt-3 flex items-center gap-3 text-[12px]">
            <button
              onClick={beginMeasure}
              className="text-accent underline-offset-2 hover:underline"
            >
              Take the test instead
            </button>
          </div>
        </div>
      )}

      {mode === 'skipped' && (
        <div className="mt-6 rounded-input bg-bg-2/60 p-4 text-[13px] text-muted">
          Using the default {current} WPM. You can fine-tune this later in
          Settings → General → Typing baseline.
          <div className="mt-3 flex items-center gap-3 text-[12px]">
            <button
              onClick={beginMeasure}
              className="text-accent underline-offset-2 hover:underline"
            >
              Change my mind — take the test
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function BaselineChoice({
  title,
  sub,
  onClick,
  primary,
}: {
  title: string;
  sub: string;
  onClick: () => void;
  primary?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'group relative flex flex-col items-start gap-1 rounded-input p-4 text-left transition-colors',
        primary
          ? 'bg-[color-mix(in_oklab,var(--accent)_14%,var(--bg-2))] text-accent hover:brightness-110'
          : 'bg-bg-2/60 text-fg hover:bg-bg-2 hover:text-fg-hi',
      )}
    >
      <span className="font-medium">{title}</span>
      <span className={cn('text-[11px]', primary ? 'text-accent/80' : 'text-muted')}>
        {sub}
      </span>
    </button>
  );
}

function TriggerStep() {
  const [pick, setPick] = useState<'toggle' | 'push_to_talk' | 'hold_to_talk'>('toggle');
  const items = [
    { id: 'toggle', label: 'Toggle', desc: 'Press once to start, press again to stop.' },
    { id: 'hold_to_talk', label: 'Hold to talk', desc: 'Hold the key while speaking; release to finalise.' },
    { id: 'push_to_talk', label: 'Push to talk', desc: 'Quick press-release starts recording; press again to stop.' },
  ] as const;

  return (
    <div className="rounded-block bg-bg-1 p-8">
      <h1 className="text-[22px] font-bold tracking-[-0.02em] text-fg-hi">
        How would you like to trigger recording?
      </h1>
      <ul className="mt-5 space-y-2">
        {items.map((it) => (
          <li key={it.id}>
            <RadioTile
              name="onboarding_trigger"
              value={it.id}
              selected={pick === it.id}
              onSelect={() => setPick(it.id as typeof pick)}
            >
              <div className="font-medium text-fg-hi">{it.label}</div>
              <div className="mt-0.5 text-[12px] leading-relaxed text-muted">{it.desc}</div>
            </RadioTile>
          </li>
        ))}
      </ul>
    </div>
  );
}

function ThemeStep() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  if (!s) return null;
  const themes = ['obsidian', 'graphite', 'paper'] as const;
  return (
    <div className="rounded-block bg-bg-1 p-8">
      <div className="flex items-center gap-2">
        <Palette className="h-5 w-5 text-accent" />
        <h1 className="text-[22px] font-bold tracking-[-0.02em] text-fg-hi">Pick a theme</h1>
      </div>
      <div className="mt-5 grid grid-cols-3 gap-3">
        {themes.map((t) => {
          const selected = s.general.theme === t;
          return (
            <button
              key={t}
              onClick={() => void patch({ general: { ...s.general, theme: t } })}
              className={cn(
                'relative rounded-input px-4 py-6 text-[13px] capitalize transition-colors',
                selected ? 'bg-bg-2 text-fg-hi' : 'bg-bg-2/40 text-fg hover:bg-bg-2/70',
              )}
            >
              <span
                className={cn(
                  'absolute left-0 top-3 bottom-3 w-[2px] rounded-r-full transition-opacity',
                  selected ? 'bg-accent opacity-100' : 'opacity-0',
                )}
                aria-hidden
              />
              {t}
            </button>
          );
        })}
      </div>
    </div>
  );
}

/**
 * GPU acceleration pick. Only shown when the native probe reports a hardware adapter.
 * Three choices: `auto` (recommended default), `direct_ml` (force GPU), `cpu` (force CPU).
 * CUDA is intentionally hidden here — the catch-all DirectML path works on NVIDIA too
 * and avoids shipping a separate CUDA runtime in v1. Power users can pick CUDA from
 * Settings → Advanced if they want.
 */
function GpuStep({ gpu }: { gpu: GpuProbeResult | null }) {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  if (!s) return null;

  const adapter = gpu?.adapters.find((a) => !a.is_software);
  const current = (s.advanced as { gpu_acceleration?: string }).gpu_acceleration ?? 'auto';

  const choose = (value: 'auto' | 'direct_ml' | 'cpu') =>
    patch({ advanced: { ...s.advanced, gpu_acceleration: value } });

  const options = [
    {
      id: 'auto' as const,
      title: 'Smart (recommended)',
      desc: 'Use your GPU when it helps, fall back to CPU for short clips. No surprises.',
    },
    {
      id: 'direct_ml' as const,
      title: 'Always use GPU',
      desc: 'Fastest transcription. May briefly slow games, 3D apps, or heavy browser tabs while you dictate.',
    },
    {
      id: 'cpu' as const,
      title: 'CPU only',
      desc: 'Keep the GPU free for other apps. Transcription is still fast, just a bit slower per utterance.',
    },
  ];

  return (
    <div className="rounded-block bg-bg-1 p-8">
      <div className="flex items-center gap-2">
        <Zap className="h-5 w-5 text-accent" />
        <h1 className="text-[22px] font-bold tracking-[-0.02em] text-fg-hi">Use your GPU?</h1>
      </div>
      <p className="mt-3 text-[14px] leading-relaxed text-muted">
        We detected{' '}
        <span className="font-medium text-fg-hi">{adapter?.name ?? 'a graphics card'}</span>
        {adapter && adapter.dedicated_video_memory_mb > 0
          ? ` with ${adapter.dedicated_video_memory_mb.toLocaleString()} MB of dedicated memory`
          : ''}
        . Running transcription on it is typically 3–10× faster than CPU. You can change this
        later in Settings → Advanced.
      </p>

      <ul className="mt-5 space-y-2">
        {options.map((opt) => (
          <li key={opt.id}>
            <RadioTile
              name="onboarding_gpu"
              value={opt.id}
              selected={current === opt.id}
              onSelect={() => void choose(opt.id)}
            >
              <div className="font-medium text-fg-hi">{opt.title}</div>
              <div className="mt-0.5 text-[12px] leading-relaxed text-muted">{opt.desc}</div>
            </RadioTile>
          </li>
        ))}
      </ul>
    </div>
  );
}

/**
 * Static preview of the floating pill anchored to a mock desktop. Uses whatever
 * theme + accent the user has picked in earlier steps, so the swatch is a true
 * "here's what you'll see" rather than a decoupled mockup.
 *
 * Note: this is the one place in the app where the legacy `.glass` utility is
 * still legitimately used — the DESIGN_SYSTEM §8.1 reserves it for the floating
 * bar preview exactly because the preview IS the bar replica.
 */
function BarStep() {
  const s = useSettings((x) => x.data);
  if (!s) return null;

  return (
    <div className="rounded-block bg-bg-1 p-8">
      <div className="flex items-center gap-2">
        <Sparkles className="h-5 w-5 text-accent" />
        <h1 className="text-[22px] font-bold tracking-[-0.02em] text-fg-hi">
          Where Utterbar lives
        </h1>
      </div>
      <p className="mt-3 max-w-lg text-[13px] leading-relaxed text-muted">
        A small bar will float near the top of your screen whenever Utterbar is running. It shows
        a live waveform while you dictate and lets you start, stop, or cancel with a click. You
        can drag it anywhere you like, and it'll remember the position.
      </p>

      {/* Mock desktop — a soft-grey rectangle with a "taskbar" strip that sells
          the "at the top of your screen" framing without needing an actual
          screenshot we'd have to keep up-to-date. */}
      <div className="relative mt-6 h-44 overflow-hidden rounded-card bg-gradient-to-b from-bg-0 to-bg-2/60">
        <div className="absolute inset-x-0 top-0 h-1 bg-border-soft/50" />
        <div className="absolute inset-x-0 bottom-0 h-5 border-t border-border-soft/40 bg-bg-2/70" />

        {/* Pill replica — mirrors FloatingBar.tsx's visual (glass + rounded-full).
            The .glass utility is intentional here per §8.1. */}
        <div className="absolute left-1/2 top-3 -translate-x-1/2">
          <div className="glass flex items-center gap-2 rounded-full px-2.5 py-1.5">
            <UtterbarMiniMark size={18} active />
            <span className="text-[12px] font-medium tracking-wide text-fg-hi">Utterbar</span>
            <div className="h-4 w-32">
              <Waveform rms={0.22} peak={0.45} active />
            </div>
            <span className="inline-flex h-5 w-5 items-center justify-center rounded-full bg-accent text-bg-0">
              <Mic className="h-2.5 w-2.5" />
            </span>
          </div>
        </div>

        <div className="ub-eyebrow absolute inset-x-0 bottom-6 text-center text-muted/60">
          your desktop
        </div>
      </div>

      <p className="mt-5 text-[13px] leading-relaxed text-muted">
        Tip: grab the bar anywhere that isn't a button to drag it. Hover to see the record, stop,
        history, and settings controls.
      </p>
    </div>
  );
}

function ReadyStep() {
  const s = useSettings((x) => x.data);
  const shortcut = (
    s?.shortcuts as { toggle_recording?: { label?: string; trigger_style?: string } } | undefined
  )?.toggle_recording;
  const label = shortcut?.label || 'Ctrl+Space';
  const triggerStyle = shortcut?.trigger_style || 'toggle';
  const keys = label.split('+');

  // Listen to the global dictation phase so the Ready step can narrate what's happening:
  // "Press the shortcut" → "Speak now — press again to stop" → "Processing" → result.
  const phase = useDictation((x) => x.phase);
  const meter = useDictation((x) => x.meter);

  const isListening = phase.kind === 'listening';
  const isProcessing = phase.kind === 'processing';

  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [value, setValue] = useState('');
  const [pressedOnce, setPressedOnce] = useState(false);

  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  useEffect(() => {
    const p = listen(EVT.shortcutTriggered, (e) => {
      if (e.payload === 'toggle_recording') setPressedOnce(true);
    });
    return () => void p.then((u) => u());
  }, []);

  useEffect(() => {
    const p = listen<{ processed_text?: string; raw_text?: string }>(
      EVT.dictationResult,
      (e) => {
        const text = e.payload.processed_text || e.payload.raw_text || '';
        if (!text) return;
        const ta = textareaRef.current;
        if (!ta) {
          setValue((v) => (v ? v + ' ' + text : text));
          return;
        }
        const start = ta.selectionStart ?? ta.value.length;
        const end = ta.selectionEnd ?? ta.value.length;
        const before = ta.value.slice(0, start);
        const after = ta.value.slice(end);
        const joiner = before && !/\s$/.test(before) ? ' ' : '';
        const next = before + joiner + text + after;
        setValue(next);
        window.setTimeout(() => {
          if (!textareaRef.current) return;
          textareaRef.current.focus();
          const caret = (before + joiner + text).length;
          textareaRef.current.setSelectionRange(caret, caret);
        }, 0);
      },
    );
    return () => void p.then((u) => u());
  }, []);

  // Phase-aware prompt. Gives the user a running monologue so they know exactly what
  // the app is doing, rather than guessing whether "listening" is real.
  const promptLine = (() => {
    if (isListening) {
      if (triggerStyle === 'hold_to_talk') return 'Speak now — release to finish.';
      if (triggerStyle === 'push_to_talk') return 'Speak now — press again to stop.';
      return 'Speak now — press the shortcut again to stop.';
    }
    if (isProcessing) return 'Processing your speech…';
    if (phase.kind === 'success') return 'Done — your words are in the box.';
    if (phase.kind === 'error') {
      return `Something went wrong: ${phase.message}`;
    }
    return 'Press the shortcut, speak, and your words will land in the box below.';
  })();

  return (
    <div className="rounded-block bg-bg-1 p-8">
      <div className="flex items-center justify-center gap-2">
        <Sparkles className="h-5 w-5 text-accent" />
        <h1 className="text-[22px] font-bold tracking-[-0.02em] text-fg-hi">You're ready</h1>
      </div>
      <p
        className={cn(
          'mx-auto mt-2 max-w-md text-center text-[13px] leading-relaxed transition-colors',
          isListening
            ? 'font-medium text-accent'
            : phase.kind === 'error'
              ? 'text-rose'
              : 'text-muted',
        )}
      >
        {promptLine}
      </p>

      <div className="mt-5 flex items-center justify-center gap-1.5">
        {keys.map((k, i) => (
          <span key={i} className="flex items-center gap-1.5">
            <Kbd
              className={cn(
                'transition-colors',
                isListening && 'text-accent',
              )}
            >
              {k}
            </Kbd>
            {i < keys.length - 1 && <span className="text-fg-dim">+</span>}
          </span>
        ))}
      </div>

      {/* Live waveform surface. When the user triggers recording, this replaces the
          static textbox hint with real-time mic activity, so "listening" is
          immediately obvious. */}
      <div className="relative mt-5 h-14 overflow-hidden rounded-input bg-bg-0">
        {isListening ? (
          <div className="flex h-full items-center px-3">
            <Waveform rms={meter.rms} peak={meter.peak} active={true} />
          </div>
        ) : (
          <div className="flex h-full items-center justify-center text-[11px] text-muted">
            {isProcessing ? (
              <span className="flex items-center gap-2">
                <span className="ub-led" />
                Transcribing…
              </span>
            ) : (
              <span>Waiting for the shortcut — the waveform will come alive when you're speaking.</span>
            )}
          </div>
        )}
      </div>

      <Textarea
        ref={textareaRef}
        value={value}
        onChange={(e) => setValue(e.target.value)}
        placeholder="Your dictated text will appear here…"
        rows={4}
        className="mt-3 w-full"
      />

      <div className="mt-3 text-center text-[11px]">
        {pressedOnce && !isListening && !isProcessing ? (
          <span className="inline-flex items-center gap-1.5 text-moss">
            <CheckCircle2 className="h-3.5 w-3.5" /> Shortcut detected — dictation is live.
          </span>
        ) : (
          <span className="text-muted">Or click Start dictating below when you're ready.</span>
        )}
      </div>

      {/* Warm-up expectation-setting. Without this, the first dictation taking 4-6s
          reads as "the app is slow" when really it's just a one-time load cost. */}
      <div className="mt-4 rounded-input bg-bg-2 p-3 text-[11px] leading-relaxed text-muted">
        <span className="font-medium text-fg-hi">Heads up:</span> the first dictation loads the
        model and takes a few seconds. Subsequent ones while the model is warm feel near-instant.
        Tune how long the model stays warm in Settings → Models.
      </div>
    </div>
  );
}

function formatSize(bytes: number): string {
  if (!bytes) return '—';
  const units = ['B', 'KB', 'MB', 'GB'];
  let i = 0;
  let v = bytes;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i += 1;
  }
  return `${v.toFixed(v >= 10 || i === 0 ? 0 : 1)} ${units[i]}`;
}

function capitalise(s: string) {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function formatSecs(secs: number): string {
  if (secs < 60) return `${secs}s`;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  if (m < 60) return s === 0 ? `${m}m` : `${m}m ${s}s`;
  const h = Math.floor(m / 60);
  const rem = m % 60;
  return rem === 0 ? `${h}h` : `${h}h ${rem}m`;
}

function formatInvokeError(e: unknown): string {
  if (e == null) return 'Unknown error';
  if (typeof e === 'string') return e;
  if (typeof e === 'object') {
    const obj = e as { message?: unknown };
    if (typeof obj.message === 'string' && obj.message.length > 0) return obj.message;
    try {
      return JSON.stringify(e);
    } catch {
      return Object.prototype.toString.call(e);
    }
  }
  return String(e);
}
