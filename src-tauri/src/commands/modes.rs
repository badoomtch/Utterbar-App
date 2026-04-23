use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::app_state::AppState;
use crate::domain::Mode;
use crate::services::tray;
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn list_modes(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<Mode>> {
    state.modes.list().map_err(OrliqError::from)
}

#[tauri::command]
pub async fn upsert_mode(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    mode: Mode,
) -> CmdResult<Mode> {
    let sort = mode.sort_order;
    let result = state.modes.upsert(mode, sort).map_err(OrliqError::from)?;
    let _ = tray::rebuild(&handle, &state).await;
    Ok(result)
}

#[tauri::command]
pub async fn delete_mode(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> CmdResult<()> {
    state.modes.delete(&id).map_err(OrliqError::from)?;
    let _ = tray::rebuild(&handle, &state).await;
    Ok(())
}

#[tauri::command]
pub async fn duplicate_mode(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> CmdResult<Mode> {
    let result = state.modes.duplicate(&id).map_err(OrliqError::from)?;
    let _ = tray::rebuild(&handle, &state).await;
    Ok(result)
}

#[tauri::command]
pub async fn set_active_mode(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<()> {
    state
        .settings
        .update(|s| s.active_mode_id = Some(id.clone()))
        .await
        .map_err(OrliqError::from)?;
    Ok(())
}

#[tauri::command]
pub async fn get_active_mode(state: State<'_, Arc<AppState>>) -> CmdResult<Option<Mode>> {
    let s = state.settings.snapshot().await;
    let id = s.active_mode_id.unwrap_or_default();
    state.modes.get(&id).map_err(OrliqError::from)
}
