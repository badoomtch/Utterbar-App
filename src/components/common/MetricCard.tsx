import type { ComponentType, SVGProps } from 'react';

import { cn } from '@/lib/cn';

/**
 * Metric tones. Per §1.6, a Dashboard viewport gets AT MOST three distinct
 * cyan elements — a full grid of accent-toned metric cards is a design
 * review failure.
 *
 *   • `accent`   — brand cyan rail. Use for ONE hero metric per grid.
 *   • `neutral`  — hairline rail in --border. Default for every other card.
 *   • `positive` — moss green rail (success-coded counts).
 *   • `warning`  — amber rail (counts near a threshold).
 *   • `danger`   — rose rail (error counts, failed jobs).
 *
 * Back-compat aliases remain (rose/amber/violet/cyan/lime/teal) so older
 * routes compile while being swept.
 */
export type MetricTone =
  | 'accent'
  | 'neutral'
  | 'positive'
  | 'warning'
  | 'danger'
  | 'rose'
  | 'amber'
  | 'violet'
  | 'cyan'
  | 'lime'
  | 'teal';

interface ToneStyle {
  rail: string;
  icon: string;
}

const TONE_STYLES: Record<MetricTone, ToneStyle> = {
  accent:   { rail: 'bg-accent',         icon: 'text-accent' },
  neutral:  { rail: 'bg-border-strong',  icon: 'text-muted' },
  positive: { rail: 'bg-moss',           icon: 'text-moss' },
  warning:  { rail: 'bg-amber',          icon: 'text-amber' },
  danger:   { rail: 'bg-rose',           icon: 'text-rose' },

  // Back-compat aliases — collapse onto the main tones.
  cyan:   { rail: 'bg-accent',        icon: 'text-accent' },
  rose:   { rail: 'bg-rose',          icon: 'text-rose' },
  amber:  { rail: 'bg-amber',         icon: 'text-amber' },
  violet: { rail: 'bg-support-violet',icon: 'text-support-violet' },
  lime:   { rail: 'bg-moss',          icon: 'text-moss' },
  teal:   { rail: 'bg-border-strong', icon: 'text-muted' },
};

/**
 * A dashboard stat tile.
 *
 * Layout per spec: bg-1 surface, 18px radius, a single 2px left rail,
 * Instrument Sans eyebrow, Geist Mono numeric value (literal data per §2.4
 * "timestamps and hash-style IDs" — counts fall in the same "numeric data"
 * bucket). No ring, no drop shadow, no gradient — surface tone does the work.
 *
 * The accent-tone card also gains the ::before bloom via .ub-card-rail so
 * it reads as the hero tile without any extra colour.
 */
export function MetricCard({
  label,
  value,
  caption,
  tone = 'neutral',
  icon: Icon,
}: {
  label: string;
  value: string;
  caption?: string;
  tone?: MetricTone;
  icon?: ComponentType<SVGProps<SVGSVGElement>>;
}) {
  const t = TONE_STYLES[tone];
  const isAccent = tone === 'accent' || tone === 'cyan';

  return (
    <div
      className={cn(
        'group relative overflow-hidden rounded-card bg-bg-1 px-5 py-4',
        // Accent tile gets the full rail-with-bloom signature; neutral
        // tiles get a quiet rail via the absolute-positioned div below.
        isAccent && 'ub-card-rail',
      )}
    >
      {!isAccent && (
        <div className={cn('absolute left-0 top-3 bottom-3 w-[2px] rounded-r-full', t.rail)} />
      )}

      <div className="flex items-center justify-between gap-3 pl-3">
        <div className="ub-eyebrow">{label}</div>
        {Icon && <Icon className={cn('h-3.5 w-3.5 shrink-0', t.icon)} />}
      </div>

      <div className="mt-1.5 pl-3 font-mono tabular-nums text-[26px] leading-none -tracking-[0.01em] text-fg-hi">
        {value}
      </div>

      {caption && (
        <div className="mt-2 pl-3 text-[12px] text-muted">{caption}</div>
      )}
    </div>
  );
}
