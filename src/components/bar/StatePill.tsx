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
          // Geist Mono, 9.5px, +0.1em tracking keeps the rhythm tight next
          // to the numeric timer that shares the pill.
          'inline-flex cursor-default select-none items-center gap-1.5 rounded-full px-2 py-0.5',
          'font-mono text-[9.5px] font-medium uppercase tracking-[0.1em]',
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

// Tone classes — direct-var Tailwind tokens, no more hsl() soup. The pill
// chrome lives on bg-bg-2 so the label feels recessed into the bar rather
// than floating on top of it.
function toneClass(tone: 'muted' | 'accent' | 'success' | 'error') {
  switch (tone) {
    case 'accent':
      // Accent tone uses a cyan-tinted surface via color-mix so it reads as
      // "lit" without burning a full bg-accent block (that would spike the
      // cyan-count budget for the whole viewport).
      return 'bg-[color-mix(in_oklab,var(--accent)_14%,var(--bg-2))] text-accent';
    case 'success':
      return 'bg-[color-mix(in_oklab,var(--moss)_14%,var(--bg-2))] text-moss';
    case 'error':
      return 'bg-[color-mix(in_oklab,var(--rose)_14%,var(--bg-2))] text-rose';
    default:
      return 'bg-bg-2 text-muted';
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
      return 'bg-fg-dim/70';
  }
}
