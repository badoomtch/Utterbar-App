use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct VocabEntry {
    pub id: String,
    pub word: String,
    pub hint: Option<String>,
    pub weight: f32,
    pub enabled: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct Replacement {
    pub id: String,
    pub pattern: String,
    pub replacement: String,
    pub is_regex: bool,
    pub case_sens: bool,
    pub enabled: bool,
    pub sort_order: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct VocabularyImport {
    pub words: Vec<VocabEntry>,
    pub replacements: Vec<Replacement>,
}
