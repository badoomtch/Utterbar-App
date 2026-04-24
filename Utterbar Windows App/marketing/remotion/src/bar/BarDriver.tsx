/**
 * BarDriver — the *only* thing Remotion compositions should render for
 * "a floating bar". It's a thin wrapper around the shipping app's
 * `src/routes/FloatingBar.tsx`, seeding the two zustand stores the real
 * component reads (`useDictation`, `useSettings`) with deterministic
 * values so the pill paints in a known phase before first paint.
 *
 * Why a wrapper at all: the real FloatingBar pulls everything from the
 * store — phase, elapsed timer, last-dictated text, size/outline/
 * visibility prefs — and Remotion compositions are expected to pass
 * `state="listening"` / `elapsedMs={…}` per-frame. The wrapper translates
 * those imperative props into a store update before mount, then renders
 * the real component unchanged.
 *
 * What we deliberately do NOT do here:
 *
 *   • Fork the pill. The FloatingBar import below is the shipping file,
 *     line-for-line. No "Remotion version".
 *   • Touch Tauri. All `invoke` / `getCurrentWindow` / `listen` calls
 *     that the real FloatingBar + settings store make are resolved to
 *     the inert stubs under `../stubs/` via webpack alias in
 *     `remotion.config.ts`. Nothing here needs to know about that.
 *   • Re-declare CSS. Theme classes (`.theme-obsidian` / `.accent-cyan`)
 *     are toggled on <html> here so the CSS var cascade from
 *     `src/styles/tokens.css` resolves to the values that paint the
 *     pill. Everything else — `.ub-bar-wave`, `.ub-bar-ripple`, the
 *     `animate-pulseDot` keyframe, gradient body, radius, shadow —
 *     comes from the shipping globals.css unchanged.
 *
 * If the app's FloatingBar changes, the next Remotion render reflects
 * the change. That's the whole point.
 */
import { useLayoutEffect } from 'react';
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
  /**
   * Dictation phase the pill should paint in. Maps to `DictationPhase.kind`
   * 1:1 — the store's shape is identical to what the real app emits from
   * the Rust side, so the pill can't tell it's being driven synthetically.
   */
  state: BarState;
  /**
   * Elapsed recording time in milliseconds. Only meaningful while
   * `state === 'listening'`; the real pill's mono timer reads from this.
   * Callers typically compute this from `useCurrentFrame() / fps * 1000`.
   */
  elapsedMs?: number;
  /**
   * Last dictated text — unlocks the "Copy last message" tile in the
   * expanded action row. Default `null` (tile is disabled, same as a
   * fresh app session before the user's first dictation).
   */
  lastText?: string | null;
  /**
   * Error message surfaced while `state === 'error'`. The real pill
   * doesn't render the message inline, but the phase kind needs a
   * `message` field to satisfy the discriminated union.
   */
  errorMessage?: string;
  /**
   * Theme to render under. Obsidian is the default and matches the
   * site's dark video section; Graphite / Paper are available for
   * future comps that need to show the pill under each theme.
   */
  theme?: Theme;
  /**
   * Accent to render under. Cyan by default — the whole brand lockup
   * cascades off this choice.
   */
  accent?: Accent;
  /**
   * Pill size (`'standard'` 52 px / `'compact'` 44 px).
   */
  barSize?: BarSize;
  /**
   * Pill outline ring. Matches the user's `general.bar_outline` pref —
   * most hero shots want this on.
   */
  barOutline?: boolean;
  /**
   * Window-show preference. Pinned to `'always'` for renders so the
   * real FloatingBar's visibility effect doesn't try to hide the
   * pill based on phase (which is how `while_recording` would react
   * in the live app).
   */
  barVisibility?: BarVisibility;
};

function buildPhase(
  state: BarState,
  elapsedMs: number,
  errorMessage: string,
): DictationPhase {
  switch (state) {
    case 'idle':
      return { kind: 'idle' };
    case 'warming':
      return { kind: 'warming', model_id: 'parakeet-ctc-0.6b' };
    case 'listening':
      // `started_at_ms` drives the timer via `Date.now() - started_at_ms`.
      // Back-date the start so the pill's timer reads the requested
      // elapsed duration on every frame.
      return { kind: 'listening', started_at_ms: Date.now() - elapsedMs };
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

/**
 * Build a minimal `Settings` object tuned to what the shipping
 * FloatingBar actually reads. The store is typed `Settings | null` so
 * the real component defensively falls back on `?? 'standard'` etc.; we
 * still hand it a full object so every consumer of `useSettings.data`
 * inside the component tree sees a consistent value.
 */
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
    // The floating bar doesn't read any of the nested non-general
    // sections — cast through unknown so we don't have to hand-build
    // every unrelated enum the generated type carries.
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
  // Seed the stores synchronously before first paint. useLayoutEffect
  // beats useEffect here — Remotion captures frame N after the first
  // paint, and we need the pill's store-driven state to be correct on
  // that first paint, not one frame late.
  useLayoutEffect(() => {
    useDictation.setState({
      phase: buildPhase(state, elapsedMs, errorMessage),
      elapsedMs,
      lastText,
    });
    useSettings.setState({
      data: buildSettings(theme, accent, barSize, barOutline, barVisibility),
    });
    // Toggle the theme + accent classes on <html> so every `var(--…)` in
    // the pill's CSS resolves off the correct cascade. The real app does
    // this via `applyThemeClasses(s)` after a settings load; we do the
    // minimal subset the pill visually depends on and skip the
    // side-effects (OS window theme, body zoom, a11y flags) which have
    // no meaning in a render context.
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
  }, [state, elapsedMs, lastText, errorMessage, theme, accent, barSize, barOutline, barVisibility]);

  return <AppFloatingBar />;
}
