import { useEffect } from 'react';

import { listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { useSettings } from '@/state/settings';
import type { DictationPhase } from '@/state/dictation';

type Cue = 'start' | 'stop' | 'success' | 'error';

// Synthesised cues: we generate short tones via the Web Audio API rather than bundling
// sample files. Keeps the app .msi leaner, gives us consistent sound across themes, and
// avoids a round of licensing paperwork on third-party SFX. Each profile is a family
// of four cues (start/stop/success/error) tailored to a single tonal character — see
// `SoundProfile` in `src-tauri/src/domain/settings.rs` for the short descriptions.
//
// Profiles also carry `lowpass` (Hz) and optional `attack`/`release` envelope overrides
// so we can build "warm" profiles that roll off the shrill 2–8 kHz band the ear is most
// sensitive to — the single biggest complaint with the old default was that the 660/880
// Hz sine pair sat right in the piercing range and the attack was a 12ms click.
type Note = { freq: number; start: number; duration: number; gain?: number };
type CueSpec = {
  notes: Note[];
  type?: OscillatorType;
  master?: number;
  /** Biquad low-pass cutoff (Hz). `null` disables the filter. Default 3500 Hz. */
  lowpass?: number | null;
  /** Exponential ramp time to peak, seconds. Default 0.012. Longer = softer onset. */
  attack?: number;
};
type Profile = Record<Cue, CueSpec>;

// Keep these keys in lockstep with the `SoundProfile` enum on the Rust side.
// Each is a complete four-cue family; picking a profile swaps all four at once
// so the acoustic feel is consistent between start-of-recording and success.
export type SoundProfileId = 'default' | 'classic' | 'subtle' | 'chime';

// Default — warm and satisfying, NOT piercing. Triangle waves, musical intervals,
// low-passed at 2.8 kHz, long exponential attacks. Start is an ascending perfect fifth
// (F4 → C5), stop is the same pair in reverse — feels like "opening" and "closing".
// Success is a clean major triad (F4–A4–C5) strummed top-down over 160ms. Error is a
// dampened low muted sawtooth with most of its brightness filtered out.
const DEFAULT_PROFILE: Profile = {
  start: {
    notes: [
      { freq: 349.2, start: 0,     duration: 0.10, gain: 0.32 },  // F4
      { freq: 523.3, start: 0.06,  duration: 0.14, gain: 0.30 },  // C5
    ],
    type: 'triangle',
    master: 0.20,
    lowpass: 2800,
    attack: 0.028,
  },
  stop: {
    notes: [
      { freq: 523.3, start: 0,     duration: 0.09, gain: 0.30 },  // C5
      { freq: 349.2, start: 0.06,  duration: 0.18, gain: 0.32 },  // F4
    ],
    type: 'triangle',
    master: 0.20,
    lowpass: 2800,
    attack: 0.028,
  },
  success: {
    notes: [
      { freq: 349.2, start: 0,     duration: 0.16, gain: 0.30 },  // F4
      { freq: 440.0, start: 0.06,  duration: 0.18, gain: 0.28 },  // A4
      { freq: 523.3, start: 0.12,  duration: 0.22, gain: 0.28 },  // C5
    ],
    type: 'triangle',
    master: 0.18,
    lowpass: 3000,
    attack: 0.030,
  },
  error: {
    notes: [
      { freq: 196.0, start: 0,     duration: 0.18, gain: 0.34 },  // G3
      { freq: 164.8, start: 0.14,  duration: 0.24, gain: 0.34 },  // E3
    ],
    type: 'triangle',
    master: 0.22,
    lowpass: 1800,
    attack: 0.035,
  },
};

// Tactile (keyed as "classic" for backwards compat with the Rust SoundProfile enum):
// premium, bell-like cues with a crisp mechanical feel. Think high-end keyboard click
// for start/stop and a satisfying two-note bell for success — the opposite of the old
// flat pager beeps this slot used to hold. Uses sine waves so the harmonics stay
// clean, with fast attack and long exponential tails for the "expensive" feel, and a
// 4.5 kHz lowpass so the top end never shrills.
const CLASSIC_PROFILE: Profile = {
  start: {
    // Quick perfect-fifth click pair — C5 → G5. 6 ms spacing makes it read as a single
    // "chk" event but the pitch rise gives it direction (opening).
    notes: [
      { freq: 523.3, start: 0,     duration: 0.08, gain: 0.42 },   // C5
      { freq: 784.0, start: 0.012, duration: 0.10, gain: 0.40 },   // G5
    ],
    type: 'sine',
    master: 0.26,
    lowpass: 4500,
    attack: 0.006,
  },
  stop: {
    // Inverse — G5 → C5, slightly longer tail so it feels like a confident "closed".
    notes: [
      { freq: 784.0, start: 0,     duration: 0.08, gain: 0.40 },   // G5
      { freq: 523.3, start: 0.012, duration: 0.14, gain: 0.42 },   // C5
    ],
    type: 'sine',
    master: 0.26,
    lowpass: 4000,
    attack: 0.008,
  },
  success: {
    // Two-note bell — E5 + C6 layered, sine waves, long exponential decay. Lands as a
    // soft "bing" that feels earned rather than triumphant.
    notes: [
      { freq: 659.3, start: 0,    duration: 0.42, gain: 0.32 },    // E5 — low partial
      { freq: 1046.5, start: 0.00, duration: 0.46, gain: 0.22 },   // C6 — shimmer on top
      { freq: 1567.9, start: 0.02, duration: 0.36, gain: 0.10 },   // G6 — subtle brightness
    ],
    type: 'sine',
    master: 0.24,
    lowpass: 5200,
    attack: 0.005,
  },
  error: {
    // Low muted descending third — reads as "no" without being harsh. A4 → F4, triangle
    // keeps it grounded, heavy lowpass strips any bite.
    notes: [
      { freq: 440.0, start: 0,    duration: 0.18, gain: 0.38 },    // A4
      { freq: 349.2, start: 0.14, duration: 0.28, gain: 0.38 },    // F4
    ],
    type: 'triangle',
    master: 0.22,
    lowpass: 1600,
    attack: 0.020,
  },
};

// Subtle: single, low, short sine blips — the "barely there" profile for users who
// want confirmation without anything melodic or bell-like. Previous master (0.15) was
// functionally muted next to the default profile (0.20–0.22); we want "quiet" not
// "can't hear it", so master is bumped to 0.38 and per-note peaks widened. Still well
// below the default's perceived loudness because it's single-note sines with no
// harmonic content.
const SUBTLE_PROFILE: Profile = {
  start: {
    notes: [{ freq: 440, start: 0, duration: 0.06, gain: 0.46 }],
    type: 'sine',
    master: 0.38,
    lowpass: 3200,
    attack: 0.010,
  },
  stop: {
    notes: [{ freq: 330, start: 0, duration: 0.08, gain: 0.46 }],
    type: 'sine',
    master: 0.38,
    lowpass: 3000,
    attack: 0.012,
  },
  success: {
    notes: [{ freq: 587, start: 0, duration: 0.10, gain: 0.50 }],
    type: 'sine',
    master: 0.38,
    lowpass: 3400,
    attack: 0.010,
  },
  error: {
    notes: [{ freq: 196, start: 0, duration: 0.16, gain: 0.58 }],
    type: 'sine',
    master: 0.42,
    lowpass: 1600,
    attack: 0.014,
  },
};

// Chime: triangle waves with longer decay, musical thirds — feels calmer and warmer
// than the default. Good for users who keep the app open all day and don't want
// a sharp tone every cycle.
const CHIME_PROFILE: Profile = {
  start: {
    notes: [
      { freq: 523, start: 0, duration: 0.22 },
      { freq: 659, start: 0.05, duration: 0.24 },
    ],
    type: 'triangle',
  },
  stop: {
    notes: [
      { freq: 659, start: 0, duration: 0.22 },
      { freq: 523, start: 0.05, duration: 0.26 },
    ],
    type: 'triangle',
  },
  success: {
    notes: [
      { freq: 659, start: 0, duration: 0.2 },
      { freq: 784, start: 0.08, duration: 0.24 },
      { freq: 988, start: 0.16, duration: 0.3 },
    ],
    type: 'triangle',
  },
  error: {
    notes: [
      { freq: 277, start: 0, duration: 0.22, gain: 0.4 },
      { freq: 233, start: 0.12, duration: 0.3, gain: 0.4 },
    ],
    type: 'triangle',
  },
};

const PROFILES: Record<SoundProfileId, Profile> = {
  default: DEFAULT_PROFILE,
  classic: CLASSIC_PROFILE,
  subtle: SUBTLE_PROFILE,
  chime: CHIME_PROFILE,
};

// User-facing profile metadata. Colocated with the sounds so the picker UI and
// preview button stay in sync with what each profile actually sounds like.
export const SOUND_PROFILES: { id: SoundProfileId; label: string; description: string }[] = [
  { id: 'default', label: 'Default', description: 'Warm triangle tones on a musical fifth — the soft, satisfying default.' },
  { id: 'classic', label: 'Tactile', description: 'Crisp mechanical clicks on start / stop, a soft bell on success.' },
  { id: 'subtle', label: 'Subtle', description: 'Quiet single-tone blips — minimal, no melody.' },
  { id: 'chime', label: 'Chime', description: 'Warm triangle-wave thirds with longer decay.' },
];

let ctx: AudioContext | null = null;
function audioCtx(): AudioContext | null {
  if (ctx) return ctx;
  try {
    const Ctor = window.AudioContext ?? (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    ctx = new Ctor();
    return ctx;
  } catch {
    return null;
  }
}

function play(profileId: SoundProfileId, cue: Cue) {
  const ac = audioCtx();
  if (!ac) return;
  const profile = PROFILES[profileId] ?? DEFAULT_PROFILE;
  const spec = profile[cue];

  // Chain: osc → per-note gain → (optional) lowpass → master gain → destination.
  // The lowpass is the biggest perceptual improvement — it rolls off the 2–8 kHz
  // "shrill" band most sine/triangle clicks concentrate their energy in, so the cue
  // still cuts through but lands on the ear as warm rather than piercing.
  const master = ac.createGain();
  master.gain.value = spec.master ?? 0.25;

  let outputNode: AudioNode = master;
  if (spec.lowpass !== null && spec.lowpass !== undefined) {
    const lp = ac.createBiquadFilter();
    lp.type = 'lowpass';
    lp.frequency.value = spec.lowpass;
    lp.Q.value = 0.707; // Butterworth — flat passband, no ringing.
    master.connect(lp);
    lp.connect(ac.destination);
    outputNode = master;
  } else {
    master.connect(ac.destination);
  }

  const attack = spec.attack ?? 0.012;
  const now = ac.currentTime;
  for (const note of spec.notes) {
    const osc = ac.createOscillator();
    osc.type = spec.type ?? 'sine';
    osc.frequency.value = note.freq;
    const g = ac.createGain();
    // Exponential attack-hold-release envelope. The attack ramp matters more than
    // the tone itself — a 12 ms attack clicks, a 28 ms attack lands as a soft swell.
    // Release is exponential from peak straight to near-zero across the remaining
    // note duration, which gives the triangle waves a warm, bell-like tail.
    const peak = note.gain ?? 0.6;
    g.gain.setValueAtTime(0.0001, now + note.start);
    g.gain.exponentialRampToValueAtTime(peak, now + note.start + attack);
    g.gain.exponentialRampToValueAtTime(0.0001, now + note.start + note.duration);
    osc.connect(g);
    g.connect(outputNode);
    osc.start(now + note.start);
    osc.stop(now + note.start + note.duration + 0.02);
  }
}

/**
 * Preview a profile by playing its `success` cue. Exposed so the picker UI can
 * preview on hover or on-select without wiring up the full phase subscription.
 */
export function previewProfile(profileId: SoundProfileId, cue: Cue = 'success') {
  play(profileId, cue);
}

/**
 * True for the `main` webview only — false for `floating` and `onboarding`.
 * Each window runs its own copy of `AppBootstrap`, and Tauri `listen()` subscribes
 * app-globally, so without this gate the `dictationPhase` event fires the cue in
 * every window simultaneously (3 tones per transition). Only `main` should emit.
 * Hash patterns: `#/` or `#/settings/*` → main, `#/floating` → floating,
 * `#/onboarding*` → onboarding.
 */
function isMainWindow(): boolean {
  const h = typeof window !== 'undefined' ? window.location.hash : '';
  return !h.startsWith('#/floating') && !h.startsWith('#/onboarding');
}

/** Subscribe to dictation phases and emit the matching sound cue when enabled. */
export function useSoundCues() {
  const enabled = useSettings((s) => s.data?.general.sound_cues_enabled ?? false);
  const profileId = useSettings(
    (s) => (s.data?.general as { sound_profile?: SoundProfileId } | undefined)?.sound_profile ?? 'default',
  );

  useEffect(() => {
    if (!enabled) return;
    if (!isMainWindow()) return;
    const p = listen<DictationPhase>(EVT.dictationPhase, (e) => {
      switch (e.payload.kind) {
        case 'listening':
          play(profileId, 'start');
          break;
        case 'processing':
          play(profileId, 'stop');
          break;
        case 'success':
          play(profileId, 'success');
          break;
        case 'error':
          play(profileId, 'error');
          break;
      }
    });
    return () => {
      void p.then((u) => u()).catch(() => undefined);
    };
  }, [enabled, profileId]);
}
