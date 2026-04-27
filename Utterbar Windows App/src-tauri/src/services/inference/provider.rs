use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub pcm: Vec<i16>,
    pub sample_rate: u32,
    /// Channel count the PCM is laid out in. The active pipeline always feeds mono
    /// (capture + loopback both downmix before reaching the provider), so no provider
    /// reads this today — but the field is part of the buffer contract so multi-channel
    /// sources can be added later without changing the struct shape.
    #[allow(dead_code)]
    pub channels: u16,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsrOptions {
    pub language_hint: Option<String>,
    pub want_word_timings: bool,
    pub vocabulary_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTiming {
    pub text: String,
    pub start_ms: u32,
    pub end_ms: u32,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrResult {
    pub text: String,
    pub words: Option<Vec<WordTiming>>,
    pub language: Option<String>,
    pub confidence: Option<f32>,
    pub model_version: String,
    pub duration_ms: u32,
}

#[async_trait]
pub trait AsrProvider: Send + Sync {
    fn id(&self) -> &str;
    /// Languages the backing model claims to support. Not read yet — the language hint
    /// in `AsrOptions` is passed through without validation — but kept so the settings
    /// UI can gate the language dropdown against real capabilities once we wire it.
    #[allow(dead_code)]
    fn supported_languages(&self) -> &[&'static str];
    async fn warm(&self) -> Result<()>;
    async fn transcribe(&self, audio: AudioBuffer, opts: AsrOptions) -> Result<AsrResult>;
    async fn unload(&self) -> Result<()>;
}

// ============================================================================
// Streaming
// ============================================================================
//
// Streaming is a separate trait, not a method on AsrProvider. The reasons:
//
//   1. Most providers don't support it (Whisper has no native streaming
//      mode in sherpa-onnx; Zipformer's offline export and streaming export
//      are different ONNX files; py_sidecar / stub will never support
//      streaming). Adding a `streaming_session()` method to AsrProvider
//      forces every impl to either return None or implement a fake.
//
//   2. The session lifecycle is fundamentally different shape from
//      `transcribe(buffer) -> result`. A streaming session has push / poll /
//      finish; conflating them under one trait obscures both.
//
// A given model can implement both AsrProvider AND StreamingAsrProvider —
// the streaming Zipformer will. The orchestrator picks the right path
// based on what the loaded provider implements.

/// One frame of streaming output. `text` is the cumulative transcript
/// the recogniser knows about so far; the caller computes the diff if
/// it wants per-token deltas.
///
/// `is_final` is true once the session has been told no more audio is
/// coming (`StreamingSession::finish()` was called). At that point the
/// text is the recogniser's best final guess and won't change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingPartial {
    pub text: String,
    pub is_final: bool,
}

/// A live streaming recognition session. One per recording. Created via
/// `StreamingAsrProvider::start_session`, fed audio chunks via
/// `accept_chunk`, drained via `partial_text`, finalised via `finish`.
///
/// Implementations are responsible for tracking their own per-stream
/// state (typically by holding the sherpa OnlineStream). The orchestrator
/// holds the trait object on the recording's lifecycle.
#[async_trait]
pub trait StreamingSession: Send {
    /// Push a chunk of i16 PCM samples (mono, at the model's expected
    /// sample rate — 16 kHz for every shipping streaming model).
    /// Returns the current cumulative transcript if it grew, or None if
    /// the chunk wasn't enough to advance the decoder.
    async fn accept_chunk(&mut self, samples: &[i16], sample_rate: u32) -> Result<Option<StreamingPartial>>;

    /// Mark end-of-audio. Drains remaining buffered samples and returns
    /// the final transcript. After calling this, no more `accept_chunk`
    /// calls should happen.
    async fn finish(&mut self) -> Result<StreamingPartial>;
}

/// Providers that can serve a streaming session. Optional capability:
/// only the streaming-Zipformer (and future streaming-Parakeet) impls
/// expose this.
#[async_trait]
#[allow(dead_code)] // wired up in 1B.6 — the impl exists, the orchestrator hookup lands next
pub trait StreamingAsrProvider: Send + Sync {
    /// Open a fresh streaming session. Each recording gets its own;
    /// streams aren't reusable across recordings. Returns an error if
    /// the underlying recogniser can't allocate (out of memory, model
    /// not loaded, etc).
    async fn start_session(&self) -> Result<Box<dyn StreamingSession>>;
}
