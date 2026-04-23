// Event names — must match `src-tauri/src/domain/events.rs` constants.
export const EVT = {
  dictationPhase: 'dictation:phase',
  dictationMeter: 'dictation:meter',
  dictationResult: 'dictation:result',
  dictationError: 'dictation:error',
  modelProgress: 'model:progress',
  modelChanged: 'model:changed',
  settingsChanged: 'settings:changed',
  shortcutTriggered: 'shortcut:triggered',
  fileJobProgress: 'file-job:progress',
  trayStartRecording: 'tray:start-recording',
  trayPauseToggled: 'tray:pause-toggled',
  mainNavigate: 'main:navigate',
} as const;
