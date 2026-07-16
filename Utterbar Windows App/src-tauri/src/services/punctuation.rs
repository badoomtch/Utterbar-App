//! Optional punctuation restoration via sherpa-onnx's offline
//! CT-Transformer. Off by default; driven by
//! `text_output.restore_punctuation` and only active when the punct
//! model (`sherpa-punct-ct-transformer`) is installed.
//!
//! This adds punctuation (commas, full stops, question marks) to a raw
//! transcript with a token classifier. It is NOT an LLM rewrite — it
//! never generates or reorders words, so it stays inside the "no LLM
//! rewrite of your speech" product line.
//!
//! Fail-safe: every failure path (empty input, model missing, native
//! error) returns the input unchanged, so the caller can apply it
//! unconditionally without risking dropped text.

use std::path::Path;
use std::sync::Mutex;

use anyhow::{anyhow, Result};

/// Lazily-loaded sherpa punctuation instance. `sherpa_rs::Punctuation`
/// is `Send + Sync`, but `add_punctuation` takes `&mut self` (it calls
/// into native code), so the instance lives behind a `Mutex`. It loads
/// on first use from the installed model directory and stays resident;
/// a change of model path triggers a reload.
#[derive(Default)]
pub struct PunctuationService {
    inner: Mutex<Option<Loaded>>,
}

struct Loaded {
    model_path: String,
    punct: sherpa_rs::punctuate::Punctuation,
}

impl PunctuationService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Restore punctuation on `text`, loading the model from `model_dir`
    /// on first use. Returns the input unchanged on any failure.
    pub fn restore(&self, model_dir: &Path, text: &str) -> String {
        if text.trim().is_empty() {
            return text.to_string();
        }
        match self.ensure_and_run(model_dir, text) {
            Ok(out) if !out.trim().is_empty() => out,
            Ok(_) => {
                // Native call returned empty; never replace real text with nothing.
                text.to_string()
            }
            Err(e) => {
                tracing::warn!(?e, "punctuation restore failed; returning text unchanged");
                text.to_string()
            }
        }
    }

    fn ensure_and_run(&self, model_dir: &Path, text: &str) -> Result<String> {
        let model_path = resolve_model_file(model_dir)?;
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| anyhow!("punctuation mutex poisoned"))?;

        let need_reload = guard
            .as_ref()
            .map(|l| l.model_path != model_path)
            .unwrap_or(true);
        if need_reload {
            let punct =
                sherpa_rs::punctuate::Punctuation::new(sherpa_rs::punctuate::PunctuationConfig {
                    model: model_path.clone(),
                    debug: false,
                    num_threads: Some(1),
                    // CPU only: the model is tiny, and this keeps it off
                    // the DirectML path entirely.
                    provider: Some("cpu".into()),
                })
                .map_err(|e| anyhow!("punctuation init: {e:?}"))?;
            *guard = Some(Loaded { model_path, punct });
        }

        // Safe: we just ensured `Some`.
        let loaded = guard.as_mut().expect("punctuation loaded");
        Ok(loaded.punct.add_punctuation(text))
    }
}

/// Resolve the CT-Transformer ONNX file inside an installed punct model
/// directory. Prefers the int8 weights (what the catalogue ships).
fn resolve_model_file(dir: &Path) -> Result<String> {
    for name in ["model.int8.onnx", "model.onnx"] {
        let p = dir.join(name);
        if p.is_file() {
            return Ok(p.to_string_lossy().into_owned());
        }
    }
    Err(anyhow!(
        "no punctuation model (model.int8.onnx / model.onnx) in {}",
        dir.display()
    ))
}
