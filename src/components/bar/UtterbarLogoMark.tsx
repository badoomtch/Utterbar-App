import { motion } from 'framer-motion';
import { cn } from '@/lib/cn';

/**
 * Utterbar glyph — the canonical pill mark (DESIGN_SYSTEM.md §10.1).
 *
 * Geometry is a direct port of `resources/views/components/logo.blade.php` in
 * the marketing-site repo, which is the authoritative rendering. Do NOT
 * redraw — copy this geometry. If you need a new size, scale.
 *
 * Structure:
 *   • Dark charcoal pill body (`--pill-top → --pill-bot` gradient) with a
 *     crisp 2px white stroke. The white stroke is the single most
 *     recognisable visual feature.
 *   • White hole-punch on the left (acts as a "power indicator").
 *   • Seven cyan bars inside, stepping up toward centre, with a central
 *     speech-bubble tail tapering down-right.
 *   • Accent-driven gradient. Previously hardcoded cyan stops (#00f6ff →
 *     #00a3ff) so the logo matched the marketing site regardless of the
 *     user's in-app accent choice. The trade-off was that picking a non-
 *     cyan accent looked half-applied — wordmark and CTA turned violet
 *     but the logo stayed cyan. We now drive the stops off --accent-2
 *     and --accent, which the theme-picker overrides in full, so the
 *     whole lockup re-tones in one click. The site retains its locked
 *     cyan via its own tokens; the app wins consistency inside itself.
 *
 * When `active` is true the waveform pulses gently — used on the floating
 * bar while the app is listening. The pulse lives inside the pill so the
 * border / hole-punch stay rock-steady.
 *
 * Gradient and path ids are namespaced (`ub-mark-…`) so they don't collide
 * with other SVGs on the same page.
 */
export function UtterbarLogoMark({
  active,
  size = 24,
  className,
}: {
  active?: boolean;
  size?: number;
  className?: string;
}) {
  // Canonical aspect ratio is 120×60 plus 2px padding on each side for the
  // white stroke, giving a 124×64 box. Preserve that ratio regardless of
  // `size` — callers treat `size` as the box HEIGHT.
  const width = Math.round(size * (124 / 64));

  return (
    <svg
      viewBox="-2 -2 124 64"
      width={width}
      height={size}
      className={cn('flex-shrink-0', className)}
      // linearRGB keeps the cyan gradient banding-free, per §10.1 note.
      color-interpolation="linearRGB"
      aria-hidden
    >
      <defs>
        <linearGradient id="ub-mark-pill" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stopColor="var(--pill-top)" />
          <stop offset="100%" stopColor="var(--pill-bot)" />
        </linearGradient>
        {/* Five logical stops so the gradient still has depth even when the
            chosen accent only provides two hand-picked colours: we ease
            between --accent-2 (top) and --accent (bottom) with a 50/50 mix
            in the middle via color-mix. WebView2 / Chromium 111+ supports
            color-mix in SVG style attributes. */}
        <linearGradient id="ub-mark-wave" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%"   style={{ stopColor: 'var(--accent-2)' }} />
          <stop offset="25%"  style={{ stopColor: 'color-mix(in oklab, var(--accent-2) 70%, var(--accent))' }} />
          <stop offset="50%"  style={{ stopColor: 'color-mix(in oklab, var(--accent-2) 45%, var(--accent))' }} />
          <stop offset="75%"  style={{ stopColor: 'color-mix(in oklab, var(--accent-2) 20%, var(--accent))' }} />
          <stop offset="100%" style={{ stopColor: 'var(--accent)' }} />
        </linearGradient>
      </defs>

      {/* Pill body + crisp white border */}
      <rect
        x="0"
        y="0"
        width="120"
        height="60"
        rx="30"
        fill="url(#ub-mark-pill)"
        stroke="#ffffff"
        strokeWidth="2"
      />

      {/* White hole-punch on the left */}
      <circle cx="20" cy="30" r="8" fill="#ffffff" />

      {/* Internal waveform with central speech-bubble tail (points down-right) */}
      <motion.g
        fill="url(#ub-mark-wave)"
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
