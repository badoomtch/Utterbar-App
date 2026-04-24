//! Real Parakeet-TDT provider backed by sherpa-onnx.
//!
//! Thread/async model:
//!
//! * The recognizer is held in an `Arc<std::sync::Mutex<Option<...>>>` so it can be
//!   moved into `tokio::task::spawn_blocking` callbacks without contending with the
//!   async runtime. sherpa-rs declares its recognizer `Send + Sync`.
//! * Both warm-up and transcribe run through `spawn_blocking`, which works on every
//!   Tokio runtime (multi-thread and current-thread). The previous version used
//!   `tokio::task::block_in_place`, which panics on a current-thread runtime — almost
//!   certainly the crash the user saw mid-transcribe.
//! * We never hold the mutex across an `.await`, which would deadlock if another task
//!   on the same thread tried to lock it.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;

use crate::services::inference::provider::{AsrOptions, AsrProvider, AsrResult, AudioBuffer};

type Recognizer = Option<sherpa_rs::transducer::TransducerRecognizer>;

pub struct SherpaParakeet {
    model_id: String,
    model_dir: PathBuf,
    provider: String,
    inner: Arc<Mutex<Recognizer>>,
}

impl SherpaParakeet {
    pub fn new(model_id: String, model_dir: PathBuf, provider: String) -> Self {
        Self {
            model_id,
            model_dir,
            provider,
            inner: Arc::new(Mutex::new(None)),
        }
    }

    fn resolve_files(&self) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf)> {
        let encoder = first_matching(&self.model_dir, "encoder")?;
        let decoder = first_matching(&self.model_dir, "decoder")?;
        let joiner = first_matching(&self.model_dir, "joiner")?;
        let tokens = self.model_dir.join("tokens.txt");
        if !tokens.exists() {
            return Err(anyhow!(
                "tokens.txt not found in {}",
                self.model_dir.display()
            ));
        }
        Ok((encoder, decoder, joiner, tokens))
    }

    async fn ensure_loaded(&self) -> Result<()> {
        {
            let guard = self
                .inner
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            if guard.is_some() {
                tracing::info!(model = %self.model_id, "sherpa::ensure_loaded cache hit");
                return Ok(());
            }
        }

        tracing::info!(model = %self.model_id, dir = %self.model_dir.display(), "sherpa::ensure_loaded begin");
        let (encoder, decoder, joiner, tokens) = self.resolve_files()?;
        tracing::info!(
            encoder = %encoder.display(),
            decoder = %decoder.display(),
            joiner = %joiner.display(),
            tokens = %tokens.display(),
            "sherpa model files resolved"
        );

        // fp16 encoders only decode sanely on a GPU provider (DirectML / CUDA). On CPU
        // the ONNX runtime either refuses half-precision kernels or grinds to 0.1x
        // realtime, so we refuse to load the combo with a human-readable error instead
        // of letting the user wait 30 seconds for every utterance.
        let is_fp16 =
            has_fp16_suffix(&encoder) || has_fp16_suffix(&decoder) || has_fp16_suffix(&joiner);
        if is_fp16 && self.provider == "cpu" {
            return Err(anyhow!(
                "This model ships half-precision (fp16) weights that need GPU acceleration. \
                 Turn on GPU acceleration in Settings → Advanced, or install the compact \
                 (int8) Parakeet model which runs full speed on any modern CPU."
            ));
        }
        let encoder = encoder.to_string_lossy().into_owned();
        let decoder = decoder.to_string_lossy().into_owned();
        let joiner = joiner.to_string_lossy().into_owned();
        let tokens = tokens.to_string_lossy().into_owned();

        let inner = self.inner.clone();
        let model_id = self.model_id.clone();
        let provider = self.provider.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            tracing::info!(%model_id, %provider, "sherpa: building TransducerConfig");
            let config = sherpa_rs::transducer::TransducerConfig {
                decoder,
                encoder,
                joiner,
                tokens,
                num_threads: 2,
                sample_rate: 16_000,
                feature_dim: 80,
                decoding_method: "greedy_search".into(),
                // Parakeet TDT is NeMo's Token-Duration-Transducer — a NeMo-specific
                // transducer variant with different exported metadata (no vocab_size
                // attribute, a "model_type=EncDecRNNTBPEModel" attribute, etc). Passing
                // "transducer" makes sherpa-onnx look up the wrong metadata keys, which
                // triggers std::terminate in the native layer because the lookup throws
                // before any C++ exception can cross the FFI boundary. "nemo_transducer"
                // is the correct key for Parakeet TDT models.
                model_type: "nemo_transducer".into(),
                provider: Some(provider.clone()),
                debug: false,
                ..Default::default()
            };
            tracing::info!(%model_id, "sherpa: TransducerRecognizer::new calling into native");
            let recognizer =
                sherpa_rs::transducer::TransducerRecognizer::new(config).map_err(|e| {
                    tracing::error!(%model_id, error = ?e, "sherpa init failed");
                    anyhow!("sherpa init: {e:?}")
                })?;
            tracing::info!(%model_id, "sherpa: recognizer constructed");
            let mut guard = inner
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            *guard = Some(recognizer);
            tracing::info!(%model_id, "sherpa: recognizer cached");
            Ok(())
        })
        .await
        .context("sherpa warm spawn_blocking failed")??;

        Ok(())
    }
}

