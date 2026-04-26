import { cn } from '@/lib/cn';

/**
 * On/off toggle. Single canonical implementation — every Toggle in the
 * app uses this component. If a switch ever looks "off" elsewhere, the
 * fix lives here, not in a one-off override.
 *
 * Geometry (audited 2026-04-27, fourth pass — see history at the bottom
 * of this comment for the full saga). Numbers chosen so the on/off knob
 * positions are mirror-symmetric AND the visible track shape stays the
 * same in both states:
 *
 *   Track       w-11 h-6      → 44×24 outer (border-box)
 *   Border      1px each side → 42×22 content box, present in BOTH states
 *   Knob        h-[18px] w-[18px]
 *   Inset       2px each side from the content box
 *   Travel      content_w - knob - 2*inset = 42 - 18 - 4 = 20px
 *
 *   OFF      translate-x-[2px]   → 2px inset from left
 *   ON       translate-x-[22px]  → 22+18 = 40, 2px from right
 *
 * The previous version used `border-transparent` for the ON state, which
 * was mathematically symmetric but perceptually asymmetric: a transparent
 * border meant the cyan fill bled to the OUTER edge of the track, so the
 * ON knob's 2px right gap looked like it was sitting on the very rim
 * while the OFF knob's 2px left gap had a visible grey border to butt
 * against. Fix: keep a same-coloured (accent) border in the ON state so
 * the visible track terminates at the same place in both states.
 *
 * Focus ring: 2px cyan outline per DESIGN_SYSTEM §6.1, no glow.
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
        'relative inline-flex h-6 w-11 shrink-0 items-center rounded-pill border transition-colors duration-180',
        'focus-visible:outline-2 focus-visible:outline focus-visible:outline-accent focus-visible:outline-offset-2',
        checked
          ? 'bg-accent border-accent'
          : 'bg-bg-2 border-border-strong hover:border-accent/40',
        disabled && 'cursor-not-allowed opacity-50',
      )}
    >
      <span
        className={cn(
          'pointer-events-none inline-block h-[18px] w-[18px] rounded-full transition-all duration-180',
          checked
            ? 'translate-x-[22px] bg-[#06121A] shadow-[0_1px_2px_rgba(0,0,0,0.35)]'
            : 'translate-x-[2px] bg-fg-dim',
        )}
      />
    </button>
  );
}
