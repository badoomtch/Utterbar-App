import { motion } from 'framer-motion';
import { cn } from '@/lib/cn';

/**
 * Compact Utterbar glyph for tight spaces like the floating bar.
 *
 * Geometry mirrors the interior of the canonical pill mark (§10.1) — same
 * seven cyan bars + central speech-bubble tail on the same 120×60 grid —
 * but strips the pill body, white border and hole-punch. The floating bar
 * already IS a pill; rendering the full mark inside it would read as a
 * little app-tile-within-a-bar, which the spec explicitly calls out in
 * §11.8 ("Don't repeat-brand inside the app").
 *
 * The tight viewBox (44–108 × 10–50) frames just the waveform so size maps
 * cleanly to the caller's intended footprint.
 */
export function UtterbarMiniMark({
  active,
  size = 20,
  className,
}: {
  active?: boolean;
  size?: number;
  className?: string;
}) {
  // Waveform occupies x: 44→108 (width 64), y: 10→50 (height 40).
  // Ratio ≈ 1.6:1. `size` is the rendered HEIGHT.
  const width = Math.round(size * (64 / 40));

  return (
    <svg
      viewBox="44 10 64 40"
      width={width}
      height={size}
      className={cn('flex-shrink-0', className)}
      color-interpolation="linearRGB"
      aria-hidden
    >
      <defs>
        {/* Same 5-stop cyan track the full mark uses — see UtterbarLogoMark. */}
        {/* Accent-driven stops — matches UtterbarLogoMark. See tokens.css for
            the `.accent-*` cascade that sets --accent-2 / --accent for each
            theme. */}
        <linearGradient id="ub-mini-wave" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%"   style={{ stopColor: 'var(--accent-2)' }} />
          <stop offset="25%"  style={{ stopColor: 'color-mix(in oklab, var(--accent-2) 70%, var(--accent))' }} />
          <stop offset="50%"  style={{ stopColor: 'color-mix(in oklab, var(--accent-2) 45%, var(--accent))' }} />
          <stop offset="75%"  style={{ stopColor: 'color-mix(in oklab, var(--accent-2) 20%, var(--accent))' }} />
          <stop offset="100%" style={{ stopColor: 'var(--accent)' }} />
        </linearGradient>
      </defs>
      <motion.g
        fill="url(#ub-mini-wave)"
        animate={active ? { opacity: [0.8, 1, 0.8] } : { opacity: 1 }}
        transition={active ? { duration: 1.4, repeat: Infinity, ease: 'easeInOut' } : {}}
      >
        <rect x="44"  y="25" width="4" height="10" rx="2" />
        <rect x="54"  y="21" width="4" height="18" rx="2" />
        <rect x="64"  y="16" width="4" height="28" rx="2" />
        <path d="M 74 10 A 2 2 0 0 1 78 10 V 50 L 74 44 Z" />
        <rect x="84"  y="16" width="4" height="28" rx="2" />
        <rect x="94"  y="21" width="4" height="18" rx="2" />
        <rect x="104" y="25" width="4" height="10" rx="2" />
      </motion.g>
    </svg>
  );
}
