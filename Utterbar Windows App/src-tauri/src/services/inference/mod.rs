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

        // Single write lock around the whole check-then-build sequence.
        //
        // Before v0.6.2 this was a read-lock hit test followed by a
        // separate write-lock build, which is a classic TOCTOU: two
        // transcribe() calls arriving during a model swap could each
        // (a) read the stale cache, (b) see it was invalid, (c) drop
        // their read lock, (d) both acquire the write lock one after
        // the other, and (e) both call into sherpa-onnx's native
        // `TransducerRecognizer::new`. The DirectML provider's session
        // construction touches global D3D12 device state, so two
        // concurrent builds inside the same process would corrupt the
        // ORT heap — reliably killing the app with
        // STATUS_HEAP_CORRUPTION (0xC0000374).
        //
        // Holding a write lock for the whole sequence costs us some
        // parallelism on the rare genuine model-swap case, but every
        // other code path hits the fast-return inside the same lock
        // with no contention worth measuring. The safety is worth it.
        let mut guard = self.active.write().await;

        if let Some(active) = guard.as_ref() {
            if active.provider.id() == model_id && active.provider_string == desired {
                return Ok(active.provider.clone());
            }
        }

        // Either no provider, or it doesn't match. Drop the previous
        // one (if any) before building a new one so we don't keep two
        // sets of ONNX sessions resident in memory during the swap.
        //
        // But only if no other caller holds an Arc clone of the old
        // provider. If they do (a transcribe is mid-flight with the
        // previous model), we must wait for them to finish — dropping
        // the native recognizer while their tokio task is still calling
        // into its C++ code would race sherpa-onnx's teardown.
        if let Some(prev) = guard.take() {
            if Arc::strong_count(&prev.provider) > 1 {
                // Put it back. The caller was asking for a model swap,
                // but there's still a transcribe running against the
                // current model — bail out for this call and let the
                // caller retry, so the active transcribe can finish
                // before we tear its recognizer down.
                tracing::warn!(
                    "ensure_loaded: model swap deferred — previous provider still in use by another transcribe"
                );
                *guard = Some(prev);
                return Err(anyhow!(
                    "model swap pending: another transcribe is still in flight — retry shortly"
                ));
            }
            // Drop the guard across the await so `unload()` — which
            // does its own synchronous mutex dance inside the provider
            // — doesn't deadlock with another caller that's blocked
            // waiting for our write lock. Safe because `guard.take()`
            // already transferred ownership of the old `Active` to a
            // local, so nothing else can see it mid-unload.
            drop(guard);
            let _ = prev.provider.unload().await;
            guard = self.active.write().await;
        }

        let provider = build_provider(model_id, &self.registry, &desired).await?;
        // `warm()` runs native `TransducerRecognizer::new` internally;
        // while we hold the write lock no other transcribe() can be
        // racing this path. See the bug comment above.
        provider.warm().await?;
        *guard = Some(Active {
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
        //
        // We must not unload while another concurrent transcribe holds a
        // clone of this same provider Arc. Our local `provider` above is
        // one clone; `active.provider` is another. So strong_count == 2
        // means "just us". Higher means another transcribe is in flight
        // and unloading would race its native teardown.
        let warm_secs = self.settings.snapshot().await.models.warm_duration_secs;
        if warm_secs == 0 {
            // Drop our local `provider` Arc BEFORE the check so strong_count
            // reflects only other in-flight callers, not ourselves.
            drop(provider);
            let mut guard = self.active.write().await;
            let safe_to_unload = guard
                .as_ref()
                .is_some_and(|a| Arc::strong_count(&a.provider) == 1);
            if safe_to_unload {
                if let Some(prev) = guard.take() {
                    drop(guard);
                    let _ = prev.provider.unload().await;
                }
            } else {
                tracing::info!(
                    "inference: inline unload (warm=0) deferred — another transcribe still in flight"
                );
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
        self.active
            .read()
            .await
            .as_ref()
            .map(|a| a.provider.id().to_string())
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
    ///
    /// v0.6.3 hardening: the sweeper now refuses to unload while any
    /// caller still holds a clone of the current provider Arc. Before
    /// this check, the sweeper could fire in between `ensure_loaded`
    /// returning an `Arc<dyn AsrProvider>` to a caller and that caller
    /// finishing `provider.transcribe().await`. On the active happy
    /// path the inner mutex serialises well enough to avoid UB, but
    /// the post-transcribe read-lock + `last_used` update runs AFTER
    /// the native call — if the sweeper dropped the provider's native
    /// recognizer mid-way, the drop invoked sherpa-onnx's native ORT
    /// teardown concurrently with another tokio task that still held
    /// the Arc for its own cleanup. That racing native teardown is the
    /// most likely source of the remaining STATUS_HEAP_CORRUPTION
    /// (0xC0000374) crashes post-v0.6.2.
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
                        Some(a) => {
                            let elapsed = a
                                .last_used
                                .lock()
                                .map(|last| last.elapsed())
                                .unwrap_or_default();
                            let idle = elapsed >= Duration::from_secs(warm_secs as u64);
                            // Refuse to unload if another caller still holds
                            // a clone of this provider Arc — they're mid-use,
                            // and dropping the native recognizer out from
                            // under them races the sherpa-onnx native teardown
                            // with their in-flight transcribe. One extra live
                            // clone (besides ours in `a.provider`) means at
                            // least one transcribe hasn't returned yet.
                            let in_use = Arc::strong_count(&a.provider) > 1;
                            if idle && in_use {
                                tracing::info!(
                                    strong_count = Arc::strong_count(&a.provider),
                                    "inference: idle sweeper deferring — provider still in use"
                                );
                            }
                            idle && !in_use
                        }
                        None => false,
                    }
                };

                if should_unload {
                    // Re-check strong_count under the write lock. Another
                    // transcribe may have cloned between our read-lock
                    // check and here. Cheap guard, avoids the read→write
                    // upgrade race.
                    let mut guard = self.active.write().await;
                    let still_single = guard
                        .as_ref()
                        .is_some_and(|a| Arc::strong_count(&a.provider) == 1);
                    if still_single {
                        if let Some(prev) = guard.take() {
                            // Drop the write lock across the await so a
                            // transcribe blocked on our lock isn't held
                            // up by the native teardown.
                            drop(guard);
                            let _ = prev.provider.unload().await;
                            tracing::info!(warm_secs, "inference: idle unload");
                        }
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
