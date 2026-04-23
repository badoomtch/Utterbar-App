use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::services::files::FileJob;
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn enqueue_file(
    state: State<'_, Arc<AppState>>,
    source_path: String,
    mode_id: Option<String>,
    model_id: Option<String>,
) -> CmdResult<FileJob> {
    state
        .files
        .enqueue(source_path, mode_id, model_id)
        .map_err(OrliqError::from)
}

#[tauri::command]
pub async fn list_jobs(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<FileJob>> {
    state.files.list().map_err(OrliqError::from)
}

#[tauri::command]
pub async fn cancel_job(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<()> {
    state.files.cancel(&id).await.map_err(OrliqError::from)
}
