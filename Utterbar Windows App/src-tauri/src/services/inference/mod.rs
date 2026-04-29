//! Inference orchestrator. Owns the active ASR provider and serialises access.

pub mod diarize;
pub mod diarize_assets;
pub mod provider;
pub mod py_sidecar;
pub mod sherpa_parakeet;
pub mod sherpa_whisper;
pub mod sherpa_zipformer;
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

pub use provider::{AsrOptions, AsrProvider, AsrResult, AudioBuffer, WordTiming};

pub struct InferenceOrchestrator {
    registry: Arc<ModelRegistry>,
    settings: Arc<SettingsStore>,
    active: RwLock<Option<Active>>,
}

struct Active {
    provider: Arc<dyn AsrProvider>,
    /// ONNX Runtime provider string the recognizer was built with. If the user changes
    /// their GPU preference we need to rebuild, so we cache the resolved value.
    /// Same as `resolution.used`; kept as a separate field so the
    /// existing rebuild-on-change check stays a one-string compare.
    provider_string: String,
    /// Full provider-resolution telemetry — what the user requested,
    /// what we actually used, whether a fallback fired, and why.
    /// Surfaced through `current_provider_resolution()` so dictation's
    /// transcribe-complete log can record the gap. v0.15.5+.
    resolution: ProviderResolution,
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
        let resolution = resolve_provider_for_settings(&self.settings).await;
        let desired = resolution.used.clone();
        if resolution.fallback_used {
            tracing::info!(
                requested = %resolution.requested,
                used = %resolution.used,
                reason = %resolution.fallback_reason,
                "inference: provider downgraded — runtime EP unavailable"
            );
        }

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

