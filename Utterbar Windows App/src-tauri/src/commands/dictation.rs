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
use crate::services::inference::{AsrOptions, AudioBuffer};
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
    tracing::info!(?device_id, "dictation::start_recording enter");

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
            tracing::warn!("start_recording rejected: another start is already in progress");
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

    let settings = state.settings.snapshot().await;

    use crate::domain::settings::CaptureSource;
    let source = settings.audio.capture_source;

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
    let mic_capture = if matches!(source, CaptureSource::Microphone | CaptureSource::Mixed) {
        match state.audio.start_capture(device_id).await {
            Ok(handle) => Some(handle),
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
    tracing::info!(?source, "audio capture armed");
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

/// Schedule a return-to-idle phase transition after `delay_ms`. Used by the error and
/// cancel paths so the UI briefly shows the transient state before falling back to Idle.
fn schedule_idle(handle: &AppHandle, delay_ms: u64) {
    let handle = handle.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        set_phase(&handle, DictationPhase::Idle).await;
    });
}

#[tauri::command]
pub async fn stop_recording(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> CmdResult<HistoryEntry> {
    tracing::info!("dictation::stop_recording enter");
    let final_ = state.audio.stop_capture().await;
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
    set_phase(&handle, DictationPhase::Processing).await;

    // Optional pre-ASR conditioning. All three passes are cheap (~O(n)) so we apply them
    // inline rather than spawning a worker. Order matters: noise suppress first (we want
    // to measure the noise floor before normalisation scales it up), then silence trim
    // (noise-suppressed silence is easier to detect), then normalise at the end.
    {
        let audio_settings = &state.settings.snapshot().await.audio;
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
    }

    let settings = state.settings.snapshot().await;
    let active_mode_id = settings
        .active_mode_id
        .clone()
        .unwrap_or_else(|| "voice-to-text".into());
    let mode = state
        .modes
        .get(&active_mode_id)
        .ok()
        .flatten()
        .ok_or_else(|| OrliqError::User {
            message: "active mode not found".into(),
        })?;

    let model_id = mode
        .asr_model_id
        .clone()
        .or(settings.models.default_asr_model_id.clone())
        .unwrap_or_else(|| "stub".into());

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
    let asr = match state
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
    {
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

    // Capture context for post-processing and history — gated by both the mode's rules
    // and the user's global privacy toggles.
    let include_app =
        mode.context_rules.use_active_app && settings.privacy.allow_context_active_app;
    let include_selection =
        mode.context_rules.use_selection && settings.privacy.allow_context_selection;
    let include_clipboard =
        mode.context_rules.use_clipboard && settings.privacy.allow_context_clipboard;
    let ctx_snapshot = if include_app || include_selection || include_clipboard {
        let s = state.context.snapshot(include_selection, include_clipboard);
        crate::services::llm::ProcContext {
            app_exe: if include_app { s.exe.clone() } else { None },
            app_title: if include_app { s.title.clone() } else { None },
            selection: s.selection.clone(),
            clipboard: s.clipboard.clone(),
        }
    } else {
        crate::services::llm::ProcContext::default()
    };

    let llm = crate::services::llm::best_available().await;
    let mut processed = match &mode.post_processing {
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
    processed = crate::services::dialect::normalise(&processed, mode.english_variant);

    // Text-output settings. These are deliberately applied last, after any LLM/mode
    // rewriting, so they're the final word on the string that hits the user's app.
    if settings.text_output.auto_capitalise {
        processed = capitalise_first_letter(&processed);
    }
    if settings.text_output.trim_trailing_space {
        processed = processed.trim_end().to_string();
    }

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

    // Attempt text delivery.
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
    set_phase(&handle, DictationPhase::Success).await;
    // Return to idle after a brief pause (client-driven; we just emit the current phase).
    schedule_idle(&handle, 1200);

    Ok(entry)
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
    *phase_cell().write().await = phase.clone();
    let _ = handle.emit(EVT_DICTATION_PHASE, &phase);
    if let DictationPhase::Error { message } = &phase {
        let _ = handle.emit(EVT_DICTATION_ERROR, message.clone());
    }
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
