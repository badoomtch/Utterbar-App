import type { InputHTMLAttributes, TextareaHTMLAttributes } from 'react';
import { forwardRef } from 'react';

import { cn } from '@/lib/cn';

/**
 * Input — text field. Per DESIGN_SYSTEM.md §6.1 the input sits on bg-0
 * (one step BELOW the surrounding surface) so it reads as "type into this
 * hole" rather than "floating control". 10px radius, 2px cyan outline on
 * focus, hairline border at rest.
 *
 * The actual styling lives in globals.css (.ub-input) so the site and app
 * share one definition.
 */
export const Input = forwardRef<HTMLInputElement, InputHTMLAttributes<HTMLInputElement>>(
  function Input({ className, type = 'text', ...rest }, ref) {
    return <input ref={ref} type={type} className={cn('ub-input', className)} {...rest} />;
  },
);

/**
 * Textarea — same visual as Input but with variable height. Keep the
 * character-width cap modest; long-form text fields in a dense UI invite
 * paragraph-dumping, which the "one sentence of intent" editorial voice
 * does not want.
 */
export const Textarea = forwardRef<
  HTMLTextAreaElement,
  TextareaHTMLAttributes<HTMLTextAreaElement>
>(function Textarea({ className, rows = 3, ...rest }, ref) {
  return (
    <textarea
      ref={ref}
      rows={rows}
      className={cn('ub-input resize-y leading-[1.55]', className)}
      {...rest}
    />
  );
});
