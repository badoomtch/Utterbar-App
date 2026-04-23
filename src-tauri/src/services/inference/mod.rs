//! Inference orchestrator. Owns the active ASR provider and serialises access.

pub mod provider;
pub mod py_sidecar;
pub mod sherpa_parakeet;
pub mod stub;

use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use tokio::sync::RwLock;

use crate::domain::settings::GpuAcceleration;
use crate::services::gpu;
use crate::services::models::ModelRegistry;
use crate::services::settings::SettingsStore;

pub use provider::{AsrOptions, AsrProvider, AsrResult, AudioBuffer};

pub struct InferenceOrchestrator {
    registry: Arc<ModelRegistry>,
    settings: Arc<SettingsStore>,
    active: RwLock<Option<Active>>,
}

struct Active {
    provider: Arc<dyn AsrProvider>,
    /// ONNX Runtime provider string the recognizer was built with. If the user changes
    /// their GPU preference we need to rebuild, so we cache the resolved value.
    provider_string: String,
    /// Last time `transcribe` completed against this provider. Used by the idle
    /// sweeper to honour `models.warm_duration_secs`. Wrapped in a `std::sync::Mutex`
    /// so we can update it while only holding a read lock on `active` — updates are
    /// nanosecond-scale so the blocking mutex is fine.
    last_used: StdMutex<Instant>,
}

impl InferenceOrchestrator {
    pub fn new(registry: Arc<ModelRegistry>, settings: Arc<SettingsStore>) -> Self {
        Self {
            registry,
            settings,
            active: RwLock::new(None),
        }
    }

    pub async fn ensure_loaded(&self, model_id: &str) -> Result<Arc<dyn AsrProvider>> {
        let desired = resolve_provider_string(&self.settings).await;

        if let Some(active) = self.active.read().await.as_ref() {
            if active.provider.id() == model_id && active.provider_string == desired {
                return Ok(active.provider.clone());
            }
        }

        // Unload any currently-active recognizer before building a new one so we don't
        // keep two sets of ONNX sessions resident in memory.
        if let Some(prev) = self.active.write().await.take() {
            let _ = prev.provider.unload().await;
        }

        let provider = build_provider(model_id, &self.registry, &desired).await?;
        provider.warm().await?;
        *self.active.write().await = Some(Active {
            provider: provider.clone(),
            provider_string: desired,
            last_used: StdMutex::new(Instant::now()),
        });
        Ok(provider)
    }

    pub async fn transcribe(
        &self,
        model_id: &str,
        audio: AudioBuffer,
        opts: AsrOptions,
    ) -> Result<AsrResult> {
        let provider = self.ensure_loaded(model_id).await?;
        let result = provider.transcribe(audio, opts).await;

        // Update last-used bookkeeping for the idle sweeper. Read-lock is enough —
        // `last_used` has its own blocking mutex. Do this even on error: a failed
        // transcribe still counts as "user is actively dictating" so we shouldn't
        // evict the model right out from under a retry.
        if let Some(active) = self.active.read().await.as_ref() {
            if let Ok(mut slot) = active.last_used.lock() {
                *slot = Instant::now();
            }
        }

        // `warm_duration_secs == 0` means "don't keep it warm" — unload the moment
        // this transcription is done. The next dictation pays a full load (~1s on
        // a warm disk) but uses zero RAM in between. Sweeper handles the `> 0`
        // case; we just shortcut here so the user sees the effect immediately
        // rather than on the next sweeper tick.
        let warm_secs = self.settings.snapshot().await.models.warm_duration_secs;
        if warm_secs == 0 {
            if let Some(prev) = self.active.write().await.take() {
                let _ = prev.provider.unload().await;
            }
        }

        result
    }

    /// Which model (if any) is currently resident in memory. `None` when the
    /// orchestrator is cold — the next dictation will pay the full load cost
    /// (1–3 s depending on disk cache / GPU init). `Some(id)` means the model
    /// is warm and the next transcribe will start as fast as audio arrives.
    ///
    /// Used by Settings → Models to surface a subtle "Ready / Loading" state
    /// so users know why the first dictation after a cold start feels slower
    /// than subsequent ones, without having to guess from latency alone.
    pub async fn loaded_model_id(&self) -> Option<String> {
        self.active.read().await.as_ref().map(|a| a.provider.id().to_string())
    }

