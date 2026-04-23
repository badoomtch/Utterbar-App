use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct ModelInfo {
    pub id: String,
    pub kind: ModelKind,
    pub display_name: String,
    /// Short human-readable rationale: "why would I pick this one?". Shown under each
    /// card in the onboarding + Settings pickers.
    pub description: String,
    pub version: String,
    pub size_bytes: u64,
    pub backend: ModelBackend,
    pub languages: Vec<String>,
    pub installed: bool,
    pub path: Option<String>,
    pub sha256: String,
    pub download_url: Option<String>,
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Asr,
    Llm,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "kebab-case")]
pub enum ModelBackend {
    Onnx,
    PySidecar,
    LlamaCpp,
}
