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
