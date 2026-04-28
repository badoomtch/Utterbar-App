//! Dictation lifecycle. Exposes start / stop / cancel / phase.

use std::sync::{Arc, Mutex, OnceLock};

use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;

use crate::app_state::AppState;
use crate::domain::{
    DictationPhase, HistoryEntry, HistoryStatus, EVT_DICTATION_ERROR, EVT_DICTATION_PHASE,
    EVT_DICTATION_RESULT,
};
use crate::services::audio::meter;
use crate::services::inference::{AsrOptions, AsrResult, AudioBuffer};
use crate::util::result::{CmdResult, OrliqError};
use crate::util::time::now_ms;

// Phase mirrored here for the poll fallback. Events are the authoritative source of truth.
static PHASE: OnceLock<RwLock<DictationPhase>> = OnceLock::new();

fn phase_cell() -> &'static RwLock<DictationPhase> {
    PHASE.get_or_init(|| RwLock::new(DictationPhase::Idle))
}

/// Tracks the tail of the last successful delivery so we can stitch a missing space onto
/// the next one when the user is clearly continuing in the same target field. Intentionally
/// narrow — we only key off app exe + a short time window to keep false positives down.
#[derive(Clone)]
struct LastDelivery {
    at_ms: i64,
    app_exe: Option<String>,
    ends_with_whitespace_or_open_punct: bool,
}

static LAST_DELIVERY: OnceLock<Mutex<Option<LastDelivery>>> = OnceLock::new();

fn last_delivery() -> &'static Mutex<Option<LastDelivery>> {
    LAST_DELIVERY.get_or_init(|| Mutex::new(None))
}

/// ~45 s window. Long enough to cover the natural pause between two dictations
/// describing the same thought, short enough that if the user switched contexts and
/// came back we won't glue an errant space onto what they're now typing manually.
const AUTO_SPACE_WINDOW_MS: i64 = 45_000;

fn should_prepend_space(now_ms: i64, current_app_exe: Option<&str>, next_text: &str) -> bool {
    let Some(prev) = last_delivery().lock().ok().and_then(|g| g.clone()) else {
        return false;
    };
    if now_ms - prev.at_ms > AUTO_SPACE_WINDOW_MS {
        return false;
    }
    // Same-app gate: if the active app changed, assume the user is in a different field.
    if prev.app_exe.as_deref() != current_app_exe {
        return false;
    }
    if prev.ends_with_whitespace_or_open_punct {
        return false;
    }
    // Only prepend when the new text starts with an alnum — don't inject a space before
    // punctuation like a closing quote, a period, or a comma.
    next_text
        .chars()
        .next()
        .is_some_and(|c| c.is_alphanumeric())
}

fn record_delivery(now_ms: i64, app_exe: Option<&str>, delivered: &str) {
    let ends_open = delivered
        .chars()
        .next_back()
        .is_none_or(|c| c.is_whitespace() || matches!(c, '(' | '[' | '{' | '"' | '\''));
    let _ = last_delivery().lock().map(|mut g| {
        *g = Some(LastDelivery {
            at_ms: now_ms,
            app_exe: app_exe.map(|s| s.to_string()),
            ends_with_whitespace_or_open_punct: ends_open,
        });
    });
}

