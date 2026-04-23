import type { ReactNode } from 'react';

import { cn } from '@/lib/cn';

/**
 * Highlight — the React equivalent of the site's `<x-hl>` Blade component.
 * Wraps a word (or two) inside a heading in the brand cyan gradient
 * (`.gradient-cyan-text`, defined in globals.css under §10.3).
 *
 * Design system rules (DESIGN_SYSTEM.md §2.1, §1.5):
 *   • ONE highlight per heading. Never two in the same line.
 *   • AT MOST ONE highlighted heading per viewport. If Dashboard already
 *     spends its cyan budget on the hero metric card rail, the page title
 *     must read in plain --fg-hi.
 *   • Never use inside body copy — gradient type only reads right at
 *     display sizes (22px+). Use on h1 / h2 only.
 *
 * Because it inherits font settings from the parent heading, the caller
 * controls size and tracking. Highlight only paints.
 */
export function Highlight({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <span className={cn('gradient-cyan-text', className)}>{children}</span>;
}
