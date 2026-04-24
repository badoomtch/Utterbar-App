use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::services::analytics::{DailyPoint, StatsSummary};
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn get_stats_summary(state: State<'_, Arc<AppState>>) -> CmdResult<StatsSummary> {
    state.analytics.summary().map_err(OrliqError::from)
}

#[tauri::command]
pub async fn get_stats_series(
    state: State<'_, Arc<AppState>>,
    metric: String,
    days: u32,
) -> CmdResult<Vec<DailyPoint>> {
    state
        .analytics
        .series(&metric, days)
        .map_err(OrliqError::from)
}
