import type { ReactNode } from 'react';

/**
 * Top-of-page header.
 *
 * Hierarchy per DESIGN_SYSTEM.md §2.3 (scaled down for app density §11.1):
 *   eyebrow  (Instrument Sans 500, 11px, +0.14em tracking, UPPERCASE, --muted)
 *            — signals which section the user is in
 *   title    (Instrument Sans 800, ~24px, tight tracking)
 *            — the page's single loudest typographic moment
 *   subtitle (muted, 58ch max, line-height 1.55)
 *            — one sentence of intent, never a paragraph
 *
 * Weight 600 is explicitly forbidden by the spec — do not reach for it if
 * 800 feels "too much" at this scale; the brief IS to feel confident.
 */
export function PageHeader({
  title,
  subtitle,
  eyebrow,
  actions,
}: {
  title: string;
  subtitle?: string;
  eyebrow?: string;
  actions?: ReactNode;
}) {
  return (
    <header className="flex items-start justify-between gap-6">
      <div className="min-w-0">
        {eyebrow && <div className="ub-eyebrow mb-3">{eyebrow}</div>}
        <h1 className="text-[26px] font-extrabold text-fg-hi -tracking-[0.03em] leading-[1.05]">
          {title}
        </h1>
        {subtitle && (
          <p className="mt-2.5 text-[14px] font-light leading-[1.55] text-muted max-w-[58ch]">
            {subtitle}
          </p>
        )}
      </div>
      {actions && <div className="flex items-center gap-2 shrink-0">{actions}</div>}
    </header>
  );
}
