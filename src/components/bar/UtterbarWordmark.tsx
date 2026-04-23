import { cn } from '@/lib/cn';

/**
 * Utterbar wordmark — "Utter" in foreground, "bar" in the brand cyan gradient.
 *
 * Per DESIGN_SYSTEM.md §8.3 / §10.2 and matching the marketing site's
 * `resources/views/components/logo.blade.php`, the wordmark is Instrument
 * Sans at weight 600 with -0.04em tracking. The "bar" half renders with
 * the theme-picker's accent-aware `--gradient-wordmark` token — each
 * accent variant (cyan / teal / amber / violet / coral / lime / rose)
 * defines its own two-stop gradient so the entire logo re-tones when the
 * user changes accent. Previously this was a hardcoded cyan gradient;
 * combined with the new accent cascade in tokens.css the whole lockup
 * now reads consistently across themes.
 *
 * Inline SVG so the gradient works identically across the web-view and
 * any canvas/raster target we render it into (tray icon, splash, etc.)
 * without depending on `background-clip: text`.
 */
export function UtterbarWordmark({
  size = 24,
  className,
}: {
  size?: number;
  className?: string;
}) {
  const width = size * 4.2; // tuned to the aspect of the lockup

  return (
    <svg
      viewBox="0 0 290 64"
      width={width}
      height={size}
      className={cn('flex-shrink-0', className)}
      aria-label="Utterbar"
    >
      <defs>
        {/* Accent-driven horizontal gradient, matching the accent cascade in
            tokens.css. `--accent-2` is the lit/hover sibling (always the
            lighter of the pair) so it reads left; `--accent` is the base
            tone on the right. Changing accent (cyan → violet → coral…)
            re-tones the "bar" half without a remount. */}
        <linearGradient id="ub-wordmark-cyan" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" style={{ stopColor: 'var(--accent-2)' }} />
          <stop offset="100%" style={{ stopColor: 'var(--accent)' }} />
        </linearGradient>
      </defs>
      <text
        x="0"
        y="52"
        fontFamily="'Instrument Sans Variable', 'Instrument Sans', ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif"
        fontSize="64"
        fontWeight="600"
        letterSpacing="-2.56"
      >
        <tspan fill="currentColor">Utter</tspan>
        <tspan fill="url(#ub-wordmark-cyan)">bar</tspan>
      </text>
    </svg>
  );
}
