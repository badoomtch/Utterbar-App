use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A keyboard shortcut. Stored as an accelerator string (`"Ctrl+Shift+O"`) plus a friendly label.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct Shortcut {
    pub accelerator: String,
    pub label: String,
    pub trigger_style: TriggerStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum TriggerStyle {
    Toggle,
    PushToTalk,
    HoldToTalk,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct ShortcutSlot {
    pub id: ShortcutId,
    pub label: String,
    pub current: Option<Shortcut>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum ShortcutId {
    ToggleRecording,
    PushToTalk,
    HoldToTalk,
    CancelRecording,
    CycleMode,
    OpenSettings,
    OpenHistory,
    /// Quick add-to-vocabulary. When pressed we synthesise Ctrl+C against the
    /// foreground app, read the resulting clipboard, restore the user's
    /// previous clipboard, and open the small vocab-add window prefilled
    /// with the captured text. Ships unbound by default — there is no
    /// generally-safe accelerator that doesn't risk colliding with another
    /// app's binding, so the user is invited to bind one from
    /// Settings → Shortcuts (or the prompt on the Vocabulary page).
    AddToVocabulary,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct ShortcutConflict {
    pub slot_a: ShortcutId,
    pub slot_b: ShortcutId,
    pub accelerator: String,
}