#[tauri::command]
pub async fn start_recording(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    device_id: Option<String>,
) -> CmdResult<()> {
    // Group-policy gate. An IT admin in a managed deployment can pin
    // `RequireLicence=1` under HKLM\SOFTWARE\Policies\Utterbar to
    // disallow the free tier — typically because Pro is the approved
    // SKU and the IT team has already pushed a licence to the machine.
    // We surface a clear OrliqError::User so the floating bar shows
    // "your IT admin requires a Pro licence" rather than a generic
    // refusal. Cheap registry read; only happens at the start of a
    // dictation, not in the audio loop.
    if crate::services::policy::Policy::load().licence_required() {
        let status = state.licence.status().await;
        if !crate::util::gating::is_pro(&status) {
            tracing::warn!(
                "start_recording refused: RequireLicence policy active and licence is not Pro"
            );
            return Err(OrliqError::User {
                message: "Your IT administrator requires a Pro licence on this machine. Contact them to activate Utterbar.".into(),
            });
        }
    }

    // Log the guard state on every entry. The flags are the canonical
    // source of "is a capture/transcribe actually in-flight?" — visualising
    // their value at entry makes stuck-state bugs diagnosable from a log
    // tail alone rather than needing a live debugger.
    let rec_flag = state
        .recording_in_progress
        .load(std::sync::atomic::Ordering::Acquire);
    let trx_flag = state
        .transcribe_in_progress
        .load(std::sync::atomic::Ordering::Acquire);
    // Capture an Instant here so we can log total hotkey-to-listening
    // latency at the bottom of start_recording. Useful for diagnosing
    // perceived sluggishness without needing the user to grok wall-
    // clock differences between log lines.
    let start_t = std::time::Instant::now();
    tracing::info!(
        ?device_id,
        recording_in_progress = rec_flag,
        transcribe_in_progress = trx_flag,
        "dictation::start_recording enter"
    );

    // Re-entry guard. Before v0.6.2, a single hotkey press could fire three
    // concurrent invocations (one per webview receiving the broadcast
    // `shortcut:triggered` event). All three armed their own capture stream
    // and raced on sherpa-onnx's recognizer construction — reliably
    // corrupting the heap (STATUS_HEAP_CORRUPTION, 0xC0000374). The
    // frontend fan-out fix gates the broadcast; this CAS is defence in
    // depth so a future frontend regression can't re-introduce the same
    // crash.
    //
    // v0.6.3 self-heal: if the flag is stuck true but the audio service
    // has no active capture, the guard has desynced from reality — most
    // likely because an earlier transcribe returned an error that
    // short-circuited the normal `stop_recording → clear flag` path,
    // or because a dropped dictation-phase event left the frontend
    // thinking it was idle while the Rust flag was still set. Without
    // the self-heal, the user's hotkey silently does nothing until
    // they restart the app. With it, we detect the stale flag, clear
    // it, log a warning (so a real regression is still visible in the
    // log), and proceed with the start as normal.
    if state
        .recording_in_progress
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire,
        )
        .is_err()
    {
        // Probe the audio service: is there genuinely a live capture?
        let has_live_capture = state.audio.current().await.is_some();
        if has_live_capture {
            // Re-emit the authoritative Listening phase. When the frontend's
            // dictation store has desynced from Rust (observed 2026-04-24:
            // phase store was stuck at idle/processing/success while the
            // capture was still running), the UI thinks it isn't recording
            // and every hotkey press routes to start — all of which we
            // reject. Pushing a fresh `Listening` phase resets the frontend
            // so the user's NEXT press hits the stop branch correctly.
            // We don't persist the original recording_started_at_ms, so the
            // re-emitted Listening phase carries the current wallclock.
            // This is strictly a UI resync path — the user has already
            // been recording for some time, but we don't have the exact
            // start timestamp handy. The duration counter in FloatingBar
            // resets, which is a small UX compromise vs. staying wedged.
            tracing::warn!(
                "start_recording rejected: another start is already in progress — re-emitting Listening phase to re-sync frontend"
            );
            set_phase(
                &handle,
                DictationPhase::Listening {
                    started_at_ms: now_ms(),
                },
            )
            .await;
            emit_busy(&handle);
            return Err(OrliqError::User {
                message: "recording already in progress".into(),
            });
        }
        // Flag was true but no capture is running — stale. Clear it and
        // fall through. We deliberately don't re-CAS here: the flag is
        // already true from our perspective, so we just keep it that
        // way for the rest of this invocation.
        tracing::warn!(
            "start_recording: recording_in_progress flag was stuck true with no active capture — clearing as stale and proceeding"
        );
    }

    // Transcribe-in-flight rejection. The inference orchestrator
    // serialises with a single write lock, so even if we armed a new
    // capture here we'd stall when `stop_recording` tries to transcribe
    // — and we'd tempt the concurrent-native-teardown race that's the
    // suspected cause of the v0.6.3 heap-corruption regression.
    //
    // Stale self-heal: if `transcribe_in_progress` has been true for
    // more than 30 s it is stale (transcribe hung, panic in post-
    // processing, dropped error path). Clear it + proceed rather than
    // wedge the user out of the app. 30 s covers real transcribe times
    // with plenty of headroom — the log shows a 25-second recording
    // finishing its transcribe in 1.2 s.
    const TRANSCRIBE_STALE_MS: i64 = 30_000;
    if state
        .transcribe_in_progress
        .load(std::sync::atomic::Ordering::Acquire)
    {
        let started_at = state
            .transcribe_started_at_ms
            .load(std::sync::atomic::Ordering::Acquire);
        let stale = started_at > 0 && (now_ms() - started_at) > TRANSCRIBE_STALE_MS;
        if !stale {
            tracing::warn!(
                transcribe_started_at_ms = started_at,
                "start_recording rejected: transcribe still in flight"
            );
            // Clear our recording_in_progress CAS — we're about to bail
            // without actually starting, so leaving it true would wedge
            // the next press.
            state
                .recording_in_progress
                .store(false, std::sync::atomic::Ordering::Release);
            emit_busy(&handle);
            return Err(OrliqError::User {
                message: "still transcribing — wait a moment".into(),
            });
        }
        tracing::warn!(
            elapsed_ms = now_ms() - started_at,
            "start_recording: transcribe_in_progress flag was stuck for >30s — clearing as stale and proceeding"
        );
        state
            .transcribe_in_progress
            .store(false, std::sync::atomic::Ordering::Release);
        state
            .transcribe_started_at_ms
            .store(0, std::sync::atomic::Ordering::Release);
    }

    let settings = state.settings.snapshot().await;

    use crate::domain::settings::CaptureSource;

    // Meeting mode override. When the active mode is flagged as a meeting,
    // force capture to Mixed (mic + system audio) regardless of the global
    // capture_source preference. The override is per-recording — we don't
    // mutate the persisted setting. A user who has set capture_source to
    // System for everyday use, then runs a meeting once, gets Mixed for
    // that recording and System back on the next non-meeting press.
    //
    // Resolved here at start_recording rather than at stop_recording so
    // the loopback handle can be armed alongside the mic in the same
    // start path. Stop time only sees the already-running streams.
    let active_mode_for_capture = settings
        .active_mode_id
        .as_deref()
        .and_then(|id| state.modes.get(id).ok().flatten());
    let is_meeting = active_mode_for_capture
        .as_ref()
        .map(|m| m.is_meeting_mode)
        .unwrap_or(false);
    let source = if is_meeting {
        tracing::info!(
            mode = active_mode_for_capture.as_ref().map(|m| m.id.as_str()),
            "start_recording: meeting mode — forcing Mixed capture (mic + loopback)"
        );
        CaptureSource::Mixed
    } else {
        settings.audio.capture_source
    };

    // Track which side-effects have fired so we can roll them back if capture fails.
    // We only flip the visible phase to `Listening` once we know the mic (or loopback)
    // is armed — the UI must not show a live waveform when nothing is recording.
    let mut media_paused = false;
    let mut conferencing_ducked = false;

    // Pause any media that's currently playing (Spotify / YouTube / etc.) so the user's
    // speech isn't mixing with music. Skipped for System/Mixed capture — pausing would
    // defeat the entire point of transcribing what's playing.
    let should_pause_media =
        settings.privacy.pause_media_while_recording && matches!(source, CaptureSource::Microphone);
    if should_pause_media {
        let media = state.media.clone();
        tokio::task::spawn_blocking(move || media.pause_playing());
        media_paused = true;
    }

    // Mute Teams / Slack / Discord / Zoom output while recording, so incoming call
    // audio doesn't distract. Only relevant when we're actually capturing the mic —
    // `System`/`Mixed` sources deliberately want to include those apps' audio.
    if settings.privacy.mute_conferencing_while_recording
        && matches!(source, CaptureSource::Microphone)
    {
        let ducking = state.ducking.clone();
        let override_list = settings.privacy.conferencing_apps_override.clone();
        let targets: Vec<String> = if override_list.is_empty() {
            crate::services::audio_ducking::DEFAULT_CONFERENCING_APPS
                .iter()
                .map(|s| (*s).to_string())
                .collect()
        } else {
            override_list
        };
        tokio::task::spawn_blocking(move || ducking.mute_matching(&targets));
        conferencing_ducked = true;
    }

    // Start whichever source(s) the user selected. Microphone is the default; System
    // uses WASAPI loopback; Mixed runs both and sums them at stop time. On failure we
    // undo every side-effect we accrued above — nothing must linger in a half-armed
    // state, and the visible phase must stay at whatever it was before (i.e. not
    // Listening), so the UI shows an error instead of a dead waveform.
    // Device-id resolution: front-end callers (the floating bar's
    // hotkey path, the tray-menu start, AppBootstrap recovery) pass
    // `device_id: None` because they don't read the audio settings
    // store directly. The contract is "None means use whatever the
    // user has configured, falling back to the host default if
    // they haven't configured one". Pre-fix we treated `None` as
    // "always Windows default" which silently swapped the user's
    // selected mic for whatever Windows had defaulted to — typically
    // the laptop's built-in array mic on machines with a USB
    // interface plugged in. Reproduced 2026-04-28 on Jay's machine:
    // Focusrite selected in Audio settings, recording captured from
    // Intel Smart Sound array.
    let configured_device_id = settings.audio.input_device_id.clone();
    let effective_device_id = device_id.clone().or_else(|| configured_device_id.clone());
    tracing::info!(
        requested_device_id = ?device_id,
        configured_device_id = ?configured_device_id,
        effective_device_id = ?effective_device_id,
        "start_recording: resolving input device"
    );

    let mic_capture = if matches!(source, CaptureSource::Microphone | CaptureSource::Mixed) {
        let mic_t = std::time::Instant::now();
        match state.audio.start_capture(effective_device_id.clone()).await {
            Ok((cap_handle, meta)) => {
                tracing::info!(
                    mic_arm_ms = mic_t.elapsed().as_millis() as u64,
                    resolved_device_name = %meta.resolved_device_name,
                    fallback = ?meta.fallback_reason,
                    "start_capture returned"
                );
                // Surface a fallback (configured device missing,
                // enumeration failed) to the user as a transient
                // dictation error. It's not fatal — recording still
                // works on the host default — but the user has to
                // know they aren't being recorded by their selected
                // mic, otherwise transcripts come out garbled and
                // the failure mode is invisible from the bar.
                if let Some(reason) = &meta.fallback_reason {
                    let msg = reason.user_message(&meta.resolved_device_name);
                    tracing::warn!(
                        ?reason,
                        resolved_device_name = %meta.resolved_device_name,
                        "input-device fallback occurred — notifying user"
                    );
                    let _ = handle.emit(EVT_DICTATION_ERROR, &msg);
                }
                Some(cap_handle)
            }
            Err(e) => {
                tracing::error!(error = %e, "audio.start_capture failed");
                rollback_start_sideeffects(&state, media_paused, conferencing_ducked);
                state
                    .recording_in_progress
                    .store(false, std::sync::atomic::Ordering::Release);
                set_phase(
                    &handle,
                    DictationPhase::Error {
                        message: format!("mic unavailable: {e}"),
                    },
                )
                .await;
                schedule_idle(&handle, 1200);
                return Err(OrliqError::Device {
                    message: e.to_string(),
                });
            }
        }
    } else {
        None
    };

    #[cfg(windows)]
    if matches!(source, CaptureSource::System | CaptureSource::Mixed) {
        if let Err(e) = state.audio.start_loopback().await {
            tracing::error!(error = %e, "audio.start_loopback failed");
            // A mic was already armed — shut it back down so we don't strand a live
            // capture thread when the secondary source failed.
            if mic_capture.is_some() {
                let _ = state.audio.stop_capture().await;
            }
            rollback_start_sideeffects(&state, media_paused, conferencing_ducked);
            state
                .recording_in_progress
                .store(false, std::sync::atomic::Ordering::Release);
            set_phase(
                &handle,
                DictationPhase::Error {
                    message: format!("loopback unavailable: {e}"),
                },
            )
            .await;
            schedule_idle(&handle, 1200);
            return Err(OrliqError::Device {
                message: e.to_string(),
            });
        }
    }

    // All capture paths armed — only now do we flip the visible phase to Listening. The
    // `started_at_ms` tag drives the elapsed-time counter on the floating bar, so it has
    // to line up with the moment real audio started flowing, not the moment the command
    // entered.
    set_phase(
        &handle,
        DictationPhase::Listening {
            started_at_ms: now_ms(),
        },
    )
    .await;

    // Drive the meter from the mic when available; fall back to a no-op when we're only
    // capturing loopback (the waveform animation still has its own fallback jitter).
    if let Some(cap) = mic_capture {
        meter::spawn_pump(handle, cap);
    }
    tracing::info!(
        ?source,
        elapsed_ms = start_t.elapsed().as_millis() as u64,
        "audio capture armed"
    );
    Ok(())
}

