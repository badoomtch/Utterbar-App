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
    /// Accuracy score on a 1–10 scale, relative to other models in the
    /// catalogue. Editorial judgement, not a measured WER — the absolute
    /// scale is unstable (Parakeet variants are clustered in a tight band)
    /// and users care about "which of these is better for me" rather than
    /// a WER percentage they can't interpret. Used by the Settings → Models
    /// and Onboarding pickers to render paired accuracy/speed comparison
    /// bars next to each option. 0 means "no comparison data available" —
    /// the bars hide in that case.
    pub accuracy_score: u8,
    /// Inference speed on a 1–10 scale, same semantics as `accuracy_score`.
    /// Factors in quantisation (int8 vs fp16) and the cost of the GPU path
    /// on machines that have one — a GPU-only fp16 model scores lower here
    /// because most installs will fall back to CPU and run slowly (or fail
    /// to load at all).
    pub speed_score: u8,
    /// Pro-gated model. The Rust side rejects `install_model` with a
    /// `pro_required` licence error; the frontend shows a `<ProNudge>`
    /// under the card and disables the Install button. fp16 carries this
    /// flag because the accuracy-ceiling experience is part of the paid
    /// promise — plus it's the only variant that needs a real GPU, so
    /// pairing the gate with the hardware reality keeps the funnel honest.
    #[serde(default)]
    pub requires_pro: bool,
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
