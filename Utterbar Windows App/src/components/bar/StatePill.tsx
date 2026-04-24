import { motion, AnimatePresence } from 'framer-motion';
import { cn } from '@/lib/cn';
import { springs } from '@/design/motion';
import type { DictationPhase } from '@/state/dictation';

/**
 * StatePill — the dictation-phase read inside the floating bar.
 *
 * Per DESIGN_SYSTEM.md §8 the pill is the signature surface. The state
 * label rides on it as a quiet chip:
 *
 *   • Idle / warming / cancelled — neutral dot + muted text on bg-2.
 *   • Recording / processing     — cyan LED (pulse for recording) + accent
 *                                  text. The cyan-count budget allows this
 *                                  because the rest of the bar is dark.
 *   • Inserted (success)         — moss dot + moss text.
 *   • Error                      — rose dot + rose text.
 *
 * The label uses Geist Mono (§2.4: mono reserved for timestamps, hash IDs,
 * and state reads — exactly this).
 */
export function StatePill({ phase }: { phase: DictationPhase }) {
  const { label, tone } = describe(phase);
  return (
    <AnimatePresence mode="popLayout" initial={false}>
      <motion.span
        key={phase.kind}
        layout
        initial={{ opacity: 0, y: 4, scale: 0.96 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: -4, scale: 0.96 }}
        transition={springs.pop}
        className={cn(
          // Geist Mono. Bumped from 9.5px to 11px — the old size read as
          // decorative ornament; 'Ready' was barely legible on the floating
          // bar's gradient, which defeated the whole point of surfacing the
          // idle state. 11px + 0.08em tracking keeps the chip compact while
          // actually being readable at a glance.
          'inline-flex cursor-default select-none items-center gap-1.5 rounded-full px-2 py-0.5',
          'font-mono text-[11px] font-medium uppercase tracking-[0.08em]',
          toneClass(tone),
        )}
      >
        <span className={cn('h-1.5 w-1.5 rounded-full', dotClass(tone))} />
        {label}
      </motion.span>
    </AnimatePresence>
  );
}

function describe(phase: DictationPhase) {
  switch (phase.kind) {
    case 'idle':
      return { label: 'Ready', tone: 'muted' as const };
    case 'warming':
      return { label: 'Warming', tone: 'muted' as const };
    case 'listening':
      return { label: 'Recording', tone: 'accent' as const };
    case 'processing':
      return { label: 'Processing', tone: 'accent' as const };
    case 'success':
      return { label: 'Inserted', tone: 'success' as const };
    case 'error':
      return { label: 'Error', tone: 'error' as const };
    case 'cancelled':
      return { label: 'Cancelled', tone: 'muted' as const };
  }
}

// Tone classes. StatePill only appears inside the floating bar, so every
// tone mixes its accent colour with the pill-local surface (`--pill-surface`)
// rather than the app-shell `--bg-2`. That was the 'Ready' bug on Paper: a
// dark-grey chip on a cream pill looked like a paste error. Mixing against
// `--pill-surface` flips correctly per theme (dark tile on dark pill, warm
// tile on Paper's light pill) and keeps the chip recessed into the bar
// instead of floating on top.
function toneClass(tone: 'muted' | 'accent' | 'success' | 'error') {
  switch (tone) {
    case 'accent':
      return 'bg-[color-mix(in_oklab,var(--accent)_16%,var(--pill-surface))] text-accent';
    case 'success':
      return 'bg-[color-mix(in_oklab,var(--moss)_16%,var(--pill-surface))] text-moss';
    case 'error':
      return 'bg-[color-mix(in_oklab,var(--rose)_16%,var(--pill-surface))] text-rose';
    default:
      // Muted 'Ready' uses the pill-local surface + muted ink tokens so it
      // reads cleanly on both dark and Paper pills without hand-tuning.
      return 'bg-[var(--pill-surface)] text-[var(--pill-fg-muted)]';
  }
}

function dotClass(tone: 'muted' | 'accent' | 'success' | 'error') {
  switch (tone) {
    case 'accent':
      return 'bg-accent animate-pulseDot';
    case 'success':
      return 'bg-moss';
    case 'error':
      return 'bg-rose';
    default:
      // Matches the muted-text colour of the surrounding chip so the dot
      // doesn't out-contrast the label on Paper (where `fg-dim` is the
      // app-shell token, not the pill-local one).
      return 'bg-[var(--pill-fg-muted)]';
  }
}
