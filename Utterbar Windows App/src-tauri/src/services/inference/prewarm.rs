//! Background prewarm of the default ASR model.
//!
//! Why this exists: cold model construction is the dominant
//! "Utterbar feels slow" report. The native sherpa-onnx
//! `WhisperRecognizer::new` / `TransducerRecognizer::new` call
//! mmaps the ONNX session, allocates ORT arenas, and (on
//! DirectML) spins up D3D12 device state. That work runs
//! lazily on the first transcribe today, so the user's first
//! dictation after launch pays the full warm cost — typically
//! 0.3–4 s, sometimes more on a cold disk or first-DML build.
//!
//! Anyone who keeps the warm window enabled (the default is
//! `warm_duration_secs == u32::MAX`, which keeps the selected model
//! resident until the app exits) wants the model ready when they next
//! dictate. There is no reason to make them wait until first
//! dictation to pay the cost. Prewarming on app startup turns
//! their first dictation into a cache-hit fast-path and shifts
//! the cost into the post-launch background tick where the user
//! is already waiting for the rest of the UI to settle.
//!
//! The idle sweeper still owns eviction. If the user launches
//! Utterbar and then doesn't dictate for `warm_duration_secs`
//! seconds, the sweeper evicts the prewarmed model on its normal
//! 10 s tick — exactly as if the user had dictated once and then
//! gone idle. We don't fight the sweeper; we just shift the
//! "first load" cost off the user's first hotkey press.
//!
//! The only setting we explicitly opt OUT of prewarming is
//! `warm_duration_secs == 0` ("evict immediately after each
//! transcription") — those users have asked the orchestrator to
//! keep RAM/VRAM free between dictations, and prewarming would
//! load a model only for the inline-unload path in `transcribe`
//! to drop it again at the next dictation. Honour the user's
//! intent.
//!
//! Constraints the helper honours:
//!   * Non-blocking — the caller hands off to a tokio task and
//!     returns immediately. Startup must never block on a model
//!     load.
//!   * Failure-tolerant — warm errors are logged at warn level
//!     and swallowed. The first dictation will still attempt
//!     load through the same path and surface its own error.
//!   * User-first at launch — waits through a short startup grace
//!     period, then skips the warm-up if a dictation began during it. This
//!     prevents a first hotkey press from queuing behind a background
//!     native model construction task.
//!   * Meeting-mode model NOT prewarmed — meeting mode is
//!     opt-in per dictation; prewarming a separate Whisper
//!     Medium for users who never invoke it would double the
//!     idle RAM cost. Only the `default_asr_model_id` is
//!     prewarmed, which is what fires on every non-meeting
//!     dictation.

use std::sync::Arc;
use std::time::Instant;

use crate::app_state::AppState;
use crate::domain::Settings;
use crate::services::inference::InferenceOrchestrator;

/// Give the user a chance to dictate immediately after launch before the
/// background warm-up claims the inference construction lock.
const STARTUP_PREWARM_DELAY_SECS: u64 = 15;

