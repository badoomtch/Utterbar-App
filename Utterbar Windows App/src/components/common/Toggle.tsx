import { cn } from '@/lib/cn';

/**
 * On/off toggle.
 *
 * Visual language matches the spec's "at rest vs lit" button dialectic
 * applied to a switch:
 *   • Off — hollow track on bg-2, quiet border, knob sits left.
 *   • On  — cyan accent fill, knob sits right with a soft shadow.
 *
 * No glow ring on focus — a 2px cyan outline per §6.1 is sufficient.
 */
export function Toggle({
  checked,
  onChange,
  label,
  disabled,
}: {
  checked: boolean;
  onChange: (next: boolean) => void;
  label?: string;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => !disabled && onChange(!checked)}
      className={cn(
        'relative inline-flex h-6 w-10 items-center rounded-pill border transition-colors duration-180',
        'focus-visible:outline-2 focus-visible:outline focus-visible:outline-accent focus-visible:outline-offset-2',
        checked
          ? 'bg-accent border-transparent'
          : 'bg-bg-2 border-border-strong hover:border-accent/40',
        disabled && 'cursor-not-allowed opacity-50',
      )}
    >
      <span
        className={cn(
          'pointer-events-none inline-block h-4 w-4 rounded-full transition-all duration-180',
          checked
            ? 'translate-x-[21px] bg-[#06121A] shadow-[0_1px_2px_rgba(0,0,0,0.35)]'
            : 'translate-x-[3px] bg-fg-dim',
        )}
      />
    </button>
  );
}
