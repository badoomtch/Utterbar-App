import { cn } from '@/lib/cn';

/**
 * ScoreBars — paired accuracy/speed comparison bars for the model picker.
 *
 * Shown next to each Parakeet variant in Settings → Models and the
 * Onboarding picker so users can see at a glance how the choices trade
 * accuracy against speed. Scores are catalogue-assigned (1–10 scale,
 * editorial judgement — see `domain::model::ModelInfo`). Zero means
 * "no comparison data" and hides the component.
 *
 * Visual grammar:
 *   • Two labelled rows ("Accuracy", "Speed"), 10 discrete pips each.
 *   • Filled pips ride the accent gradient; unfilled ride `--bg-2` so
 *     they stay visible on both dark and Paper themes without burning
 *     an extra cyan beat.
 *   • Small caption labels use `.ub-eyebrow` sizing for the app rhythm
 *     without the uppercase tracking (which would read as heavy chrome
 *     on a compact comparison widget).
 *
 * Keeps the `<3 cyan moments per viewport` budget safe: each bar totals
 * a single cyan beat per row because the pip row reads as one object,
 * not ten.
 */
export function ScoreBars({
  accuracy,
  speed,
  compact = false,
  className,
}: {
  accuracy: number;
  speed: number;
  /** Tighter layout for list cells; default is the spacious card form. */
  compact?: boolean;
  className?: string;
}) {
  if (accuracy <= 0 && speed <= 0) return null;
  return (
    <div
      className={cn(
        'flex flex-col',
        compact ? 'gap-1' : 'gap-1.5',
        className,
      )}
    >
      <ScoreRow label="Accuracy" value={accuracy} compact={compact} />
      <ScoreRow label="Speed" value={speed} compact={compact} />
    </div>
  );
}

function ScoreRow({
  label,
  value,
  compact,
}: {
  label: string;
  value: number;
  compact: boolean;
}) {
  const clamped = Math.max(0, Math.min(10, Math.round(value)));
  return (
    <div className="flex items-center gap-2">
      <span
        className={cn(
          'shrink-0 text-muted',
          compact ? 'w-[52px] text-[10px]' : 'w-[60px] text-[11px]',
        )}
      >
        {label}
      </span>
      <div className="flex flex-1 items-center gap-[3px]">
        {Array.from({ length: 10 }).map((_, i) => {
          const filled = i < clamped;
          return (
            <span
              key={i}
              aria-hidden
              className={cn(
                'flex-1 rounded-[1.5px] transition-colors',
                compact ? 'h-1' : 'h-1.5',
                filled ? 'bg-accent' : 'bg-bg-2',
              )}
            />
          );
        })}
      </div>
      <span
        className={cn(
          'shrink-0 text-right font-mono tabular-nums text-fg-dim',
          compact ? 'w-[28px] text-[10px]' : 'w-[32px] text-[11px]',
        )}
      >
        {clamped}/10
      </span>
    </div>
  );
}
