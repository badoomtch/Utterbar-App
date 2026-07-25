use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::app_state::AppState;
use crate::domain::ModelInfo;
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn list_installed_models(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<ModelInfo>> {
    state.models.list_installed().map_err(OrliqError::from)
}

#[tauri::command]
pub async fn list_available_models(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<ModelInfo>> {
    Ok(state.models.list_available().await)
}

#[tauri::command]
pub async fn install_model(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> CmdResult<ModelInfo> {
    state
        .models
        .install(&handle, &id)
        .await
        .map_err(|e| OrliqError::Model {
            message: e.to_string(),
        })
}

#[tauri::command]
pub async fn remove_model(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<()> {
    state.models.remove(&id).map_err(OrliqError::from)
}

#[tauri::command]
pub async fn model_status(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> CmdResult<Option<ModelInfo>> {
    let installed = state.models.list_installed().map_err(OrliqError::from)?;
    Ok(installed.into_iter().find(|m| m.id == id))
}

/// Returns the id of the model currently resident in memory, or `None` if the
/// ASR orchestrator is cold. The Settings → Models page polls this so it can
/// render a subtle "Ready" (loaded) vs. "Idle" (cold) badge next to the
/// default-model picker — it explains why the first dictation after launch
/// feels slower than subsequent ones (1–3 s model load vs. warm inference).
#[tauri::command]
pub async fn asr_runtime_status(state: State<'_, Arc<AppState>>) -> CmdResult<Option<String>> {
    Ok(state.inference.loaded_model_id().await)
}
