use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::app_state::AppState;
use crate::domain::{DataPaths, Settings, EVT_SETTINGS_CHANGED};
use crate::util::paths;
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> CmdResult<Settings> {
    Ok(state.settings.snapshot().await)
}

#[tauri::command]
pub async fn update_settings(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    patch: serde_json::Value,
) -> CmdResult<Settings> {
    let updated = state
        .settings
        .update(|s| merge_patch(s, &patch))
        .await
        .map_err(OrliqError::from)?;
    apply_side_effects(&updated);
    let _ = handle.emit(EVT_SETTINGS_CHANGED, &updated);
    Ok(updated)
}

#[tauri::command]
pub async fn reset_settings(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> CmdResult<Settings> {
    let s = state.settings.reset().await.map_err(OrliqError::from)?;
    let _ = handle.emit(EVT_SETTINGS_CHANGED, &s);
    Ok(s)
}

#[tauri::command]
pub async fn get_data_paths() -> CmdResult<DataPaths> {
    Ok(DataPaths {
        config: paths::config_dir()?.to_string_lossy().into(),
        data: paths::db_dir()?.to_string_lossy().into(),
        models: paths::models_dir()?.to_string_lossy().into(),
        recordings: paths::recordings_dir()?.to_string_lossy().into(),
        logs: paths::logs_dir()?.to_string_lossy().into(),
    })
}

fn merge_patch(s: &mut Settings, patch: &serde_json::Value) {
    if let Ok(current) = serde_json::to_value(&*s) {
        let merged = merge(current, patch.clone());
        if let Ok(next) = serde_json::from_value::<Settings>(merged) {
            *s = next;
        }
    }
}

fn merge(mut a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    match (&mut a, b) {
        (serde_json::Value::Object(ao), serde_json::Value::Object(bo)) => {
            for (k, v) in bo {
                let existing = ao.remove(&k).unwrap_or(serde_json::Value::Null);
                ao.insert(k, merge(existing, v));
            }
            a
        }
        (_, other) => other,
    }
}

fn apply_side_effects(s: &Settings) {
    #[cfg(windows)]
    {
        let _ = crate::platform::windows::startup::set_launch_on_login(s.general.launch_on_login);
    }
    let _ = s; // no-op on non-Windows
}
