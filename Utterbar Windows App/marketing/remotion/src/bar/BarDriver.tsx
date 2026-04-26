/**
 * BarDriver — render the shipping FloatingBar inside Remotion with
 * every wall-clock animation source neutralised.
 *
 * Lessons from the 2026-04-24/25 jitter saga: trying to re-plumb
 * time inside the render tab (patch Date.now / RAF / setTimeout so
 * Framer Motion runs against the Remotion frame clock) produces
 * worse jitter than just snapping. Remotion captures each frame
 * before any of those queues drain, so motion components end up at
 * a different mid-tween position every frame. The right answer is
 * boring and works:
 *
 *   1. Force every Framer Motion descendant to use reducedMotion =
 *      "always" via <MotionConfig>. That snaps `motion.*` between
 *      from/to states with no per-frame interpolation.
 *   2. Disable CSS keyframes / transitions inside the pill via the
 *      `html.remotion-render` class (rules live in app.css).
 *   3. Drive the few visual cues that absolutely need to *appear*
 *      animated (waveform bar heights, recording ripple) from
 *      `useCurrentFrame()` with a pure sinusoid, applied as inline
 *      CSS custom properties.
 *
 * No `Date.now` patches. No fake schedulers. The render is fully
 * deterministic frame-to-frame because nothing reads a clock.
 */
import { useLayoutEffect } from 'react';
import { MotionConfig, MotionGlobalConfig } from 'framer-motion';
import { useCurrentFrame, useVideoConfig } from 'remotion';

// Framer Motion's global kill-switch. When true, every motion.* component
// in the entire app skips its animation and applies the target value
// instantly (the library forces duration=0 + delay=0 internally). This
// is the only thing that reliably silences the outer pill's `layout`
// width tween + the StatePill's `AnimatePresence mode="popLayout"`
// transition during headless render. `MotionConfig reducedMotion="always"`
// alone doesn't catch all of these — it works for transform/opacity but
// not for layout measurements or popLayout sequences. The global flag
// does. Set once at module load so every Remotion frame inherits it.
MotionGlobalConfig.skipAnimations = true;
import { FloatingBar as AppFloatingBar } from '@/routes/FloatingBar';
import { useDictation, type DictationPhase } from '@/state/dictation';
import { useSettings } from '@/state/settings';
import type { Settings } from '@/generated/Settings';
import type { Theme } from '@/generated/Theme';
import type { Accent } from '@/generated/Accent';
import type { BarSize } from '@/generated/BarSize';
import type { BarVisibility } from '@/generated/BarVisibility';

export type BarState =
  | 'idle'
  | 'warming'
  | 'listening'
  | 'processing'
  | 'success'
  | 'error'
  | 'cancelled';

export type BarDriverProps = {
  state: BarState;
  /** Elapsed recording time in ms; only meaningful while listening. */
  elapsedMs?: number;
  lastText?: string | null;
  errorMessage?: string;
  theme?: Theme;
  accent?: Accent;
  barSize?: BarSize;
  barOutline?: boolean;
  barVisibility?: BarVisibility;
};

/**
 * Build the phase for the dictation store. `started_at_ms` is irrelevant
 * here — we explicitly write `elapsedMs` directly into the store every
 * frame, AND we override `tickElapsed` to a no-op so the FloatingBar's
 * `setInterval(tick, 100)` can't overwrite it with whatever the wall
 * clock says. The pill renders `formatDuration(useDictation.elapsedMs)`,
 * so what we set is what gets displayed. End of story.
 */
function buildPhase(state: BarState, errorMessage: string): DictationPhase {
  switch (state) {
    case 'idle':
      return { kind: 'idle' };
    case 'warming':
      return { kind: 'warming', model_id: 'parakeet-ctc-0.6b' };
    case 'listening':
      return { kind: 'listening', started_at_ms: 0 };
    case 'processing':
      return { kind: 'processing' };
    case 'success':
      return { kind: 'success' };
    case 'error':
      return { kind: 'error', message: errorMessage };
    case 'cancelled':
      return { kind: 'cancelled' };
  }
}

