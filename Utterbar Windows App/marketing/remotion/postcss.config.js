/**
 * PostCSS config for the Remotion bundle.
 *
 * Remotion's bundler (webpack under the hood) auto-detects a
 * `postcss.config.js` at the Remotion project root and pipes every
 * imported `.css` file through it. This is how `@tailwind base/components/
 * utilities` directives inside `../../src/styles/globals.css` actually
 * produce utility classes — without this file, Tailwind's `@tailwind`
 * lines would ship as raw CSS and every `bg-accent` / `text-fg-hi` /
 * `animate-pulseDot` in the real app components would render as nothing.
 */
export default {
  plugins: {
    tailwindcss: { config: './tailwind.config.ts' },
    autoprefixer: {},
  },
};
