//! Streaming Zipformer provider.
//!
//! Implements `StreamingAsrProvider` (and the offline-fallback
//! `AsrProvider` for parity with the rest of the inference layer).
//! Backed by the safe `OnlineRecognizer` wrapper at
//! [`super::streaming::OnlineRecognizer`].
//!
//! Model on disk: extracted tarball at
//!   `<models_dir>/sherpa-streaming-zipformer-en/`
//! containing
//!   `encoder-epoch-99-avg-1-chunk-16-left-128.int8.onnx`
//!   `decoder-epoch-99-avg-1-chunk-16-left-128.int8.onnx`
//!   `joiner-epoch-99-avg-1-chunk-16-left-128.int8.onnx`
//!   `tokens.txt`
//!
//! See `services::models` catalogue entry `sherpa-streaming-zipformer-en`
//! for the download URL + pinned SHA-256.
//!
//! ## Concurrency model
//!
//! The recogniser is heavy (loads ONNX graphs into memory) and thread-
//! safe — held in `Arc<Mutex<Option<...>>>` for lazy init, same shape as
//! the other sherpa providers. Streams are per-recording; each
//! `start_session()` creates a new one and returns a boxed
//! `StreamingSession` impl that owns it.
//!
//! Audio chunks pushed into a session are i16 → f32 converted on the
//! caller's thread (cheap), then fed into the native `accept_waveform`.
//! Decoder advance + result extraction happen synchronously on whatever
//! thread is calling `accept_chunk` — this is fine because the chunk
//! sizes are small (100-500 ms typically) and sherpa's online inference
//! at this chunk size is real-time on any modern CPU.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;

use crate::services::inference::provider::{
    AsrOptions, AsrProvider, AsrResult, AudioBuffer, StreamingAsrProvider, StreamingPartial,
    StreamingSession,
};
use crate::services::inference::streaming::{
    OnlineRecognizer, OnlineRecognizerConfig, OnlineStream,
};

/// Streaming Zipformer recogniser. Holds the loaded `OnlineRecognizer`
/// and exposes both offline (full-buffer) and streaming (per-chunk)
/// transcription paths.
pub struct SherpaZipformerStreaming {
    model_id: String,
    model_dir: PathBuf,
    provider: String,
    inner: Arc<Mutex<Option<OnlineRecognizer>>>,
}

impl SherpaZipformerStreaming {
    pub fn new(model_id: String, model_dir: PathBuf, provider: String) -> Self {
        Self {
            model_id,
            model_dir,
            provider,
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Resolve the four ONNX/tokens files inside the model directory.
    ///
    /// The streaming Zipformer tarball ships BOTH fp32 (.onnx) and int8
    /// (.int8.onnx) variants of each tensor. We deliberately prefer
    /// .int8.onnx — smaller footprint, faster CPU inference, no
    /// measurable accuracy delta on streaming workloads. The fp32 files
    /// stay on disk after extract but go unused.
    fn resolve_files(&self) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf)> {
        let encoder = pick_int8(&self.model_dir, "encoder")?;
        let decoder = pick_int8(&self.model_dir, "decoder")?;
        let joiner = pick_int8(&self.model_dir, "joiner")?;
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
                tracing::info!(model = %self.model_id, "zipformer-streaming::ensure_loaded cache hit");
                return Ok(());
            }
        }

        tracing::info!(model = %self.model_id, dir = %self.model_dir.display(), "zipformer-streaming::ensure_loaded begin");
        let (encoder, decoder, joiner, tokens) = self.resolve_files()?;

        let cfg = OnlineRecognizerConfig {
            encoder: encoder.to_string_lossy().into_owned(),
            decoder: decoder.to_string_lossy().into_owned(),
            joiner: joiner.to_string_lossy().into_owned(),
            tokens: tokens.to_string_lossy().into_owned(),
            num_threads: 2,
            sample_rate: 16_000,
            feature_dim: 80,
            decoding_method: "greedy_search".into(),
            provider: self.provider.clone(),
            // Endpoint detection off — Utterbar uses push-to-talk, the
            // user decides when an utterance ends. See the design note
            // on OnlineRecognizerConfig::enable_endpoint.
            enable_endpoint: false,
            debug: false,
        };

        let inner = self.inner.clone();
        let model_id = self.model_id.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            tracing::info!(%model_id, "zipformer-streaming: OnlineRecognizer::new calling into native");
            let recognizer = OnlineRecognizer::new(cfg)
                .map_err(|e| anyhow!("zipformer-streaming init: {e}"))?;
            tracing::info!(%model_id, "zipformer-streaming: recognizer constructed");
            let mut guard = inner
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            *guard = Some(recognizer);
            Ok(())
        })
        .await
        .context("zipformer-streaming warm spawn_blocking failed")??;

        Ok(())
    }
}

#[async_trait]
impl StreamingAsrProvider for SherpaZipformerStreaming {
    async fn start_session(&self) -> Result<Box<dyn StreamingSession>> {
        self.ensure_loaded().await?;
        let inner = self.inner.clone();
        let stream = tokio::task::spawn_blocking(move || -> Result<OnlineStream> {
            let guard = inner
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            let rec = guard
                .as_ref()
                .ok_or_else(|| anyhow!("recognizer not loaded"))?;
            rec.create_stream()
        })
        .await
        .context("create_stream spawn_blocking failed")??;

        Ok(Box::new(ZipformerStreamingSession {
            recognizer: self.inner.clone(),
            stream: Some(stream),
            last_text: String::new(),
        }))
    }
}

