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
import type { DictationPhase } from '@/state/dictation';

export function StatePill({ phase }: { phase: DictationPhase }) {
  const { label, tone } = describe(phase);
  return (
    <span
      className={cn(
        'inline-flex cursor-default select-none items-center gap-1.5 rounded-full px-2 py-0.5',
        'font-mono text-[11px] font-medium uppercase tracking-[0.08em]',
        toneClass(tone),
      )}
    >
      <span className={cn('h-1.5 w-1.5 rounded-full', dotClass(tone))} />
      {label}
    </span>
  );
}

function describe(phase: DictationPhase) {
  switch (phase.kind) {
    case 'idle':
      return { label: 'Ready', tone: 'muted' as const };
    case 'warming':
      return { label: 'Initialising model', tone: 'accent' as const };
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
