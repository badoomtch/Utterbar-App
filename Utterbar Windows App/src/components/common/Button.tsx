import type { ButtonHTMLAttributes, ReactNode } from 'react';
import { forwardRef } from 'react';

import { cn } from '@/lib/cn';

/**
 * Button — the pill. Per DESIGN_SYSTEM.md §5 there are only two first-class
 * variants and they tell the same story in two states:
 *
 *   • `cta`   — fully lit pill (cyan fill, dark "LED-off" dot).
 *               Maximum visual weight. One per page.
 *   • `ghost` — at-rest pill (dark fill, glowing cyan LED dot).
 *               Secondary / quieter alternative.
 *
 * Plus two escape hatches for contexts the spec doesn't cover:
 *
 *   • `plain`  — neutral hairline fallback. Tertiary actions, toolbars.
 *                Prefer ghost in doubt.
 *   • `danger` — destructive variant. Irreversible actions (delete,
 *                remove, reset). Rose fill, same geometry as CTA.
 *
 * The actual colour language lives in globals.css (.ub-btn-*) so both the
 * app and the marketing site stay byte-identical. This component just
 * composes the right classes and adds a size hook for in-app toolbars
 * where the default size would be too loud.
 */
export type ButtonVariant = 'cta' | 'ghost' | 'plain' | 'danger';
export type ButtonSize = 'sm' | 'md';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Force-render without the LED dot (CTA / Ghost). Used when stacking
   *  inside a tight row where two dots would read as decoration. */
  noLed?: boolean;
  children?: ReactNode;
}

const VARIANT_CLASS: Record<ButtonVariant, string> = {
  cta:    'ub-btn ub-btn-cta',
  ghost:  'ub-btn ub-btn-ghost',
  plain:  'ub-btn ub-btn-plain',
  danger: 'ub-btn ub-btn-danger',
};

const SIZE_CLASS: Record<ButtonSize, string> = {
  // Compact — used for inline toolbar actions. Trims vertical padding
  // and bumps type down one step.
  sm: 'text-[12px] py-[0.4rem]',
  md: '',
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = 'ghost', size = 'md', noLed, className, children, type, ...rest },
  ref,
) {
  return (
    <button
      ref={ref}
      type={type ?? 'button'}
      className={cn(
        VARIANT_CLASS[variant],
        SIZE_CLASS[size],
        // When the LED dot is suppressed we also strip the left padding
        // that made space for it — otherwise the label looks adrift.
        noLed && '!pl-[1.15rem] !pr-[1.15rem] [&::before]:hidden',
        className,
      )}
      {...rest}
    >
      {children}
    </button>
  );
});
