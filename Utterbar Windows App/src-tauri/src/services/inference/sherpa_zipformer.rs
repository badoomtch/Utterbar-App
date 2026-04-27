//! Zipformer provider backed by sherpa-onnx.
//!
//! Same async/threading model as the other providers: recogniser is held
//! in `Arc<Mutex<Option<...>>>`, all blocking work runs through
//! `tokio::task::spawn_blocking`. Mutex never crosses an `.await`.
//!
//! Differences vs Parakeet / Whisper:
//!
//! * `ZipFormer::decode` takes `Vec<f32>` by value (consumes the buffer)
//!   and returns a plain `String` — no `OfflineRecognizerResult` wrapper.
//!   Simpler than the Whisper path; no detected-language field to thread
//!   through.
//! * Zipformer ships English-only here (the streaming bilingual variants
//!   would slot in as separate catalogue entries when needed).
//! * Same encoder/decoder/joiner+tokens file shape as Parakeet, so the
//!   resolver reuses Parakeet's prefix-match approach.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;

use crate::services::inference::provider::{AsrOptions, AsrProvider, AsrResult, AudioBuffer};

type Recognizer = Option<sherpa_rs::zipformer::ZipFormer>;

pub struct SherpaZipformer {
    model_id: String,
    model_dir: PathBuf,
    provider: String,
    inner: Arc<Mutex<Recognizer>>,
}

impl SherpaZipformer {
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
                tracing::info!(model = %self.model_id, "zipformer::ensure_loaded cache hit");
                return Ok(());
            }
        }

        tracing::info!(model = %self.model_id, dir = %self.model_dir.display(), "zipformer::ensure_loaded begin");
        let (encoder, decoder, joiner, tokens) = self.resolve_files()?;
        let encoder = encoder.to_string_lossy().into_owned();
        let decoder = decoder.to_string_lossy().into_owned();
        let joiner = joiner.to_string_lossy().into_owned();
        let tokens = tokens.to_string_lossy().into_owned();

        let inner = self.inner.clone();
        let model_id = self.model_id.clone();
        let provider = self.provider.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            tracing::info!(%model_id, %provider, "zipformer: building ZipFormerConfig");
            let config = sherpa_rs::zipformer::ZipFormerConfig {
                encoder,
                decoder,
                joiner,
                tokens,
                num_threads: Some(2),
                provider: Some(provider.clone()),
                ..Default::default()
            };
            tracing::info!(%model_id, "zipformer: ZipFormer::new calling into native");
            let recognizer = sherpa_rs::zipformer::ZipFormer::new(config).map_err(|e| {
                tracing::error!(%model_id, error = ?e, "zipformer init failed");
                anyhow!("zipformer init: {e:?}")
            })?;
            tracing::info!(%model_id, "zipformer: recognizer constructed");
            let mut guard = inner
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            *guard = Some(recognizer);
            Ok(())
        })
        .await
        .context("zipformer warm spawn_blocking failed")??;

        Ok(())
    }
}

#[async_trait]
impl AsrProvider for SherpaZipformer {
    fn id(&self) -> &str {
        &self.model_id
    }

    fn supported_languages(&self) -> &[&'static str] {
        // Catalogue ships English Zipformer entries today; bilingual /
        // multilingual zipformer variants would be added as separate
        // catalogue rows with their own language vectors.
        &["en"]
    }

    async fn warm(&self) -> Result<()> {
        self.ensure_loaded().await
    }

    async fn transcribe(&self, audio: AudioBuffer, _opts: AsrOptions) -> Result<AsrResult> {
        tracing::info!(model = %self.model_id, samples = audio.pcm.len(), sample_rate = audio.sample_rate, "zipformer::transcribe enter");
        self.ensure_loaded().await?;

        let sample_rate = audio.sample_rate;
        let pcm = audio.pcm;
        let duration_ms = (pcm.len() as u64 * 1000 / sample_rate.max(1) as u64) as u32;
        let inner = self.inner.clone();
        let model_id = self.model_id.clone();
        let model_id_task = model_id.clone();

        let result = tokio::task::spawn_blocking(move || -> Result<String> {
            let model_id = model_id_task;
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

            if samples.is_empty() {
                tracing::warn!(%model_id, "zipformer: empty audio buffer, returning empty string");
                return Ok(String::new());
            }

            // Same silence + duration gates as Parakeet/Whisper.
            // Zipformer is gentler than Whisper on silence (it produces
            // empty output rather than confabulating) but the gates
            // are cheap and consistent across providers.
            const MIN_DURATION_MS: u32 = 180;
            const SILENCE_RMS_THRESHOLD: f32 = 0.004;
            const MARGINAL_RMS_THRESHOLD: f32 = 0.010;
            let rms_sq: f64 = samples.iter().map(|s| (*s as f64) * (*s as f64)).sum::<f64>()
                / samples.len() as f64;
            let rms = rms_sq.sqrt() as f32;
            if rms < SILENCE_RMS_THRESHOLD {
                tracing::info!(%model_id, rms, "zipformer: audio below silence threshold, returning empty string");
                return Ok(String::new());
            }
            if duration_ms < MIN_DURATION_MS && rms < MARGINAL_RMS_THRESHOLD {
                tracing::info!(%model_id, rms, duration_ms, "zipformer: too short + too quiet, returning empty string");
                return Ok(String::new());
            }

            tracing::info!(%model_id, samples = samples.len(), sample_rate, rms, "zipformer: calling recognizer.decode (native)");
            let text = recognizer.decode(sample_rate, samples);
            tracing::info!(%model_id, chars = text.len(), "zipformer: recognizer.decode returned");
            Ok(text)
        })
        .await
        .context("zipformer transcribe spawn_blocking failed")?;

        let text = result?;
        tracing::info!(model = %model_id, duration_ms, chars = text.len(), "zipformer transcribe done");

        // The k2-fsa English Zipformer bundle ships an ALL-CAPS token
        // vocabulary — the model can ONLY emit uppercase tokens. The
        // raw output reads as a shouting match. We lowercase the result
        // before returning it; the standard auto-capitalise pass in
        // dictation.rs (TextOutputSettings.auto_capitalise) then puts
        // a single capital at the start of the sentence, matching what
        // Parakeet and Whisper produce naturally.
        //
        // We use to_ascii_lowercase rather than to_lowercase because
        // the Zipformer English vocabulary is ASCII-only (no accents,
        // no Unicode). ASCII is faster and avoids Unicode locale
        // surprises.
        let normalised = text.trim().to_ascii_lowercase();

        Ok(AsrResult {
            text: normalised,
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
