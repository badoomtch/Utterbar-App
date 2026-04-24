/**
 * Font registration for Remotion. Both families are self-hosted via
 * @fontsource-variable (NOT a CDN) so the renderer doesn't need network.
 *
 * Weights in use:
 *   - Instrument Sans: 300 / 400 / 500 / 700 / 800  (never 600 — the design
 *     system bans 600 as ambiguous under the Instrument Sans metrics)
 *   - Geist Mono:      400 / 500
 *
 * Import side-effect only — this module is loaded once from Root.tsx and
 * every composition inherits the registered families.
 */
import '@fontsource-variable/instrument-sans/wght.css';
import '@fontsource-variable/geist-mono/wght.css';
