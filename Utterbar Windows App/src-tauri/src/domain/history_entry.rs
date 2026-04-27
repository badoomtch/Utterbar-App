use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct HistoryEntry {
    pub id: String,
    pub created_at: i64,
    pub mode_id: Option<String>,
    pub mode_name_snapshot: String,
    pub asr_model_id: String,
    pub asr_model_version: String,
    pub audio_path: Option<String>,
    pub audio_duration_ms: i64,
    pub raw_text: String,
    pub processed_text: String,
    pub language: Option<String>,
    pub processing_ms: i64,
    pub status: HistoryStatus,
    pub error_message: Option<String>,
    pub app_exe: Option<String>,
    pub app_title: Option<String>,
    pub context_payload: Option<String>,
    pub word_timings: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum HistoryStatus {
    Success,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct HistoryFilter {
    pub mode_id: Option<String>,
    pub app_exe: Option<String>,
    pub search: Option<String>,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    /// Filter by status. None means "any status" (the previous behaviour).
    /// Stored as a HistoryStatus to keep the wire shape symmetric with the
    /// row column.
    #[serde(default)]
    pub status: Option<HistoryStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