/// Reverse the ordered side-effects we kick off at the top of `start_recording` when
/// capture arming subsequently fails. Kept as a named helper so both the mic-failed and
/// loopback-failed branches roll back identically.
fn rollback_start_sideeffects(state: &AppState, media_paused: bool, conferencing_ducked: bool) {
    if media_paused {
        let media = state.media.clone();
        tokio::task::spawn_blocking(move || media.resume_paused());
    }
    if conferencing_ducked {
        let ducking = state.ducking.clone();
        tokio::task::spawn_blocking(move || ducking.restore());
    }
}

/// Schedule a return-to-idle phase transition after `delay_ms`, but only
/// if the current phase is still a **terminal** one (success / error /
/// cancelled / no_audio). If the user has already started a new
/// recording before the timer fires, the current phase will be
/// `Listening` (or `Warming` / `Processing`) and we leave it alone.
///
/// v0.6.7 bugfix: previously this fired `set_phase(Idle)`
/// unconditionally, which caused a 1 in ~5 "the app stopped my
/// recording" failure mode. Timeline observed in the log:
///   t=0.000 stop_recording → set_phase=success, schedule_idle(1200ms)
///   t=1.000 user re-presses → start_recording → set_phase=listening
///   t=1.200 stale timer fires → set_phase=idle ← WRONG
///   t=2.200 user presses again → frontend reads phase=idle → start_recording
///                               → Rust rejects (capture still armed)
///                               → self-heal stop_recording fires
///                               → real recording aborted mid-way
///
/// Fixing at the Idle emit is the right layer: the stale timer is the
/// only code path that transitions AWAY from an active phase without a
/// corresponding command-level trigger. Checking the current phase at
/// fire time catches every race with `stop → success → schedule_idle →
/// start` sequences, regardless of how tight the user's rhythm is.
fn schedule_idle(handle: &AppHandle, delay_ms: u64) {
    let handle = handle.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        // Only decay to Idle if the phase is still the one we scheduled
        // from. If a new recording has begun (Listening / Warming /
        // Processing) the timer is stale and must not clobber it.
        let current = phase_cell().read().await.clone();
        match current {
            DictationPhase::Success { .. }
            | DictationPhase::Error { .. }
            | DictationPhase::Cancelled
            | DictationPhase::NoAudio => {
                set_phase(&handle, DictationPhase::Idle).await;
            }
            DictationPhase::Idle => {
                // Already idle (e.g. a previous scheduled timer beat us).
                // No-op.
            }
            DictationPhase::Listening { .. }
            | DictationPhase::Warming { .. }
            | DictationPhase::Processing
            | DictationPhase::TranscribingBackground => {
                // Active phase started between the scheduling and now.
                // Leaving it alone is the whole point of this guard.
                tracing::info!(
                    ?current,
                    "schedule_idle: current phase is active — skipping stale Idle emit"
                );
            }
        }
    });
}

/// Per-window RMS for speaker tagging. Walks `pcm[start..end]` once and
/// returns mean square of the i16 samples (no sqrt; we only ever compare
/// two RMS values against each other and saving the sqrt is cheaper).
/// Bounds-safe — clamps end to pcm.len() and returns 0.0 on empty range
/// so a missing-side capture trivially loses the tag race.
/// Resolve which ASR model to use for this dictation. One source of
/// truth for the early-warm probe, the cold-load Warming pill, and the
/// final transcribe call — they must all read the same value or the UI
/// flashes the wrong "Initialising model" copy when the user has
/// Meeting mode auto-routing to a different model than the global.
///
/// Rules:
///   * Meeting modes auto-route to Whisper Large v3 when it's
///     installed. The user shouldn't have to switch the global model
///     dropdown when they pick Meeting mode; the mode itself implies
///     "I want the best accuracy regardless of speed".
///   * Everything else uses the global default. The mode-level
///     `asr_model_id` field is still ignored at runtime (removed as a
///     runtime override in v0.6.6; field stays on disk for back-
///     compat).
///   * Fallback is the literal string "stub", same sentinel the
///     orchestrator already routes to a no-op provider when no real
///     model is configured. Surfacing it in the UI as a model id is
///     fine: the dictation flow surfaces the standard "no model
///     installed" prompt before any sherpa call happens.
fn resolve_target_model_id(
    mode: &crate::domain::Mode,
    settings: &crate::domain::Settings,
    models: &crate::services::models::ModelRegistry,
) -> String {
    const WHISPER_LARGE: &str = "sherpa-whisper-large-v3";
    if mode.is_meeting_mode && models.is_installed(WHISPER_LARGE) {
        return WHISPER_LARGE.to_string();
    }
    settings
        .models
        .default_asr_model_id
        .clone()
        .unwrap_or_else(|| "stub".into())
}

fn rms_over(pcm: &[i16], start: usize, end: usize) -> f64 {
    let lo = start.min(pcm.len());
    let hi = end.min(pcm.len());
    if hi <= lo {
        return 0.0;
    }
    let mut sum_sq: f64 = 0.0;
    for &s in &pcm[lo..hi] {
        let f = s as f64;
        sum_sq += f * f;
    }
    sum_sq / (hi - lo) as f64
}