    /// Manual unload path. Unused today — the inline `warm_duration_secs == 0` path and
    /// the idle sweeper together cover every eviction case we currently need — but kept
    /// as the canonical "drop the model right now" entry for settings-change flows we
    /// expect to wire up (e.g. user switches GPU preference and wants an immediate rebuild).
    #[allow(dead_code)]
    pub async fn unload(&self) -> Result<()> {
        if let Some(prev) = self.active.write().await.take() {
            prev.provider.unload().await?;
        }
        Ok(())
    }

    /// Spawn the idle-unload sweeper. Call once during startup (see main.rs).
    /// Polls every 10s — much finer than that and we're checking more often than
    /// the user's `warm_duration_secs` can meaningfully resolve; much coarser
    /// and a setting of, say, 15s takes up to half its lifetime to trigger.
    ///
    /// `warm_duration_secs == 0` is handled inline in `transcribe` — the
    /// sweeper's job is the `> 0` case where the model hangs around for N
    /// seconds after the last dictation, then is released to free RAM /
    /// VRAM for other apps.
    pub fn spawn_idle_sweeper(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(10));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                ticker.tick().await;

                let warm_secs = self.settings.snapshot().await.models.warm_duration_secs;
                if warm_secs == 0 {
                    // Inline-unload path handles this; nothing for the sweeper to do.
                    continue;
                }

                let should_unload = {
                    let guard = self.active.read().await;
                    match guard.as_ref() {
                        Some(a) => a
                            .last_used
                            .lock()
                            .map(|last| last.elapsed() >= Duration::from_secs(warm_secs as u64))
                            .unwrap_or(false),
                        None => false,
                    }
                };

                if should_unload {
                    if let Some(prev) = self.active.write().await.take() {
                        let _ = prev.provider.unload().await;
                        tracing::info!(warm_secs, "inference: idle unload");
                    }
                }
            }
        });
    }
}

/// Turn the user's GPU preference into a concrete ONNX Runtime provider string.
///
/// * `Auto`      — pick a hardware GPU if one exists, otherwise `cpu`
/// * `DirectMl`  — force `directml` (fails loudly at load time if unavailable)
/// * `Cuda`      — force `cuda`
/// * `Cpu`       — force `cpu`
async fn resolve_provider_string(settings: &SettingsStore) -> String {
    let pref = settings.snapshot().await.advanced.gpu_acceleration;
    match pref {
        GpuAcceleration::Cpu => "cpu".into(),
        GpuAcceleration::DirectMl => "directml".into(),
        GpuAcceleration::Cuda => "cuda".into(),
        GpuAcceleration::Auto => {
            let probe = gpu::probe();
            if probe.has_hardware_gpu {
                // DirectML is the safe default on Windows — runs on any DX12 GPU
                // (NVIDIA, AMD, Intel Arc, integrated Intel/AMD). CUDA would be faster
                // on NVIDIA but requires a separate DLL bundle.
                "directml".into()
            } else {
                "cpu".into()
            }
        }
    }
}

async fn build_provider(
    model_id: &str,
    registry: &ModelRegistry,
    provider_string: &str,
) -> Result<Arc<dyn AsrProvider>> {
    // Stub always available — no model installed yet, or explicit opt-out.
    if model_id == "stub" {
        return Ok(Arc::new(stub::StubProvider));
    }

    let installed = registry.list_installed()?;
    let info = installed
        .iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| anyhow!("model not installed: {model_id}"))?;

    let model_dir: PathBuf = info
        .path
        .as_ref()
        .map(|p| registry.models_root().join(p))
        .ok_or_else(|| anyhow!("installed model has no path: {model_id}"))?;

    Ok(Arc::new(sherpa_parakeet::SherpaParakeet::new(
        model_id.to_string(),
        model_dir,
        provider_string.to_string(),
    )))
}
