/**
 * Remotion-side stub for `@tauri-apps/api/window`.
 *
 * `src/routes/FloatingBar.tsx` and `src/state/settings.ts` pull
 * `getCurrentWindow()` for window-frame control (show/hide, drag,
 * always-on-top, theme, outer-position polls). The Remotion renderer
 * isn't a Tauri window; every method here returns a benign no-op so the
 * real component mounts without throwing.
 *
 * Wired via webpack `resolve.alias` in `remotion.config.ts`.
 */
type UnlistenFn = () => void;

export function getCurrentWindow() {
  return {
    async outerPosition() {
      return { x: 0, y: 0 };
    },
    async scaleFactor() {
      return 1;
    },
    async show() {
      return undefined;
    },
    async hide() {
      return undefined;
    },
    async unminimize() {
      return undefined;
    },
    async setAlwaysOnTop(_v: boolean) {
      return undefined;
    },
    async setTheme(_t: 'light' | 'dark' | null) {
      return undefined;
    },
    async startDragging() {
      return undefined;
    },
    async onMoved(_cb: (e: unknown) => void): Promise<UnlistenFn> {
      return () => undefined;
    },
  };
}
