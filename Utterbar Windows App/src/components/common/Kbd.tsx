import type { ReactNode } from 'react';

import { cn } from '@/lib/cn';

/**
 * Kbd — keyboard-key chip. Per DESIGN_SYSTEM.md §6.2 the chip gets a
 * thicker bottom border (2px) so it reads as a physical key rather than
 * a generic pill. Mono only — §6.2 is the one place in the app where
 * mono-for-decoration is permitted, because the text IS a literal key label.
 *
 * Use for: shortcut displays (Ctrl+Space), inline help ("press F6"),
 * keybind editors. NEVER for generic small labels — that's what .ub-chip
 * is for.
 */
export function Kbd({ children, className }: { children: ReactNode; className?: string }) {
  return <kbd className={cn('ub-kbd', className)}>{children}</kbd>;
}

/**
 * KbdSequence — renders a shortcut accelerator string ("Ctrl+Alt+Space")
 * as a sequence of individual Kbd chips joined by "+" glyphs. Keeps the
 * visual rhythm consistent across every shortcut surface in the app.
 */
export function KbdSequence({ accelerator, className }: { accelerator: string; className?: string }) {
  const parts = accelerator.split('+').map((p) => p.trim()).filter(Boolean);
  return (
    <span className={cn('inline-flex items-center gap-1 text-muted text-[12px]', className)}>
      {parts.map((p, i) => (
        <span key={`${p}-${i}`} className="inline-flex items-center gap-1">
          {i > 0 && <span aria-hidden className="text-fg-dim">+</span>}
          <Kbd>{p}</Kbd>
        </span>
      ))}
    </span>
  );
}