/// Meeting Mode chunked transcribe.
///
/// Slices the captured PCM into ~30 s windows with a 2 s overlap and
/// runs each chunk through the offline ASR provider. The chunk
/// boundary is wall-clock based (not silence-aware) — silence-aware
/// chunking is a future polish; the 2 s overlap lets the recogniser
/// stitch boundary words back together rather than cleaving them.
///
/// Why 30 s + 2 s: the offline transcribe path's failure mode is O(N)
/// memory + accumulated decoder state inside ONNX Runtime. 30 s is
/// well below the empirical crash floor (~30 min on the offline
/// Zipformer, but per-call memory grows long before that). 2 s is
/// roughly two-words of speech at conversational pace — enough to
/// catch the boundary word in both chunks for the dedup pass.
///
/// Stitching: each non-first chunk has its first ~6.7 % of words
/// dropped (the proportional overlap fraction, 2 s / 30 s). This is
/// imperfect — words don't distribute perfectly evenly across an audio
/// chunk — but bounded error and trivially correct.
///
/// Output formatting (v0.13.2):
///   - Each non-empty chunk is prefixed with `[mm:ss]` derived from the
///     chunk's wall-clock start (relative to the recording start).
///   - When mic and loopback PCM were both available (the typical meeting-
///     mode case where capture_source was forced to Mixed), a `**You**:`
///     or `**Other**:` speaker tag is appended after the timestamp,
///     chosen by which side had higher RMS over that chunk's audio
///     window. With only one side present (mic-only or loopback-only)
///     the tag is omitted — the label would be trivially the same on
///     every chunk.
///
/// duration_ms on the returned AsrResult is the wall-clock duration
/// of the source audio (sum of chunk durations less overlaps). Other
/// fields are taken from the first chunk's result; word_timings is
/// dropped because per-chunk timings would need rebasing to remain
/// useful and meeting-mode rows don't render them today.
#[allow(clippy::too_many_arguments)]
async fn transcribe_chunked(
    state: &Arc<AppState>,
    model_id: &str,
    pcm: Vec<i16>,
    mic_pcm: Option<Vec<i16>>,
    loop_pcm: Option<Vec<i16>>,
    sample_rate: u32,
    target_language: Option<String>,
    vocabulary_hints: Vec<String>,
) -> anyhow::Result<AsrResult> {
    const CHUNK_SECONDS: usize = 30;
    const OVERLAP_SECONDS: usize = 2;

    let chunk_samples = CHUNK_SECONDS * sample_rate as usize;
    let overlap_samples = OVERLAP_SECONDS * sample_rate as usize;
    let stride = chunk_samples.saturating_sub(overlap_samples);

    let total = pcm.len();
    if total == 0 {
        return Err(anyhow::anyhow!("empty PCM buffer"));
    }

    // Single-chunk fast path. If the whole recording fits in one
    // window, no chunking is necessary — just call transcribe directly
    // and skip the stitching dance. Saves on logging noise too.
    if total <= chunk_samples {
        return state
            .inference
            .transcribe(
                model_id,
                AudioBuffer {
                    pcm,
                    sample_rate,
                    channels: 1,
                },
                AsrOptions {
                    language_hint: target_language,
                    want_word_timings: false,
                    vocabulary_hints,
                },
            )
            .await;
    }

    // Loopback noise floor — calibrated over the whole recording so the
    // per-chunk speaker tag is robust to whatever system volume the user
    // had set. We walk the loopback PCM in non-overlapping CHUNK windows,
    // collect the RMS of each, and take the smallest as the "quiet"
    // baseline. A chunk's loopback is then "speaking" if its RMS is
    // greater than `floor * THRESHOLD_MULT` — that means the system
    // audio is producing real signal in that window, not just hiss.
    //
    // Why this beats the loudest-wins heuristic: in a real meeting setup
    // the mic picks up the loopback bleeding from the user's speakers,
    // so the mic's RMS is always (your voice + bleed) while loopback
    // alone is just (their voice). The loudest-wins comparison thus
    // always picks mic and tags every chunk as **You**, defeating the
    // entire point of the tag. Threshold-against-own-floor sidesteps
    // the comparison: it asks "did system audio produce signal at all?"
    // If yes → tag **Other** (system speaker is active, regardless of
    // whether the user is too); if no → tag **You** (only the mic was
    // active). Imperfect when both speak simultaneously — we'd tag
    // **Other** because the loopback signal is real — but that's a
    // truer label than the prior behaviour.
    const THRESHOLD_MULT: f64 = 4.0;
    let loopback_floor: Option<f64> = loop_pcm.as_ref().map(|lp| {
        let mut floor = f64::INFINITY;
        let mut probe = 0usize;
        while probe < lp.len() {
            let probe_end = (probe + chunk_samples).min(lp.len());
            let r = rms_over(lp, probe, probe_end);
            if r < floor {
                floor = r;
            }
            probe += chunk_samples;
        }
        // If we never advanced (empty loopback), return 0.0 so any
        // non-zero per-chunk RMS trivially exceeds it. Better to over-
        // tag **Other** than miss real speech.
        if floor.is_finite() {
            floor
        } else {
            0.0
        }
    });
    if let Some(floor) = loopback_floor {
        tracing::info!(
            loopback_floor_rms = floor,
            threshold = floor * THRESHOLD_MULT,
            "meeting mode: loopback noise floor calibrated"
        );
    }

    let mut start = 0usize;
    let mut chunk_index = 0u32;
    let mut stitched = String::new();
    let mut total_duration_ms: u32 = 0;
    let mut first_lang: Option<String> = None;
    let mut model_version = String::new();

    while start < total {
        let end = (start + chunk_samples).min(total);
        let slice = pcm[start..end].to_vec();
        let slice_secs = (end - start) as f32 / sample_rate as f32;
        tracing::info!(
            chunk_index,
            start_sample = start,
            end_sample = end,
            duration_secs = slice_secs,
            "meeting mode: transcribing chunk"
        );

        let result = state
            .inference
            .transcribe(
                model_id,
                AudioBuffer {
                    pcm: slice,
                    sample_rate,
                    channels: 1,
                },
                AsrOptions {
                    language_hint: target_language.clone(),
                    want_word_timings: false,
                    vocabulary_hints: vocabulary_hints.clone(),
                },
            )
            .await?;

        // Trim the leading overlap region from every chunk except the
        // first. We don't have word-level timings (offline Zipformer
        // doesn't emit them, Parakeet's are imperfect), so drop the
        // first OVERLAP/CHUNK fraction of words. Conservative — better
        // to drop a duplicate word at a boundary than to leave it in.
        let chunk_text = if chunk_index == 0 {
            result.text.clone()
        } else {
            let words: Vec<&str> = result.text.split_whitespace().collect();
            let drop_count = (words.len() * OVERLAP_SECONDS / CHUNK_SECONDS).min(words.len());
            words[drop_count..].join(" ")
        };

        if !chunk_text.trim().is_empty() {
            // Timestamp prefix derived from the chunk's wall-clock start.
            // Non-first chunks contribute starting at `start + overlap`
            // because their first ~overlap seconds were dropped from the
            // text by the dedup pass above; the listener's mental model
            // is "where in the recording does this text begin?" — that's
            // start+overlap for chunk N, start (=0) for chunk 0.
            let chunk_text_start_samples = if chunk_index == 0 {
                start
            } else {
                start + overlap_samples
            };
            let chunk_text_start_secs = chunk_text_start_samples / sample_rate as usize;
            let mm = chunk_text_start_secs / 60;
            let ss = chunk_text_start_secs % 60;

            // Speaker-tag heuristic computation, log-only in v0.14.3.
            //
            // The previous "loopback above its own floor" rule got the
            // tag wrong on real recordings: in a typical setup the mic
            // picks up bleed from the speakers, the loopback can be
            // empty (output device is headphones, BT, or virtual
            // routing), and the threshold scaling was on mean-square
            // not RMS. Suppressing the **You** / **Other** prefix
            // until the v0.14.4 signal-above-baseline heuristic ships,
            // because wrong labels are worse than no labels.
            //
            // The per-side RMS values are still emitted to the log so
            // the v0.14.4 calibration work has real-recording data to
            // tune against without re-running everyone's tests.
            if let (Some(mic), Some(lp), Some(floor)) =
                (mic_pcm.as_ref(), loop_pcm.as_ref(), loopback_floor)
            {
                let mic_rms = rms_over(mic, start, end);
                let lp_rms = rms_over(lp, start, end);
                tracing::info!(
                    chunk_index,
                    mic_rms,
                    lp_rms,
                    loopback_floor = floor,
                    "meeting mode: per-side energy (logged only; tags suppressed)"
                );
            }

            // One chunk per paragraph for legibility. Empty line between
            // chunks so a long meeting reads as a stack of timestamped
            // turns rather than a wall of text. Just the timestamp for
            // now; speaker tag returns in v0.14.4 with a heuristic
            // that actually works.
            if !stitched.is_empty() {
                stitched.push_str("\n\n");
            }
            stitched.push_str(&format!("[{:02}:{:02}]", mm, ss));
            stitched.push(' ');
            stitched.push_str(&chunk_text);
        }

        if first_lang.is_none() {
            first_lang = result.language.clone();
        }
        if model_version.is_empty() {
            model_version = result.model_version.clone();
        }

        // Sum non-overlapping audio time. The first chunk contributes
        // its full duration; later chunks contribute only their
        // post-overlap portion. The last chunk may be shorter than
        // CHUNK_SECONDS — handle that by using the actual slice length.
        let slice_ms = ((end - start) as u64 * 1000 / sample_rate as u64) as u32;
        let contribution_ms = if chunk_index == 0 {
            slice_ms
        } else {
            slice_ms.saturating_sub((OVERLAP_SECONDS as u32) * 1000)
        };
        total_duration_ms = total_duration_ms.saturating_add(contribution_ms);

        if end >= total {
            break;
        }
        start += stride;
        chunk_index += 1;
    }

    tracing::info!(
        chunks = chunk_index + 1,
        chars = stitched.len(),
        total_duration_ms,
        "meeting mode: stitch complete"
    );

    Ok(AsrResult {
        text: stitched,
        words: None,
        language: first_lang,
        confidence: None,
        model_version,
        duration_ms: total_duration_ms,
    })
}

