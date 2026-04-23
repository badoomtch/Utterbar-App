import type { ReactNode } from 'react';
import { cn } from '@/lib/cn';

/**
 * Card — the canonical panel. Per DESIGN_SYSTEM.md §4, cards are elevated
 * via background colour (bg-1), not border or shadow. Adding a 1px outline
 * or a drop-shadow explicitly fails design review.
 *
 * Props:
 *   • `eyebrow` — optional uppercase label above the title (Instrument Sans
 *     500 at 11px, +0.14em tracking, `--muted`). NEVER mono — the spec
 *     retired mono-for-decoration in §2.4.
 *   • `title` / `description` — standard group header.
 *   • `rail` — when true, applies the 2px cyan left-rail signature motif
 *     (§4.5). Reserve for "featured / selected / important" cards. Do NOT
 *     put a rail on every card — the rail is signal, not atmosphere.
 *   • `tone` — `'raised'` lifts to bg-2, used for cards that are themselves
 *     the "lit" / featured tile of a page (e.g. the Pro-equivalent in a
 *     pair). Default `'default'` is bg-1.
 */
export function Card({
  children,
  className,
  title,
  description,
  eyebrow,
  rail = false,
  tone = 'default',
}: {
  children: ReactNode;
  className?: string;
  title?: string;
  description?: string;
  eyebrow?: string;
  rail?: boolean;
  tone?: 'default' | 'raised';
}) {
  return (
    <section
      className={cn(
        'rounded-card p-6',
        tone === 'raised' ? 'bg-bg-2' : 'bg-bg-1',
        rail && 'ub-card-rail',
        className,
      )}
    >
      {(title || description || eyebrow) && (
        <header className="mb-5">
          {eyebrow && <div className="ub-eyebrow mb-2">{eyebrow}</div>}
          {title && (
            // Weight 700 (H3 tier per §2.3). Weight 600 is forbidden.
            <h3 className="text-[17px] font-bold text-fg-hi -tracking-[0.02em] leading-tight">
              {title}
            </h3>
          )}
          {description && (
            <p className="mt-1.5 text-[13px] leading-relaxed text-muted max-w-[60ch]">
              {description}
            </p>
          )}
        </header>
      )}
      {children}
    </section>
  );
}

/**
 * Row — a label + control pair inside a Card. Separated by a single
 * hairline (§4.3); the first row drops its top border so the card header
 * isn't double-lined.
 */
export function Row({
  label,
  description,
  children,
  align = 'center',
}: {
  label: string;
  description?: ReactNode;
  children: ReactNode;
  /** Control alignment. Most rows want `center`; long-form controls
   *  (sliders, multi-line inputs) read better with `start`. */
  align?: 'center' | 'start';
}) {
  return (
    <div
      className={cn(
        'flex gap-6 border-t border-border-soft/70 py-4 first:border-t-0 first:pt-0',
        align === 'center' ? 'items-center' : 'items-start',
      )}
    >
      <div className="flex-1 min-w-0">
        <div className="text-[13px] font-medium text-fg-hi">{label}</div>
        {description && (
          <div className="mt-1 text-[12px] leading-relaxed text-muted">
            {description}
          </div>
        )}
      </div>
      <div className="flex items-center gap-2 shrink-0">{children}</div>
    </div>
  );
}
