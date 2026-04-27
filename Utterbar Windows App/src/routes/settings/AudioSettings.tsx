import { useEffect, useRef, useState } from 'react';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { Select, type SelectOption } from '@/components/common/Select';
import { invoke, listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { useSettings } from '@/state/settings';
import { cn } from '@/lib/cn';

const CAPTURE_SOURCES = [
  {
    id: 'microphone' as const,
    label: 'Microphone',
    note: 'Record your voice from the selected microphone. Best for everyday dictation.',
  },
  {
    id: 'system' as const,
    label: 'System audio',
    note: 'Record whatever is playing through your speakers, meetings, videos, voice notes. WASAPI loopback: no virtual cable needed.',
  },
  {
    id: 'mixed' as const,
    label: 'Mixed (mic + system)',
    note: 'Both at once, summed. Useful for transcribing calls where you also speak.',
  },
];

type InputDevice = {
  id: string;
  name: string;
  is_default: boolean;
  supported_sample_rates: number[];
};

type NoiseLevel = 'off' | 'light' | 'balanced' | 'aggressive';

const NOISE_LEVELS: {
  id: NoiseLevel;
  label: string;
  blurb: string;
  /** High-pass cutoff (Hz) used for the in-UI audio preview. 0 = bypass. These
   *  mirror the backend's level_params(), see services/audio/preprocess.rs.
   *  Preview is a short synthesised noise-plus-rumble clip run through a
   *  biquad HPF at this frequency, so you can hear each level cleaning up. */
  previewCutoffHz: number;
}[] = [
  {
    id: 'off',
    label: 'Off',
    blurb: 'No processing. Lowest latency, keeps every nuance for the model.',
    previewCutoffHz: 0,
  },
  {
    id: 'light',
    label: 'Light',
    blurb: 'Trim subwoofer-band rumble (under ~60 Hz). Good for quiet rooms.',
    previewCutoffHz: 60,
  },
  {
    id: 'balanced',
    label: 'Balanced',
    blurb: 'Cuts fan hum, HVAC, and keyboard clatter between words. Best for most setups.',
    previewCutoffHz: 120,
  },
  {
    id: 'aggressive',
    label: 'Aggressive',
    blurb: 'Harder high-pass + higher gate. For cafés and noisy rooms; can clip soft consonants.',
    previewCutoffHz: 200,
  },
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

  const deviceOptions: SelectOption<string>[] = [
    { value: '', label: 'System default' },
    ...devices.map((d) => ({
      value: d.id,
      label: d.name,
      caption: d.is_default ? 'system default' : undefined,
    })),
  ];

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader title="Audio" subtitle="Input devices, preprocessing, and mic testing." />
      <div className="mt-6 space-y-4">
        <Card
          title="Capture source"
          description="What Utterbar listens to. Switch to System audio to transcribe a meeting or a video playing in another app; Mixed records both so a call where you also speak gets both sides."
        >
          {/* Capture-source picker, surface-elevation selection (selected
              tile lifts to bg-2 + gets the cyan left rail) consistent with
              the rest of the app's active-state vocabulary. */}
          <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
            {CAPTURE_SOURCES.map((opt) => {
              const selected = (s.audio.capture_source ?? 'microphone') === opt.id;
              return (
                <button
                  key={opt.id}
                  onClick={() => void patch({ audio: { ...s.audio, capture_source: opt.id } })}
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
                  <div className="pl-2 text-[13px] font-medium">{opt.label}</div>
                  <div className="pl-2 mt-1 text-[11.5px] leading-relaxed">{opt.note}</div>
                </button>
              );
            })}
          </div>
        </Card>

        <Card title="Input device">
          <Row
            label="Microphone"
            description="Which device Utterbar records from. System default follows whatever Windows has set."
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
          <Row label="Live meter">
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
                {metering ? 'Stop' : 'Test'}
              </button>
            </div>
          </Row>
        </Card>

        <Card
          title="Noise handling"
          description="Clean up the recording before it reaches Parakeet. Everything here runs locally in sub-millisecond time, there's no cloud pass."
        >
          <Row
            label="Suppression strength"
            description="Broadband filter that kills rumble, fan hum, and HVAC noise. Start with Balanced; step up to Aggressive only if there's still a lot of room noise in the background."
          >
            <div className="flex flex-wrap gap-2">
              {NOISE_LEVELS.map((lvl) => {
                const selected = noiseLevel === lvl.id;
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
                    title={`${lvl.blurb} · Click to preview`}
                    className={cn('ub-chip', selected && 'ub-chip--active')}
                  >
                    {lvl.label}
                  </button>
                );
              })}
            </div>
          </Row>
          <div className="-mt-2 pl-1 pr-1 text-[11px] leading-relaxed text-muted">
            {NOISE_LEVELS.find((x) => x.id === noiseLevel)?.blurb}{' '}
            <span className="opacity-70">Tap any level to hear a ~1 s sample.</span>
          </div>

          <Row
            label="Between-word gate"
            description="Zero out short quiet gaps so keyboard clicks, mouse-clicks, lip-smacks, and breath sounds between phrases don't reach the model. Complements the suppression above, safe to use at any level."
          >
            <Toggle
              checked={Boolean((s.audio as Record<string, unknown>).noise_gate)}
              onChange={(v) =>
                void patch({ audio: { ...s.audio, noise_gate: v } as typeof s.audio })
              }
            />
          </Row>
          <Row
            label="Dynamic normalisation"
            description="Boost quiet recordings to a consistent level before transcription. Helps when the microphone is far or the speaker is soft."
          >
            <Toggle
              checked={Boolean(s.audio.dynamic_normalisation)}
              onChange={(v) => void patch({ audio: { ...s.audio, dynamic_normalisation: v } })}
            />
          </Row>
          <Row
            label="Silence trimming"
            description="Drop silent leading / trailing audio before the model sees it. Faster transcriptions, identical quality."
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
