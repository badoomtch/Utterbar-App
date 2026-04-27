use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use ts_rs::TS;

use crate::app_state::AppState;
use crate::domain::Mode;
use crate::services::tray;
use crate::util::gating::{is_pro, pro_required};
use crate::util::result::{CmdResult, OrliqError};

/// Slim view of the currently-focused window — exe name, title, class.
/// Used by the auto-activation editor's "test this rule" live indicator.
/// All three fields can be `None` when the OS reports no foreground
/// window (e.g. the user is on the desktop or a UAC prompt is up).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct FocusedWindow {
    pub exe: Option<String>,
    pub title: Option<String>,
    pub class: Option<String>,
}

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
    // Pro gate: Free tier may edit built-ins and existing custom modes,
    // but cannot create new custom modes. "New" = an id that doesn't match
    // any existing row. Grandfathering: Pro-authored custom modes remain
    // editable after deactivation.
    let status = state.licence.status().await;
    if !is_pro(&status) && !mode.is_builtin {
        let existing = state.modes.list().map_err(OrliqError::from)?;
        let is_new = !existing.iter().any(|m| m.id == mode.id);
        if is_new {
            return Err(pro_required("Custom modes"));
        }
    }

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
    // Duplicating is just a specialised new-mode creation, so same gate.
    let status = state.licence.status().await;
    if !is_pro(&status) {
        return Err(pro_required("Custom modes"));
    }

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

/// Return the currently-focused window's exe / title / class. Used by
/// the auto-activation rule editor to show a live "this rule matches /
/// doesn't match" indicator while the user is editing.
///
/// Reuses the same `context::snapshot()` plumbing the rest of the app
/// uses for active-app context, but skips the expensive selection /
/// clipboard reads — this command is called every ~1s while the editor
/// is open and we do not need either field.
#[tauri::command]
pub async fn current_focused_window() -> CmdResult<FocusedWindow> {
    let svc = crate::services::context::ContextService::new();
    let ctx = tauri::async_runtime::spawn_blocking(move || svc.snapshot(false, false))
        .await
        .map_err(|e| OrliqError::from(anyhow::anyhow!("focused-window join: {e}")))?;
    Ok(FocusedWindow {
        exe: ctx.exe,
        title: ctx.title,
        class: ctx.class,
    })
}
