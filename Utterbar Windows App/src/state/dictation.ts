import { create } from 'zustand';

export type DictationSuccessSummary = {
  duration_seconds: number;
  chunks: number;
  chars: number;
  /** v0.15.3+: short model name (e.g. "Whisper Medium") rendered
   * in the success pill so the user can correlate quality vs
   * speed across meetings. Always set on Rust-side meeting summaries. */
  model_label: string;
};

export type DictationPhase =
  | { kind: 'idle' }
  | { kind: 'warming'; model_id: string }
  | { kind: 'listening'; started_at_ms: number }
  | { kind: 'processing' }
  // Meeting-mode chunked transcribe is running. Visually distinct from
  // Processing so the user can tell that a long meeting is in flight
  // (and roughly how long it'll take — chunked transcribe is ~real-time
  // on Parakeet, so "5 minutes of audio" → "expect ~17 s of this state").
  | { kind: 'transcribing_background' }
  // `summary` is populated for meeting-mode recordings so the floating
  // bar can render "5:01 · 11 chunks · 799 chars" instead of the generic
  // "Inserted" pill. Optional — ordinary dictations omit it.
  //
  // `notice` is populated when the stop path produced a successful
  // transcript but wants to flag a non-error condition the user
  // should see. The single-shot auto-stop at the 5-minute cap
  // sets it to "Auto-stopped at 5 min. Use Meeting Mode for
  // longer recordings." Distinct from `summary` because the
  // notice fires for non-meeting recordings (which never have a
  // summary), and distinct from `error` because the recording did
  // produce a transcript and the user got their text inserted.
  | { kind: 'success'; summary?: DictationSuccessSummary; notice?: string }
  | { kind: 'error'; message: string }
  | { kind: 'cancelled' }
  // v0.6.6 — the clip had no usable speech; bar shows "No audio detected"
  // and decays back to idle without writing a history row or firing the
  // success cue. Keep in lockstep with the Rust `DictationPhase` enum in
  // src-tauri/src/domain/events.rs (and with the generated type at
  // src/generated/DictationPhase.ts which ts-rs produces from the Rust
  // source — this hand-maintained copy exists for the zustand store).
  //
  // Optional `notice` carries a short label when the drop was a
  // validator-noticed case (e.g. "Output filtered" for repeated-token
  // or foreign-script hallucinations) instead of plain silence. The
  // floating bar substitutes the default "No audio detected" pill for
  // the notice text in the same muted tone.
  | { kind: 'no_audio'; notice?: string };

export type MeterFrame = { rms: number; peak: number; ts_ms: number };

// Meeting Mode chunked-transcribe progress. Populated from
// `dictation:chunk-progress` events in AppBootstrap; reset on every
// phase transition that leaves `transcribing_background`. `null` when
// no meeting is currently being transcribed (the common case).
export type MeetingProgress = { chunkIndex: number; totalChunks: number };

// One live-transcribed meeting segment. Populated from
// `meeting:live-segment` events as each speech span is recognised
// during a live meeting. `speakerId` is 0-based (0 = you / mic,
// 1 = others / system audio). Reset on `meeting:live-reset`.
export type LiveMeetingSegment = {
  id: number;
  speakerId: number;
  startSecs: number;
  text: string;
};

export type DictationState = {
  phase: DictationPhase;
  meter: MeterFrame;
  elapsedMs: number;
  meetingProgress: MeetingProgress | null;
  // Live meeting transcript segments, in arrival order. Empty when no
  // live meeting is in progress. The view sorts by `startSecs` for
  // display.
  liveMeeting: LiveMeetingSegment[];
  // True from the moment a live meeting starts (its reset event) until
  // the recording winds down to idle. Lets the live panel show a
  // "listening…" state before the first segment lands.
  liveMeetingActive: boolean;
  // Last successfully-delivered dictation text. Populated from the
  // `dictation:result` event in AppBootstrap so any window (floating bar
  // especially) can surface "copy last message" without re-querying SQLite.
  // Prefer `processed_text`; fall back to raw when processing returned nothing.
  // Null until the user dictates once in the current session.
  lastText: string | null;
  setPhase(phase: DictationPhase): void;
  setMeter(frame: MeterFrame): void;
  setMeetingProgress(progress: MeetingProgress | null): void;
  resetLiveMeeting(): void;
  addLiveSegment(segment: LiveMeetingSegment): void;
  dismissLiveMeeting(): void;
  setLastText(text: string | null): void;
  tickElapsed(): void;
};

export const useDictation = create<DictationState>((set, get) => ({
  phase: { kind: 'idle' },
  meter: { rms: 0, peak: 0, ts_ms: 0 },
  elapsedMs: 0,
  meetingProgress: null,
  liveMeeting: [],
  liveMeetingActive: false,
  lastText: null,
  setPhase(phase) {
    // Clear meeting progress whenever we leave the
    // transcribing_background phase. Otherwise a stale
    // "11/12 chunks" would linger past the success pill into
    // the next recording.
    const meetingProgress =
      phase.kind === 'transcribing_background' ? get().meetingProgress : null;
    // Reset the meter slot when entering a fresh `listening` phase.
    // Without this, the floating bar's MeterBars rAF loop would read
    // the stale frame from the *previous* recording on its first tick
    // (the meter pump only emits while a capture is live, so the slot
    // freezes at the last frame after stop). For ~30 ms (until the
    // next live frame arrives) the bars would visibly start tall on a
    // recording the user has just begun, contradicting the whole
    // "react to detected audio" point of the meter-driven waveform.
    const meter =
      phase.kind === 'listening' ? { rms: 0, peak: 0, ts_ms: 0 } : get().meter;
    // A live meeting stops being "active" once the recording winds all
    // the way down to idle. Its segments are kept (so the finished
    // transcript stays on screen to review) until the user dismisses it
    // or the next meeting resets it.
    const liveMeetingActive = phase.kind === 'idle' ? false : get().liveMeetingActive;
    set({
      phase,
      elapsedMs: phase.kind === 'listening' ? 0 : get().elapsedMs,
      meetingProgress,
      meter,
      liveMeetingActive,
    });
  },
  setMeter(frame) {
    set({ meter: frame });
  },
  setMeetingProgress(progress) {
    set({ meetingProgress: progress });
  },
  resetLiveMeeting() {
    // Fires at the start of a live meeting: clear the prior transcript
    // and mark the session active so the panel shows immediately.
    set({ liveMeeting: [], liveMeetingActive: true });
  },
  addLiveSegment(segment) {
    // De-dupe on id (Tauri can double-deliver during window focus
    // changes) and keep arrival order; the view sorts by startSecs.
    const existing = get().liveMeeting;
    if (existing.some((s) => s.id === segment.id)) return;
    set({ liveMeeting: [...existing, segment] });
  },
  dismissLiveMeeting() {
    set({ liveMeeting: [], liveMeetingActive: false });
  },
  setLastText(text) {
    set({ lastText: text });
  },
  tickElapsed() {
    const p = get().phase;
    if (p.kind === 'listening') set({ elapsedMs: Date.now() - p.started_at_ms });
  },
}));
