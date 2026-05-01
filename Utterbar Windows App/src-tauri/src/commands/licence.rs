//! Tauri commands for the licence system.
//!
//! Three verbs. The UI reads status on mount via `licence_status`, listens to
//! the `licence_changed` event for subsequent updates, and calls
//! `licence_activate` / `licence_deactivate` from the Settings → Licence page.

use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::app_state::AppState;
use crate::domain::licence::LicenceStatus;
use crate::util::result::CmdResult;

#[tauri::command]
pub async fn licence_status(state: State<'_, Arc<AppState>>) -> CmdResult<LicenceStatus> {
    Ok(state.licence.status().await)
}

#[tauri::command]
pub async fn licence_activate(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    key: String,
) -> CmdResult<LicenceStatus> {
    state.licence.activate(handle, &key).await
}

#[tauri::command]
pub async fn licence_deactivate(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> CmdResult<LicenceStatus> {
    state.licence.deactivate(handle).await
}

/// True when the active Pro status came from a managed (fleet) licence
/// file at `<data_dir>/licence.bin` rather than the keyring cache or the
/// paste-flow activation. The Settings > Licence page calls this on
/// mount to swap the activation form for a "managed by your administrator"
/// notice. Frozen at startup; matches the rest of the policy/managed
/// stack - managed-licence changes require restarting Utterbar.
#[tauri::command]
pub fn licence_is_managed(state: State<'_, Arc<AppState>>) -> CmdResult<bool> {
    Ok(state.licence.is_managed())
}
