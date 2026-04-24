//! Event payloads emitted over Tauri's event bus.
//! Event names are the canonical source of truth (see `src/lib/events.ts` in the frontend).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const EVT_DICTATION_PHASE: &str = "dictation:phase";
pub const EVT_DICTATION_METER: &str = "dictation:meter";
pub const EVT_DICTATION_RESULT: &str = "dictation:result";
pub const EVT_DICTATION_ERROR: &str = "dictation:error";
pub const EVT_MODEL_PROGRESS: &str = "model:progress";
pub const EVT_MODEL_CHANGED: &str = "model:changed";
pub const EVT_SETTINGS_CHANGED: &str = "settings:changed";
pub const EVT_SHORTCUT_TRIGGERED: &str = "shortcut:triggered";
pub const EVT_FILE_JOB_PROGRESS: &str = "file-job:progress";
/// Navigation intent for the main window. Emitted by `show_main_at` so the
/// floating bar's History / Settings buttons can land the user on the right
/// page instead of dumping them wherever they last were.
pub const EVT_MAIN_NAVIGATE: &str = "main:navigate";
/// Fired when `start_recording` refuses to arm a capture because the app
/// is already busy (a transcribe is in flight, or the re-entry guard is
/// still set from a previous recording). The FloatingBar listens for
/// this and plays a one-second border flash so the user gets a visible
/// "busy, wait" cue instead of pressing the hotkey into a silent void.
pub const EVT_DICTATION_BUSY: &str = "dictation:busy";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DictationPhase {
    Idle,
    Warming { model_id: String },
    Listening { started_at_ms: i64 },
    Processing,
    Success,
    Error { message: String },
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct MeterFrame {
    pub rms: f32,
    pub peak: f32,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct ModelProgress {
    pub model_id: String,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub phase: ModelProgressPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum ModelProgressPhase {
    Starting,
    Downloading,
    Verifying,
    Extracting,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct FileJobProgress {
    pub id: String,
    pub progress: f32,
    pub status: String,
}
