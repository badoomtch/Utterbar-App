// Event names — must match `src-tauri/src/domain/events.rs` constants.
export const EVT = {
  dictationPhase: 'dictation:phase',
  dictationMeter: 'dictation:meter',
  dictationResult: 'dictation:result',
  dictationError: 'dictation:error',
  // Fired when start_recording is rejected because the app is already
  // busy (previous recording mid-transcribe, guard still held, etc.).
  // FloatingBar listens for this and runs a one-second border flash.
  dictationBusy: 'dictation:busy',
  // Fired during chunked Meeting Mode transcription, once per chunk
  // boundary. Payload: { chunk_index, total_chunks }. Listened to in
  // AppBootstrap; stored in `useDictation.meetingProgress` so StatePill
  // can paint a thin progress bar under "Transcribing meeting…" while
  // a long meeting is being stitched. Single-chunk fast path doesn't
  // emit; non-meeting recordings never enter the chunked path.
  dictationChunkProgress: 'dictation:chunk-progress',
  modelProgress: 'model:progress',
  modelChanged: 'model:changed',
  settingsChanged: 'settings:changed',
  shortcutTriggered: 'shortcut:triggered',
  fileJobProgress: 'file-job:progress',
  trayStartRecording: 'tray:start-recording',
  trayPauseToggled: 'tray:pause-toggled',
  mainNavigate: 'main:navigate',
  licenceChanged: 'licence_changed',
  /// Emitted by `open_vocab_quick_add` (commands/vocabulary.rs) after the
  /// vocab-add window is shown. Payload is `VocabCapturePayload`.
  vocabCapture: 'vocab:capture',
} as const;