function buildSettings(
  theme: Theme,
  accent: Accent,
  barSize: BarSize,
  barOutline: boolean,
  barVisibility: BarVisibility,
): Settings {
  return {
    schema_version: 1,
    onboarding_completed: true,
    general: {
      launch_on_login: false,
      start_minimised: false,
      close_to_tray: true,
      check_updates_automatically: false,
      theme,
      accent,
      reduce_motion: false,
      sound_cues_enabled: false,
      sound_profile: 'default' as unknown as Settings['general']['sound_profile'],
      ui_scale: 100,
      high_contrast: false,
      larger_focus_rings: false,
      reduce_transparency: false,
      notifications_enabled: false,
      bar_size: barSize,
      bar_visibility: barVisibility,
      bar_outline: barOutline,
      releases_email: null,
      custom_accent_hex: null,
    },
    audio: {} as unknown as Settings['audio'],
    models: {} as unknown as Settings['models'],
    text_output: {} as unknown as Settings['text_output'],
    privacy: {} as unknown as Settings['privacy'],
    advanced: {} as unknown as Settings['advanced'],
    shortcuts: {} as unknown as Settings['shortcuts'],
    typing_baseline_wpm: 40,
    active_mode_id: null,
  };
}

/**
 * Frame-derived waveform bar heights.
 *
 * The shipping pill's `.ub-bar-wave__bar` elements normally animate via
 * CSS keyframes. With `html.remotion-render` set, app.css disables the
 * keyframe and reads each bar's height from `--ub-wave-h-N`. We compute
 * those values here per frame as a smooth, deterministic sinusoid: same
 * "speech wave" feel, identical on every render of frame N.
 *
 * Period = 1.4 s. Per-bar phase offset = 0.14 s. Range tuned to match
 * what the keyframe naturally produces under playback.
 */
function computeWaveHeights(frame: number, fps: number, active: boolean): Record<string, string> {
  if (!active) {
    return {
      '--ub-wave-h-1': '10px',
      '--ub-wave-h-2': '14px',
      '--ub-wave-h-3': '22px',
      '--ub-wave-h-4': '26px',
      '--ub-wave-h-5': '22px',
      '--ub-wave-h-6': '14px',
      '--ub-wave-h-7': '10px',
    };
  }
  const heights: Record<string, string> = {};
  const seconds = frame / fps;
  const period = 1.4;
  const peaks = [14, 18, 24, 28, 24, 18, 14];
  const troughs = [8, 10, 12, 14, 12, 10, 8];
  for (let i = 0; i < 7; i++) {
    const phase = i * 0.14;
    const x = ((seconds + phase) * (Math.PI * 2)) / period;
    const wave = 0.5 + 0.5 * Math.sin(x);
    const h = troughs[i] + (peaks[i] - troughs[i]) * wave;
    heights[`--ub-wave-h-${i + 1}`] = `${h.toFixed(2)}px`;
  }
  return heights;
}

/**
 * Frame-derived recording ripple intensity. The shipping pill renders
 * a red recording dot with a `::after` pseudo-element that pulses via
 * `@keyframes ub-bar-ripple`. We disable the keyframe via app.css and
 * substitute a frame-derived `--ub-ripple-scale` + `--ub-ripple-opacity`
 * pair so the dot still pulses smoothly without a wall clock.
 */
function computeRippleVars(frame: number, fps: number, active: boolean): Record<string, string> {
  if (!active) {
    return { '--ub-ripple-scale': '1', '--ub-ripple-opacity': '0' };
  }
  const seconds = frame / fps;
  const period = 1.5; // matches the original keyframe period
  const t = (seconds % period) / period;
  const scale = 1 + 1.4 * t;
  const opacity = 0.55 * (1 - t);
  return {
    '--ub-ripple-scale': scale.toFixed(3),
    '--ub-ripple-opacity': opacity.toFixed(3),
  };
}

/**
 * Pin Date.now() to a frame-derived value during render.
 *
 * Why: the shipping FloatingBar reads Date.now() in two state-gating
 * paths that aren't covered by overriding `useDictation.tickElapsed`:
 *   1. `terminalUntil = Date.now() + 1400` when the phase becomes terminal
 *   2. `now = useState(Date.now())` + setTimeout that re-reads it
 *
 * Both gate `expanded`, the boolean that controls whether the pill is
 * collapsed or expanded. If real wall-clock drifts between Remotion
 * frame captures, `expanded` flips true/false at random and the bar
 * "zooms in/out" jitterily.
 *
 * We anchor Date.now to a deterministic frame-derived value so every
 * read inside the render tree is identical for frame N. Nothing else
 * is patched — Framer Motion's RAF-based animations are silenced by
 * MotionConfig reducedMotion="always" (set further down).
 */
