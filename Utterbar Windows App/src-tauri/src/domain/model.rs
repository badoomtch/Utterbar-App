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
    /// Which sherpa-onnx recogniser class this model needs at load
    /// time. The inference layer dispatches on this so the catalogue
    /// can hold multiple ASR engine families side by side. Only meaningful
    /// when `kind == Asr`. LLM entries leave this at the default
    /// (`Parakeet`) and the field is ignored on that path.
    #[serde(default)]
    pub engine: ModelEngine,
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
    /// Offline punctuation + casing restoration (sherpa CT-Transformer).
    /// Not an ASR model — kept out of the dictation-model pickers; drives
    /// the optional "Restore punctuation" text-output pass.
    Punct,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "kebab-case")]
pub enum ModelBackend {
    Onnx,
    PySidecar,
    LlamaCpp,
}

/// ASR recogniser family. Each maps to a different sherpa-rs class at
/// load time. New engines land here once the inference dispatch knows
/// how to spin them up.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, Default, PartialEq, Eq)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "kebab-case")]
pub enum ModelEngine {
    /// NVIDIA Parakeet TDT via sherpa_rs::transducer::TransducerRecognizer.
    /// Today's only shipping engine and the default. English only.
    #[default]
    ParakeetTdt,
    /// OpenAI Whisper via sherpa_rs::whisper::WhisperRecognizer.
    /// Multilingual. Wired in the inference layer in v0.8.1 onwards.
    Whisper,
    /// Zipformer streaming via sherpa_rs::zipformer::ZipFormer.
    /// English, fast cold start. Wired in v0.8.2 onwards.
    Zipformer,
    /// Offline punctuation restoration via sherpa_rs::punctuate.
    /// Never built as an ASR provider; the punctuation service loads it
    /// directly. Present only so `ModelKind::Punct` catalogue entries
    /// have a non-lying engine value.
    Punct,
}
