use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct Mode {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub is_builtin: bool,
    pub is_favourite: bool,
    pub sort_order: i64,
    pub asr_model_id: Option<String>,
    pub target_language: Option<String>,
    pub post_processing: PostProcessing,
    pub context_rules: ContextRules,
    pub auto_activation: Vec<AutoActivationRule>,
    pub shortcut_override: Option<crate::domain::Shortcut>,
    pub formatting: Formatting,
    /// English dialect to normalise the transcript into. Parakeet itself is a
    /// monolingual English model with no locale switch, and its training set
    /// leans North American — so if the user wants British spellings we have
    /// to swap them in post-processing. Deterministic dictionary swap runs in
    /// microseconds and is safe on any transcript.
    #[serde(default)]
    pub english_variant: EnglishVariant,
    /// Treat this mode as a meeting capture. When true, `start_recording`
    /// overrides the global capture_source to Mixed (mic + system audio)
    /// for the duration of the recording, and `stop_recording` slices the
    /// captured PCM into ~30 s windows and transcribes each window
    /// against the mode's offline ASR model rather than handing the whole
    /// buffer in one call. The chunked path bypasses the 15-min offline
    /// cap because no single transcribe call sees more than 30 s of
    /// audio. Post-processing is skipped — meetings want raw transcript,
    /// not LLM polish.
    #[serde(default)]
    pub is_meeting_mode: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Which English dialect the transcript should read as.
/// * `Auto` — don't touch. Whatever Parakeet emitted stands.
/// * `American` — normalise British spellings to American (`colour` → `color`,
///   `organise` → `organize`). Rarely useful given the model's bias, but
///   completes the axis.
/// * `British` — normalise American spellings to British (`color` → `colour`,
///   `organize` → `organise`, `favorite` → `favourite`). The common case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum EnglishVariant {
    #[default]
    Auto,
    American,
    British,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PostProcessing {
    None,
    Deterministic {
        template: String,
    },
    Llm {
        prompt: String,
        template: Option<String>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct ContextRules {
    pub use_clipboard: bool,
    pub use_selection: bool,
    pub use_active_app: bool,
}

// The `By*` prefix is deliberate: it mirrors the serde tag values (`by_exe`,
// `by_title`, `by_class`) surfaced in persisted JSON and in the ts-rs contract.
// Renaming would break on-disk modes and the frontend types — not worth it for a
// stylistic lint.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(tag = "match", rename_all = "snake_case")]
pub enum AutoActivationRule {
    ByExe { pattern: String },
    ByTitle { pattern: String },
    ByClass { pattern: String },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct Formatting {
    pub capitalisation: Capitalisation,
    pub strip_fillers: bool,
    pub auto_punctuation: bool,
    pub bullet_style: BulletStyle,
    pub preserve_linebreaks: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum Capitalisation {
    #[default]
    Sentence,
    None,
    Title,
    Upper,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum BulletStyle {
    #[default]
    None,
    Dash,
    Asterisk,
    Numbered,
}