#[async_trait]
impl AsrProvider for SherpaParakeet {
    fn id(&self) -> &str {
        &self.model_id
    }

    fn supported_languages(&self) -> &[&'static str] {
        &["en"]
    }

    async fn warm(&self) -> Result<()> {
        self.ensure_loaded().await
    }

    async fn transcribe(&self, audio: AudioBuffer, _opts: AsrOptions) -> Result<AsrResult> {
        tracing::info!(model = %self.model_id, samples = audio.pcm.len(), sample_rate = audio.sample_rate, "sherpa::transcribe enter");
        self.ensure_loaded().await?;

        // Convert to f32 off the Tokio runtime — this allocates O(duration) memory and
        // shouldn't run on an async worker thread.
        let sample_rate = audio.sample_rate;
        let pcm = audio.pcm;
        let duration_ms = (pcm.len() as u64 * 1000 / sample_rate.max(1) as u64) as u32;
        let inner = self.inner.clone();
        let model_id = self.model_id.clone();
        let model_id_task = model_id.clone();

        let duration_ms_captured = duration_ms;
        let result = tokio::task::spawn_blocking(move || -> Result<String> {
            let model_id = model_id_task;
            let duration_ms = duration_ms_captured;
            tracing::info!(%model_id, samples = pcm.len(), "sherpa: converting i16 -> f32");
            let samples: Vec<f32> = pcm
                .into_iter()
                .map(|s| s as f32 / i16::MAX as f32)
                .collect();

            let mut guard = inner
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            let recognizer = guard
                .as_mut()
                .ok_or_else(|| anyhow!("recognizer not loaded"))?;

            // Defensive: sherpa panics on empty input. Short-circuit so we don't bring
            // the whole process down for a silent recording.
            if samples.is_empty() {
                tracing::warn!(%model_id, "sherpa: empty audio buffer, returning empty string");
                return Ok(String::new());
            }

            // Silence / hallucination guard. Parakeet-TDT fabricates short filler
            // tokens ("yeah", "so", "okay", "mm") when fed near-silent or noise-floor
            // audio — the decoder always emits *something*, and with no vocal content
            // the highest-probability path is a common English filler that matches
            // the pre-emphasis + mel features of room noise. We pre-reject anything
            // that's either (a) too short to contain a word or (b) whose RMS energy
            // is below the threshold the VAD/noise-gate would have trimmed.
            //
            // RMS thresholds are in normalised-sample units (i16 → f32 / i16::MAX):
            //   • < 0.004 — essentially silence (room tone, AC hum). Human speech at
            //     normal distance from a mic clears 0.02–0.05 even on quiet syllables.
            //   • 0.004–0.010 — marginal; return empty so the user can retry rather
            //     than get a phantom word pasted into their focused app.
            //
            // Duration floor is 180 ms — shorter than the "Ctrl+Space tap, oh wait"
            // accidental press that motivated this fix, but long enough to contain
            // a real single-syllable word ("yes", "no", "stop"). Any recording
            // shorter than 180 ms with RMS < 0.010 is treated as "nothing spoken".
            const MIN_DURATION_MS: u32 = 180;
            const SILENCE_RMS_THRESHOLD: f32 = 0.004;
            const MARGINAL_RMS_THRESHOLD: f32 = 0.010;
            let rms_sq: f64 = samples.iter().map(|s| (*s as f64) * (*s as f64)).sum::<f64>()
                / samples.len() as f64;
            let rms = rms_sq.sqrt() as f32;
            if rms < SILENCE_RMS_THRESHOLD {
                tracing::info!(%model_id, rms, "sherpa: audio below silence threshold, returning empty string");
                return Ok(String::new());
            }
            if duration_ms < MIN_DURATION_MS && rms < MARGINAL_RMS_THRESHOLD {
                tracing::info!(%model_id, rms, duration_ms, "sherpa: too short + too quiet, likely accidental press, returning empty string");
                return Ok(String::new());
            }

            tracing::info!(%model_id, samples = samples.len(), sample_rate, rms, "sherpa: calling recognizer.transcribe (native)");
            let text = recognizer.transcribe(sample_rate, &samples);
            tracing::info!(%model_id, chars = text.len(), "sherpa: recognizer.transcribe returned");

            // Post-transcribe hallucination filter. Even when audio clears the RMS
            // gate, Parakeet-TDT fabricates single-filler outputs when the signal is
            // mostly breath/room noise with a brief "something" that wasn't speech.
            // Drop the text if the model returned exactly one short filler token AND
            // the input was quiet — that combination is the fingerprint of
            // hallucination, not a real single-word utterance. Real words like "yes"
            // / "no" / "stop" have RMS well above the marginal threshold (the user
            // is deliberately speaking), so this filter does not eat real single-
            // word dictations. Kept simple: case-insensitive exact-match of the
            // trimmed output against a known-fillers list.
            // Kept deliberately short — no "yes" / "no" / "okay" because a real
            // deliberate single-word dictation of those clears the 0.010 RMS gate
            // comfortably, and listing them here would eat legitimate input. The
            // list is fillers that have NO meaning as a stand-alone dictation.
            const HALLUCINATION_FILLERS: &[&str] = &[
                "yeah", "so", "mm", "mhm", "uh", "uhh", "um", "hmm", "oh", "ah", "mm-hmm",
            ];
            let trimmed = text.trim().to_ascii_lowercase();
            let trimmed = trimmed.trim_end_matches(['.', ',', '!', '?']);
            if rms < MARGINAL_RMS_THRESHOLD && HALLUCINATION_FILLERS.contains(&trimmed) {
                tracing::info!(%model_id, rms, returned = %text, "sherpa: dropping likely hallucinated filler on quiet input");
                return Ok(String::new());
            }

            Ok(text)
        })
        .await
        .context("sherpa transcribe spawn_blocking failed")?;

        let text = result?;
        tracing::info!(model = %model_id, duration_ms, chars = text.len(), "parakeet transcribe done");

        Ok(AsrResult {
            text: text.trim().to_string(),
            words: None,
            language: Some("en".into()),
            confidence: None,
            model_version: model_id,
            duration_ms,
        })
    }

    async fn unload(&self) -> Result<()> {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = None;
        }
        Ok(())
    }
}

fn has_fp16_suffix(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_ascii_lowercase().contains(".fp16."))
        .unwrap_or(false)
}

fn first_matching(dir: &Path, prefix: &str) -> Result<PathBuf> {
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("read_dir {}", dir.display()))?
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with(prefix) && name.ends_with(".onnx") {
            return Ok(path);
        }
    }
    Err(anyhow!("no {prefix}*.onnx file in {}", dir.display()))
}