#[tauri::command]
pub async fn stop_recording(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> CmdResult<HistoryEntry> {
    let rec_flag = state
        .recording_in_progress
        .load(std::sync::atomic::Ordering::Acquire);
    let trx_flag = state
        .transcribe_in_progress
        .load(std::sync::atomic::Ordering::Acquire);
    tracing::info!(
        recording_in_progress = rec_flag,
        transcribe_in_progress = trx_flag,
        "dictation::stop_recording enter"
    );
    // Pull mic and loopback streams separately. Non-meeting-mode paths
    // mix them immediately and continue with the existing single-buffer
    // flow; meeting mode stashes both sides so the chunked transcribe
    // can derive per-chunk speaker tags from per-side RMS before mixing
    // for ASR. The split is meeting-aware here rather than at the audio
    // service so the audio layer's contract stays simple — one path,
    // returns whatever the user asked for.
    let (mic_final, loop_final) = state.audio.stop_capture_split().await;
    let final_ = match (mic_final.clone(), loop_final.clone()) {
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (Some(a), Some(b)) => Some(crate::services::audio::mix_two(a, b)),
        (None, None) => None,
    };
    let Some(mut final_) = final_ else {
        tracing::warn!("stop_recording called with no active recording");
        // Clear the guard even on "no-op stop" — the post-v0.6.2 fan-out
        // fix means this branch is only hit when a client explicitly
        // double-taps the hotkey faster than Listening flips to Idle,
        // but we still want the next genuine start to succeed without
        // having to click "Reset" anywhere.
        state
            .recording_in_progress
            .store(false, std::sync::atomic::Ordering::Release);
        return Err(OrliqError::User {
            message: "no active recording".into(),
        });
    };
    // Capture drained — now we own the session. Release the guard so a
    // user who starts a new dictation while transcription is still
    // running isn't blocked. The orchestrator serialises work on its
    // own side (see InferenceOrchestrator::ensure_loaded), so
    // overlapping start-after-stop is safe from here.
    state
        .recording_in_progress
        .store(false, std::sync::atomic::Ordering::Release);
    tracing::info!(
        samples = final_.pcm.len(),
        sample_rate = final_.sample_rate,
        duration_ms = final_.duration_ms,
        "audio captured"
    );

    // Resolve the target model id ONCE, before the cold-load probe, so
    // every downstream consumer (early Warming label, ensure_loaded
    // call, final transcribe, history row) reads the same value.
    //
    // Resolution rules (see `resolve_target_model_id` below):
    //   * Meeting modes auto-route to Whisper Large v3 when it's
    //     installed. The user doesn't have to pick a meeting model
    //     manually; meeting mode picks the best one available.
    //   * All other modes use the global `default_asr_model_id`.
    //
    // The mode-level `asr_model_id` field on the Mode struct is still
    // ignored at runtime (removed as a runtime override in v0.6.6;
    // the field stays on disk for back-compat with old DB rows).
    //
    // Why the probe must use the same id as the final transcribe:
    // before this, the early probe read `default_asr_model_id` and the
    // final read the same — coincidentally identical. With the meeting
    // override they could diverge (probe reads default Parakeet,
    // transcribe routes to Whisper Large) and the user would see
    // "Processing" flash before the real "Initialising model" cold-load
    // pill. Resolving once and reusing the binding sidesteps that.
    let pre_settings = state.settings.snapshot().await;
    let pre_active_mode_id = pre_settings
        .active_mode_id
        .clone()
        .unwrap_or_else(|| "voice-to-text".into());
    let pre_active_mode = state
        .modes
        .get(&pre_active_mode_id)
        .ok()
        .flatten()
        .ok_or_else(|| OrliqError::User {
            message: "active mode not found".into(),
        })?;
    let resolved_model_id = resolve_target_model_id(&pre_active_mode, &pre_settings, &state.models);
    let was_cold =
        state.inference.loaded_model_id().await.as_deref() != Some(resolved_model_id.as_str());
    if was_cold {
        tracing::info!(
            model_id = %resolved_model_id,
            mode_id = %pre_active_mode.id,
            is_meeting_mode = pre_active_mode.is_meeting_mode,
            "cold-load detected at stop_recording entry — emitting Warming, skipping initial Processing"
        );
        set_phase(
            &handle,
            DictationPhase::Warming {
                model_id: resolved_model_id.clone(),
            },
        )
        .await;
    } else {
        set_phase(&handle, DictationPhase::Processing).await;
    }

    // Optional pre-ASR conditioning. All three passes are cheap (~O(n)) so we apply them
    // inline rather than spawning a worker. Order matters: noise suppress first (we want
    // to measure the noise floor before normalisation scales it up), then silence trim
    // (noise-suppressed silence is easier to detect), then normalise at the end.
    {
        let audio_settings = &state.settings.snapshot().await.audio;
        // Diagnostic-only: when enabled, snapshot the raw post-capture
        // PCM before any preprocess pass touches it. Paired with the
        // post-preprocess dump below so a user can compare exactly
        // what was captured vs exactly what was handed to the ASR.
        // Default off; gated behind `audio.diagnostic_audio_dump`.
        let dump_diag = audio_settings.diagnostic_audio_dump;
        let raw_pcm_for_dump: Option<Vec<i16>> = if dump_diag {
            Some(final_.pcm.clone())
        } else {
            None
        };
        // Broadband DC-blocker + high-pass + adaptive floor gate. `Off` is a no-op,
        // so we can call it unconditionally and let the level variant decide.
        crate::services::audio::preprocess::suppress_noise(
            &mut final_.pcm,
            audio_settings.noise_suppression,
        );
        // Standalone inter-word gate — complements the broadband suppressor for
        // users who want keyboard clicks and lip-smacks gone between phrases.
        if audio_settings.noise_gate {
            crate::services::audio::preprocess::noise_gate(&mut final_.pcm);
        }
        if audio_settings.silence_removal {
            let trimmed = crate::services::audio::preprocess::trim_silence(
                std::mem::take(&mut final_.pcm),
                (i16::MAX as f32 * 0.003) as i16,
            );
            final_.pcm = trimmed;
            final_.duration_ms =
                (final_.pcm.len() as u64 * 1000) / final_.sample_rate.max(1) as u64;
        }
        if audio_settings.dynamic_normalisation {
            crate::services::audio::preprocess::normalise(&mut final_.pcm);
        }
        if let Some(raw) = raw_pcm_for_dump {
            // Best-effort. Diagnostic dumps must never fail a real recording.
            if let Err(e) = write_diagnostic_audio_pair(&raw, &final_.pcm, final_.sample_rate) {
                tracing::warn!(?e, "diagnostic audio dump failed");
            }
        }
    }

    // Re-snapshot settings here because audio preprocessing took a
    // measurable amount of time on long recordings, and the user may
    // have toggled a setting (privacy, text-output) in another window
    // since the cold-load probe at the top. The mode + resolved model
    // come from the earlier snapshot — those don't change mid-stop.
    let settings = state.settings.snapshot().await;
    let mode = pre_active_mode.clone();
    let model_id = resolved_model_id.clone();

    let vocab_hints = state
        .vocabulary
        .list_words()
        .unwrap_or_default()
        .into_iter()
        .filter(|w| w.enabled)
        .map(|w| w.word)
        .collect();

    // Persist the recording to disk if the user has opted in. We do this before handing
    // the PCM to the inference layer (which consumes the vec) so we're writing the
    // exact same audio that was transcribed. File naming is `<ulid>.wav` under
    // `%APPDATA%\Utterbar\recordings\` — same place the history row references.
    //
    // Pro gate: audio retention is a paid feature. Even if the user flipped
    // the stored flag via some other path, we refuse to persist on Free.
    // The Privacy settings panel also disables the toggle.
    let retention_allowed =
        settings.privacy.retain_audio && crate::util::gating::is_pro(&state.licence.status().await);
    let recorded_path = if retention_allowed {
        match write_recording_wav(&final_.pcm, final_.sample_rate) {
            Ok(path) => {
                // Best-effort pruning of old recordings. Runs inline because it's just a
                // directory scan + unlinks — cheap even with hundreds of entries.
                prune_old_recordings(settings.privacy.retain_audio_days);
                Some(path)
            }
            Err(e) => {
                tracing::warn!(?e, "failed to persist recording WAV");
                None
            }
        }
    } else {
        None
    };

    let started = now_ms();
    tracing::info!(%model_id, mode_id = %mode.id, "inference.transcribe begin");

    // Cold-load: Warming vs Processing was decided at the top of
    // stop_recording from the resolved model id. `model_id` here is
    // the same `resolved_model_id` value the early probe used (we
    // cloned it out above), so probe and transcribe trivially agree.
    // Nothing more to assert.
    //
    // All that's left on the cold path is to run `ensure_loaded`
    // explicitly before transcribe, then snap the phase to Processing
    // the instant the native warm finishes. `transcribe` does
    // ensure_loaded internally too, but that path doesn't give us a UI
    // hook between warm-start and inference-start — so we call it here.

    // Set the transcribe-in-flight guard. `start_recording` rejects
    // while this is true so a new recording can't race the current
    // transcribe's write lock on the inference orchestrator.
    // `transcribe_started_at_ms` timestamp lets the start-recording
    // stale-guard self-heal if the transcribe hangs longer than 30 s.
    state
        .transcribe_in_progress
        .store(true, std::sync::atomic::Ordering::Release);
    state
        .transcribe_started_at_ms
        .store(now_ms(), std::sync::atomic::Ordering::Release);

    if was_cold {
        if let Err(e) = state.inference.ensure_loaded(&model_id).await {
            tracing::error!(%model_id, error = %e, "cold ensure_loaded failed");
            // Fall through — `transcribe` below will try again and surface
            // the same error via its own error path. We don't abort here
            // to keep one unified failure handler.
        } else {
            // Warm finished — snap to the right phase for the final
            // inference stage. Meeting mode gets its own phase so the bar
            // reads "Transcribing meeting…" while the chunked transcribe
            // runs (otherwise a 5-minute meeting and a 5-second dictation
            // both just read "Processing", which gives the user no
            // indication of what's happening or how long it'll take).
            let phase = if mode.is_meeting_mode {
                DictationPhase::TranscribingBackground
            } else {
                DictationPhase::Processing
            };
            set_phase(&handle, phase).await;
        }
    } else if mode.is_meeting_mode {
        // Warm path: we're already past Listening into Processing
        // (emitted earlier in stop_recording). Snap to the meeting-
        // specific phase so the bar shows the "Transcribing meeting…"
        // copy instead of the generic "Processing" pill while the
        // chunked transcribe runs.
        set_phase(&handle, DictationPhase::TranscribingBackground).await;
    }
    // Meeting mode: chunk the audio into 30 s windows and transcribe
    // each, stitching the results back together. The non-meeting path
    // hands the whole buffer to transcribe() in a single call as
    // before. Meeting mode bypasses the 15-min cap inside transcribe()
    // because no single chunk hits it.
    let asr_result = if mode.is_meeting_mode {
        tracing::info!(
            samples = final_.pcm.len(),
            sample_rate = final_.sample_rate,
            mic_present = mic_final.is_some(),
            loop_present = loop_final.is_some(),
            "stop_recording: meeting mode — chunked transcribe"
        );
        // Hand the chunked path the per-side PCMs so it can derive
        // speaker tags via RMS comparison per chunk window. The mixed
        // PCM (final_.pcm) is what the recogniser sees; the per-side
        // streams are only used for the speaker-label heuristic and
        // never reach ASR.
        transcribe_chunked(
            state.inner(),
            &model_id,
            final_.pcm,
            mic_final.map(|f| f.pcm),
            loop_final.map(|f| f.pcm),
            final_.sample_rate,
            mode.target_language.clone(),
            vocab_hints,
        )
        .await
    } else {
        state
            .inference
            .transcribe(
                &model_id,
                AudioBuffer {
                    pcm: final_.pcm,
                    sample_rate: final_.sample_rate,
                    channels: 1,
                },
                AsrOptions {
                    language_hint: mode.target_language.clone(),
                    want_word_timings: false,
                    vocabulary_hints: vocab_hints,
                },
            )
            .await
    };
    // Clear the guard the instant the future settles. All exit paths
    // below (error, success) observe it cleared; any new recording that
    // comes in while the outer stop_recording is still doing text-inject
    // / history-write (post-transcribe work) is safe, because the
    // inference orchestrator's write lock is already released.
    state
        .transcribe_in_progress
        .store(false, std::sync::atomic::Ordering::Release);
    state
        .transcribe_started_at_ms
        .store(0, std::sync::atomic::Ordering::Release);
    let asr = match asr_result {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, %model_id, "inference.transcribe failed");
            // Un-do the start-time side effects so the user's music / conferencing
            // volume comes back even when ASR blew up — otherwise they're left muted
            // with the app stuck in Processing.
            {
                let media = state.media.clone();
                tokio::task::spawn_blocking(move || media.resume_paused());
            }
            {
                let ducking = state.ducking.clone();
                tokio::task::spawn_blocking(move || ducking.restore());
            }
            set_phase(
                &handle,
                DictationPhase::Error {
                    message: format!("transcribe: {e}"),
                },
            )
            .await;
            schedule_idle(&handle, 1800);
            return Err(OrliqError::Model {
                message: e.to_string(),
            });
        }
    };
    tracing::info!(chars = asr.text.len(), "inference.transcribe done");

    let raw = state
        .vocabulary
        .apply_replacements(&asr.text)
        .unwrap_or(asr.text.clone());

    // Capture context for post-processing and history. v0.6.6: globals are
    // authoritative — the mode-level `context_rules` toggles are ignored.
    // Previously these were ANDed with the privacy globals, which meant
    // toggling a mode context flag silently did nothing when the
    // corresponding global was off, and surfaced a confusing "why isn't
    // clipboard context working in this mode" question. Privacy alone
    // controls context now. The mode struct still carries `context_rules`
    // on disk for back-compat with older DB rows; the field is unread.
    let include_app = settings.privacy.allow_context_active_app;
    let include_selection = settings.privacy.allow_context_selection;
    let include_clipboard = settings.privacy.allow_context_clipboard;
    let ctx_snapshot = if include_app || include_selection || include_clipboard {
        // The context snapshot invokes Win32 + (for selection) the
        // dedicated UIA thread. The UIA path blocks on a reply channel
        // with a 200ms timeout, so we must not run this on the tokio
        // async runtime — spawn_blocking moves it to the blocking pool.
        // Also shields the async path from any panic in the Win32 FFI,
        // which would otherwise poison the whole task set.
        let ctx_svc = state.context.clone();
        let s = tokio::task::spawn_blocking(move || {
            ctx_svc.snapshot(include_selection, include_clipboard)
        })
        .await
        .unwrap_or_default();
        crate::services::llm::ProcContext {
            app_exe: if include_app { s.exe.clone() } else { None },
            app_title: if include_app { s.title.clone() } else { None },
            selection: s.selection.clone(),
            clipboard: s.clipboard.clone(),
        }
    } else {
        crate::services::llm::ProcContext::default()
    };

    // Meeting mode: skip the entire post-processing pipeline. Meeting
    // transcripts are reference material — the user wants the literal
    // record of what was said, not LLM-polished prose. We still apply
    // dialect normalisation (cheap dictionary swap, semantically a
    // spelling fix not a rewrite) but skip:
    //   - PostProcessing::Deterministic (filler removal, sentence
    //     punctuation, sentence-case)
    //   - PostProcessing::Llm (template-based rewrites)
    //   - text_output.numbers_as_digits (a stylistic transform, not
    //     useful for meeting verbatim)
    //   - text_output.auto_capitalise (the raw transcript has
    //     whatever case the ASR emitted, which is typically already
    //     sentence-cased; forcing capitalise_first_letter on a
    //     multi-paragraph meeting transcript only capitalises the very
    //     first letter and reads as a no-op)
    //   - text_output.trim_trailing_space (harmless either way)
    //
    // The user can still flip any of these on per-mode by un-flagging
    // is_meeting_mode and using a non-meeting mode; meeting mode is the
    // opinionated default for "press-record-talk-stop, get verbatim".
    let mut processed = if mode.is_meeting_mode {
        crate::services::dialect::normalise(&raw, mode.english_variant)
    } else {
        let llm = crate::services::llm::best_available().await;
        let post = match &mode.post_processing {
            crate::domain::PostProcessing::None => raw.clone(),
            crate::domain::PostProcessing::Deterministic { .. } => {
                crate::services::llm::deterministic::DeterministicProvider.rewrite_sync(&raw)
            }
            crate::domain::PostProcessing::Llm { prompt, .. } => llm
                .rewrite(&raw, prompt, &ctx_snapshot)
                .await
                .unwrap_or(raw.clone()),
        };

        // English dialect normalisation. Parakeet is a monolingual English model trained
        // mostly on North American data, so British users see "color" / "favorite" /
        // "organize" come out of the ASR even when they spoke the BrE word. Deterministic
        // dictionary swap fixes that for negligible cost. `Auto` is a passthrough — we
        // never re-spell unless the mode opts in.
        let mut p = crate::services::dialect::normalise(&post, mode.english_variant);

        // Text-output settings. These are deliberately applied last, after any LLM/mode
        // rewriting, so they're the final word on the string that hits the user's app.
        if settings.text_output.numbers_as_digits {
            p = crate::services::numbers::words_to_digits(&p);
        }
        if settings.text_output.auto_capitalise {
            p = capitalise_first_letter(&p);
        }
        if settings.text_output.trim_trailing_space {
            p = p.trim_end().to_string();
        }
        p
    };

    // Auto-space on paste: if the previous dictation in this same app didn't end with
    // whitespace and the new one starts with a letter/digit, prepend a space. Keeps
    // "hello" + "world" from colliding into "helloworld" when dictating successive
    // chunks in the same field.
    if settings.text_output.auto_space_on_paste
        && should_prepend_space(now_ms(), ctx_snapshot.app_exe.as_deref(), &processed)
    {
        processed.insert(0, ' ');
    }

    // Shift-to-auto-send: snapshot the modifier state *now*, before we do any text
    // delivery — the user was holding Shift when they released the hotkey, and our
    // synthesised keypresses below will clear it. The Enter keystroke fires after the
    // paste has landed so Teams / Slack / Discord submit the line.
    let auto_send = settings.text_output.shift_to_auto_send && state.text_inject.shift_is_held();

    // No-audio-detected gate. Two contributors put empty strings on this
    // path:
    //   • sherpa_parakeet's hallucination filter (see sherpa_parakeet.rs)
    //     returns "" on a quiet clip that Parakeet-TDT filled with a
    //     fabricated filler token.
    //   • Genuine silence clips where the model simply had nothing to
    //     produce.
    // Either way we have no usable speech. Previously we fell through to
    // text_inject with an empty `processed`, which paste-injected nothing
    // but still emitted Success + wrote a history row + played the success
    // cue. That reads to the user as "it worked, but nothing came out" —
    // confusing and wrong. Replace that with a NoAudio phase, skip
    // delivery + history + analytics, but still resume paused media /
    // ducking so we don't leave the user silent.
    let is_silence = processed.trim().is_empty();
    if is_silence {
        tracing::info!(
            asr_chars = asr.text.len(),
            raw_chars = raw.len(),
            processed_chars = processed.len(),
            "no-audio: skipping inject/history, emitting NoAudio phase"
        );
        {
            let media = state.media.clone();
            tokio::task::spawn_blocking(move || media.resume_paused());
        }
        {
            let ducking = state.ducking.clone();
            tokio::task::spawn_blocking(move || ducking.restore());
        }
        set_phase(&handle, DictationPhase::NoAudio).await;
        // Longer settle than Success (1.2s) — the message is the whole
        // point so we want the user to actually read it before the bar
        // decays back to idle.
        schedule_idle(&handle, 1800);
        // Return a synthetic history-shaped Ok so callers that inspect
        // the result don't error. The entry is NOT persisted; it's a
        // marker value only.
        return Ok(HistoryEntry {
            id: String::new(),
            created_at: now_ms(),
            mode_id: Some(mode.id.clone()),
            mode_name_snapshot: mode.name.clone(),
            asr_model_id: model_id,
            asr_model_version: asr.model_version,
            audio_path: recorded_path,
            audio_duration_ms: final_.duration_ms as i64,
            raw_text: String::new(),
            processed_text: String::new(),
            language: asr.language,
            processing_ms: now_ms() - started,
            status: HistoryStatus::Success,
            error_message: None,
            app_exe: None,
            app_title: None,
            context_payload: None,
            word_timings: None,
            is_meeting: mode.is_meeting_mode,
        });
    }

    // Meeting mode: skip text injection entirely. A meeting transcript is
    // reference material — saving 5+ minutes of dialogue into whatever
    // window happened to have focus is the wrong default. The history row
    // (and, in v0.14, the Meetings page) is the destination. We also skip
    // the `record_delivery` bookkeeping because there's no delivery to
    // remember — the auto-space-on-paste heuristic only applies to
    // dictation continuations.
    if !mode.is_meeting_mode {
        let strategy = if settings.text_output.paste_automatically {
            crate::services::text_inject::Strategy::Paste {
                restore_clipboard: settings.text_output.restore_clipboard,
                fallback_to_keystrokes: settings.text_output.keystroke_fallback,
            }
        } else {
            crate::services::text_inject::Strategy::Keystrokes
        };
        let _ = state.text_inject.deliver(&processed, strategy);

        if auto_send {
            // Small pause so the host has time to accept the paste before we fire Enter —
            // some apps (Teams, specifically) briefly freeze on receiving a paste and would
            // drop the Enter if we fired it synchronously.
            tokio::time::sleep(std::time::Duration::from_millis(80)).await;
            if let Err(e) = state.text_inject.send_enter() {
                tracing::warn!(?e, "auto-send Enter failed");
            }
        }

        // Remember this delivery so the next dictation can decide whether to prepend a space.
        record_delivery(now_ms(), ctx_snapshot.app_exe.as_deref(), &processed);
    } else {
        tracing::info!(
            chars = processed.len(),
            "meeting mode: skipping text injection — transcript saved to history only"
        );
    }

    let context_payload = serde_json::to_string(&ctx_snapshot).ok();
    let entry = HistoryEntry {
        id: ulid::Ulid::new().to_string(),
        created_at: now_ms(),
        mode_id: Some(mode.id.clone()),
        mode_name_snapshot: mode.name.clone(),
        asr_model_id: model_id,
        asr_model_version: asr.model_version,
        audio_path: recorded_path,
        audio_duration_ms: final_.duration_ms as i64,
        raw_text: raw,
        processed_text: processed.clone(),
        language: asr.language,
        processing_ms: now_ms() - started,
        status: HistoryStatus::Success,
        error_message: None,
        app_exe: ctx_snapshot.app_exe.clone(),
        app_title: ctx_snapshot.app_title.clone(),
        context_payload,
        word_timings: None,
        is_meeting: mode.is_meeting_mode,
    };
    state.history.insert(&entry).map_err(OrliqError::from)?;

    let words = processed.split_whitespace().count() as u32;
    let _ = state.analytics.record_dictation(
        words,
        (final_.duration_ms / 1000) as u32,
        settings.typing_baseline_wpm,
        Some(&mode.id),
    );

    // Resume any media we paused when the recording started.
    // (see: privacy.retain_audio — if on, the WAV we wrote above is kept; otherwise
    // the recordings dir is left untouched.)
    {
        let media = state.media.clone();
        tokio::task::spawn_blocking(move || media.resume_paused());
    }
    // Un-mute any conferencing apps we ducked on start. Runs unconditionally —
    // `DuckingService::restore` is a no-op if we never muted anything.
    {
        let ducking = state.ducking.clone();
        tokio::task::spawn_blocking(move || ducking.restore());
    }

    let _ = handle.emit(EVT_DICTATION_RESULT, &entry);
    // Meeting recordings get a substantive success pill — duration,
    // chunk count, char count — for ~3 s before decay so the user can
    // see "5:01 · 11 chunks · 799 chars" instead of a generic
    // "Inserted" cue. Non-meeting dictations keep the current sparse
    // payload (None) so the floating bar's existing Success treatment
    // is unchanged.
    let success_summary = if mode.is_meeting_mode {
        Some(crate::domain::DictationSuccessSummary {
            duration_seconds: (asr.duration_ms / 1000),
            // The chunked path doesn't expose its chunk count to the
            // outer scope, so compute it here from the audio length —
            // matches the decision the chunker made (CHUNK 30s, OVERLAP 2s,
            // stride 28s). For a single-chunk recording this gives 1.
            chunks: chunk_count_for_duration(asr.duration_ms),
            chars: processed.len() as u32,
        })
    } else {
        None
    };
    set_phase(
        &handle,
        DictationPhase::Success {
            summary: success_summary,
        },
    )
    .await;
    // Meeting mode lingers a beat longer (3 s instead of the dictation
    // 1.2 s) so the user has time to read the duration + chunk pill
    // before it decays. The pill is the only visible "your meeting was
    // saved" signal until the v0.14 Meetings page lands.
    let idle_delay = if mode.is_meeting_mode { 3000 } else { 1200 };
    schedule_idle(&handle, idle_delay);

    Ok(entry)
}

