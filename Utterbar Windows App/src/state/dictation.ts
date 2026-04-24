import { create } from 'zustand';

export type DictationPhase =
  | { kind: 'idle' }
  | { kind: 'warming'; model_id: string }
  | { kind: 'listening'; started_at_ms: number }
  | { kind: 'processing' }
  | { kind: 'success' }
  | { kind: 'error'; message: string }
  | { kind: 'cancelled' };

export type MeterFrame = { rms: number; peak: number; ts_ms: number };

export type DictationState = {
  phase: DictationPhase;
  meter: MeterFrame;
  elapsedMs: number;
  // Last successfully-delivered dictation text. Populated from the
  // `dictation:result` event in AppBootstrap so any window (floating bar
  // especially) can surface "copy last message" without re-querying SQLite.
  // Prefer `processed_text`; fall back to raw when processing returned nothing.
  // Null until the user dictates once in the current session.
  lastText: string | null;
  setPhase(phase: DictationPhase): void;
  setMeter(frame: MeterFrame): void;
  setLastText(text: string | null): void;
  tickElapsed(): void;
};

export const useDictation = create<DictationState>((set, get) => ({
  phase: { kind: 'idle' },
  meter: { rms: 0, peak: 0, ts_ms: 0 },
  elapsedMs: 0,
  lastText: null,
  setPhase(phase) {
    set({ phase, elapsedMs: phase.kind === 'listening' ? 0 : get().elapsedMs });
  },
  setMeter(frame) {
    set({ meter: frame });
  },
  setLastText(text) {
    set({ lastText: text });
  },
  tickElapsed() {
    const p = get().phase;
    if (p.kind === 'listening') set({ elapsedMs: Date.now() - p.started_at_ms });
  },
}));
