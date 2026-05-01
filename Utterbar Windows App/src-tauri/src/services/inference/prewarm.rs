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
//! When the user has set `models.warm_duration_secs == u32::MAX`
//! they have already opted into "keep the model resident
//! forever" — the idle sweeper in
//! `InferenceOrchestrator::spawn_idle_sweeper` no-ops on that
//! sentinel. There is no reason to make them wait until first
//! dictation to pay the cost. Prewarming on app startup turns
//! their first dictation into a cache-hit fast-path and
//! shifts the cost into the post-launch background tick where
//! the user is already waiting for the rest of the UI to
//! settle.
//!
//! Constraints the helper honours:
//!   * Non-blocking — the caller hands off to a tokio task and
//!     returns immediately. Startup must never block on a model
//!     load.
//!   * Failure-tolerant — warm errors are logged at warn level
//!     and swallowed. The first dictation will still attempt
//!     load through the same path and surface its own error.
//!   * Race-free with active dictation — reuses
//!     `InferenceOrchestrator::ensure_loaded`, which holds the
//!     write lock around build+warm. If a dictation arrives
//!     mid-prewarm, the orchestrator's existing TOCTOU defence
//!     handles it. If a dictation has already loaded the model
//!     before the prewarm task runs, `ensure_loaded` returns
//!     the cached provider in microseconds.
//!   * Meeting-mode model NOT prewarmed — meeting mode is
//!     opt-in per dictation; prewarming a separate Whisper
//!     Medium for users who never invoke it would double the
//!     idle RAM cost. Only the `default_asr_model_id` is
//!     prewarmed, which is what fires on every non-meeting
//!     dictation.

use std::sync::Arc;
use std::time::Instant;

use crate::domain::Settings;
use crate::services::inference::InferenceOrchestrator;

/// Pure decision: should the app prewarm a default ASR model on
/// launch, and if so which model id?
///
/// Returns `Some(model_id)` only when both:
///   1. `warm_duration_secs == u32::MAX` (the "keep loaded
///      forever" sentinel — see the orchestrator's idle
///      sweeper). Other settings either evict the model after
///      transcribe (0) or after N seconds (other), and prewarming
///      against those would either be wasted work or fight the
///      sweeper.
///   2. `default_asr_model_id == Some(non-empty id)`. Empty or
///      missing means the user has no default picked yet
///      (post-onboarding stub / post-v0.8.0 retirement state),
///      and there's nothing to load.
///
/// Pure helper — caller passes a settings snapshot, no I/O. Lets
/// the unit tests pin every branch without spinning up state.
pub fn should_prewarm_default(settings: &Settings) -> Option<String> {
    if settings.models.warm_duration_secs != u32::MAX {
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
/// against the given orchestrator. Always returns immediately.
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
pub fn spawn(orchestrator: Arc<InferenceOrchestrator>, model_id: String) {
    tauri::async_runtime::spawn(async move {
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
    fn no_prewarm_when_warm_duration_is_zero() {
        let s = settings_with(0, Some("sherpa-parakeet-tdt-0.6b-v3-int8"));
        assert_eq!(should_prewarm_default(&s), None);
    }

    #[test]
    fn no_prewarm_when_warm_duration_is_finite() {
        // The default of 120 s is the "evict after 2 minutes idle"
        // setting. Prewarm would race the sweeper.
        let s = settings_with(120, Some("sherpa-parakeet-tdt-0.6b-v3-int8"));
        assert_eq!(should_prewarm_default(&s), None);
    }

    #[test]
    fn no_prewarm_when_default_model_is_none() {
        // u32::MAX is the "keep loaded forever" sentinel, but
        // there's nothing to keep loaded.
        let s = settings_with(u32::MAX, None);
        assert_eq!(should_prewarm_default(&s), None);
    }

    #[test]
    fn no_prewarm_when_default_model_is_empty_string() {
        // Empty / whitespace defensively coerced to "no model".
        // settings.json hand-edits or migration artefacts might
        // produce these.
        let s = settings_with(u32::MAX, Some(""));
        assert_eq!(should_prewarm_default(&s), None);
        let s = settings_with(u32::MAX, Some("   "));
        assert_eq!(should_prewarm_default(&s), None);
    }

    #[test]
    fn prewarm_when_keep_forever_and_default_set() {
        let s = settings_with(u32::MAX, Some("sherpa-whisper-medium"));
        assert_eq!(
            should_prewarm_default(&s),
            Some("sherpa-whisper-medium".into())
        );
    }

    #[test]
    fn prewarm_trims_surrounding_whitespace() {
        let s = settings_with(u32::MAX, Some("  sherpa-whisper-small  "));
        assert_eq!(
            should_prewarm_default(&s),
            Some("sherpa-whisper-small".into())
        );
    }
}
