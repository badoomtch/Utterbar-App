/**
 * StatePillStatic — a render-time replacement for the shipping StatePill.
 *
 * The shipping component (src/components/bar/StatePill.tsx) wraps the
 * chip in `<AnimatePresence mode="popLayout">` and keys it on
 * `phase.kind`. Every time the phase changes, the old chip exits while
 * the new one enters, with a layout-tween between them. In a normal
 * browser session that reads as a soft pop. Inside Remotion's
 * frame-by-frame headless render, the old + new chips are briefly
 * both mounted, which the eye reads as a sideways "fly-in".
 *
 * This module is aliased into the Remotion build via webpack-override.mjs
 * so any `import { StatePill } from '@/components/bar/StatePill'` inside
 * the FloatingBar tree resolves here. The shipping app is unaffected.
 *
 * Visual output is identical to the live chip on its resting frame —
 * same dot, same label, same tone classes — just without the wrapping
 * AnimatePresence + motion.span. Snap-changes between chips, no
 * mid-transition states.
 */
import { cn } from '@/lib/cn';
import { useDictation, type DictationPhase, type MeetingProgress } from '@/state/dictation';

export function StatePill({ phase }: { phase: DictationPhase }) {
  // Read meeting progress from the store the same way the shipping
  // StatePill does. BarDriver writes the slot for the
  // transcribing_background state; everything else gets null.
  const meetingProgress = useDictation((s) => s.meetingProgress);
  const { label, tone } = describe(phase, meetingProgress);
  // Show a thin progress track under the chip when the meeting is
  // mid-transcribe with known total chunks, mirroring the shipping
  // StatePill. Single-chunk meetings (no progress payload) and
  // every non-meeting phase fall through to the bare chip.
  const showProgress =
    phase.kind === 'transcribing_background' &&
    meetingProgress !== null &&
    meetingProgress.totalChunks > 0;
  const fraction = showProgress
    ? Math.min(
        1,
        Math.max(0, (meetingProgress.chunkIndex + 1) / meetingProgress.totalChunks),
      )
    : 0;
  return (
    <span
      className={cn(
        'relative inline-flex cursor-default select-none items-center gap-1.5 rounded-full px-2 py-0.5',
        'font-mono text-[11px] font-medium uppercase tracking-[0.08em]',
        toneClass(tone),
      )}
    >
      <span className={cn('h-1.5 w-1.5 rounded-full', dotClass(tone))} />
      {label}
      {showProgress && (
        <span
          aria-hidden
          className="pointer-events-none absolute inset-x-2 bottom-0.5 h-[1.5px] overflow-hidden rounded-full bg-[color-mix(in_oklab,var(--accent)_18%,transparent)]"
        >
          <span
            className="block h-full origin-left bg-accent"
            style={{ width: `${(fraction * 100).toFixed(1)}%` }}
          />
        </span>
      )}
    </span>
  );
}

function describe(phase: DictationPhase, meetingProgress: MeetingProgress | null) {
  switch (phase.kind) {
    case 'idle':
      return { label: 'Ready', tone: 'muted' as const };
    case 'warming':
      return { label: 'Initialising model', tone: 'accent' as const };
    case 'listening':
      return { label: 'Recording', tone: 'accent' as const };
    case 'processing':
      return { label: 'Processing', tone: 'accent' as const };
    case 'transcribing_background':
      // Mirror the shipping StatePill: render "Transcribing
      // meeting · N/M" when chunk progress is known, fall back
      // to the bare ellipsis copy otherwise (single-chunk
      // meetings and the brief moment between phase change and
      // first progress event arrival).
      if (meetingProgress && meetingProgress.totalChunks > 0) {
        const current = meetingProgress.chunkIndex + 1;
        return {
          label: `Transcribing meeting · ${current}/${meetingProgress.totalChunks}`,
          tone: 'accent' as const,
        };
      }
      return { label: 'Transcribing meeting…', tone: 'accent' as const };
    case 'success':
      return { label: 'Inserted', tone: 'success' as const };
    case 'error':
      return { label: 'Error', tone: 'error' as const };
    case 'cancelled':
      return { label: 'Cancelled', tone: 'muted' as const };
    case 'no_audio':
      return { label: 'No audio detected', tone: 'muted' as const };
  }
}

function toneClass(tone: 'muted' | 'accent' | 'success' | 'error') {
  switch (tone) {
    case 'accent':
      return 'bg-[color-mix(in_oklab,var(--accent)_16%,var(--pill-surface))] text-accent';
    case 'success':
      return 'bg-[color-mix(in_oklab,var(--moss)_16%,var(--pill-surface))] text-moss';
    case 'error':
      return 'bg-[color-mix(in_oklab,var(--rose)_16%,var(--pill-surface))] text-rose';
    default:
      return 'bg-[var(--pill-surface)] text-[var(--pill-fg-muted)]';
  }
}

function dotClass(tone: 'muted' | 'accent' | 'success' | 'error') {
  switch (tone) {
    case 'accent':
      return 'bg-accent';
    case 'success':
      return 'bg-moss';
    case 'error':
      return 'bg-rose';
    default:
      return 'bg-[var(--pill-fg-muted)]';
  }
}
