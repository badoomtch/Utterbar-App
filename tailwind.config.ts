import type { Config } from 'tailwindcss';
import animate from 'tailwindcss-animate';

/*
 * Tailwind config — bridge between tokens.css and component utility classes.
 *
 * The palette is a direct port of the shared DESIGN_SYSTEM.md (see the
 * Utterbar Website repo). Surfaces are exposed as `bg-bg-0 / bg-1 / bg-2`
 * so code reads like the spec; legacy `background / surface / surface-raised`
 * names remain as aliases so older components continue to compile during the
 * refit.
 *
 * Rules:
 *   • Adding a colour here = adding it to the design system. Don't.
 *     Try a tonal shift on bg-1 / bg-2 first.
 *   • No elevation shadows. Cards elevate via background, not box-shadow.
 *   • No gradient-background tokens other than --gradient-wordmark
 *     (reserved for the wordmark "bar" and the <Highlight> word).
 */

export default {
  darkMode: ['class'],
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    container: {
      center: true,
      padding: '1rem',
    },
    extend: {
      colors: {
        // --- Canonical surface ladder ------------------------------------
        'bg-0': 'var(--bg-0)',
        'bg-1': 'var(--bg-1)',
        'bg-2': 'var(--bg-2)',

        // --- Text tiers --------------------------------------------------
        'fg-hi':  'var(--fg-hi)',
        'fg':     'var(--fg)',
        'muted':  'var(--muted)',
        'fg-dim': 'var(--fg-dim)',

        // --- Brand cyan trio --------------------------------------------
        accent:   'var(--accent)',
        'accent-2': 'var(--accent-2)',
        'accent-3': 'var(--accent-3)',

        // --- Semantic (app only; site is monochrome + cyan) -------------
        amber: 'var(--amber)',
        rose:  'var(--rose)',
        moss:  'var(--moss)',

        // --- Borders as literal rgba tokens -----------------------------
        'border-soft':   'rgba(255, 255, 255, 0.06)',
        'border-strong': 'rgba(255, 255, 255, 0.10)',

        // --- Legacy aliases kept for in-flight components ---------------
        // Tailwind's default `border-border` / `bg-background` etc. resolve
        // through hsl() so we keep those compatible. New code should prefer
        // the named tokens above.
        border: 'hsl(var(--border-hsl) / 0.08)',
        input:  'hsl(var(--input) / 0.10)',
        ring:   'hsl(var(--ring))',
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        surface: {
          DEFAULT: 'hsl(var(--surface))',
          raised: 'hsl(var(--surface-raised))',
          elevated: 'hsl(var(--surface-raised))',
          active: 'hsl(var(--surface-active))',
          frost: 'hsl(var(--surface-frost))',
        },
        text: {
          strong: 'hsl(var(--text-strong))',
          mid:    'hsl(var(--text-mid))',
          low:    'hsl(var(--text-low))',
          dim:    'hsl(var(--text-dim))',
        },
        brand: {
          cyan: {
            DEFAULT: 'hsl(var(--brand-cyan))',
            deep:    'hsl(var(--brand-cyan-deep))',
            petrol:  'hsl(var(--brand-cyan-deep) / 0.2)',
          },
        },
        'accent-soft':       'hsl(var(--accent-hsl) / 0.14)',
        'accent-foreground': 'hsl(var(--accent-foreground))',
        muted2: {
          DEFAULT:    'hsl(var(--muted-hsl))',
          foreground: 'hsl(var(--muted-foreground))',
        },
        destructive: {
          DEFAULT:    'hsl(var(--destructive))',
          foreground: 'hsl(var(--destructive-foreground))',
        },
        success: 'hsl(var(--success))',
        warning: 'hsl(var(--warning))',

        // Support palette retained for categorisation chips. Rare in new code.
        support: {
          lime:   'var(--moss)',
          amber:  'var(--amber)',
          violet: 'hsl(258 90% 66%)',
          coral:  'var(--rose)',
        },
      },
      borderRadius: {
        // Spec radii: cards 18, block 24, inputs 10, kbd 6, pill 999.
        card:  'var(--radius-card)',
        block: 'var(--radius-block)',
        input: 'var(--radius-input)',
        kbd:   'var(--radius-kbd)',
        pill:  'var(--radius-pill)',

        // Tailwind standard aliases mapped to spec values so `rounded-lg`
        // used across older code still produces on-system geometry.
        sm: 'var(--radius-kbd)',
        md: 'var(--radius-input)',
        lg: '14px',
        xl: 'var(--radius-card)',
        '2xl': 'var(--radius-block)',
      },
      fontFamily: {
        // Instrument Sans Variable for everything that isn't literal code/data.
        sans: [
          'Instrument Sans Variable',
          'Instrument Sans',
          'Segoe UI Variable',
          'Segoe UI',
          'system-ui',
          'sans-serif',
        ],
        // Geist Mono Variable only for versions, paths, timestamps, hashes,
        // keyboard chip labels. Never decorative.
        mono: [
          'Geist Mono Variable',
          'Geist Mono',
          'JetBrains Mono',
          'Consolas',
          'ui-monospace',
          'monospace',
        ],
        // Wordmark stack — pinned so the "bar" gradient is stable even when
        // Instrument Sans hasn't painted yet.
        wordmark: [
          'Instrument Sans Variable',
          'Instrument Sans',
          'Segoe UI',
          'system-ui',
          'sans-serif',
        ],
      },
      fontSize: {
        // Eyebrow — the spec's 12px at 17px site maps to ~11px at 14px app.
        eyebrow: ['0.6875rem', { lineHeight: '1.2', letterSpacing: '0.14em' }],
      },
      letterSpacing: {
        // Site display tracking, used on page hero in dense app contexts.
        display: '-0.045em',
        h2: '-0.035em',
        h3: '-0.02em',
      },
      boxShadow: {
        // Cards and panels have NO shadow — the spec is explicit about this.
        // These tokens exist only for the single permitted case: the LED
        // glow primitive used on the floating pill and CTA button LEDs.
        led:      '0 0 10px var(--accent), 0 0 3px color-mix(in oklab, var(--accent) 90%, #fff)',
        'led-lg': '0 0 14px var(--accent), 0 0 4px color-mix(in oklab, var(--accent) 90%, #fff)',
        // Floor shadow under the hero pill (website only; app reuses for
        // the large onboarding pill illustration).
        floor:    '0 80px 120px -40px rgba(0, 0, 0, 0.55)',
      },
      backdropBlur: {
        xs: '2px',
      },
      keyframes: {
        // Pill LED — slow breathing when idle, fast pulse during record.
        pulseDot: {
          '0%, 100%': { opacity: '1',   transform: 'scale(1)' },
          '50%':      { opacity: '0.75', transform: 'scale(1.3)' },
        },
        // Gentle opacity fade for the pill's live-state underline. Ease only.
        seam: {
          '0%, 100%': { opacity: '0.55' },
          '50%':      { opacity: '1' },
        },
        // Waveform bar bounce for the pill's recording state.
        wave: {
          '0%, 100%': { transform: 'scaleY(0.4)' },
          '50%':      { transform: 'scaleY(1)' },
        },
      },
      animation: {
        pulseDot: 'pulseDot 1.6s ease-in-out infinite',
        seam:     'seam 1.8s ease-in-out infinite',
        wave:     'wave 900ms ease-in-out infinite',
      },
      transitionTimingFunction: {
        // The one eased curve used across the site. Matches nav underline,
        // view transitions, FAQ accordion, etc.
        sweep: 'cubic-bezier(0.2, 0.8, 0.2, 1)',
      },
    },
  },
  plugins: [animate],
} satisfies Config;
