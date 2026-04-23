use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::app_state::AppState;
use crate::domain::{Shortcut, ShortcutConflict, ShortcutId, ShortcutSlot};
use crate::services::hotkeys;
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn list_shortcuts(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<ShortcutSlot>> {
    let s = state.settings.snapshot().await;
    Ok(vec![
        slot(ShortcutId::ToggleRecording, "Toggle recording", "Start or stop a dictation.", s.shortcuts.toggle_recording.clone()),
        slot(ShortcutId::PushToTalk, "Push to talk", "Record only while held (press-release).", s.shortcuts.push_to_talk.clone()),
        slot(ShortcutId::HoldToTalk, "Hold to talk", "Record while key held; release to finalise.", s.shortcuts.hold_to_talk.clone()),
        slot(ShortcutId::CancelRecording, "Cancel recording", "Discard the current recording.", s.shortcuts.cancel_recording.clone()),
        slot(ShortcutId::CycleMode, "Cycle mode", "Switch to the next enabled mode.", s.shortcuts.cycle_mode.clone()),
        slot(ShortcutId::OpenSettings, "Open settings", "Show the main settings window.", s.shortcuts.open_settings.clone()),
        slot(ShortcutId::OpenHistory, "Open history", "Show the history panel.", s.shortcuts.open_history.clone()),
    ])
}

fn slot(id: ShortcutId, label: &str, desc: &str, current: Option<Shortcut>) -> ShortcutSlot {
    ShortcutSlot { id, label: label.into(), description: desc.into(), current }
}

#[tauri::command]
pub async fn bind_shortcut(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: ShortcutId,
    shortcut: Shortcut,
) -> CmdResult<()> {
    state.hotkeys.bind(id.clone(), shortcut.clone()).await.map_err(OrliqError::from)?;
    state.settings.update(|s| assign(s, &id, Some(shortcut))).await.map_err(OrliqError::from)?;
    hotkeys::arm_from_settings(&handle, &state).await.map_err(OrliqError::from)?;
    Ok(())
}

#[tauri::command]
pub async fn unbind_shortcut(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: ShortcutId,
) -> CmdResult<()> {
    state.hotkeys.unbind(id.clone()).await.map_err(OrliqError::from)?;
    state.settings.update(|s| assign(s, &id, None)).await.map_err(OrliqError::from)?;
    hotkeys::arm_from_settings(&handle, &state).await.map_err(OrliqError::from)?;
    Ok(())
}

#[tauri::command]
pub async fn detect_conflicts(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<ShortcutConflict>> {
    Ok(state.hotkeys.conflicts().await)
}

fn assign(settings: &mut crate::domain::Settings, id: &ShortcutId, val: Option<Shortcut>) {
    match id {
        ShortcutId::ToggleRecording => settings.shortcuts.toggle_recording = val,
        ShortcutId::PushToTalk => settings.shortcuts.push_to_talk = val,
        ShortcutId::HoldToTalk => settings.shortcuts.hold_to_talk = val,
        ShortcutId::CancelRecording => settings.shortcuts.cancel_recording = val,
        ShortcutId::CycleMode => settings.shortcuts.cycle_mode = val,
        ShortcutId::OpenSettings => settings.shortcuts.open_settings = val,
        ShortcutId::OpenHistory => settings.shortcuts.open_history = val,
    }
}