/// Mirror of the chunking arithmetic in `transcribe_chunked` — used by
/// stop_recording's success-pill payload to report how many chunks the
/// meeting was split into without threading the count back from the
/// chunker. Hard-coded to the same CHUNK_SECONDS=30 / OVERLAP_SECONDS=2
/// pair the chunker uses; if those are ever moved to settings, both
/// sites read from the same source.
fn chunk_count_for_duration(duration_ms: u32) -> u32 {
    const CHUNK_S: u32 = 30;
    const OVERLAP_S: u32 = 2;
    const STRIDE_S: u32 = CHUNK_S - OVERLAP_S;
    let secs = duration_ms.div_ceil(1000);
    if secs == 0 {
        return 0;
    }
    if secs <= CHUNK_S {
        return 1;
    }
    // Chunk N starts at N * stride; covers the whole recording when
    // (N-1) * stride + chunk >= total. Solve: N = ceil((total - chunk) / stride) + 1.
    ((secs - CHUNK_S).div_ceil(STRIDE_S)) + 1
}

#[tauri::command]
pub async fn cancel_recording(handle: AppHandle, state: State<'_, Arc<AppState>>) -> CmdResult<()> {
    let _ = state.audio.stop_capture().await;
    // Always release the re-entry guard — cancel is the user's "get me
    // out of whatever state I'm in" command, and if we stranded the
    // flag high they'd lose the ability to start the next recording
    // without restarting the app.
    state
        .recording_in_progress
        .store(false, std::sync::atomic::Ordering::Release);
    // Resume whatever was paused on start, even on cancel — the user expects their
    // music to pick back up as soon as they hit the cancel hotkey.
    {
        let media = state.media.clone();
        tokio::task::spawn_blocking(move || media.resume_paused());
    }
    {
        let ducking = state.ducking.clone();
        tokio::task::spawn_blocking(move || ducking.restore());
    }
    set_phase(&handle, DictationPhase::Cancelled).await;
    schedule_idle(&handle, 600);
    Ok(())
}