function pinDateNow(frameMs: number) {
  const w = window as typeof window & {
    __ubRemotionDateNow?: typeof Date.now;
    __ubRemotionFrameMs?: number;
  };
  if (!w.__ubRemotionDateNow) {
    w.__ubRemotionDateNow = Date.now.bind(Date);
    Date.now = () => 1_700_000_000_000 + (w.__ubRemotionFrameMs ?? 0);
  }
  w.__ubRemotionFrameMs = frameMs;
}

/**
 * Override `useDictation.tickElapsed` to a no-op so the FloatingBar's
 * `setInterval(tick, 100)` can't overwrite `elapsedMs` with a wall-
 * clock-derived value. We write `elapsedMs` ourselves, frame by frame,
 * and `formatDuration(elapsedMs)` produces the timer string the pill
 * displays. Nothing else needs to know what time it is.
 */
function neutraliseTick() {
  const w = window as typeof window & { __ubRemotionTickPatched?: boolean };
  if (w.__ubRemotionTickPatched) return;
  // Replace tickElapsed in the store with a no-op. zustand stores actions
  // alongside state; setState(merge) accepts both. Cast through unknown
  // because the type bag treats actions as readonly.
  useDictation.setState({
    tickElapsed: () => {
      /* render-time no-op — elapsedMs is owned by BarDriver. */
    },
  } as unknown as Partial<ReturnType<typeof useDictation.getState>>);
  w.__ubRemotionTickPatched = true;
}

export function BarDriver({
  state,
  elapsedMs = 0,
  lastText = null,
  errorMessage = 'Something went wrong',
  theme = 'obsidian' as Theme,
  accent = 'cyan' as Accent,
  barSize = 'standard' as BarSize,
  barOutline = true,
  barVisibility = 'always' as BarVisibility,
}: BarDriverProps) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const frameMs = (frame * 1000) / fps;
  if (typeof window !== 'undefined') {
    pinDateNow(frameMs);
    neutraliseTick();
  }
  const waveActive = state === 'listening' || state === 'processing' || state === 'warming';
  const waveVars = computeWaveHeights(frame, fps, waveActive);
  const rippleVars = computeRippleVars(frame, fps, state === 'listening');

  // Seed the stores synchronously before first paint. useLayoutEffect
  // beats useEffect — Remotion captures the screenshot after the first
  // paint, and we want the pill in the right phase for that paint.
  useLayoutEffect(() => {
    useDictation.setState({
      phase: buildPhase(state, errorMessage),
      // elapsedMs is the timer value the pill displays. tickElapsed has
      // been replaced with a no-op (see neutraliseTick) so this value
      // sticks for the full frame, no wall-clock interference.
      elapsedMs,
      lastText,
    });
    useSettings.setState({
      data: buildSettings(theme, accent, barSize, barOutline, barVisibility),
    });

    const root = document.documentElement;
    root.classList.remove('theme-obsidian', 'theme-graphite', 'theme-paper');
    root.classList.add(`theme-${theme}`);
    root.classList.remove(
      'accent-teal',
      'accent-amber',
      'accent-violet',
      'accent-rose',
      'accent-cyan',
      'accent-lime',
      'accent-coral',
      'accent-custom',
    );
    root.classList.add(`accent-${accent}`);
    if (theme === ('paper' as Theme)) root.classList.remove('dark');
    else root.classList.add('dark');
    // app.css keys its render-only freeze rules off this class.
    root.classList.add('remotion-render');
  }, [state, elapsedMs, lastText, errorMessage, theme, accent, barSize, barOutline, barVisibility]);

  // Pin per-frame CSS custom properties on <html>. <html> persists across
  // Remotion's per-frame re-mounts, so without an explicit set/unset every
  // frame N would inherit frame N-1's values.
  useLayoutEffect(() => {
    const root = document.documentElement;
    for (const [k, v] of Object.entries(waveVars)) root.style.setProperty(k, v);
    for (const [k, v] of Object.entries(rippleVars)) root.style.setProperty(k, v);
  }, [waveVars, rippleVars]);

  // MotionConfig with reducedMotion="always" snaps every motion.* descendant
  // between states instantly. That's exactly what we want for video: no
  // mid-tween partial frames, no wall-clock dependency, no Framer Motion
  // jitter. Spring overshoots are sacrificed; smoothness is gained.
  return (
    <MotionConfig reducedMotion="always">
      <AppFloatingBar />
    </MotionConfig>
  );
}
