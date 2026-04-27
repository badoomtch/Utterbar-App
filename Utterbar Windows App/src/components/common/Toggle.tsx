import { cn } from '@/lib/cn';

/**
 * On/off toggle. Single canonical implementation — every Toggle in the
 * app uses this component. If a switch ever looks "off" elsewhere, the
 * fix lives here, not in a one-off override.
 *
 * Geometry — fifth pass, 2026-04-27. Previous attempts used Tailwind
 * `translate-x-[Npx]` against a flex parent and kept producing the same
 * visual bug: the ON-state knob protruded past the right edge of the
 * cyan track. The arithmetic looked right on paper but something in the
 * flex/translate/border-box stack was offsetting the child by ~2px more
 * than expected at runtime. Rather than keep guessing, this version
 * uses absolute positioning with explicit `left` — pixel-perfect, no
 * flexbox interaction, no translate, can't drift.
 *
 * Track:        44 × 24, 1px border (border-box), 22px radius (full pill)
 * Inner usable: 42 × 22 (track minus border)
 * Knob:         18 × 18, absolute-positioned within the track
 * OFF position: left: 2px              → knob spans 2 → 20 (within 0..42)
 * ON  position: left: calc(100% - 20px) → knob spans 22 → 40 (within 0..42)
 *
 * Both states leave a 2px gap from the inner border. Both states keep
 * a same-coloured 1px border so the visible track ends in the same
 * place regardless of state. Vertical: knob is 18, inner height is 22,
 * so 2px above/below — top: 2px does it.
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
        'relative inline-block shrink-0 rounded-pill border transition-colors duration-180',
        'focus-visible:outline-2 focus-visible:outline focus-visible:outline-accent focus-visible:outline-offset-2',
        checked
          ? 'bg-accent border-accent'
          : 'bg-bg-2 border-border-strong hover:border-accent/40',
        disabled && 'cursor-not-allowed opacity-50',
      )}
      style={{ width: 44, height: 24, boxSizing: 'border-box' }}
    >
      <span
        aria-hidden="true"
        className={cn(
          'absolute pointer-events-none rounded-full transition-all duration-180',
          checked ? 'bg-[#06121A] shadow-[0_1px_2px_rgba(0,0,0,0.35)]' : 'bg-fg-dim',
        )}
        style={{
          width: 18,
          height: 18,
          top: 2,
          // Position from the left in OFF state, from the right in ON
          // state. Using `right` for ON so the gap-from-edge matches OFF
          // exactly, regardless of how the browser resolves border-box.
          left: checked ? 'auto' : 2,
          right: checked ? 2 : 'auto',
        }}
      />
    </button>
  );
}
