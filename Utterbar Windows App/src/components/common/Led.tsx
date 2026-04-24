import { cn } from '@/lib/cn';

/**
 * Led — the cyan LED dot. Per DESIGN_SYSTEM.md §8.1 this primitive
 * communicates "live / ready / active / connected / recording":
 *
 *   • On a Ghost button (handled by the ::before on .ub-btn-ghost)
 *   • As a status indicator next to labels ("Recording", "Mic active")
 *   • On the floating pill
 *
 * Variants:
 *   • `calm`   — static glow (default). Use for "connected / ready".
 *   • `pulse`  — slow breath. Use for "idle / warm" states.
 *   • `record` — brighter, faster pulse. Use during active recording.
 *   • `error`  — rose-tinted variant. ONLY surface; the pill itself
 *                uses it during fatal ASR errors.
 *
 * Size defaults to 7px to match the button LED; pass 6px for inline
 * label-adjacent use (the 1px gap against body text at 14px is perfect).
 */
export function Led({
  variant = 'calm',
  size = 7,
  className,
}: {
  variant?: 'calm' | 'pulse' | 'record' | 'error';
  size?: number;
  className?: string;
}) {
  const isError = variant === 'error';

  return (
    <span
      aria-hidden
      className={cn(
        'inline-block rounded-full',
        isError ? 'bg-rose' : 'bg-accent',
        variant === 'pulse' && 'animate-pulseDot',
        // Record is just a faster pulse — same keyframes, tighter duration.
        variant === 'record' && 'animate-pulseDot [animation-duration:900ms]',
        className,
      )}
      style={{
        width: size,
        height: size,
        boxShadow: isError
          ? '0 0 10px var(--rose), 0 0 3px color-mix(in oklab, var(--rose) 90%, #fff)'
          : '0 0 10px var(--accent), 0 0 3px color-mix(in oklab, var(--accent) 90%, #fff)',
      }}
    />
  );
}