#[tauri::command]
pub async fn get_phase() -> CmdResult<DictationPhase> {
    Ok(phase_cell().read().await.clone())
}

async fn set_phase(handle: &AppHandle, phase: DictationPhase) {
    // Log every phase transition. The frontend's zustand dictation store
    // is driven entirely by the `dictation:phase` event emitted here; when
    // the store falls out of sync with Rust (observed 2026-04-24T17:59 —
    // see stuck-hotkey investigation), the log is the only window into
    // what Rust THOUGHT the phase was vs. what the UI actually rendered.
    // Using a short discriminant rather than the full JSON so the signal
    // stays high in a busy log.
    let kind = match &phase {
        DictationPhase::Idle => "idle",
        DictationPhase::Warming { .. } => "warming",
        DictationPhase::Listening { .. } => "listening",
        DictationPhase::Processing => "processing",
        DictationPhase::TranscribingBackground => "transcribing_background",
        DictationPhase::Success { .. } => "success",
        DictationPhase::Error { .. } => "error",
        DictationPhase::Cancelled => "cancelled",
        DictationPhase::NoAudio => "no_audio",
    };
    tracing::info!(phase = %kind, "set_phase");
    *phase_cell().write().await = phase.clone();
    let emit_res = handle.emit(EVT_DICTATION_PHASE, &phase);
    if let Err(e) = &emit_res {
        // `emit` failures are silent today — they swallow the error and
        // the frontend never learns about the phase change. If that ever
        // starts happening (e.g. a webview was destroyed mid-dictation),
        // the log tells us so we don't spend another session wondering
        // why the UI disagrees with Rust.
        tracing::error!(error = ?e, phase = %kind, "phase emit failed");
    }
    if let DictationPhase::Error { message } = &phase {
        let _ = handle.emit(EVT_DICTATION_ERROR, message.clone());
    }
}

