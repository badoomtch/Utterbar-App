//! Audio capture service. cpal-backed for microphones; direct WASAPI for system-audio
//! loopback. Produces 16 kHz mono Int16 PCM regardless of source.

pub mod capture;
pub mod decode;
#[cfg(windows)]
pub mod loopback;
pub mod meter;
pub mod preprocess;

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use self::capture::CaptureHandle;

pub struct AudioService {
    inner: RwLock<Option<Arc<CaptureHandle>>>,
    #[cfg(windows)]
    loopback_inner: RwLock<Option<Arc<loopback::LoopbackHandle>>>,
}

impl AudioService {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(None),
            #[cfg(windows)]
            loopback_inner: RwLock::new(None),
        }
    }

    pub async fn devices(&self) -> Result<Vec<InputDevice>> {
        capture::list_input_devices()
    }

    /// Start capture from the user's microphone. If a capture is already active (either
    /// from a previous recording that wasn't stopped, or a double-fire of start_recording
    /// before the previous phase transition landed), we MUST stop it first. The old
    /// `CaptureHandle` owns a `stop_flag` that the capture thread reads every 50ms, but
    /// the thread holds its own Arc clone — so letting the handle drop does NOT terminate
    /// the thread. Without the explicit stop here, every recording leaked a live cpal
    /// stream + an ever-growing PCM buffer (observed: 11 GB RSS after a long session).
    pub async fn start_capture(&self, device_id: Option<String>) -> Result<Arc<CaptureHandle>> {
        // Stop any existing capture first so the cpal stream + audio thread terminate and
        // release the PCM buffer. `.stop()` is idempotent at the thread level.
        if let Some(prev) = self.inner.write().await.take() {
            tracing::warn!("start_capture: previous capture still active — stopping it first");
            let _ = prev.stop().await;
        }
        let handle = Arc::new(capture::start(device_id.as_deref())?);
        *self.inner.write().await = Some(handle.clone());
        Ok(handle)
    }

    /// Start a WASAPI loopback capture (system audio). Windows-only. Same zombie-thread
    /// problem as `start_capture` — we stop any previous loopback before starting a new one.
    #[cfg(windows)]
    pub async fn start_loopback(&self) -> Result<Arc<loopback::LoopbackHandle>> {
        if let Some(prev) = self.loopback_inner.write().await.take() {
            tracing::warn!("start_loopback: previous loopback still active — stopping it first");
            let _ = prev.stop().await;
        }
        let handle = Arc::new(loopback::start()?);
        *self.loopback_inner.write().await = Some(handle.clone());
        Ok(handle)
    }

    pub async fn stop_capture(&self) -> Option<capture::CaptureFinal> {
        let mic_final = if let Some(h) = self.inner.write().await.take() {
            Some(h.stop().await)
        } else {
            None
        };
        #[cfg(windows)]
        let loop_final = if let Some(h) = self.loopback_inner.write().await.take() {
            Some(h.stop().await)
        } else {
            None
        };
        #[cfg(not(windows))]
        let loop_final: Option<capture::CaptureFinal> = None;

        match (mic_final, loop_final) {
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (Some(a), Some(b)) => Some(mix_two(a, b)),
            (None, None) => None,
        }
    }

    /// Read the currently-active primary capture handle. Unused today — dictation and
    /// meter paths each stash their own handle — but kept so diagnostics / future probes
    /// can ask "is the mic currently hot?" without racing the start/stop APIs.
    #[allow(dead_code)]
    pub async fn current(&self) -> Option<Arc<CaptureHandle>> {
        self.inner.read().await.clone()
    }
}

/// Sum two PCM streams sample-by-sample with hard clipping. Assumes both streams are at
/// the same sample rate (both capture paths resample to 16 kHz before returning).
fn mix_two(mut a: capture::CaptureFinal, b: capture::CaptureFinal) -> capture::CaptureFinal {
    let len = a.pcm.len().max(b.pcm.len());
    a.pcm.resize(len, 0);
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let av = *a.pcm.get(i).unwrap_or(&0) as i32;
        let bv = *b.pcm.get(i).unwrap_or(&0) as i32;
        let sum = (av + bv).clamp(i16::MIN as i32, i16::MAX as i32);
        out.push(sum as i16);
    }
    capture::CaptureFinal {
        pcm: out,
        sample_rate: a.sample_rate,
        duration_ms: (len as u64 * 1000) / a.sample_rate.max(1) as u64,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct InputDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub supported_sample_rates: Vec<u32>,
}
