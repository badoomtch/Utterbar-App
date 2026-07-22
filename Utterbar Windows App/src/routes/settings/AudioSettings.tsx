import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { Select, type SelectOption } from '@/components/common/Select';
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { useSettings } from '@/state/settings';
import { cn } from '@/lib/cn';

// Stable id registry. ids match the `audio.capture_source` field's
// serialised values on the Rust side, so the patch wiring continues to
// write `'microphone' | 'system' | 'mixed'`. Visible label and note
// resolve via t() at render time.
const CAPTURE_SOURCE_IDS = ['microphone', 'system', 'mixed'] as const;

type InputDevice = {
  id: string;
  name: string;
  is_default: boolean;
  supported_sample_rates: number[];
};

type NoiseLevel = 'off' | 'light' | 'balanced' | 'aggressive';

// Stable id + structural numeric registry. ids match the
// `audio.noise_suppression` field's serialised string values on the
// Rust side and previewCutoffHz mirrors the backend's level_params()
// (see services/audio/preprocess.rs); both are structural and stay in
// source. Visible label and blurb resolve via t() at render time.
const NOISE_LEVELS: {
  id: NoiseLevel;
  /** High-pass cutoff (Hz) used for the in-UI audio preview. 0 = bypass.
   *  Preview is a short synthesised noise-plus-rumble clip run through a
   *  biquad HPF at this frequency, so you can hear each level cleaning up. */
  previewCutoffHz: number;
}[] = [
  { id: 'off', previewCutoffHz: 0 },
  { id: 'light', previewCutoffHz: 60 },
  { id: 'balanced', previewCutoffHz: 120 },
  { id: 'aggressive', previewCutoffHz: 200 },
];

/** Lazy-shared AudioContext. Creating one per click leaks nodes, Web Audio
 *  holds contexts for the page lifetime. One global context, resumed on first
 *  user gesture, is the idiomatic pattern. */
let sharedAudioCtx: AudioContext | null = null;
function getAudioCtx(): AudioContext | null {
  if (typeof window === 'undefined') return null;
  const Ctx = window.AudioContext ?? (window as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!Ctx) return null;
  if (!sharedAudioCtx) sharedAudioCtx = new Ctx();
  // Autoplay policy pauses the context until the first user gesture.
  if (sharedAudioCtx.state === 'suspended') void sharedAudioCtx.resume();
  return sharedAudioCtx;
}

/** Play a ~900 ms preview of "noisy room audio" through the given HPF cutoff.
 *  The sample is synthesised in-process (no asset bundled): brown-ish noise
 *  mixed with a 50 Hz mains-hum sine and a 90 Hz fan-hum sine, so each level's
 *  cutoff frequency has distinct content to remove. Off bypasses the filter
 *  entirely (you hear the full noise); Aggressive cuts most of it. Returns a
 *  cancel fn so callers can interrupt an in-flight preview when the user
 *  clicks another level mid-play. */
function playNoisePreview(cutoffHz: number): () => void {
  const ctx = getAudioCtx();
  if (!ctx) return () => {};
  const duration = 0.9;
  const now = ctx.currentTime;

  // Pre-computed brown-noise buffer, 1 ch, mono, same as our capture pipeline.
  const buf = ctx.createBuffer(1, Math.floor(ctx.sampleRate * duration), ctx.sampleRate);
  const data = buf.getChannelData(0);
  let last = 0;
  for (let i = 0; i < data.length; i++) {
    // Random walk (brown-noise approximation) biased toward low freq.
    last = (last + (Math.random() * 2 - 1) * 0.12) * 0.985;
    data[i] = last * 0.4;
  }

  const noise = ctx.createBufferSource();
  noise.buffer = buf;

  // Two tonal hums mixed in, gives the HPF something to bite on above the
  // brown noise floor so the difference between Light/Balanced/Aggressive is
  // audible rather than just a slightly-quieter hiss.
  const hum50 = ctx.createOscillator();
  hum50.type = 'sine';
  hum50.frequency.value = 50;
  const hum50Gain = ctx.createGain();
  hum50Gain.gain.value = 0.18;
  hum50.connect(hum50Gain);

  const hum90 = ctx.createOscillator();
  hum90.type = 'sine';
  hum90.frequency.value = 90;
  const hum90Gain = ctx.createGain();
  hum90Gain.gain.value = 0.12;
  hum90.connect(hum90Gain);

  // Mix, then optionally filter, then envelope so we don't click on stop.
  const mixer = ctx.createGain();
  mixer.gain.value = 1;
  noise.connect(mixer);
  hum50Gain.connect(mixer);
  hum90Gain.connect(mixer);

  let outNode: AudioNode = mixer;
  if (cutoffHz > 0) {
    const hpf = ctx.createBiquadFilter();
    hpf.type = 'highpass';
    hpf.frequency.value = cutoffHz;
    hpf.Q.value = 0.7;
    mixer.connect(hpf);
    outNode = hpf;
  }

  const env = ctx.createGain();
  env.gain.setValueAtTime(0, now);
  env.gain.linearRampToValueAtTime(0.35, now + 0.04);
  env.gain.setValueAtTime(0.35, now + duration - 0.08);
  env.gain.linearRampToValueAtTime(0, now + duration);
  outNode.connect(env);
  env.connect(ctx.destination);

  noise.start(now);
  hum50.start(now);
  hum90.start(now);
  noise.stop(now + duration);
  hum50.stop(now + duration);
  hum90.stop(now + duration);

  return () => {
    try {
      noise.stop();
      hum50.stop();
      hum90.stop();
    } catch {
      // Already stopped, ignore.
    }
  };
}

