//! Stub provider for testing and pre-install flows. Returns a placeholder transcript.

use anyhow::Result;
use async_trait::async_trait;

use super::{AsrOptions, AsrProvider, AsrResult, AudioBuffer};

#[derive(Default)]
pub struct StubProvider;

#[async_trait]
impl AsrProvider for StubProvider {
    fn id(&self) -> &str {
        "stub"
    }
    fn supported_languages(&self) -> &[&'static str] {
        &["en"]
    }
    async fn warm(&self) -> Result<()> {
        Ok(())
    }

    async fn transcribe(&self, audio: AudioBuffer, _opts: AsrOptions) -> Result<AsrResult> {
        let dur_ms = (audio.pcm.len() as u64 * 1000 / audio.sample_rate.max(1) as u64) as u32;
        Ok(AsrResult {
            text: "Install a Parakeet model in Settings → Models to enable real transcription."
                .into(),
            words: None,
            language: Some("en".into()),
            confidence: None,
            model_version: "stub-0".into(),
            duration_ms: dur_ms,
        })
    }

    async fn unload(&self) -> Result<()> {
        Ok(())
    }
}
