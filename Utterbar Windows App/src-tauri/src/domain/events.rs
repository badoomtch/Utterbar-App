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
    Warming {
        model_id: String,
    },
    Listening {
        started_at_ms: i64,
    },
    Processing,
    /// Meeting-mode-specific phase, emitted between capture-drain and
    /// the final Success. Indistinguishable from Processing for the
    /// recogniser's perspective — the chunked transcribe is running —
    /// but the bar shows "Transcribing meeting…" instead of
    /// "Processing", and the user can start a new recording during
    /// this window because the orchestrator's transcribe-in-flight
    /// guard has been released early. The new capture's eventual
    /// transcribe will queue behind the in-flight one via the
    /// orchestrator's existing write-lock.
    TranscribingBackground,
    /// Recording finished cleanly. Optional `summary` is populated for
    /// meeting-mode recordings so the floating bar can show
    /// "5:01 · 11 chunks · 799 chars" for ~3 s before decaying to idle —
    /// without it, a 5-minute meeting and a 3-second dictation look
    /// identical in the bar (both transition Listening → Processing →
    /// Success). Default None for ordinary dictations so the existing
    /// "Inserted" treatment is unchanged.
    Success {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        summary: Option<DictationSuccessSummary>,
    },
    Error {
        message: String,
    },
    Cancelled,
    /// The capture completed but produced no usable speech — either silence
    /// the gate caught before ASR, or the ASR returned an empty / all-filler
    /// string we declined to inject. Distinct from `Error` because it's not
    /// a failure to handle, it's a non-event — the bar shows a neutral
    /// "No audio detected" message and settles back to idle without writing
    /// a history row or firing the success cue.
    NoAudio,
}

/// Payload appended to a `Success` phase event for meeting-mode
/// recordings. The floating bar uses these fields to render a
/// substantive success pill ("5:01 · 11 chunks · 799 chars") instead of
/// the generic "Inserted" copy. None of the fields are required to
/// reconstruct the history row — the Rust side has all of that — they
/// exist purely so the UI can paint the success state without an
/// extra IPC round-trip.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct DictationSuccessSummary {
    /// Wall-clock duration of the captured audio, in seconds. Rendered
    /// as `mm:ss` in the bar.
    pub duration_seconds: u32,
    /// Number of chunks the meeting was split into. 1 for non-meeting
    /// recordings (and they shouldn't carry a summary at all in that
    /// case, but the field is here so meetings that happened to fit
    /// inside a single chunk still produce a coherent payload).
    pub chunks: u32,
    /// Character count of the final stitched transcript. Rough proxy
    /// for "is this a real meeting or two seconds of silence" the user
    /// can read at a glance.
    pub chars: u32,
    /// Short, human-readable name of the ASR model used for this
    /// meeting (v0.15.3+). Examples: `"Whisper Small"`, `"Whisper
    /// Medium"`, `"Whisper Large v3"`. Rendered in the success pill
    /// between the time and the chunk count so the user can correlate
    /// quality vs speed across meetings without opening logs.
    pub model_label: String,
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