export function AudioSettings() {
  const { t } = useTranslation('settings');
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  const [devices, setDevices] = useState<InputDevice[]>([]);
  const [meter, setMeter] = useState(0);
  const [metering, setMetering] = useState(false);
  // Cancel handle for the current noise preview, so clicking another level
  // mid-play swaps cleanly instead of overlapping.
  const previewCancelRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    void invoke<InputDevice[]>('list_input_devices').then(setDevices).catch(() => setDevices([]));
  }, []);

  useEffect(() => {
    const p = listen<{ rms: number }>(EVT.dictationMeter, (e) => setMeter(e.payload.rms));
    return () => void p.then((u) => u());
  }, []);

  const startMeter = async () => {
    setMetering(true);
    await invoke('start_meter', { deviceId: s?.audio.input_device_id ?? null });
  };
  const stopMeter = async () => {
    setMetering(false);
    await invoke('stop_meter');
    setMeter(0);
  };

  if (!s) return null;

  // Noise suppression is now an enum (off/light/balanced/aggressive) but older
  // settings.json files stored a bool. The backend deserializer translates
  // bool→level transparently, but we still defensively coerce here so the UI
  // doesn't flash the wrong card if the Rust and TS types briefly disagree
  // on a single frame.
  const rawNoise = s.audio.noise_suppression as unknown;
  const noiseLevel: NoiseLevel =
    typeof rawNoise === 'string'
      ? (rawNoise as NoiseLevel)
      : rawNoise === true
        ? 'balanced'
        : 'off';

  // d.name is the OS-supplied device name and stays verbatim. d.id is
  // the structural device key the Rust side uses to match the chosen
  // microphone. The visible "System default" empty option and the
  // per-device "system default" caption flag are frontend-owned chrome
  // and resolve through t().
  const deviceOptions: SelectOption<string>[] = [
    { value: '', label: t('audio.input_device.system_default') },
    ...devices.map((d) => ({
      value: d.id,
      label: d.name,
      caption: d.is_default ? t('audio.input_device.system_default_caption') : undefined,
    })),
  ];

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader title={t('audio.title')} subtitle={t('audio.subtitle')} />
      <div className="mt-6 space-y-4">
        <Card
          title={t('audio.capture_source.title')}
          description={t('audio.capture_source.description')}
        >
          {/* Capture-source picker, surface-elevation selection (selected
              tile lifts to bg-2 + gets the cyan left rail) consistent with
              the rest of the app's active-state vocabulary. */}
          <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
            {CAPTURE_SOURCE_IDS.map((id) => {
              const selected = (s.audio.capture_source ?? 'microphone') === id;
              return (
                <button
                  key={id}
                  onClick={() => void patch({ audio: { ...s.audio, capture_source: id } })}
                  className={cn(
                    'group relative rounded-input p-3 text-left transition-colors',
                    selected ? 'bg-bg-2 text-fg-hi' : 'bg-bg-2/40 text-muted hover:bg-bg-2/70',
                  )}
                >
                  <span
                    className={cn(
                      'absolute left-0 top-2 bottom-2 w-[2px] rounded-r-full transition-opacity',
                      selected ? 'bg-accent opacity-100' : 'opacity-0',
                    )}
                    aria-hidden
                  />
                  <div className="pl-2 text-[13px] font-medium">
                    {t(`audio.capture_source.options.${id}.label` as const)}
                  </div>
                  <div className="pl-2 mt-1 text-[11.5px] leading-relaxed">
                    {t(`audio.capture_source.options.${id}.note` as const)}
                  </div>
                </button>
              );
            })}
          </div>
        </Card>

        {(s.audio.capture_source === 'system' || s.audio.capture_source === 'mixed') && (
          // Cross-suggestion. The user has just opted into a system-audio
          // capture, which is exactly what Meeting Mode automates. We
          // surface a callout so the connection isn't missed — the link
          // below jumps to /modes where they can flip the built-in
          // "Meeting" mode on or set the flag on a custom mode. Only
          // shown when the source is System or Mixed; the default
          // Microphone capture has no need for the suggestion.
          <Card
            title={t('audio.meeting_callout.title')}
            description={t('audio.meeting_callout.description')}
          >
            <a
              href="#/settings/modes"
              onClick={(e) => {
                e.preventDefault();
                void invoke('show_main_at', { path: '/modes' });
              }}
              className="text-[13px] text-accent hover:underline"
            >
              {t('audio.meeting_callout.link')}
            </a>
          </Card>
        )}

        <Card title={t('audio.input_device.title')}>
          <Row
            label={t('audio.input_device.microphone.label')}
            description={t('audio.input_device.microphone.description')}
          >
            <Select
              value={(s.audio.input_device_id as string | null) ?? ''}
              onChange={(v) =>
                void patch({ audio: { ...s.audio, input_device_id: v || null } })
              }
              options={deviceOptions}
              minWidth="min-w-[18rem]"
            />
          </Row>
          <Row label={t('audio.input_device.live_meter.label')}>
            <div className="flex items-center gap-3">
              <div className="h-2 w-40 overflow-hidden rounded-full bg-bg-2">
                <div
                  className="h-full bg-accent transition-[width] duration-75"
                  style={{ width: `${Math.min(100, meter * 220)}%` }}
                />
              </div>
              <button
                onClick={() => (metering ? void stopMeter() : void startMeter())}
                className="rounded-full bg-bg-2 px-3 py-1 text-[12px] text-muted transition-colors hover:text-fg-hi"
              >
                {metering
                  ? t('audio.input_device.live_meter.stop')
                  : t('audio.input_device.live_meter.test')}
              </button>
            </div>
          </Row>
        </Card>

        <Card
          title={t('audio.capture_tail.title')}
          description={t('audio.capture_tail.description')}
        >
          <Row
            label={t('audio.capture_tail.delay.label')}
            description={t('audio.capture_tail.delay.description', {
              milliseconds: Number((s.audio as Record<string, unknown>).capture_tail_pad_ms ?? 500),
            })}
          >
            <div className="flex w-full items-center gap-3">
              <input
                type="range"
                min={250}
                max={1500}
                step={250}
                value={Number((s.audio as Record<string, unknown>).capture_tail_pad_ms ?? 500)}
                onChange={(e) =>
                  void patch({
                    audio: {
                      ...s.audio,
                      capture_tail_pad_ms: Number(e.target.value),
                    } as typeof s.audio,
                  })
                }
                className="flex-1"
              />
              <span className="w-16 text-right font-mono text-[12px] tabular-nums text-fg-hi">
                {t('audio.capture_tail.delay.value', {
                  milliseconds: Number((s.audio as Record<string, unknown>).capture_tail_pad_ms ?? 500),
                })}
              </span>
            </div>
          </Row>
        </Card>

        <Card
          title={t('audio.noise_handling.title')}
          description={t('audio.noise_handling.description')}
        >
          <Row
            label={t('audio.noise_handling.suppression.label')}
            description={t('audio.noise_handling.suppression.description')}
          >
            <div className="flex flex-wrap gap-2">
              {NOISE_LEVELS.map((lvl) => {
                const selected = noiseLevel === lvl.id;
                const blurb = t(`audio.noise_handling.suppression.levels.${lvl.id}.blurb` as const);
                return (
                  <button
                    key={lvl.id}
                    onClick={() => {
                      // Persist the new level AND audibly preview it in one
                      // click, the old flow required a separate Preview
                      // button tap after selection, which made comparing
                      // levels tedious. Cancel any preview already in flight
                      // so rapid clicks don't stack.
                      previewCancelRef.current?.();
                      previewCancelRef.current = playNoisePreview(lvl.previewCutoffHz);
                      void patch({
                        audio: { ...s.audio, noise_suppression: lvl.id },
                      });
                    }}
                    title={t('audio.noise_handling.suppression.preview_tooltip', { blurb })}
                    className={cn('ub-chip', selected && 'ub-chip--active')}
                  >
                    {t(`audio.noise_handling.suppression.levels.${lvl.id}.label` as const)}
                  </button>
                );
              })}
            </div>
          </Row>
          <div className="-mt-2 pl-1 pr-1 text-[11px] leading-relaxed text-muted">
            {t(`audio.noise_handling.suppression.levels.${noiseLevel}.blurb` as const)}{' '}
            <span className="opacity-70">
              {t('audio.noise_handling.suppression.preview_hint')}
            </span>
          </div>

          <Row
            label={t('audio.noise_handling.noise_gate.label')}
            description={t('audio.noise_handling.noise_gate.description')}
          >
            <Toggle
              checked={Boolean((s.audio as Record<string, unknown>).noise_gate)}
              onChange={(v) =>
                void patch({ audio: { ...s.audio, noise_gate: v } as typeof s.audio })
              }
            />
          </Row>
          <Row
            label={t('audio.noise_handling.dynamic_normalisation.label')}
            description={t('audio.noise_handling.dynamic_normalisation.description')}
          >
            <Toggle
              checked={Boolean(s.audio.dynamic_normalisation)}
              onChange={(v) => void patch({ audio: { ...s.audio, dynamic_normalisation: v } })}
            />
          </Row>
          <Row
            label={t('audio.noise_handling.silence_removal.label')}
            description={t('audio.noise_handling.silence_removal.description')}
          >
            <Toggle
              checked={Boolean(s.audio.silence_removal)}
              onChange={(v) => void patch({ audio: { ...s.audio, silence_removal: v } })}
            />
          </Row>
        </Card>
      </div>
    </div>
  );
}
