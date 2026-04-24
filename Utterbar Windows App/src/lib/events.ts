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
  modelProgress: 'model:progress',
  modelChanged: 'model:changed',
  settingsChanged: 'settings:changed',
  shortcutTriggered: 'shortcut:triggered',
  fileJobProgress: 'file-job:progress',
  trayStartRecording: 'tray:start-recording',
  trayPauseToggled: 'tray:pause-toggled',
  mainNavigate: 'main:navigate',
  licenceChanged: 'licence_changed',
} as const;
