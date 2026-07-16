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
/// Key-up for a bound global shortcut. Only the hold-to-record styles
/// (push-to-talk / hold-to-talk) act on this; every other shortcut is a
/// one-shot on key-down and ignores it. Emitted for all shortcuts on
/// release; the frontend filters.
pub const EVT_SHORTCUT_RELEASED: &str = "shortcut:released";
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
/// Fired during chunked Meeting Mode transcription. The payload reports
/// progress as `chunk_index` (0-based, of the chunk about to be
/// transcribed) and `total_chunks` (precomputed at the top of
/// `transcribe_chunked`). Emitted at the START of each chunk iteration
/// so a long-running model can paint partial progress instead of leaving
/// the bar stuck on "Transcribing meeting…" for minutes. Single-chunk
/// fast path (`total <= chunk_samples`) deliberately does NOT emit:
/// nothing meaningful to plot, and the existing phase pill already
/// covers the case. Regular dictation never enters this code path so
/// non-meeting recordings never see chunk-progress events.
pub const EVT_DICTATION_CHUNK_PROGRESS: &str = "dictation:chunk-progress";
/// Experimental Teams + Discord mic-mute status event. Emitted at four
/// load-bearing moments: when the start-time mute keystroke is
/// attempted, when the stop-time restore keystroke succeeds, when the
/// stop-time restore fails, and when restore-on-launch fails for a
/// session that crashed mid-dictation. Payload shape pinned in
/// `MicMuteStatus` so the floating bar's banner has a single source
/// of truth for which apps were targeted and the user-facing copy.
///
/// The payload deliberately carries NONE of: window titles, process
/// IDs, hotkey strings, transcripts, paths, or PIDs. The banned-fields
/// snapshot test in `services/mic_mute.rs` covers the persistence
/// file; the same banned set applies here.
pub const EVT_MIC_MUTE_STATUS: &str = "mic-mute:status";

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
    ///
    /// Optional `notice` is populated for stop paths that produced a
    /// successful transcript but want to flag a non-error condition
    /// the user should see, e.g. the single-shot auto-stop at the
    /// 5-minute dictation cap. Distinct from `summary` because the
    /// notice fires for non-meeting recordings (which never have a
    /// summary) and distinct from `Error` because the recording did
    /// produce a transcript and the user got their text inserted.
    Success {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        summary: Option<DictationSuccessSummary>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        notice: Option<String>,
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
    ///
    /// Optional `notice` carries a short label when the drop was a
    /// validator-noticed case (repeated token, repeated punctuation,
    /// unexpected foreign script) rather than plain silence. The
    /// floating bar swaps the default "No audio detected" pill for
    /// the notice text, in the same muted tone - non-error, non-alarming.
    /// Quiet drops (empty/whitespace/single punctuation) leave this None
    /// so the existing UX is unchanged.
    NoAudio {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        notice: Option<String>,
    },
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

/// Progress payload for chunked Meeting Mode transcription. See the
/// `EVT_DICTATION_CHUNK_PROGRESS` constant for emission semantics.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct MeetingChunkProgress {
    /// 0-based index of the chunk about to be transcribed.
    pub chunk_index: u32,
    /// Total number of chunks the recording was split into. Precomputed
    /// up front in `transcribe_chunked` via `projected_chunk_count`,
    /// matches the eventual `DictationSuccessSummary::chunks` count.
    pub total_chunks: u32,
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

/// Phase of a `MicMuteStatus` event. Closed enum so the React banner
/// can switch on it cleanly, and so adding a phase requires a TS
/// regeneration that surfaces the change to the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum MicMutePhase {
    /// We attempted to send the mute keystroke at recording start.
    /// Banner shows the active "Tried to mute X while recording" sub-line.
    RecordingStarted,
    /// All restore keystrokes returned `Ok` from `SendInput`. The
    /// banner shows the success copy and auto-dismisses after 5 s.
    RestoreSucceeded,
    /// One or more restore keystrokes failed (process gone, send
    /// errored, no hotkey). Sticky banner + tray notification.
    RestoreFailed,
    /// Restore was attempted at app boot from a persisted state file.
    /// Always treated as a failure-grade event because the user has
    /// been muted unknowingly for some interval; we surface the banner
    /// regardless of keystroke outcome so they look at the call apps.
    RestoreOnLaunchFailed,
}

/// One per-app entry in a `MicMuteStatus` payload.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct MicMuteStatusApp {
    /// Closed-string app id matching the persistence schema in
    /// `services/mic_mute.rs::MicMuteApp`. Currently `"teams"` or
    /// `"discord"`.
    pub app: String,
    /// Display label the banner uses verbatim. `"Teams"` or `"Discord"`.
    pub label: String,
    /// Outcome string: `"applied"` / `"restored"` / `"not_running"` /
    /// `"send_failed"` / `"no_hotkey"`. Closed set; the React layer
    /// matches on these strings to pick banner copy. Optional because
    /// the `RecordingStarted` phase predates any outcome.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
}

/// Payload for `EVT_MIC_MUTE_STATUS`. Small by design, no secrets.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct MicMuteStatus {
    pub phase: MicMutePhase,
    pub apps: Vec<MicMuteStatusApp>,
    /// Short human-readable copy the banner can show as a fallback if
    /// the React layer wants to render the message verbatim rather
    /// than build it from per-app entries. The Rust side composes this
    /// once at emit time so all consumers see the same copy.
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct FileJobProgress {
    pub id: String,
    pub progress: f32,
    pub status: String,
}
