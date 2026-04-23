//! Python sidecar provider — spawns `scripts/asr_sidecar.py` and talks length-prefixed JSON
//! over stdio. Lets users leverage NeMo's reference Parakeet pipeline when ONNX export isn't
//! viable on their hardware.
//!
//! Scaffolded but not wired to the provider selector yet — sherpa-onnx covers every
//! shipped model. We keep the module so the selector can opt-in to a Python backend
//! once we publish a model that needs it, without re-deriving the stdio protocol.

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use super::provider::{AsrOptions, AsrProvider, AsrResult, AudioBuffer, WordTiming};

pub struct PySidecarProvider {
    model_id: String,
    child: Mutex<Option<Child>>,
}

#[derive(Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum SidecarRequest {
    Warm { model_id: String },
    Transcribe {
        model_id: String,
        pcm_b64: String,
        sample_rate: u32,
        want_word_timings: bool,
        language_hint: Option<String>,
    },
    Shutdown,
}

#[derive(Deserialize)]
struct SidecarResponse {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    words: Option<Vec<WordTiming>>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    duration_ms: Option<u32>,
}

impl PySidecarProvider {
    pub fn new(model_id: String) -> Result<Self> {
        Ok(Self { model_id, child: Mutex::new(None) })
    }

    async fn ensure_child(&self) -> Result<()> {
        let mut guard = self.child.lock().await;
        if guard.is_some() { return Ok(()); }
        let python = which_python()?;
        let script = script_path()?;
        let child = Command::new(python)
            .arg(&script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow!("spawn sidecar: {e}"))?;
        *guard = Some(child);
        Ok(())
    }

    async fn exchange(&self, req: SidecarRequest) -> Result<SidecarResponse> {
        self.ensure_child().await?;
        let mut guard = self.child.lock().await;
        let child = guard.as_mut().ok_or_else(|| anyhow!("sidecar not spawned"))?;
        let stdin = child.stdin.as_mut().ok_or_else(|| anyhow!("no stdin"))?;
        let stdout = child.stdout.as_mut().ok_or_else(|| anyhow!("no stdout"))?;

        let payload = serde_json::to_vec(&req)?;
        let len = (payload.len() as u32).to_le_bytes();
        stdin.write_all(&len).await?;
        stdin.write_all(&payload).await?;
        stdin.flush().await?;

        let mut len_buf = [0u8; 4];
        stdout.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        stdout.read_exact(&mut buf).await?;
        Ok(serde_json::from_slice(&buf)?)
    }
}

#[async_trait]
impl AsrProvider for PySidecarProvider {
    fn id(&self) -> &str { &self.model_id }
    fn supported_languages(&self) -> &[&'static str] { &["en"] }

    async fn warm(&self) -> Result<()> {
        let r = self.exchange(SidecarRequest::Warm { model_id: self.model_id.clone() }).await?;
        if !r.ok { return Err(anyhow!(r.error.unwrap_or_else(|| "warm failed".into()))); }
        Ok(())
    }

    async fn transcribe(&self, audio: AudioBuffer, opts: AsrOptions) -> Result<AsrResult> {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let bytes: Vec<u8> = audio.pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
        let pcm_b64 = STANDARD.encode(&bytes);
        let resp = self.exchange(SidecarRequest::Transcribe {
            model_id: self.model_id.clone(),
            pcm_b64,
            sample_rate: audio.sample_rate,
            want_word_timings: opts.want_word_timings,
            language_hint: opts.language_hint,
        }).await?;
        if !resp.ok { return Err(anyhow!(resp.error.unwrap_or_default())); }
        Ok(AsrResult {
            text: resp.text.unwrap_or_default(),
            words: resp.words,
            language: resp.language,
            confidence: None,
            model_version: format!("{}-sidecar", self.model_id),
            duration_ms: resp.duration_ms.unwrap_or(0),
        })
    }

    async fn unload(&self) -> Result<()> {
        let _ = self.exchange(SidecarRequest::Shutdown).await;
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

fn which_python() -> Result<std::path::PathBuf> {
    for name in ["python", "python3", "py"] {
        if let Ok(output) = std::process::Command::new(name).arg("--version").output() {
            if output.status.success() {
                return Ok(name.into());
            }
        }
    }
    Err(anyhow!("no python found on PATH; install Python 3.10+ or use the ONNX backend"))
}

fn script_path() -> Result<std::path::PathBuf> {
    let exe = std::env::current_exe()?;
    let mut root = exe.clone();
    // resources/ lives next to the bundled binary on Windows.
    for _ in 0..3 {
        if let Some(parent) = root.parent() {
            let candidate = parent.join("scripts").join("asr_sidecar.py");
            if candidate.exists() { return Ok(candidate); }
            root = parent.to_path_buf();
        }
    }
    // Dev: fall back to repo path
    Ok(std::path::PathBuf::from("scripts/asr_sidecar.py"))
}