/// Fire the `dictation:busy` event. The FloatingBar listens for this
/// and runs a one-second accent border flash — the user pressed the
/// hotkey while the app was busy (either mid-recording or mid-
/// transcribe), and a silent no-op would be confusing. A flash plus
/// the rejection error (swallowed by the frontend) together read as
/// "noted, try again in a moment" without any modal interruption.
fn emit_busy(handle: &AppHandle) {
    use crate::domain::EVT_DICTATION_BUSY;
    let _ = handle.emit(EVT_DICTATION_BUSY, ());
}

/// Capitalise the first alphabetic character of `s`, leaving everything else alone.
/// Intentionally narrow — we don't sentence-split here because the LLM / deterministic
/// post-processor already handles mid-string capitalisation when the mode asks for it.
fn capitalise_first_letter(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut done = false;
    for ch in s.chars() {
        if !done && ch.is_alphabetic() {
            for c in ch.to_uppercase() {
                out.push(c);
            }
            done = true;
        } else {
            out.push(ch);
        }
    }
    out
}

// Sync wrapper around the deterministic provider for call sites that don't want to await.
impl crate::services::llm::deterministic::DeterministicProvider {
    pub fn rewrite_sync(&self, input: &str) -> String {
        let normalised = crate::services::llm::deterministic::normalise_whitespace(input);
        let stripped = crate::services::llm::deterministic::strip_fillers(&normalised);
        let punctuated =
            crate::services::llm::deterministic::ensure_terminal_punctuation(&stripped);
        crate::services::llm::deterministic::capitalise_sentences(&punctuated)
    }
}

/// Write mono 16-bit PCM to a WAV file under the recordings directory. Filename is a
/// fresh ULID so concurrent dictations never clash. Returns the absolute path, which is
/// what gets stored on the HistoryEntry so the History UI can play it back later.
fn write_recording_wav(pcm: &[i16], sample_rate: u32) -> anyhow::Result<String> {
    let dir = crate::util::paths::recordings_dir()?;
    let filename = format!("{}.wav", ulid::Ulid::new());
    let path = dir.join(&filename);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec)?;
    for &s in pcm {
        writer.write_sample(s)?;
    }
    writer.finalize()?;
    Ok(path.to_string_lossy().to_string())
}

/// Diagnostic dump (gated by `audio.diagnostic_audio_dump`): write the
/// raw post-capture PCM and the exact post-preprocess PCM that the ASR
/// will see, side-by-side under `%APPDATA%\Utterbar\diagnostics\audio\`.
/// Self-rotating: keeps at most the latest 6 pairs (12 files) so a
/// forgotten-on flag doesn't fill the drive.
fn write_diagnostic_audio_pair(
    raw: &[i16],
    processed: &[i16],
    sample_rate: u32,
) -> anyhow::Result<()> {
    let dir = crate::util::paths::diagnostics_audio_dir()?;
    let stamp = ulid::Ulid::new();
    let raw_path = dir.join(format!("{stamp}-raw.wav"));
    let proc_path = dir.join(format!("{stamp}-processed.wav"));

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    {
        let mut w = hound::WavWriter::create(&raw_path, spec)?;
        for &s in raw {
            w.write_sample(s)?;
        }
        w.finalize()?;
    }
    {
        let mut w = hound::WavWriter::create(&proc_path, spec)?;
        for &s in processed {
            w.write_sample(s)?;
        }
        w.finalize()?;
    }

    // Retention: keep the 6 most recent pairs only (12 files). Cheap
    // single-pass scan, runs every dump so an always-on user never
    // accumulates more than ~12 short recordings worth of audio.
    if let Ok(entries) = std::fs::read_dir(&dir) {
        let mut wavs: Vec<(std::path::PathBuf, std::time::SystemTime)> = entries
            .flatten()
            .filter(|e| e.path().extension().map(|x| x == "wav").unwrap_or(false))
            .filter_map(|e| {
                e.metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(|t| (e.path(), t))
            })
            .collect();
        wavs.sort_by_key(|b| std::cmp::Reverse(b.1));
        for (path, _) in wavs.into_iter().skip(12) {
            let _ = std::fs::remove_file(path);
        }
    }
    Ok(())
}

/// Best-effort cleanup of old recordings. Runs inline at stop time; we don't care about
/// errors since the user's active dictation should never be held up by housekeeping.
fn prune_old_recordings(retain_days: u32) {
    if retain_days == 0 {
        return; // 0 would mean "delete everything immediately", which almost certainly
                // isn't what the user wants — treat as "don't prune".
    }
    let Ok(dir) = crate::util::paths::recordings_dir() else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(retain_days as u64 * 86_400));
    let Some(cutoff) = cutoff else { return };
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let modified = meta.modified().ok();
        if modified.is_some_and(|m| m < cutoff) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}