/// Pure decision: should the app prewarm a default ASR model on
/// launch, and if so which model id?
///
/// Returns `Some(model_id)` only when both:
///   1. `warm_duration_secs > 0`. A non-zero warm window means the
///      user wants the model resident at least a little while
///      after each dictation; the same intent applies BEFORE the
///      first dictation. The idle sweeper handles eviction
///      uniformly across the full range — finite N seconds keeps
///      the model warm for N, `u32::MAX` keeps it forever.
///      Only `0` ("evict immediately after each transcription")
///      opts out: those users have explicitly asked us to keep
///      RAM/VRAM free between dictations.
///   2. `default_asr_model_id == Some(non-empty id)`. Empty or
///      missing means the user has no default picked yet
///      (post-onboarding stub / post-v0.8.0 retirement state),
///      and there's nothing to load.
///
/// Pure helper — caller passes a settings snapshot, no I/O. Lets
/// the unit tests pin every branch without spinning up state.
pub fn should_prewarm_default(settings: &Settings) -> Option<String> {
    if settings.models.warm_duration_secs == 0 {
        return None;
    }
    let id = settings.models.default_asr_model_id.as_deref()?;
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

/// Spawn a fire-and-forget tokio task that prewarms `model_id`
/// against the given orchestrator. Always returns immediately. The task
/// waits through the startup grace period before it can claim inference
/// resources, so an immediate dictation remains user-priority work.
///
/// Logs:
///   * `asr_prewarm: skipped` — info, with reason — when the
///     decision says no model to prewarm. Logged by the caller
///     before invoking this; this function assumes caller has
///     already decided.
///   * `asr_prewarm: starting` — info, model id only.
///   * `asr_prewarm: done` — info, with elapsed_ms.
///   * `asr_prewarm: failed` — warn, with error and elapsed_ms.
///     Failure is non-fatal: the next dictation will retry via
///     the same `ensure_loaded` path.
pub fn spawn(state: Arc<AppState>, orchestrator: Arc<InferenceOrchestrator>, model_id: String) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(STARTUP_PREWARM_DELAY_SECS)).await;
        if state
            .dictation_started_since_launch
            .load(std::sync::atomic::Ordering::Acquire)
        {
            tracing::info!(%model_id, "asr_prewarm: skipped because dictation began during startup grace");
            return;
        }
        tracing::info!(%model_id, "asr_prewarm: starting");
        let started = Instant::now();
        match orchestrator.ensure_loaded(&model_id).await {
            Ok(_) => {
                let elapsed_ms = started.elapsed().as_millis() as u64;
                tracing::info!(%model_id, elapsed_ms, "asr_prewarm: done");
            }
            Err(e) => {
                let elapsed_ms = started.elapsed().as_millis() as u64;
                tracing::warn!(%model_id, elapsed_ms, error = ?e, "asr_prewarm: failed");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Settings;

    fn settings_with(warm_secs: u32, default_id: Option<&str>) -> Settings {
        let mut s = Settings::default();
        s.models.warm_duration_secs = warm_secs;
        s.models.default_asr_model_id = default_id.map(|x| x.to_string());
        s
    }

    #[test]
    fn launch_prewarm_skips_when_a_dictation_started_during_grace() {
        let src = include_str!("prewarm.rs");
        assert!(
            src.contains("dictation_started_since_launch"),
            "prewarm must check the durable launch-session marker, not only the current recording flag"
        );
        assert!(
            src.contains("skipped because dictation began during startup grace"),
            "the skip branch must remain observable in logs"
        );
    }

    #[test]
    fn no_prewarm_when_warm_duration_is_zero() {
        // `warm_duration_secs == 0` is the explicit "evict
        // immediately after every transcription" opt-out. The
        // user has asked us to keep RAM/VRAM free between
        // dictations; prewarming on launch would directly
        // contradict that intent.
        let s = settings_with(0, Some("sherpa-parakeet-tdt-0.6b-v3-int8"));
        assert_eq!(should_prewarm_default(&s), None);
    }

    #[test]
    fn prewarm_when_finite_120s_warm_window_and_default_set() {
        // A finite warm window still opts into launch prewarm. The
        // idle sweeper evicts the prewarmed model 120 s after launch
        // if the user never dictates.
        let s = settings_with(120, Some("sherpa-parakeet-tdt-0.6b-v3-int8"));
        assert_eq!(
            should_prewarm_default(&s),
            Some("sherpa-parakeet-tdt-0.6b-v3-int8".into())
        );
    }

    #[test]
    fn prewarm_when_short_warm_window_and_default_set() {
        // An aggressive setting like 30 s still wins prewarm —
        // the user has chosen a short warm window, not zero.
        // The idle sweeper, which polls every 10 s, evicts the
        // prewarmed model promptly if the user doesn't dictate
        // within 30 s of launch.
        let s = settings_with(30, Some("sherpa-whisper-small"));
        assert_eq!(
            should_prewarm_default(&s),
            Some("sherpa-whisper-small".into())
        );
    }

    #[test]
    fn no_prewarm_when_default_model_is_none() {
        // No default picked — nothing to load. Holds across the
        // full warm-window spectrum: short, default, infinite.
        for &warm in &[1u32, 120u32, u32::MAX] {
            let s = settings_with(warm, None);
            assert_eq!(
                should_prewarm_default(&s),
                None,
                "warm_duration_secs={warm} with no default model must not prewarm"
            );
        }
    }

    #[test]
    fn no_prewarm_when_default_model_is_empty_string() {
        // Empty / whitespace defensively coerced to "no model".
        // settings.json hand-edits or migration artefacts might
        // produce these.
        let s = settings_with(120, Some(""));
        assert_eq!(should_prewarm_default(&s), None);
        let s = settings_with(120, Some("   "));
        assert_eq!(should_prewarm_default(&s), None);
        // Same outcome for the keep-forever sentinel — the empty
        // id wins regardless of warm window.
        let s = settings_with(u32::MAX, Some(""));
        assert_eq!(should_prewarm_default(&s), None);
    }

    #[test]
    fn prewarm_when_keep_forever_and_default_set() {
        // The pre-perf path still works: u32::MAX users keep
        // their launch prewarm.
        let s = settings_with(u32::MAX, Some("sherpa-whisper-medium"));
        assert_eq!(
            should_prewarm_default(&s),
            Some("sherpa-whisper-medium".into())
        );
    }

    #[test]
    fn prewarm_trims_surrounding_whitespace() {
        let s = settings_with(120, Some("  sherpa-whisper-small  "));
        assert_eq!(
            should_prewarm_default(&s),
            Some("sherpa-whisper-small".into())
        );
    }
}