        // Snapshot settings before building so the Whisper language hint
        // is consistent across this load+warm. Don't read it inside
        // build_provider directly — keeping the call free of *self lets
        // it stay testable in isolation.
        let snapshot = self.settings.snapshot().await;
        let whisper_language = snapshot.models.whisper_language.clone();
        let provider =
            build_provider(model_id, &self.registry, &desired, &whisper_language).await?;
        // `warm()` runs native `TransducerRecognizer::new` internally;
        // while we hold the write lock no other transcribe() can be
        // racing this path. See the bug comment above.
        provider.warm().await?;
        *guard = Some(Active {
            provider: provider.clone(),
            provider_string: desired,
            resolution,
            last_used: StdMutex::new(Instant::now()),
        });
        Ok(provider)
    }

    /// Latest provider resolution, if any model is loaded. Read by
    /// `commands::dictation` so the `meeting transcribe complete` and
    /// `inference.transcribe done` log lines can carry
    /// `provider_requested` / `provider_used` / `fallback_used` /
    /// `fallback_reason` alongside the existing audio/RTF data.
    /// Returns `None` when no model is loaded — the caller should
    /// log empty-string sentinels in that edge case.
    pub async fn current_provider_resolution(&self) -> Option<ProviderResolution> {
        self.active
            .read()
            .await
            .as_ref()
            .map(|a| a.resolution.clone())
    }

    pub async fn transcribe(
        &self,
        model_id: &str,
        audio: AudioBuffer,
        opts: AsrOptions,
    ) -> Result<AsrResult> {
        // Long-input guard. Every shipping provider today (sherpa Parakeet,
        // sherpa Whisper, sherpa Zipformer) calls into the offline native
        // recogniser, which allocates O(duration) memory inside ONNX
        // Runtime. Crashes have been observed at ~30+ minutes (see the
        // analyse-crash dump for utterbar.exe.73872.dmp on 2026-04-27 —
        // FAIL_FAST_FATAL_APP_EXIT_c0000409 inside
        // SherpaOnnxDecodeOfflineStream after ~5 min of processing on a
        // 30-min recording).
        //
        // Until the streaming-recogniser path lands (Phase 1 of the
        // STREAMING_PLAN), refuse anything over 5 min with a clear,
        // recoverable user-facing error rather than silently OOMing the
        // whole process. The user keeps the recording (the audio buffer
        // is owned by the caller — we don't take it on early-return) and
        // can split it manually if they need to transcribe today.
        const MAX_TRANSCRIBE_SECONDS: u32 = 5 * 60; // 5 minutes
        let duration_seconds = (audio.pcm.len() as u64 / audio.sample_rate.max(1) as u64) as u32;
        if duration_seconds > MAX_TRANSCRIBE_SECONDS {
            tracing::warn!(
                model_id,
                duration_seconds,
                cap_seconds = MAX_TRANSCRIBE_SECONDS,
                "transcribe refused: input exceeds the offline-recogniser safety cap. \
                 the streaming path is on the way."
            );
            return Err(anyhow!(
                "Recording is {duration_seconds}s. Until the streaming path ships, \
                 transcription is capped at {cap}s to avoid a known offline-decoder \
                 crash on long inputs. Split the audio into shorter clips, or wait \
                 for the next beta which removes this limit.",
                cap = MAX_TRANSCRIBE_SECONDS
            ));
        }

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
        // Three semantic regimes:
        //   • 0          unload immediately after this transcription (handled inline below).
        //   • u32::MAX   keep loaded forever (no inline unload, sweeper also no-ops).
        //   • else       keep warm for N seconds, idle sweeper handles eviction.
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
                if warm_secs == u32::MAX {
                    // Sentinel for "keep loaded forever" — user has explicitly
                    // chosen to keep the model resident in RAM/VRAM at all times
                    // for the lowest possible latency. Skip the eviction check;
                    // the model only unloads on app exit or an explicit reload
                    // from Settings (e.g. switching default model).
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

/// Telemetry-shaped result of the provider resolution + downgrade.
///
/// v0.15.5 (Phase A) made the gap between "what the user asked for"
/// and "what actually ran" observable. Pre-v0.15.5 the resolver
/// returned a single string and we passed it through to sherpa-onnx
/// regardless of whether the loaded ORT could honour it; the user's
/// "DirectML" choice was silently ignored or — worse — produced an
/// obscure session-create error. Phase A:
///
///   1. Detects unavailable execution providers up front (file-presence
///      check via `gpu::runtime_capabilities`).
///   2. Downgrades to `"cpu"` BEFORE engine construction so the
///      provider string we hand to sherpa-onnx is the one we actually
///      ran on.
///   3. Carries the requested + used + fallback_reason through to the
///      transcribe-complete log line so the gap is visible in `tail`.
///
/// `requested` strings — UI-derived:
///   * `"cpu"` / `"directml"` / `"cuda"` — user explicitly chose
///   * `"auto-cpu"` — Auto resolved to CPU because no hardware GPU
///   * `"auto-directml"` — Auto resolved to DirectML because hardware GPU detected
///
/// `used` is one of `"cpu"` / `"directml"` / `"cuda"` — what the
/// engine actually receives. Phase A: always `"cpu"` because the
/// shipping runtime has no DML/CUDA EP. Phase B (v0.15.6) will start
/// returning `"directml"` here for real.
///
/// `fallback_reason` enum (string constants — pinned by a test):
///   * `""` — no downgrade, requested == used
///   * `"dml_ep_unavailable"` — DirectML requested, runtime can't serve it
///   * `"cuda_ep_unavailable"` — CUDA requested, runtime can't serve it
#[derive(Debug, Clone)]
pub struct ProviderResolution {
    pub requested: String,
    pub used: String,
    pub fallback_used: bool,
    pub fallback_reason: String,
}

impl ProviderResolution {
    fn no_fallback(provider: &str) -> Self {
        Self {
            requested: provider.to_string(),
            used: provider.to_string(),
            fallback_used: false,
            fallback_reason: String::new(),
        }
    }
}

/// Turn the user's GPU preference + the shipping runtime's capabilities
/// into a `ProviderResolution`. The `used` string is what flows into
/// engine construction.
///
/// Pure helper — takes the inputs explicitly so tests can drive every
/// arm without standing up a SettingsStore or a runtime fixture.
pub(crate) fn resolve_provider(
    pref: GpuAcceleration,
    has_hardware_gpu: bool,
    caps: &gpu::RuntimeCapabilities,
) -> ProviderResolution {
    match pref {
        GpuAcceleration::Cpu => ProviderResolution::no_fallback("cpu"),

        GpuAcceleration::DirectMl => {
            if caps.has_dml_ep {
                ProviderResolution::no_fallback("directml")
            } else {
                ProviderResolution {
                    requested: "directml".into(),
                    used: "cpu".into(),
                    fallback_used: true,
                    fallback_reason: "dml_ep_unavailable".into(),
                }
            }
        }

        GpuAcceleration::Cuda => {
            if caps.has_cuda_ep {
                ProviderResolution::no_fallback("cuda")
            } else {
                ProviderResolution {
                    requested: "cuda".into(),
                    used: "cpu".into(),
                    fallback_used: true,
                    fallback_reason: "cuda_ep_unavailable".into(),
                }
            }
        }

        GpuAcceleration::Auto => {
            // Auto picks the best provider that's BOTH supported by
            // the hardware AND served by the loaded runtime. Without
            // both, Auto resolves honestly to CPU — it's not a
            // downgrade in the fallback sense, it's the only path
            // that was ever going to work on this build, so
            // `fallback_used = false`.
            //
            // DirectML is the safe Windows GPU choice when both
            // conditions hold — runs on any DX12 GPU (NVIDIA, AMD,
            // Intel Arc, integrated). CUDA is never picked by Auto
            // because we've never bundled the CUDA runtime.
            if has_hardware_gpu && caps.has_dml_ep {
                ProviderResolution {
                    requested: "auto-directml".into(),
                    used: "directml".into(),
                    fallback_used: false,
                    fallback_reason: String::new(),
                }
            } else {
                ProviderResolution {
                    requested: "auto-cpu".into(),
                    used: "cpu".into(),
                    fallback_used: false,
                    fallback_reason: String::new(),
                }
            }
        }
    }
}

/// Async wrapper that snapshots the user's pref + probes the runtime
/// before delegating to the pure `resolve_provider` helper. This is
/// the orchestrator-facing entry point.
async fn resolve_provider_for_settings(settings: &SettingsStore) -> ProviderResolution {
    let pref = settings.snapshot().await.advanced.gpu_acceleration;
    let probe = gpu::probe();
    let caps = gpu::runtime_capabilities();
    resolve_provider(pref, probe.has_hardware_gpu, &caps)
}

async fn build_provider(
    model_id: &str,
    registry: &ModelRegistry,
    provider_string: &str,
    whisper_language: &str,
) -> Result<Arc<dyn AsrProvider>> {
    // Stub always available — no model installed yet, or explicit opt-out.
    if model_id == "stub" {
        return Ok(Arc::new(stub::StubProvider));
    }

    // Use list_available so we get the catalogue's engine field merged
    // with installed-state. list_installed() alone returns the DB row
    // which doesn't persist engine — every loaded model would fall
    // back to ParakeetTdt (the enum default), masking misconfigurations
    // until they crashed at first inference.
    let available = registry.list_available().await;
    let info = available
        .iter()
        .find(|m| m.id == model_id && m.installed)
        .ok_or_else(|| anyhow!("model not installed: {model_id}"))?;

    let model_dir: PathBuf = info
        .path
        .as_ref()
        .map(|p| registry.models_root().join(p))
        .ok_or_else(|| anyhow!("installed model has no path: {model_id}"))?;

    // Engine dispatch. Today only the Parakeet path is wired into the
    // inference layer; Whisper + Zipformer cases land in v0.8.1 / v0.8.2
    // alongside their respective recogniser providers. Catalogue entries
    // for those engines are guarded so the user can't install one and
    // hit a runtime panic — install_model rejects engines the loader
    // doesn't yet support.
    use crate::domain::ModelEngine;
    match info.engine {
        ModelEngine::ParakeetTdt => Ok(Arc::new(sherpa_parakeet::SherpaParakeet::new(
            model_id.to_string(),
            model_dir,
            provider_string.to_string(),
        ))),
        ModelEngine::Whisper => Ok(Arc::new(sherpa_whisper::SherpaWhisper::new(
            model_id.to_string(),
            model_dir,
            whisper_language.to_string(),
            provider_string.to_string(),
        ))),
        ModelEngine::Zipformer => Ok(Arc::new(sherpa_zipformer::SherpaZipformer::new(
            model_id.to_string(),
            model_dir,
            provider_string.to_string(),
        ))),
    }
}

#[cfg(test)]
mod resolver_tests {
    //! v0.15.5 Phase A: provider resolution + downgrade matrix.
    //!
    //! Pinning the user-spec contract:
    //!   * If DML/CUDA is requested but the runtime can't serve it,
    //!     `used` is `"cpu"` and `fallback_reason` is set.
    //!   * If CPU is explicitly requested, no fallback.
    //!   * If Auto resolves to CPU because the runtime is CPU-only,
    //!     that's not a fallback — it's the only path that was ever
    //!     going to work; `requested` carries the `"auto-cpu"` /
    //!     `"auto-directml"` distinction so logs can tell them apart.
    use super::*;
    use crate::services::gpu::RuntimeCapabilities;

    fn cpu_only_runtime() -> RuntimeCapabilities {
        RuntimeCapabilities {
            has_dml_ep: false,
            has_cuda_ep: false,
        }
    }

    fn dml_capable_runtime() -> RuntimeCapabilities {
        RuntimeCapabilities {
            has_dml_ep: true,
            has_cuda_ep: false,
        }
    }

    fn cuda_capable_runtime() -> RuntimeCapabilities {
        RuntimeCapabilities {
            has_dml_ep: false,
            has_cuda_ep: true,
        }
    }

    #[test]
    fn cpu_requested_uses_cpu_no_fallback() {
        let r = resolve_provider(GpuAcceleration::Cpu, false, &cpu_only_runtime());
        assert_eq!(r.requested, "cpu");
        assert_eq!(r.used, "cpu");
        assert!(!r.fallback_used);
        assert_eq!(r.fallback_reason, "");
    }

    #[test]
    fn cpu_requested_with_hardware_gpu_still_uses_cpu() {
        // Hardware GPU presence must never override an explicit CPU choice.
        let r = resolve_provider(GpuAcceleration::Cpu, true, &dml_capable_runtime());
        assert_eq!(r.used, "cpu");
        assert!(!r.fallback_used);
    }

    #[test]
    fn directml_requested_dml_unavailable_downgrades_to_cpu() {
        let r = resolve_provider(GpuAcceleration::DirectMl, true, &cpu_only_runtime());
        assert_eq!(r.requested, "directml");
        assert_eq!(
            r.used, "cpu",
            "must pass `cpu` to engine when DML EP is unavailable"
        );
        assert!(r.fallback_used);
        assert_eq!(r.fallback_reason, "dml_ep_unavailable");
    }

    #[test]
    fn directml_requested_dml_available_uses_directml() {
        // Phase B test surface — when v0.15.6 ships the DML runtime,
        // this test pins that an explicit DirectML request is honoured.
        let r = resolve_provider(GpuAcceleration::DirectMl, true, &dml_capable_runtime());
        assert_eq!(r.used, "directml");
        assert!(!r.fallback_used);
        assert_eq!(r.fallback_reason, "");
    }

    #[test]
    fn cuda_requested_cuda_unavailable_downgrades_to_cpu() {
        let r = resolve_provider(GpuAcceleration::Cuda, true, &cpu_only_runtime());
        assert_eq!(r.requested, "cuda");
        assert_eq!(r.used, "cpu");
        assert!(r.fallback_used);
        assert_eq!(r.fallback_reason, "cuda_ep_unavailable");
    }

    #[test]
    fn cuda_requested_cuda_available_uses_cuda() {
        let r = resolve_provider(GpuAcceleration::Cuda, true, &cuda_capable_runtime());
        assert_eq!(r.used, "cuda");
        assert!(!r.fallback_used);
    }

    #[test]
    fn auto_no_hardware_gpu_uses_cpu_no_fallback() {
        let r = resolve_provider(GpuAcceleration::Auto, false, &cpu_only_runtime());
        assert_eq!(
            r.requested, "auto-cpu",
            "auto with no hardware GPU labels itself auto-cpu"
        );
        assert_eq!(r.used, "cpu");
        assert!(
            !r.fallback_used,
            "auto-resolves-to-cpu is not a fallback — it's the only path that was going to work"
        );
        assert_eq!(r.fallback_reason, "");
    }

    #[test]
    fn auto_with_hardware_gpu_but_no_dml_runtime_uses_cpu_no_fallback() {
        // The user's spec: "Auto with unavailable EP does not pretend
        // GPU is active". Auto is honest about CPU when the runtime
        // can't deliver DML, but it doesn't call this a fallback —
        // the user never asked for DML directly.
        let r = resolve_provider(GpuAcceleration::Auto, true, &cpu_only_runtime());
        assert_eq!(r.requested, "auto-cpu");
        assert_eq!(r.used, "cpu");
        assert!(!r.fallback_used);
        assert_eq!(r.fallback_reason, "");
    }

    #[test]
    fn auto_with_hardware_gpu_and_dml_runtime_uses_directml() {
        // Phase B path: Auto picks DirectML when both conditions hold.
        let r = resolve_provider(GpuAcceleration::Auto, true, &dml_capable_runtime());
        assert_eq!(r.requested, "auto-directml");
        assert_eq!(r.used, "directml");
        assert!(!r.fallback_used);
    }

    /// Pins the `fallback_reason` enum strings so log-greppers don't
    /// silently break in a future refactor.
    #[test]
    fn fallback_reason_enum_strings_are_stable() {
        let dml = resolve_provider(GpuAcceleration::DirectMl, true, &cpu_only_runtime());
        assert_eq!(dml.fallback_reason, "dml_ep_unavailable");

        let cuda = resolve_provider(GpuAcceleration::Cuda, true, &cpu_only_runtime());
        assert_eq!(cuda.fallback_reason, "cuda_ep_unavailable");

        let cpu = resolve_provider(GpuAcceleration::Cpu, false, &cpu_only_runtime());
        assert_eq!(cpu.fallback_reason, "");
    }
}