/// Per-recording streaming session. Owns the `OnlineStream` and a clone
/// of the recogniser handle so it can drive `is_ready` / `decode` /
/// `result` independently of the provider.
struct ZipformerStreamingSession {
    recognizer: Arc<Mutex<Option<OnlineRecognizer>>>,
    /// Option so we can take ownership of the stream during finish() —
    /// the stream is dropped at end-of-session and the field becomes None.
    stream: Option<OnlineStream>,
    /// The cumulative transcript at the time of the last `accept_chunk`
    /// or `finish` call. Used to short-circuit "no new tokens" → return
    /// None instead of re-emitting the same string.
    last_text: String,
}

#[async_trait]
impl StreamingSession for ZipformerStreamingSession {
    async fn accept_chunk(
        &mut self,
        samples: &[i16],
        sample_rate: u32,
    ) -> Result<Option<StreamingPartial>> {
        // Take the stream out so we can move it into the blocking task
        // without holding the &mut self across the await. We put it back
        // afterwards. If the task panics or errors and we don't restore
        // it, subsequent accept_chunk calls will fail cleanly with
        // "stream not present" rather than deadlocking.
        let mut stream = self
            .stream
            .take()
            .ok_or_else(|| anyhow!("streaming session already finished"))?;

        // Convert i16 → f32 on the caller's thread before the blocking
        // task. Cheap (one multiply per sample); doing it here lets us
        // borrow `samples` rather than copying it into the closure.
        let f32_samples: Vec<f32> = samples
            .iter()
            .map(|s| *s as f32 / i16::MAX as f32)
            .collect();

        let recognizer = self.recognizer.clone();
        let result = tokio::task::spawn_blocking(move || -> Result<(OnlineStream, String)> {
            let guard = recognizer
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            let rec = guard
                .as_ref()
                .ok_or_else(|| anyhow!("recognizer not loaded"))?;

            stream.accept_waveform(sample_rate, &f32_samples);
            // Drain the decoder. is_ready returns true while there's
            // enough buffered audio for one decoder step; we loop until
            // it's caught up.
            while rec.is_ready(&stream) {
                rec.decode(&stream);
            }
            let text = rec.result(&stream);
            Ok((stream, text))
        })
        .await
        .context("accept_chunk spawn_blocking failed")?;

        let (stream, new_text) = result?;
        self.stream = Some(stream);

        // Suppress emit if the cumulative text didn't grow. Sherpa's
        // result() returns the same string until a new token lands,
        // and we don't want to spam the IPC channel with no-deltas.
        if new_text == self.last_text {
            return Ok(None);
        }
        self.last_text = new_text.clone();
        Ok(Some(StreamingPartial {
            text: new_text,
            is_final: false,
        }))
    }

    async fn finish(&mut self) -> Result<StreamingPartial> {
        let mut stream = self
            .stream
            .take()
            .ok_or_else(|| anyhow!("streaming session already finished"))?;

        let recognizer = self.recognizer.clone();
        let final_text = tokio::task::spawn_blocking(move || -> Result<String> {
            let guard = recognizer
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            let rec = guard
                .as_ref()
                .ok_or_else(|| anyhow!("recognizer not loaded"))?;

            stream.input_finished();
            // Drain any trailing audio sherpa was holding.
            while rec.is_ready(&stream) {
                rec.decode(&stream);
            }
            let text = rec.result(&stream);
            // stream is dropped here at end-of-scope, freeing the
            // sherpa OnlineStream via its Drop impl.
            Ok(text)
        })
        .await
        .context("finish spawn_blocking failed")??;

        self.last_text = final_text.clone();
        Ok(StreamingPartial {
            text: final_text,
            is_final: true,
        })
    }
}

// ============================================================================
// Offline-fallback AsrProvider impl.
//
// Lets the streaming model also serve full-buffer transcribe() calls
// (used by the file-transcription path until Phase 2D rewires it
// through chunked refinement). Implementation: open a temporary
// session, push the whole buffer, finish, return the result. Same
// memory profile as the chunked path because sherpa's OnlineRecognizer
// only ever holds chunk-sized state.
// ============================================================================

#[async_trait]
impl AsrProvider for SherpaZipformerStreaming {
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
        let duration_ms =
            (audio.pcm.len() as u64 * 1000 / audio.sample_rate.max(1) as u64) as u32;
        let mut session = self.start_session().await?;
        // Push the whole buffer in one call. The streaming recogniser
        // chunks it internally; memory stays bounded.
        session
            .accept_chunk(&audio.pcm, audio.sample_rate)
            .await?;
        let final_partial = session.finish().await?;

        Ok(AsrResult {
            text: final_partial.text.trim().to_ascii_lowercase(),
            words: None,
            language: Some("en".into()),
            confidence: None,
            model_version: self.model_id.clone(),
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

/// Find the int8 ONNX file in `dir` matching the given prefix.
///
/// The streaming Zipformer tarball ships both `<prefix>-...onnx` (fp32)
/// and `<prefix>-....int8.onnx` (int8). We want int8 specifically; a
/// generic `first_matching` would non-deterministically pick fp32 about
/// half the time.
fn pick_int8(dir: &std::path::Path, prefix: &str) -> Result<PathBuf> {
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("read_dir {}", dir.display()))?
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with(prefix) && name.ends_with(".int8.onnx") {
            return Ok(path);
        }
    }
    Err(anyhow!(
        "no {prefix}*.int8.onnx file in {} — was the streaming Zipformer tarball extracted correctly?",
        dir.display()
    ))
}
