/**
 * state-parity-still — NOT a shipping composition.
 *
 * A single 1920 × 1080 still that renders the real `src/routes/FloatingBar.tsx`
 * in one dictation state. Rendered six times with different `inputProps`
 * by `render.mjs`, producing six PNGs under `out/parity/` — one per
 * meaningful state (idle / warming / listening / processing / success /
 * error).
 *
 * Used once, at the batch-3 refactor's sign-off gate:
 *
 *   1. `node render.mjs --parity` renders the six PNGs.
 *   2. Jay / website-Claude opens the shipping Tauri app, forces the
 *      pill through the same six states, takes six screenshots.
 *   3. Diff app-side vs Remotion-side pair by pair. If the pill is
 *      pixel-identical, the import wire-up is correct and we're clear
 *      to re-render the twelve real comps.
 *
 * The composition cannot stack multiple states into one frame because
 * the real `useDictation` store is a module singleton — two BarDrivers
 * on the same page would race to write the store and only the last one
 * would win. One state per render = one store state at a time.
 */
import { AbsoluteFill } from 'remotion';
import { BarDriver, type BarState } from '../bar/BarDriver';

type Props = {
  /** Dictation state to force on the store. */
  state: BarState;
  /** Timer reading, in ms. Only meaningful while state === 'listening'. */
  elapsedMs: number;
  /** Mono-case label painted top-left so PNGs are self-describing. */
  label: string;
};

// Default props let the composition render cleanly in `remotion studio`
// without explicit inputProps — `render.mjs` overrides per-still.
export const parityStillDefaults: Props = {
  state: 'idle',
  elapsedMs: 0,
  label: 'IDLE',
};

export function StateParitySheet({ state, elapsedMs, label }: Props) {
  return (
    <AbsoluteFill
      style={{
        background: '#0A0E17',
        fontFamily:
          "'Instrument Sans Variable', 'Instrument Sans', system-ui, sans-serif",
      }}
    >
      {/* Self-labelling caption — so the saved PNG files carry the
          state name on their face, not just in the filename. Kept
          small and mono so it reads as metadata, not UI. */}
      <div
        style={{
          position: 'absolute',
          top: 48,
          left: 64,
          display: 'flex',
          flexDirection: 'column',
          gap: 4,
        }}
      >
        <div
          style={{
            fontFamily: "'Geist Mono Variable', ui-monospace, monospace",
            fontSize: 18,
            color: '#5A5A64',
            letterSpacing: '0.12em',
            textTransform: 'uppercase',
          }}
        >
          state-parity / {label}
        </div>
        <div
          style={{
            fontSize: 20,
            color: '#8A8A94',
          }}
        >
          Rendered from <code style={{ fontFamily: "'Geist Mono Variable', ui-monospace, monospace" }}>
            src/routes/FloatingBar.tsx
          </code>
        </div>
      </div>

      {/* Centre-stage pill. Scaled 2.4× so the pill reads at ~73 % of
          the 1920 frame — same hero size the shipping videos land at.
          Matches the `hero-pill-loop` scale so app screenshots taken
          at 100 % zoom diff directly against this render without a
          resize step. */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <div style={{ transform: 'scale(2.4)', transformOrigin: 'center' }}>
          <BarDriver
            state={state}
            elapsedMs={elapsedMs}
            // Seed lastText so the Copy-last action tile lights up
            // rather than rendering disabled — matches the common case
            // where the user has dictated at least once this session.
            lastText={'Sample dictation.'}
          />
        </div>
      </div>
    </AbsoluteFill>
  );
}
