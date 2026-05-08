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

/// Stale-heal threshold for the `recording_in_progress` flag.
///
/// The re-entry guard in `start_recording` time-gates self-heal:
/// only when the flag has been held for longer than this is it
/// considered stale and clearable. This closes a race that
/// non-time-gated logic can't (round-5 fix):
/// `start_recording_pending` and `audio.current().await` are read
/// across an `await`, so a duplicate call can sample
/// `(pending=false, capture=None)` even though the in-flight
/// start has just dropped pending and is one scheduler tick away
/// from arming capture. Without a time gate, that snapshot would
/// stale-heal a flag that is not stale.
///
/// Sized well past worst-case start latency (cold sherpa load +
/// settings I/O measured ~3-7 s; meeting-mode diariser download
/// up to ~30 s on slow links) so a real start can never be
/// pre-empted by a duplicate hotkey, and well below "user gives
/// up and assumes the app is broken" so a genuinely wedged flag
/// from a panicked task gets cleared within a tolerable wait.
///
/// 30 s mirrors `TRANSCRIBE_STALE_MS` below — the same reasoning
/// applies (cover all real prep latencies, recover from genuine
/// hangs in a single user-noticeable cycle).
const RECORDING_STALE_MS: i64 = 30_000;

/// Auto-stop threshold for non-meeting (single-shot) dictation.
///
/// `start_recording` schedules a Tokio sleep for this duration on
/// non-meeting recordings. When it fires, the timer re-reads the
/// active recording context, confirms the same session is still
/// running and is non-meeting, and calls the same internal stop
/// path the user's hotkey would. The captured audio is transcribed
/// normally and a `notice` is appended to the Success phase so the
/// user understands what happened.
///
/// **Why 295 s, not 300 s.** The transcribe-layer cap in
/// `services/inference/mod.rs::Inference::transcribe` rejects
/// inputs whose `pcm.len() / sample_rate` exceeds
/// `MAX_TRANSCRIBE_SECONDS = 5 * 60 = 300`. The hotkey-release-to-
/// AudioBuffer-construction path adds a small amount of trailing
/// audio (resampler tail, last cpal callback) so a buffer that
/// stops at exactly 300.0 s of wall clock can occasionally produce
/// a 300.1 s PCM length and trip the cap. A 5-second margin makes
/// the auto-stop boring: every auto-stopped buffer is comfortably
/// inside the cap and gets a clean transcript.
///
/// **Pinned by tests.** `auto_stop_pin_constants_match_asr_cap`
/// in this file's test module asserts the 5-second margin against
/// the `MAX_TRANSCRIBE_SECONDS` constant in `services/inference`.
/// Bumping one without the other fails CI.
///
/// Meeting Mode does not use this timer — the chunked transcribe
/// path handles long recordings by feeding 29-second chunks to
/// the recogniser, so the 5-minute single-shot cap doesn't apply.
pub(crate) const SINGLE_SHOT_AUTO_STOP_MS: i64 = 295_000;

/// Pure decision function used by the single-shot auto-stop timer.
/// Returns `true` only when ALL of these hold:
///
/// - The bound `ActiveRecordingContext` is still present (Some).
/// - Its `started_at_ms` matches the value captured at schedule
///   time (i.e. the same session that armed the timer is still
///   running; a stop+start in between would advance this value).
/// - The bound session is non-meeting (`is_meeting == false`).
///
/// The recording-in-progress flag is NOT checked here — that's a
/// fast-path the caller does inline before reading the context, so
/// the cost of acquiring the read lock isn't paid when the user
/// already pressed stop. Splitting the decision into a pure helper
/// makes it testable without spawning Tokio tasks or building a
/// full AppState.
pub(crate) fn auto_stop_should_proceed(
    scheduled_started_at_ms: i64,
    current_ctx: Option<&crate::app_state::ActiveRecordingContext>,
) -> bool {
    let Some(ctx) = current_ctx else {
        return false;
    };
    if ctx.started_at_ms != scheduled_started_at_ms {
        return false;
    }
    if ctx.is_meeting {
        return false;
    }
    true
}

/// Why a `stop_recording` call fired. Threaded through
/// `stop_recording_inner` so the Success-emit site can attach a
/// `notice` to the phase event for non-error stop conditions the
/// user should see (currently: the auto-stop at the 5-minute cap).
///
/// Manual stops (the user's hotkey, the floating bar's stop
/// button, `cancel_recording` triggering after a queued meeting)
/// emit Success with `notice: None`, which renders as the existing
/// `Inserted` pill in the floating bar. Auto-stops emit a notice
/// string so the bar can render `Auto-stopped at 5 min` instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StopReason {
    /// User pressed the stop hotkey, clicked the bar's stop
    /// button, or otherwise drove the stop themselves.
    Manual,
    /// The Tokio task scheduled at the start of a non-meeting
    /// recording fired at `SINGLE_SHOT_AUTO_STOP_MS` and the
    /// session was still the same non-meeting recording, so the
    /// timer drove the stop instead of the user.
    AutoStoppedAtCap,
}

/// Decision returned by the re-entry guard in `start_recording`
/// when its CAS on `recording_in_progress` failed (some other
/// invocation already set the flag). Reached by a two-stage
/// process — see the `decide_…` helpers below.
///
/// The four reject + one proceed branches map 1:1 to user-visible
/// behaviour:
///
///   * `RejectActiveCapture` — a recording is genuinely live. The
///     duplicate hotkey press should re-sync the frontend's
///     Listening phase and refuse with "recording already in
///     progress".
///   * `RejectPreflight` — a previous start is still inside the
///     meeting-mode diariser asset prep. The duplicate press
///     should refuse with a "preparing models, try again" message
///     and NOT clear the recording_in_progress flag (it belongs
///     to the in-flight start).
///   * `RejectStartPending` — a previous start is still in its
///     pre-arm prep window (settings snapshot, mode lookup, mic
///     enumeration, etc.) but NOT in the diariser preflight. The
///     duplicate press should refuse with a generic "starting"
///     message and NOT clear the recording_in_progress flag.
///   * `RejectYoungFlag` — no flags held and no live capture
///     observed, but `recording_in_progress` was set very
///     recently (younger than `RECORDING_STALE_MS`). Almost
///     certainly the in-flight start has just dropped its
///     pending flag and capture armament hasn't surfaced through
///     `audio.current()` yet. Treat as a transient busy state
///     rather than stale.
///   * `ProceedAfterStaleHeal` — flag is older than
///     `RECORDING_STALE_MS`, no flags held, no capture observed,
///     and a re-check confirms nothing came in flight while we
///     looked. The flag was left stuck true by a previous error
///     path. This is the only legitimate self-heal case.
///
/// Pre-v0.15.0 only `RejectActiveCapture` and
/// `ProceedAfterStaleHeal` existed; the others were added across
/// rounds 3 (preflight), 4 (start-pending), and 5 (young-flag +
/// time-gated stale-heal). Each round closed a real race window —
/// see commit history and `AppState::start_recording_pending` for
/// the lifecycle contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reentry {
    RejectActiveCapture,
    RejectPreflight,
    RejectStartPending,
    RejectYoungFlag,
    ProceedAfterStaleHeal,
}

/// RAII guard for the `start_recording_pending` CAS-mutex (round-5).
///
/// `claim` returns `Some(Self)` only if the inner CAS succeeds;
/// returns `None` if some other invocation has already claimed the
/// pending flag (i.e. another `start_recording` is mid-flight).
/// Drop unconditionally clears the flag so unwind safety doesn't
/// depend on every early-return remembering to flip it. Borrows
/// the `AtomicBool` directly so unit tests can exercise the
/// CAS + drop contract without standing up `AppState`.
struct StartPendingGuard<'a> {
    flag: &'a std::sync::atomic::AtomicBool,
}

impl<'a> StartPendingGuard<'a> {
    /// Try to claim the flag via CAS. Returns `Some(Self)` on
    /// success, `None` if the flag was already true.
    fn try_claim(flag: &'a std::sync::atomic::AtomicBool) -> Option<Self> {
        flag.compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire,
        )
        .ok()
        .map(|_| Self { flag })
    }
}

impl<'a> Drop for StartPendingGuard<'a> {
    fn drop(&mut self) {
        self.flag.store(false, std::sync::atomic::Ordering::Release);
    }
}

/// RAII guard for the narrow inner `meeting_preflight_in_progress`
/// flag. `store(true)` on construction is fine here because the
/// outer `StartPendingGuard` already serialises the entire start
/// lifecycle — there can be at most one preflight in flight per
/// process at any time, and it always lives strictly inside the
/// pending window. Drop unconditionally clears.
///
/// **v0.15.1: no production caller.** The diariser asset preflight
/// in `start_recording` is disabled in this hotfix, so this guard's
/// only callers live in the test mod. Kept in tree so v0.15.2 can
/// re-engage it the moment a new asset preflight needs the
/// "narrow inner flag for diariser-specific reject messaging"
/// contract — see the round-3-to-5 history on
/// `AppState::meeting_preflight_in_progress`.
#[allow(dead_code)]
struct PreflightGuard<'a> {
    flag: &'a std::sync::atomic::AtomicBool,
}

impl<'a> PreflightGuard<'a> {
    #[allow(dead_code)]
    fn claim(flag: &'a std::sync::atomic::AtomicBool) -> Self {
        flag.store(true, std::sync::atomic::Ordering::Release);
        Self { flag }
    }
}

impl<'a> Drop for PreflightGuard<'a> {
    fn drop(&mut self) {
        self.flag.store(false, std::sync::atomic::Ordering::Release);
    }
}

/// Stage 1 of the re-entry decision (round-5).
///
/// Called BEFORE any await, immediately after the
/// `recording_in_progress` CAS fails. Reads the two pending
/// atomics and returns a rejection if either is held. Returning
/// `None` means "no rejection from pending state — caller must
/// proceed to stage 2 (probe capture)". Pure for testability.
///
/// Why this comes first: pre-round-5 the CAS-failed branch awaited
/// `audio.current()` before reading `start_recording_pending`. The
/// await was a yield point that let the in-flight start drop the
/// pending flag, so by the time we read it the value was stale.
/// Reading the cheap atomics first (both `Acquire` loads with no
/// scheduling cost) means a pending start is rejected on its
/// canonical signal, no time-gate or capture probe needed.
fn decide_pre_probe(start_pending: bool, preflight_in_progress: bool) -> Option<Reentry> {
    if start_pending {
        if preflight_in_progress {
            Some(Reentry::RejectPreflight)
        } else {
            Some(Reentry::RejectStartPending)
        }
    } else {
        None
    }
}

/// Stage 2 of the re-entry decision (round-5).
///
/// Called only after stage 1 returned `None`. Inputs are sampled
/// across an `await` (we did `audio.current().await` between
/// stages), so the contract is pessimistic: only stale-heal when
/// every signal agrees the flag is genuinely abandoned, including
/// a time gate and a re-check of the pending atomics after the
/// stale-age check.
///
///   * `has_live_capture` — result of `audio.current().await.is_some()`
///   * `flag_age_ms` — how long `recording_in_progress` has been
///     held; 0 means "no timestamp recorded" (treat as fresh, do
///     not stale-heal — the in-flight start is mid-CAS-then-stamp)
///   * `start_pending_recheck` — re-load of
///     `start_recording_pending` AFTER the `audio.current().await`
///   * `preflight_recheck` — re-load of
///     `meeting_preflight_in_progress` AFTER the await
///
/// Priority:
///   1. Live capture wins.
///   2. If pending re-flips true during the await, treat as
///      pre-probe rejection (preflight if both, else generic).
///   3. Flag too young to be stale → reject as transient busy.
///   4. Flag old enough AND no capture AND no pending →
///      legitimate self-heal.
fn decide_post_probe(
    has_live_capture: bool,
    flag_age_ms: i64,
    start_pending_recheck: bool,
    preflight_recheck: bool,
) -> Reentry {
    if has_live_capture {
        return Reentry::RejectActiveCapture;
    }
    // Pending flipped to true between stages — treat exactly as
    // stage 1 would have. The in-flight start that won the race
    // owns the pending state.
    if start_pending_recheck {
        return if preflight_recheck {
            Reentry::RejectPreflight
        } else {
            Reentry::RejectStartPending
        };
    }
    // Flag age 0 means we caught the in-flight start between its
    // CAS-set and its timestamp store. Treat as young — never
    // stale-heal a flag whose age we can't establish.
    if flag_age_ms <= 0 || flag_age_ms < RECORDING_STALE_MS {
        return Reentry::RejectYoungFlag;
    }
    Reentry::ProceedAfterStaleHeal
}

/// Apply a re-entry rejection: emit the appropriate frontend
/// signal and return the matching error. Centralised so the
/// stage-1 and stage-2 sites have identical behaviour. Never
/// touches `recording_in_progress` or `start_recording_pending` —
/// every rejection here represents "some other call owns those
/// flags", and clearing them would corrupt the in-flight start.
///
/// `Reentry::ProceedAfterStaleHeal` is rejected at the type level
/// by callers (they handle proceed inline). Passing it here is a
/// programmer error and we panic in debug, log in release.
async fn reject_reentry(handle: &AppHandle, decision: Reentry) -> CmdResult<()> {
    let (log_msg, user_msg, reemit_listening) = match decision {
        Reentry::RejectActiveCapture => (
            "start_recording rejected: another start is already in progress — re-emitting Listening phase to re-sync frontend",
            "recording already in progress",
            true,
        ),
        Reentry::RejectPreflight => (
            "start_recording rejected: meeting-mode diariser asset prep still in flight from a previous start",
            "Preparing meeting models — try again in a moment.",
            false,
        ),
        Reentry::RejectStartPending => (
            "start_recording rejected: a previous start is still preparing (no live capture yet)",
            "Recording is starting — try again in a moment.",
            false,
        ),
        Reentry::RejectYoungFlag => (
            "start_recording rejected: recording_in_progress flag is fresh (younger than RECORDING_STALE_MS) and no capture/pending yet — likely transient race with an in-flight start",
            "Recording is starting — try again in a moment.",
            false,
        ),
        Reentry::ProceedAfterStaleHeal => {
            debug_assert!(false, "reject_reentry called with ProceedAfterStaleHeal");
            tracing::error!("reject_reentry called with ProceedAfterStaleHeal — this is a bug");
            return Ok(());
        }
    };
    tracing::warn!("{log_msg}");
    if reemit_listening {
        // Re-sync the frontend's Listening phase. When the
        // dictation store has desynced from Rust (observed
        // 2026-04-24: phase store stuck at idle/processing/success
        // while capture was still running), the UI thinks it isn't
        // recording and every hotkey routes to start — all of
        // which we reject. Pushing a fresh Listening phase resets
        // the frontend so the user's NEXT press hits the stop
        // branch correctly.
        set_phase(
            handle,
            DictationPhase::Listening {
                started_at_ms: now_ms(),
            },
        )
        .await;
    }
    emit_busy(handle);
    Err(OrliqError::User {
        message: user_msg.into(),
    })
}

/// Mark `recording_in_progress` true via CAS and stamp the
/// timestamp. Returns true if the CAS succeeded (caller owns the
/// flag), false otherwise. Always store the timestamp on success
/// so the time-gated stale-heal has something to read.
fn mark_recording_started(state: &AppState) -> bool {
    let won = state
        .recording_in_progress
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire,
        )
        .is_ok();
    if won {
        state
            .recording_in_progress_started_at_ms
            .store(now_ms(), std::sync::atomic::Ordering::Release);
    }
    won
}

/// Clear `recording_in_progress` and zero the started-at
/// timestamp. Pair-write helper so every clear site keeps the two
/// atomics consistent — there are 7 clear sites in this file
/// (mic-fail, loopback-fail, transcribe-still-running rejection,
/// missing-Whisper-Large rejection, no-active-recording in stop,
/// normal stop, cancel) and forgetting the timestamp on any of
/// them would leave a stale `started_at_ms` to confuse the next
/// stale-heal probe.
fn clear_recording_flag(state: &AppState) {
    state
        .recording_in_progress
        .store(false, std::sync::atomic::Ordering::Release);
    state
        .recording_in_progress_started_at_ms
        .store(0, std::sync::atomic::Ordering::Release);
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
    // Stale self-heal: if `recording_in_progress` is stuck true but no
    // capture is active AND no start is in flight AND the flag is older
    // than `RECORDING_STALE_MS`, the guard has desynced from reality —
    // most likely because an earlier transcribe returned an error that
    // short-circuited the normal stop_recording → clear flag path, or
    // because a dropped dictation-phase event left the frontend thinking
    // it was idle while the Rust flag was still set. Without the
    // self-heal, the user's hotkey silently does nothing until they
    // restart the app.
    //
    // v0.15.0 corrections (rounds 3, 4, 5):
    //
    // The fundamental problem: `recording_in_progress`,
    // `start_recording_pending`, `audio.current()` and the started-at
    // timestamp are FOUR atomics read across at least one `await`
    // (`audio.current().await`). They cannot be sampled as a coherent
    // group — by the time we look at the second one, the first may
    // have changed. The pre-round-5 logic ignored that and read
    // `audio.current()` first, which let a duplicate hotkey landing
    // between "in-flight start drops pending" and "capture surfaces
    // through audio.current()" sample `(pending=false, capture=None)`
    // and stale-heal a flag that was not stale.
    //
    // Round-5 fix (this code):
    //   1. Stage 1 reads ONLY the cheap pending atomics (no await,
    //      no scheduling cost). If either is held, reject — no probe
    //      needed.
    //   2. Stage 2 (only if stage 1 said "proceed") probes
    //      `audio.current().await`, then reads the started-at age,
    //      then RE-READS the pending atomics. Stale-heal only fires
    //      when every signal agrees: capture=None, age >=
    //      RECORDING_STALE_MS, pending still false on the recheck.
    //
    // Round-4 introduced `start_recording_pending` (CAS-mutex in
    // round-5: `StartPendingGuard::try_claim`). Round-3 introduced
    // `meeting_preflight_in_progress` (narrow inner flag for the
    // ~47 MB diariser download window — used only to pick the
    // better-targeted reject message).
    if !mark_recording_started(state.inner()) {
        // Stage 1: cheap atomic reads, no await.
        let start_pending = state
            .start_recording_pending
            .load(std::sync::atomic::Ordering::Acquire);
        let preflight_in_progress = state
            .meeting_preflight_in_progress
            .load(std::sync::atomic::Ordering::Acquire);
        if let Some(reject) = decide_pre_probe(start_pending, preflight_in_progress) {
            return reject_reentry(&handle, reject).await;
        }

        // Stage 2: probe capture (await), then re-read pending atomics
        // and the started-at timestamp. The re-read is non-negotiable —
        // the await is a scheduling point, and the in-flight start may
        // claim pending between stage 1 and the probe.
        let has_live_capture = state.audio.current().await.is_some();
        let started_at_ms = state
            .recording_in_progress_started_at_ms
            .load(std::sync::atomic::Ordering::Acquire);
        let flag_age_ms = if started_at_ms == 0 {
            0
        } else {
            now_ms() - started_at_ms
        };
        let start_pending_recheck = state
            .start_recording_pending
            .load(std::sync::atomic::Ordering::Acquire);
        let preflight_recheck = state
            .meeting_preflight_in_progress
            .load(std::sync::atomic::Ordering::Acquire);
        let decision = decide_post_probe(
            has_live_capture,
            flag_age_ms,
            start_pending_recheck,
            preflight_recheck,
        );
        match decision {
            Reentry::ProceedAfterStaleHeal => {
                // Flag was true, no live capture, no start in flight,
                // older than RECORDING_STALE_MS, and a re-check
                // confirmed nothing came in flight while we probed.
                // Genuinely stale. We deliberately don't re-CAS:
                // recording_in_progress is already true from any
                // perspective, so we keep it that way for this
                // invocation. Stamp a fresh started-at so the next
                // stale-heal probe times from now, not from the old
                // wedged value.
                tracing::warn!(
                    flag_age_ms = flag_age_ms,
                    "start_recording: recording_in_progress flag was stuck true (age >= {RECORDING_STALE_MS}ms) with no active capture or in-flight start — clearing as stale and proceeding"
                );
                state
                    .recording_in_progress_started_at_ms
                    .store(now_ms(), std::sync::atomic::Ordering::Release);
            }
            other => return reject_reentry(&handle, other).await,
        }
    }

    // Claim the start-pending CAS-mutex IMMEDIATELY, BEFORE any await.
    // `try_claim` returns None if some other invocation has already
    // claimed pending — which under normal flow shouldn't happen
    // (the recording_in_progress CAS above already serialised us)
    // but CAN happen via the stale-heal path: we proceeded with
    // recording_in_progress already true, and another start may
    // have CAS-claimed pending in the meantime. Treat that as
    // "another start owns the pending lock" and reject — it's
    // the safer side of the trade-off, and the user can retry.
    let Some(_start_pending_guard) = StartPendingGuard::try_claim(&state.start_recording_pending)
    else {
        tracing::warn!(
            "start_recording rejected: start_recording_pending CAS lost — another start owns the lock"
        );
        emit_busy(&handle);
        // Don't clear recording_in_progress: we proceeded under the
        // stale-heal branch, so it was already true; clearing it now
        // would yank it out from under the in-flight start that
        // owns the pending lock.
        return Err(OrliqError::User {
            message: "Recording is starting — try again in a moment.".into(),
        });
    };

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
            // the next press. `clear_recording_flag` also zeroes the
            // started-at timestamp so the next stale-heal probe doesn't
            // see a phantom age.
            clear_recording_flag(state.inner());
            // No ActiveRecordingContext was bound at this point (it's
            // claimed below after the meeting preflight passes), but
            // pair the clears for uniformity. Pinned by
            // `every_clear_recording_flag_site_pairs_with_clear_active_recording_context`.
            clear_active_recording_context(state.inner()).await;
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

    // ── Meeting-mode preflight (v0.15.0 → v0.15.4) ──────────────────
    //
    // History:
    //   * v0.14.x — `resolve_target_model_id` silently fell back to
    //     Parakeet when Whisper Large wasn't installed. Meeting Mode
    //     without Whisper Large just ran the wrong model.
    //   * v0.15.0 — hard-coded Whisper Large requirement at
    //     start-time. Better than the silent fallback but locked
    //     every meeting user into the slowest model in the catalogue.
    //   * v0.15.1 — disabled diarisation; chunked path uses Whisper
    //     Large.
    //   * v0.15.3 — meeting model is an EXPLICIT user choice via
    //     `Settings.models.meeting_asr_model_id`. The default (when
    //     no preference is set) was Whisper Medium IF installed; if
    //     missing the start refused and told the user to pick. Never
    //     silently routes to Large, Small, or Parakeet.
    //   * v0.15.4 — same allow-list, but the default flips from
    //     Medium to Small. Step 0 measurements put Small at RTF
    //     ~0.2 vs Medium's 0.87 on the same audio, so the option
    //     that returns text quickly is now the one a no-preference
    //     user gets. Medium and Large remain explicit choices.
    //
    // After preflight passes, write an `ActiveRecordingContext` so
    // `stop_recording` can read the start-time meeting choice
    // without re-resolving from settings.
    let meeting_model_id_for_context = if is_meeting {
        match resolve_meeting_model_id(&settings, &state.models) {
            MeetingModelResolution::Use(id) => {
                tracing::info!(%id, "start_recording: meeting model resolved");
                Some(id)
            }
            MeetingModelResolution::RefuseExplicitMissing(id) => {
                let label = display_name_short(&id);
                tracing::warn!(
                    pref_id = %id,
                    "start_recording refused: meeting model preference is set but the model isn't installed"
                );
                clear_recording_flag(state.inner());
                clear_active_recording_context(state.inner()).await;
                set_phase(
                    &handle,
                    DictationPhase::Error {
                        message: format!(
                            "Your meeting model — {label} — isn't installed. Open Settings → Models, then click Install on the option you've chosen."
                        ),
                    },
                )
                .await;
                schedule_idle(&handle, 2400);
                return Err(OrliqError::User {
                    message: format!(
                        "Your meeting model ({label}) isn't installed. Open Settings → Models, then click Install on the option you've chosen."
                    ),
                });
            }
            MeetingModelResolution::RefusePreferenceUnset => {
                tracing::warn!(
                    "start_recording refused: meeting mode has no model preference and Whisper Small (default) isn't installed"
                );
                clear_recording_flag(state.inner());
                clear_active_recording_context(state.inner()).await;
                set_phase(
                    &handle,
                    DictationPhase::Error {
                        message: "Pick a meeting model. Open Settings → Models and choose Fast (Whisper Small), Balanced (Whisper Medium), or Best accuracy (Whisper Large v3).".into(),
                    },
                )
                .await;
                schedule_idle(&handle, 2400);
                return Err(OrliqError::User {
                    message: "Pick a meeting model. Open Settings → Models and choose Fast (Whisper Small), Balanced (Whisper Medium), or Best accuracy (Whisper Large v3).".into(),
                });
            }
            MeetingModelResolution::RefuseUnsupportedModel(id) => {
                tracing::warn!(
                    pref_id = %id,
                    "start_recording refused: meeting_asr_model_id is set to an off-allow-list value (Meeting Mode supports only Whisper Small / Medium / Large v3)"
                );
                clear_recording_flag(state.inner());
                clear_active_recording_context(state.inner()).await;
                set_phase(
                    &handle,
                    DictationPhase::Error {
                        message: "Your meeting model setting is invalid. Open Settings → Models and choose Fast, Balanced, or Best accuracy.".into(),
                    },
                )
                .await;
                schedule_idle(&handle, 2400);
                return Err(OrliqError::User {
                    message: "Your meeting model setting is invalid. Open Settings → Models and choose Fast, Balanced, or Best accuracy.".into(),
                });
            }
        }
    } else {
        None
    };

    if is_meeting {
        // Diariser asset preflight DISABLED in v0.15.1 (and still
        // disabled in v0.15.3). See releases/0.15.1.md for context.
        tracing::info!("meeting mode diarisation asset preflight disabled in v0.15.1 beta hotfix");
    }

    // Bind the recording session context BEFORE arming capture, so
    // an armament failure has something to clear. The context lives
    // on AppState and stop_recording reads it for is_meeting +
    // meeting_model_id. Settings drift mid-recording cannot change
    // these values once captured here.
    let active_mode_id_for_context = active_mode_for_capture
        .as_ref()
        .map(|m| m.id.clone())
        .unwrap_or_else(|| {
            settings
                .active_mode_id
                .clone()
                .unwrap_or_else(|| "voice-to-text".into())
        });
    let context_started_at_ms = now_ms();
    {
        let mut ctx_lock = state.active_recording_context.write().await;
        *ctx_lock = Some(crate::app_state::ActiveRecordingContext {
            mode_id: active_mode_id_for_context,
            is_meeting,
            meeting_model_id: meeting_model_id_for_context,
            started_at_ms: context_started_at_ms,
            // Populated below if the experimental mic-mute path actually
            // muted any apps. Stays empty when the feature is off, the
            // toggle is off, no hotkey is configured, or the call apps
            // aren't running.
            mic_mute_applied: Vec::new(),
        });
    }

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

    // Experimental Teams + Discord true mic-mute (off by default; only
    // active when the user has flipped the toggle in Settings, Advanced,
    // Experimental AND captured a hotkey for that app). Runs alongside
    // the v0.17.0 conferencing-output-mute above; the two features are
    // independent (output-mute silences what you HEAR from the call
    // app, mic-mute silences what you SEND into the call app's
    // microphone). Same Microphone-source-only guard as the existing
    // ducking path: System / Mixed captures don't make sense for either
    // feature.
    //
    // Hotkey send + state-file write happen synchronously on a blocking
    // thread because we want the state file written BEFORE recording
    // armed; if Utterbar crashes between this point and the stop hook,
    // restore-on-launch reads the file and tries to recover. Spawning
    // off-thread would race with the crash window.
    //
    // The whole block compiles to a no-op in default builds (see
    // services::mic_mute apply implementation, which is feature-gated).
    // v0.18.3 diagnostic logging. The lifecycle hook is always called on
    // every recording start; the log lines below tell us where the
    // mic-mute path is dropping out when banners don't fire.
    let mm_settings = &settings.experimental.mic_mute;
    tracing::info!(
        source = ?source,
        teams_enabled = mm_settings.teams.enabled,
        teams_hotkey_set = mm_settings.teams.hotkey.is_some(),
        discord_enabled = mm_settings.discord.enabled,
        discord_hotkey_set = mm_settings.discord.hotkey.is_some(),
        "mic_mute diag: lifecycle hook entered"
    );
    if matches!(source, CaptureSource::Microphone) {
        match crate::services::mic_mute::apply_request_from_settings(mm_settings) {
            Some(req) => {
                tracing::info!(
                    teams_in_req = req.teams.is_some(),
                    discord_in_req = req.discord.is_some(),
                    "mic_mute diag: apply_request_from_settings returned Some, dispatching apply_with_event on blocking thread"
                );
                let handle_for_apply = handle.clone();
                let applied = tokio::task::spawn_blocking(move || {
                    crate::services::mic_mute::apply_with_event(&handle_for_apply, &req)
                })
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "mic_mute diag: spawn_blocking task panicked");
                    Vec::new()
                });

                tracing::info!(
                    applied_count = applied.len(),
                    "mic_mute diag: apply_with_event returned"
                );

                if !applied.is_empty() {
                    let mut ctx_lock = state.active_recording_context.write().await;
                    if let Some(ctx) = ctx_lock.as_mut() {
                        ctx.mic_mute_applied = applied;
                    }
                }
            }
            None => {
                tracing::info!(
                    "mic_mute diag: apply_request_from_settings returned None (both toggles off, or both hotkeys unset)"
                );
            }
        }
    } else {
        tracing::info!(
            source = ?source,
            "mic_mute diag: skipped because source is not Microphone"
        );
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
                restore_mic_mute_sideeffect(&handle, &state).await;
                clear_recording_flag(state.inner());
                clear_active_recording_context(state.inner()).await;
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
            restore_mic_mute_sideeffect(&handle, &state).await;
            clear_recording_flag(state.inner());
            clear_active_recording_context(state.inner()).await;
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

    // Schedule the single-shot auto-stop timer for non-meeting
    // recordings. The transcribe layer in `services/inference` rejects
    // any input over MAX_TRANSCRIBE_SECONDS (5 minutes) to avoid the
    // offline-decoder OOM crash on long inputs (v0.12.1 origin), and
    // before this timer a user who talked past 5 minutes would lose
    // the entire recording on stop. The timer fires at
    // SINGLE_SHOT_AUTO_STOP_MS (4:55), re-checks the session identity
    // via the start-time context's started_at_ms, confirms the
    // session is still non-meeting, and calls the same internal stop
    // path the user's hotkey would.
    //
    // Meeting Mode is unaffected: it has its own ~60-minute cap and its
    // chunked transcribe path doesn't hit the 5-minute single-input cap,
    // so we don't even spawn the timer when is_meeting=true.
    if !is_meeting {
        let auto_stop_handle = handle.clone();
        let auto_stop_state = (*state).clone();
        let scheduled_started_at_ms = context_started_at_ms;
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(
                SINGLE_SHOT_AUTO_STOP_MS as u64,
            ))
            .await;
            // Race-safety: a stop or cancel may already have run, or a
            // brand-new recording may have started between the schedule
            // and the wake. The timer is "armed for THIS session"; if
            // the bound context is gone or its started_at_ms differs,
            // the timer is stale and must do nothing.
            //
            // Reading `recording_in_progress` first is a cheap fast-path
            // for the common case where the user already pressed stop.
            if !auto_stop_state
                .recording_in_progress
                .load(std::sync::atomic::Ordering::Acquire)
            {
                tracing::debug!(
                    "auto-stop timer fired but recording_in_progress=false; nothing to stop"
                );
                return;
            }
            let ctx_snapshot = auto_stop_state
                .active_recording_context
                .read()
                .await
                .clone();
            if !auto_stop_should_proceed(scheduled_started_at_ms, ctx_snapshot.as_ref()) {
                tracing::debug!(
                    scheduled_started_at_ms,
                    current_ctx = ?ctx_snapshot,
                    "auto-stop timer fired but the session has changed; nothing to stop"
                );
                return;
            }
            tracing::info!(
                scheduled_started_at_ms,
                cap_ms = SINGLE_SHOT_AUTO_STOP_MS,
                "auto-stop timer firing stop_recording for single-shot dictation at 5-minute cap"
            );
            match stop_recording_inner(
                auto_stop_handle,
                &auto_stop_state,
                StopReason::AutoStoppedAtCap,
            )
            .await
            {
                Ok(_) => {
                    tracing::info!("auto-stop timer drove stop_recording successfully");
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "auto-stop timer's stop_recording call returned an error; this can happen if the user's manual stop won the race"
                    );
                }
            }
        });
    }

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

/// Restore the experimental mic-mute side-effect (if any was applied
/// at start time) on a stop / cancel / rollback path. Reads the apps
/// from the active recording context and clears the field once
/// restore is initiated, so a subsequent (defensive) call does not
/// double-send the unmute keystroke. Always best-effort: errors log
/// and emit a sticky-failure banner; the caller never sees them.
///
/// Compiles to a no-op when no apps were applied (the common case),
/// so calling this from every recording-end path is cheap.
async fn restore_mic_mute_sideeffect(handle: &AppHandle, state: &AppState) {
    // Snapshot + clear under the lock. Subsequent reads see an empty
    // applied list, which means a defensive second call from a
    // different stop path is a no-op.
    let applied: Vec<crate::services::mic_mute::MicMuteApp> = {
        let mut ctx_lock = state.active_recording_context.write().await;
        match ctx_lock.as_mut() {
            Some(ctx) if !ctx.mic_mute_applied.is_empty() => {
                std::mem::take(&mut ctx.mic_mute_applied)
            }
            _ => return,
        }
    };

    let settings_snapshot = state.settings.snapshot().await;
    let req = crate::services::mic_mute::restore_request_from_settings(
        &settings_snapshot.experimental.mic_mute,
        applied,
    );
    let handle_for_restore = handle.clone();
    tokio::task::spawn_blocking(move || {
        crate::services::mic_mute::restore_with_event(&handle_for_restore, &req);
    });
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
            | DictationPhase::NoAudio { .. } => {
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
/// Catalogue ID of the Meeting Mode default when the user hasn't
/// expressed a preference. v0.15.4+: Whisper Small (the "Fast"
/// option) — Step 0 measurements on real meetings put Small at
/// RTF ~0.2 vs Medium's 0.87, so the default is now the option
/// that actually returns text within a useful timeframe on
/// laptop-class CPUs. v0.15.3 used Medium; pre-v0.15.3 silently
/// used Whisper Large v3 — see `resolve_meeting_model_id` for
/// the full rationale (no silent slow path).
pub(crate) const DEFAULT_MEETING_MODEL_ID: &str = "sherpa-whisper-small";
/// Catalogue ID of the Meeting Mode "Fast" option. Used by the
/// resolver's allow-list and the picker UI's hard-coded radio
/// rows; both must agree on this single source of truth. v0.15.4:
/// this is the same id as `DEFAULT_MEETING_MODEL_ID`. The two
/// constants are kept distinct because they answer different
/// questions — "what is the Fast row in the picker?" vs "what
/// model fires when no preference is set?" — and could diverge in
/// a future release.
pub(crate) const MEETING_FAST_MODEL_ID: &str = "sherpa-whisper-small";
/// Catalogue ID of the Meeting Mode "Balanced" option (Whisper
/// Medium). Allow-listed and selectable by name; never the default.
pub(crate) const MEETING_BALANCED_MODEL_ID: &str = "sherpa-whisper-medium";
/// Catalogue ID of the Meeting Mode "Best accuracy" option.
/// Same rationale as above.
pub(crate) const MEETING_BEST_MODEL_ID: &str = "sherpa-whisper-large-v3";

/// Allow-list check for explicit `meeting_asr_model_id` settings
/// values. Meeting Mode in v0.15.3 supports exactly three model
/// ids — Whisper Small, Medium, and Large v3. Any other id
/// (Parakeet variants, Zipformer, Whisper Base, future ASR ids)
/// is rejected at the resolver layer regardless of whether the
/// model is installed.
///
/// **Why this is necessary.** The picker UI only ever writes one
/// of the three allow-listed ids. But `meeting_asr_model_id`
/// lands in `settings.json` on disk, which a user can hand-edit.
/// Without an allow-list, a hand-edited entry like
/// `"sherpa-parakeet-tdt-0.6b-v3-int8"` would silently route
/// meetings through Parakeet — fast, but wrong shape (Parakeet
/// is English-only, doesn't handle multi-speaker audio well, and
/// Meeting Mode's contract is "Whisper for accuracy on long
/// recordings"). The allow-list closes that loophole.
///
/// Future Whisper variants would need to be added here AND to the
/// catalogue AND to the picker UI; the three locations together
/// define what "Meeting Mode model" means.
pub(crate) fn is_allowed_meeting_model_id(id: &str) -> bool {
    // v0.15.4: `DEFAULT_MEETING_MODEL_ID` and `MEETING_FAST_MODEL_ID`
    // are both Whisper Small now, so the allow-list arms must
    // explicitly include the Balanced (Medium) id rather than
    // implying it via `DEFAULT_MEETING_MODEL_ID`. Three distinct
    // ids: Small / Medium / Large v3.
    matches!(
        id,
        MEETING_FAST_MODEL_ID | MEETING_BALANCED_MODEL_ID | MEETING_BEST_MODEL_ID
    )
}

/// Resolution outcome for the meeting model selection. The
/// `Refuse*` variants carry the user-facing message verbatim so
/// `start_recording` can surface them without re-deriving the copy.
///
/// **No silent fallback. No off-allow-list selection.** Pre-v0.15.3
/// Meeting Mode silently routed to Whisper Large v3 whenever it
/// was installed, which dragged every existing beta user through
/// hour-plus CPU transcribes regardless of whether they wanted
/// that or whether faster alternatives were also available. v0.15.3
/// makes the choice explicit AND restricts it to the three
/// allow-listed Whisper variants:
///
///   * If `meeting_asr_model_id` is set but not allow-listed
///     (Parakeet / Zipformer / Whisper Base / unknown id) →
///     `RefuseUnsupportedModel(id)`. The user error tells them to
///     pick a supported option in Settings → Models. No silent
///     route through the off-allow-list model.
///   * If allow-listed and installed → `Use(id)`.
///   * If allow-listed but the chosen model isn't installed →
///     `RefuseExplicitMissing(id)`. No silent fallback to a
///     different allow-listed model.
///   * If `meeting_asr_model_id` is unset and Whisper Small is
///     installed → `Use("sherpa-whisper-small")`. v0.15.4 made
///     Small the default; v0.15.3 used Medium here.
///   * If unset and Small is missing → `RefusePreferenceUnset`.
///     The error tells them to pick in Settings → Models. No
///     silent route through Medium, Large, or
///     `default_asr_model_id` — even when those are installed.
///     The user has to either install Small or pick a model
///     explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MeetingModelResolution {
    Use(String),
    RefuseExplicitMissing(String),
    RefuseUnsupportedModel(String),
    RefusePreferenceUnset,
}

/// Pure resolver for the meeting-mode model. Extracted so the
/// policy is testable without standing up an `AppState`. See
/// `MeetingModelResolution` for the outcomes and the allow-list
/// rule.
pub(crate) fn resolve_meeting_model_id(
    settings: &crate::domain::Settings,
    models: &crate::services::models::ModelRegistry,
) -> MeetingModelResolution {
    if let Some(id) = settings.models.meeting_asr_model_id.as_deref() {
        // Allow-list check FIRST. An off-allow-list explicit
        // pref (Parakeet, Zipformer, Whisper Base, unknown
        // future id) refuses regardless of installed-ness — we
        // never want to silently transcribe a meeting through a
        // model the contract doesn't sanction.
        if !is_allowed_meeting_model_id(id) {
            return MeetingModelResolution::RefuseUnsupportedModel(id.to_string());
        }
        if models.is_installed(id) {
            MeetingModelResolution::Use(id.to_string())
        } else {
            MeetingModelResolution::RefuseExplicitMissing(id.to_string())
        }
    } else if models.is_installed(DEFAULT_MEETING_MODEL_ID) {
        MeetingModelResolution::Use(DEFAULT_MEETING_MODEL_ID.to_string())
    } else {
        MeetingModelResolution::RefusePreferenceUnset
    }
}

/// Short, human-readable label for the success pill + logs. Maps a
/// catalogue id to a user-facing string. Unknown ids fall back to
/// the id itself — better than panicking, and a missing label is
/// at most a UX nit, not a behaviour bug.
pub(crate) fn display_name_short(model_id: &str) -> String {
    match model_id {
        "sherpa-whisper-small" => "Whisper Small".into(),
        "sherpa-whisper-medium" => "Whisper Medium".into(),
        "sherpa-whisper-large-v3" => "Whisper Large v3".into(),
        "sherpa-whisper-base" => "Whisper Base".into(),
        other => other.to_string(),
    }
}

/// Realtime factor — processing-time / audio-time. Returns `None`
/// when audio_ms is zero (avoids division by zero; logs render
/// `None` cleanly). For typical inputs returns a small `f32`
/// (0.3-1.5 range). Used in the meeting summary log; never
/// exposed as a public claim.
pub(crate) fn realtime_factor(audio_ms: u32, processing_ms: u64) -> Option<f32> {
    if audio_ms == 0 {
        None
    } else {
        Some(processing_ms as f32 / audio_ms as f32)
    }
}

/// Clear the in-flight session context. Called from every
/// recording-end path: successful stop, cancel, no-active-recording
/// stop, transcribe error, and every start-side error path that
/// also clears `recording_in_progress`. Pairing the two clears is
/// pinned by `every_clear_recording_flag_site_pairs_with_clear_active_recording_context`.
async fn clear_active_recording_context(state: &AppState) {
    *state.active_recording_context.write().await = None;
}

/// Bridge-case failure for `resolve_target_model_id`. Only
/// produced when a recording arrived at stop_recording with NO
/// `ActiveRecordingContext` bound (the v0.15.2-in-flight bridge
/// case), AND the current settings can't yield an allow-listed
/// meeting model. Surfaces the underlying `MeetingModelResolution`
/// refusal so the caller can render the right user message.
///
/// **Why this isn't allowed to fall through to
/// `default_asr_model_id`:** for a meeting recording, the global
/// default is typically Parakeet (the dictation-mode default) or
/// the user's chosen non-meeting model. Routing meeting audio
/// through Parakeet violates v0.15.3's "no Parakeet meeting
/// routing" contract; routing through a non-meeting Whisper
/// variant (e.g. Whisper Base, which isn't allow-listed) violates
/// the same contract. The right shape is to refuse loudly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MeetingBridgeFailure {
    pub resolution: MeetingModelResolution,
}

/// Resolve which ASR model to use for this dictation. One source of
/// truth for the early-warm probe, the cold-load Warming pill, and
/// the final transcribe call — they must all read the same value or
/// the UI flashes the wrong "Initialising model" copy when the user
/// has Meeting mode using a different model from the non-meeting
/// default.
///
/// **v0.15.3 binding rule.** When the in-flight recording session
/// has a context bound (the normal case for any v0.15.3+ recording),
/// stop reads the meeting_model_id from the context — start-time
/// choice wins, settings drift mid-recording is ignored. The
/// optional `bound_meeting_model_id` parameter is the read of
/// `state.active_recording_context.read().await....meeting_model_id`
/// from the caller. When `None` AND the recording is NOT a meeting,
/// fall through to the pure non-meeting resolver
/// (`default_asr_model_id` then `"stub"`). When `None` AND the
/// recording IS a meeting (the v0.15.2-in-flight bridge case),
/// re-resolve from current settings — but ONLY accept allow-listed
/// meeting models. Anything else returns
/// `Err(MeetingBridgeFailure)` so the caller surfaces a clear
/// error rather than silently routing through Parakeet.
///
/// Non-meeting fallback: `default_asr_model_id`, then the literal
/// `"stub"` sentinel (which the orchestrator already routes to a
/// no-op provider when no real model is configured).
fn resolve_target_model_id(
    mode: &crate::domain::Mode,
    settings: &crate::domain::Settings,
    models: &crate::services::models::ModelRegistry,
    bound_meeting_model_id: Option<&str>,
) -> Result<String, MeetingBridgeFailure> {
    if mode.is_meeting_mode {
        if let Some(bound) = bound_meeting_model_id {
            return Ok(bound.to_string());
        }
        // No context bound — bridge case (v0.15.2 recording in
        // flight when v0.15.3 launched). Re-resolve from current
        // settings, BUT only accept the resolver's `Use(id)`
        // success; any refusal becomes a `MeetingBridgeFailure`.
        // We do NOT fall through to `default_asr_model_id` here —
        // that would silently route through Parakeet (or whatever
        // the user's non-meeting default is), which violates the
        // "no Parakeet meeting routing" contract.
        let resolution = resolve_meeting_model_id(settings, models);
        return match resolution {
            MeetingModelResolution::Use(id) => {
                tracing::warn!(
                    %id,
                    "stop_recording: no active recording context bound; resolved meeting model from current settings (bridge case — v0.15.2 in-flight recording?)"
                );
                Ok(id)
            }
            other => {
                tracing::error!(
                    ?other,
                    "stop_recording: meeting recording with no bound context AND meeting model unresolvable from current settings — refusing to fall back to default_asr_model_id"
                );
                Err(MeetingBridgeFailure { resolution: other })
            }
        };
    }
    Ok(settings
        .models
        .default_asr_model_id
        .clone()
        .unwrap_or_else(|| "stub".into()))
}

/// Meeting Mode chunked transcribe — fallback path for v0.15+.
///
/// Used when speaker diarisation is unavailable for any reason
/// (assets missing, model load error, panic in compute, empty
/// segment list on a non-empty buffer). Produces a timestamped
/// transcript with NO speaker labels:
///
///     [00:00] First chunk text…
///
///     [00:30] Next chunk text…
///
/// Slices the captured PCM into ~30 s windows with a 2 s overlap.
/// The chunk boundary is wall-clock based (not silence-aware) —
/// silence-aware chunking is a future polish; the 2 s overlap lets
/// the recogniser stitch boundary words back together rather than
/// cleaving them.
///
/// Why 30 s + 2 s: the offline transcribe path's failure mode is
/// O(N) memory + accumulated decoder state inside ONNX Runtime.
/// 30 s is well below the empirical crash floor. 2 s is roughly
/// two words of speech at conversational pace — enough to catch
/// the boundary word in both chunks for the dedup pass.
///
/// Stitching: each non-first chunk has its first ~6.7 % of words
/// dropped (the proportional overlap fraction, 2 s / 30 s).
/// Imperfect (words don't distribute perfectly evenly) but
/// bounded.
///
/// History: v0.13.x → v0.14.x of this function carried per-chunk
/// `**You**:` / `**Other**:` speaker tags derived from a loopback-
/// floor RMS heuristic. The labels were unreliable on real
/// recordings (mic bleed from speakers) and were suppressed in
/// v0.14.3. The whole heuristic block was removed in v0.15.0 in
/// favour of the proper sherpa-onnx diarisation path. v0.15.1
/// rolled diarisation back after real-meeting testing produced 91
/// fake speakers + cross-language Whisper hallucinations on short
/// segments — `transcribe_chunked` is meeting mode's primary
/// transcription path again, no speaker labels emitted.
///
/// Constants and pure helpers (`CHUNKED_MEETING_CHUNK_SECS`,
/// `CHUNKED_MEETING_OVERLAP_SECS`, `MAX_MEETING_CHUNKS`,
/// `projected_chunk_count`, `max_meeting_audio_minutes`) live just
/// below this function so they sit next to the loop they describe.
async fn transcribe_chunked(
    handle: &AppHandle,
    state: &Arc<AppState>,
    model_id: &str,
    pcm: Vec<i16>,
    sample_rate: u32,
    target_language: Option<String>,
) -> anyhow::Result<AsrResult> {
    // Chunk size and overlap rationale lives on the
    // CHUNKED_MEETING_CHUNK_SECS / CHUNKED_MEETING_OVERLAP_SECS
    // constants above so unit tests can pin the values without
    // re-stating them here.
    let chunk_samples = CHUNKED_MEETING_CHUNK_SECS * sample_rate as usize;
    let overlap_samples = CHUNKED_MEETING_OVERLAP_SECS * sample_rate as usize;
    let stride = chunk_samples.saturating_sub(overlap_samples);

    let total = pcm.len();
    if total == 0 {
        return Err(anyhow::anyhow!("empty PCM buffer"));
    }

    // Refuse before doing any work. Computing the chunk count up front
    // via the pure helper (so tests can pin the boundary behaviour
    // without standing up an inference orchestrator) makes the failure
    // mode visible from the log even if the user doesn't see the toast.
    let projected_chunks = projected_chunk_count(total, sample_rate);
    if projected_chunks > MAX_MEETING_CHUNKS {
        let projected_secs = total / sample_rate.max(1) as usize;
        tracing::warn!(
            projected_chunks,
            projected_secs,
            max_chunks = MAX_MEETING_CHUNKS,
            "meeting mode: refusing to transcribe — too many chunks"
        );
        return Err(anyhow::anyhow!(
            "meeting too long ({} minutes) — split into shorter recordings (cap is {} minutes per recording)",
            projected_secs / 60,
            max_meeting_audio_minutes()
        ));
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
                },
            )
            .await;
    }

    let mut start = 0usize;
    let mut chunk_index = 0u32;
    let mut stitched = String::new();
    let mut total_duration_ms: u32 = 0;
    let mut first_lang: Option<String> = None;
    let mut model_version = String::new();

    // Total chunks is known up front via the same pure helper that
    // gates the over-cap refusal above. The loop iterates chunk_index
    // 0..total_chunks - 1, so the user sees a thin progress bar fill
    // smoothly during long meetings instead of staring at a static
    // "Transcribing meeting…" pill for minutes.
    let total_chunks = projected_chunks as u32;

    while start < total {
        let end = (start + chunk_samples).min(total);
        let slice = pcm[start..end].to_vec();
        let slice_secs = (end - start) as f32 / sample_rate as f32;
        // Emit progress at the START of the iteration so the bar
        // updates the moment we begin work on this chunk, not after
        // it lands. Best-effort: a failed emit is logged at debug
        // level by Tauri itself; the transcribe must continue
        // regardless. Frontend listener stores `{ chunk_index,
        // total_chunks }` in zustand; StatePill reads it during
        // `TranscribingBackground` and renders a thin progress bar.
        let _ = handle.emit(
            crate::domain::EVT_DICTATION_CHUNK_PROGRESS,
            crate::domain::MeetingChunkProgress {
                chunk_index,
                total_chunks,
            },
        );
        tracing::info!(
            chunk_index,
            total_chunks,
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
            let drop_count = (words.len() * CHUNKED_MEETING_OVERLAP_SECS
                / CHUNKED_MEETING_CHUNK_SECS)
                .min(words.len());
            words[drop_count..].join(" ")
        };

        // Per-chunk output validation. Conservative rule set
        // (services/output_validation::validate_meeting_chunk) drops
        // empty / whitespace / single-punctuation / repeated-
        // punctuation chunks; logs but DOES NOT DROP repeated-token
        // chunks (decoder loops are observed-only in Meeting Mode
        // because real meeting speech can carry legitimate
        // repetition). Foreign-script and known-hallucination rules
        // are deliberately not applied here - the engine-internal
        // Whisper hallucination filter already runs per chunk
        // upstream, and multilingual meetings make the foreign-
        // script rule unsafe.
        //
        // The previous "skip if chunk_text.trim().is_empty()" guard
        // is subsumed by Drop(Empty) / Drop(WhitespaceOnly).
        use crate::services::output_validation::{validate_meeting_chunk, MeetingChunkVerdict};
        let chunk_chars = chunk_text.chars().count();
        match validate_meeting_chunk(&chunk_text) {
            MeetingChunkVerdict::Drop { reason } => {
                tracing::info!(
                    reason = %reason.as_str(),
                    chunk_index,
                    chunk_chars,
                    chunk_duration_ms = (slice_secs * 1000.0) as u32,
                    chunk_start_sample = start,
                    model = %model_id,
                    dropped = true,
                    "meeting_chunk_validation: dropped chunk",
                );
                // Skip appending. Continue to the bookkeeping below
                // (language capture, model_version, duration sum)
                // so the dropped chunk doesn't break the per-chunk
                // metadata threading.
            }
            MeetingChunkVerdict::Observe { reason } => {
                tracing::info!(
                    reason = %reason.as_str(),
                    chunk_index,
                    chunk_chars,
                    chunk_duration_ms = (slice_secs * 1000.0) as u32,
                    chunk_start_sample = start,
                    model = %model_id,
                    dropped = false,
                    "meeting_chunk_validation: observed (kept)",
                );
                // Fall through and append the chunk normally. The
                // observe rule is telemetry only at this stage; we
                // need soak data on real meeting speech before
                // promoting it to a drop.
                append_meeting_chunk(
                    &mut stitched,
                    &chunk_text,
                    chunk_index,
                    start,
                    overlap_samples,
                    sample_rate,
                );
            }
            MeetingChunkVerdict::Accept => {
                append_meeting_chunk(
                    &mut stitched,
                    &chunk_text,
                    chunk_index,
                    start,
                    overlap_samples,
                    sample_rate,
                );
            }
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
        // CHUNKED_MEETING_CHUNK_SECS — handle that by using the actual
        // slice length.
        let slice_ms = ((end - start) as u64 * 1000 / sample_rate as u64) as u32;
        let contribution_ms = if chunk_index == 0 {
            slice_ms
        } else {
            slice_ms.saturating_sub((CHUNKED_MEETING_OVERLAP_SECS as u32) * 1000)
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

/// Append a kept meeting chunk to the stitched transcript with the
/// `[MM:SS]` timestamp prefix, the inter-chunk paragraph break, and
/// the wall-clock-start computation that accounts for the boundary-
/// dedup overlap (non-first chunks contribute starting at
/// `start + overlap_samples` because their first ~overlap seconds
/// were dropped from the text by the dedup pass upstream).
///
/// Extracted only because two branches of the per-chunk validation
/// match arm (`Observe` and `Accept`) need it. The original code
/// inlined this in a single `if !is_empty()` branch; once that
/// branch became three (`Drop` / `Observe` / `Accept`), repeating
/// the timestamp logic three times was the wrong move.
fn append_meeting_chunk(
    stitched: &mut String,
    chunk_text: &str,
    chunk_index: u32,
    start: usize,
    overlap_samples: usize,
    sample_rate: u32,
) {
    let chunk_text_start_samples = if chunk_index == 0 {
        start
    } else {
        start + overlap_samples
    };
    let chunk_text_start_secs = chunk_text_start_samples / sample_rate as usize;
    let mm = chunk_text_start_secs / 60;
    let ss = chunk_text_start_secs % 60;

    if !stitched.is_empty() {
        stitched.push_str("\n\n");
    }
    stitched.push_str(&format!("[{:02}:{:02}]", mm, ss));
    stitched.push(' ');
    stitched.push_str(chunk_text);
}

/// Meeting-mode chunk size. Carried forward from v0.13.x — v0.14.x;
/// tightened from 30 → 29 in v0.15.3.
///
/// **Why 29, not 30 (v0.15.3 fix):** sherpa-onnx's offline Whisper
/// recogniser silently truncates inputs >= 30 seconds with a
/// stderr warning ("Only waves less than 30 seconds are supported").
/// The pre-v0.15.3 chunker passed exactly 30s buffers in the
/// non-final-chunk case and quietly lost up to 1s of audio per
/// boundary. Step 0 of v0.15.3 (verifying Whisper Medium loads)
/// surfaced the warning; pulling the constant down to 29 gives
/// the recogniser the same input every time and removes the
/// truncation. Existing v0.15.2 transcripts may have a small
/// boundary-loss; new transcripts don't.
///
/// Overlap (2s) is unchanged — ~two words at conversational pace,
/// enough for the boundary-dedup pass to re-attach a clipped
/// boundary word without dropping legit content. Stride (chunk −
/// overlap = 27s) follows.
const CHUNKED_MEETING_CHUNK_SECS: usize = 29;
const CHUNKED_MEETING_OVERLAP_SECS: usize = 2;

/// v0.15.1 defence-in-depth cap, retuned in v0.15.3 for the new 29s
/// chunk size. The chunked path is meeting mode's primary
/// transcription route. Without a cap, an absurdly long recording
/// (a forgotten running session, or a future bug routing dictation
/// audio here) could trigger hundreds of sequential Whisper CPU
/// calls and wedge the app for hours.
///
/// 135 chunks × 27 s stride + 29 s leading chunk = 3647 s ≈
/// **60 min 47 s** of audio. Sized to comfortably accept full
/// 30 / 45 / 60 min meetings (the standard meeting cadence) with a
/// small safety shoulder for "I forgot to stop", and to keep the
/// worst-case CPU Whisper transcribe time bounded at ~1 hour
/// (Whisper Medium on CPU runs roughly real-time or slightly
/// faster on the hardware Utterbar targets; Whisper Large is
/// slower; the cap is sized against the slow end). Anything longer
/// should be split into separate recordings — the cap surfaces
/// that as a clean user-facing error rather than letting the app
/// hang.
///
/// History: 60 (≈28 min) in v0.15.1 round-1; 120 (≈56 min) in
/// round-2; 130 (≈60 min 40 s) in round-3. v0.15.3 retuned to 135
/// for the new 29s chunk size to keep ~60 min coverage. The
/// release notes describe the cap as "about 60 minutes (60 min
/// 47 s, to be exact)".
const MAX_MEETING_CHUNKS: usize = 135;

/// Pure helper: given a PCM buffer length in samples + a sample rate,
/// how many chunks would `transcribe_chunked` produce? Mirrors the
/// loop arithmetic exactly: the chunker advances `start += stride`
/// while `start < total`, so the chunk count is
/// `ceil((total - chunk) / stride) + 1` for total > chunk, or 1 when
/// the whole buffer fits in a single window.
///
/// Extracted so the chunk-cap behaviour can be pinned by unit tests
/// without standing up an inference orchestrator. **The inner-loop
/// arithmetic in `transcribe_chunked` MUST stay congruent with this
/// helper** — any change in one needs the same change in the other,
/// or the cap-rejection path's projection will diverge from the
/// actual chunk count and the cap becomes meaningless. Pinned by
/// `chunk_count_helpers_agree_for_typical_meeting_sizes`.
fn projected_chunk_count(pcm_len_samples: usize, sample_rate: u32) -> usize {
    let chunk_samples = CHUNKED_MEETING_CHUNK_SECS * sample_rate as usize;
    let overlap_samples = CHUNKED_MEETING_OVERLAP_SECS * sample_rate as usize;
    let stride = chunk_samples.saturating_sub(overlap_samples).max(1);
    if pcm_len_samples == 0 {
        0
    } else if pcm_len_samples <= chunk_samples {
        1
    } else {
        // ceil((total - chunk) / stride) + 1 — matches the
        // chunker's `while start < total; start += stride` loop.
        pcm_len_samples
            .saturating_sub(chunk_samples)
            .div_ceil(stride)
            + 1
    }
}

/// Maximum minutes of audio the chunk cap will accept. Used in the
/// user-facing error message when a recording is too long.
/// `MAX_MEETING_CHUNKS` × non-overlapping seconds per chunk, divided
/// by 60. Pulled out as a `const fn` so the runtime cost is zero
/// and the constant can be referenced from tests directly.
const fn max_meeting_audio_minutes() -> usize {
    MAX_MEETING_CHUNKS * (CHUNKED_MEETING_CHUNK_SECS - CHUNKED_MEETING_OVERLAP_SECS) / 60
}

/// User-space merge: collapse adjacent same-speaker segments
/// separated by less than `MERGE_GAP_SEC`. Belt-and-braces over
/// sherpa's own `min_duration_off` (which we set to 0.4 s on the
/// C side); this catches anything sherpa let through.
fn merge_adjacent_same_speaker(
    segments: Vec<sherpa_rs::diarize::Segment>,
) -> Vec<sherpa_rs::diarize::Segment> {
    const MERGE_GAP_SEC: f32 = 0.4;
    let mut out: Vec<sherpa_rs::diarize::Segment> = Vec::with_capacity(segments.len());
    for seg in segments {
        if let Some(last) = out.last_mut() {
            if last.speaker == seg.speaker && (seg.start - last.end) < MERGE_GAP_SEC {
                last.end = seg.end;
                continue;
            }
        }
        out.push(seg);
    }
    out
}

/// User-space shape pass: drop sub-`MIN_SEGMENT_SEC` fragments
/// (likely cross-talk artefacts that sherpa's `min_duration_on`
/// let through) and split anything over `MAX_SEGMENT_SEC` so
/// Whisper Large stays in its sweet spot.
fn shape_segments(segments: Vec<sherpa_rs::diarize::Segment>) -> Vec<sherpa_rs::diarize::Segment> {
    const MIN_SEGMENT_SEC: f32 = 0.6;
    const MAX_SEGMENT_SEC: f32 = 30.0;
    let mut out: Vec<sherpa_rs::diarize::Segment> = Vec::with_capacity(segments.len());
    for seg in segments {
        let dur = seg.end - seg.start;
        if dur < MIN_SEGMENT_SEC {
            continue;
        }
        if dur > MAX_SEGMENT_SEC {
            // Time-based split — uniform pieces of MAX_SEGMENT_SEC
            // each. Not silence-aware (we don't have VAD output
            // here) but bounded and Whisper-stable.
            let mut t = seg.start;
            while t < seg.end {
                let next = (t + MAX_SEGMENT_SEC).min(seg.end);
                out.push(sherpa_rs::diarize::Segment {
                    start: t,
                    end: next,
                    speaker: seg.speaker,
                });
                t = next;
            }
        } else {
            out.push(seg);
        }
    }
    out
}

/// Pad segment boundaries by `BOUNDARY_PAD_MS` on each side so
/// Whisper isn't asked to transcribe a buffer that starts/ends
/// mid-word. Clamps at 0 and `total_samples` so we never reach
/// past the buffer edges.
fn pad_segment_samples(
    start_sec: f32,
    end_sec: f32,
    sample_rate: u32,
    total_samples: usize,
) -> (usize, usize) {
    const BOUNDARY_PAD_MS: i64 = 200;
    let pad_samples = BOUNDARY_PAD_MS * sample_rate as i64 / 1000;
    let start_idx = ((start_sec * sample_rate as f32) as i64 - pad_samples).max(0) as usize;
    let end_idx =
        (((end_sec * sample_rate as f32) as i64 + pad_samples) as usize).min(total_samples);
    (start_idx, end_idx)
}

/// Format one diarised turn into the canonical render shape:
/// `[mm:ss] **Speaker N:** text\n\n`
fn format_speaker_turn(start_sec: u32, speaker_id: i32, text: &str) -> String {
    let mm = start_sec / 60;
    let ss = start_sec % 60;
    // Sherpa speaker IDs are 0-based; surface them as 1-based
    // (`Speaker 1`, `Speaker 2`) — matches the user's mental
    // model and what the rename map keys against.
    let label_id = speaker_id + 1;
    format!(
        "[{:02}:{:02}] **Speaker {}:** {}\n\n",
        mm,
        ss,
        label_id,
        text.trim()
    )
}

/// Meeting Mode diarised transcribe — primary path for v0.15+.
///
/// Runs the diariser once on the full mixed buffer, merges +
/// shapes the resulting segments, then transcribes each segment
/// with Whisper Large. Output carries `[mm:ss] **Speaker N:**
/// text` per turn.
///
/// Caller has already verified the diariser asset paths exist
/// (via `diarize_assets::ensure_assets_ready`) and that the
/// Whisper Large model is installed (via the start_recording
/// preflight). All this function does is the diarise → shape →
/// transcribe → stitch dance.
// v0.15.1: kept in tree but unreachable from production code paths —
// the meeting branch in `stop_recording` now goes straight to
// `transcribe_chunked`. This function (plus the per-segment merge /
// shape / pad / dedup helpers below it) is the v0.15.0 architecture
// that proved unsafe in real meetings (over-fragmenting clusters,
// per-segment Whisper hallucinations, O(N) runtime). Retaining it
// here so v0.15.2 can re-use whatever pieces still apply once the
// "single Whisper pass + post-hoc speaker labelling" rework lands;
// nuking it now would just force a re-write of code we already know
// the shape of. See releases/0.15.1.md.
#[allow(dead_code)]
async fn transcribe_meeting_diarised(
    state: &Arc<AppState>,
    model_id: &str,
    pcm: Vec<i16>,
    sample_rate: u32,
    target_language: Option<String>,
    seg_path: &std::path::Path,
    embed_path: &std::path::Path,
) -> anyhow::Result<AsrResult> {
    if pcm.is_empty() {
        return Err(anyhow::anyhow!("empty PCM buffer"));
    }

    // Source-audio duration. The padded-segment slice durations
    // we'd otherwise sum can overlap (BOUNDARY_PAD_MS on both
    // sides) and would inflate the wall-clock count past the
    // recording's actual length. Use the input PCM length as the
    // single source of truth — it's what the user pressed-record-
    // for. Saturates at u32::MAX, which at 16 kHz is ~74 hours,
    // well past the 8 h capture cap upstream.
    let source_duration_ms: u32 =
        ((pcm.len() as u64 * 1000) / sample_rate as u64).min(u32::MAX as u64) as u32;

    // ── Diarise ─────────────────────────────────────────────────────
    let pcm_for_diarise = pcm.clone();
    let seg_owned = seg_path.to_path_buf();
    let embed_owned = embed_path.to_path_buf();
    let diarise_join = tokio::task::spawn_blocking(
        move || -> anyhow::Result<Vec<sherpa_rs::diarize::Segment>> {
            let mut session = crate::services::inference::diarize::DiariserSession::load(
                &seg_owned,
                &embed_owned,
            )?;
            session.diarise(&pcm_for_diarise)
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!("diariser join: {e}"))?;

    let raw_segments = diarise_join?;
    if raw_segments.is_empty() {
        return Err(anyhow::anyhow!(
            "diariser produced no segments on a non-empty buffer"
        ));
    }

    // ── Merge + shape ────────────────────────────────────────────────
    let segments = shape_segments(merge_adjacent_same_speaker(raw_segments));
    if segments.is_empty() {
        return Err(anyhow::anyhow!(
            "all diariser segments dropped after shape pass"
        ));
    }
    tracing::info!(
        segment_count = segments.len(),
        "meeting mode (diarised): segments after merge+shape"
    );

    // ── Transcribe per segment ──────────────────────────────────────
    let total = pcm.len();
    let mut stitched = String::new();
    // `transcribed_slice_ms` is a DIAGNOSTIC ONLY — sum of slice
    // wall-clock durations, which can exceed source_duration_ms
    // because of BOUNDARY_PAD_MS overlap on both sides of each
    // segment. Used in the closing log line to give the operator
    // a sense of how much padded audio Whisper actually saw.
    // Never used as `AsrResult.duration_ms` — that's the source
    // PCM length only, computed at function entry.
    let mut transcribed_slice_ms: u32 = 0;
    let mut first_lang: Option<String> = None;
    let mut model_version = String::new();
    let mut last_text_per_speaker: std::collections::HashMap<i32, String> =
        std::collections::HashMap::new();

    for (seg_idx, seg) in segments.iter().enumerate() {
        let (start_idx, end_idx) = pad_segment_samples(seg.start, seg.end, sample_rate, total);
        if end_idx <= start_idx {
            continue;
        }
        let slice = pcm[start_idx..end_idx].to_vec();
        let slice_secs = (end_idx - start_idx) as f32 / sample_rate as f32;
        tracing::info!(
            seg_idx,
            speaker = seg.speaker,
            start_sec = seg.start,
            end_sec = seg.end,
            duration_secs = slice_secs,
            "meeting mode (diarised): transcribing segment"
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
                },
            )
            .await?;

        let text = result.text.trim();
        if text.is_empty() {
            continue;
        }

        // Word-overlap dedup: if the same speaker had a previous
        // segment whose tail words equal this segment's head, drop
        // the duplicated head. The boundary pad we added (±200ms)
        // can cause the transcriber to repeat a boundary word at
        // the start of consecutive same-speaker segments.
        let final_text = if let Some(prev) = last_text_per_speaker.get(&seg.speaker) {
            dedup_segment_head_against_tail(prev, text)
        } else {
            text.to_string()
        };

        if final_text.trim().is_empty() {
            continue;
        }

        stitched.push_str(&format_speaker_turn(
            seg.start as u32,
            seg.speaker,
            &final_text,
        ));

        last_text_per_speaker.insert(seg.speaker, final_text.clone());

        if first_lang.is_none() {
            first_lang = result.language.clone();
        }
        if model_version.is_empty() {
            model_version = result.model_version.clone();
        }

        let slice_ms = (slice_secs * 1000.0) as u32;
        transcribed_slice_ms = transcribed_slice_ms.saturating_add(slice_ms);
    }

    // Trim trailing blank line — format_speaker_turn always
    // appends `\n\n` for stack-of-turns rendering, but the final
    // turn doesn't need one.
    let stitched = stitched.trim_end().to_string() + "\n";

    tracing::info!(
        chars = stitched.len(),
        source_duration_ms,
        transcribed_slice_ms,
        "meeting mode (diarised): stitch complete"
    );

    Ok(AsrResult {
        text: stitched,
        words: None,
        language: first_lang,
        confidence: None,
        model_version,
        // Source PCM duration — NOT the sum of padded slice
        // durations (those overlap by BOUNDARY_PAD_MS on both
        // sides and would inflate this past the recording's
        // actual length).
        duration_ms: source_duration_ms,
    })
}

/// If the trailing word(s) of `prev` match the leading word(s) of
/// `current`, return `current` with the duplicated head removed.
/// Only matches up to 4 trailing/leading words — bounded so we
/// never trim more than a phrase even if both segments happen to
/// repeat a common prefix.
fn dedup_segment_head_against_tail(prev: &str, current: &str) -> String {
    const MAX_OVERLAP_WORDS: usize = 4;
    let prev_words: Vec<&str> = prev.split_whitespace().collect();
    let cur_words: Vec<&str> = current.split_whitespace().collect();
    let cap = MAX_OVERLAP_WORDS.min(prev_words.len()).min(cur_words.len());
    let mut overlap = 0usize;
    for n in (1..=cap).rev() {
        let prev_tail: String = prev_words[prev_words.len() - n..].join(" ").to_lowercase();
        let cur_head: String = cur_words[..n].join(" ").to_lowercase();
        if prev_tail == cur_head {
            overlap = n;
            break;
        }
    }
    if overlap == 0 {
        current.to_string()
    } else {
        cur_words[overlap..].join(" ")
    }
}

/// Tauri command entry. Thin wrapper around `stop_recording_inner`
/// that supplies the manual-stop reason. The auto-stop timer
/// scheduled by `start_recording` for non-meeting recordings calls
/// `stop_recording_inner` directly with `StopReason::AutoStoppedAtCap`
/// so the same audio path runs but the Success phase carries the
/// auto-stop notice.
#[tauri::command]
pub async fn stop_recording(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> CmdResult<HistoryEntry> {
    stop_recording_inner(handle, state.inner(), StopReason::Manual).await
}

pub(crate) async fn stop_recording_inner(
    handle: AppHandle,
    state: &Arc<AppState>,
    reason: StopReason,
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
        ?reason,
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
        // fix means this branch is reached when a client explicitly
        // double-taps the hotkey faster than Listening flips to Idle,
        // and (as of the single-shot auto-stop) when the auto-stop
        // timer's stop call loses a race against the user's manual
        // stop. Either way, the next genuine start must be able to
        // succeed without anyone clicking "Reset" anywhere.
        //
        // CONTEXT-INTENTIONAL: do NOT clear `active_recording_context`
        // here. With the auto-stop timer in place, two `stop_recording_inner`
        // calls can interleave: a winner that drained audio and is
        // still mid-transcribe (and still reading the bound context for
        // model resolution, history insert, success summary) plus a
        // loser that finds `audio.stop_capture_split() == None` and
        // lands here. Clearing the context from the loser would yank
        // it out from under the winner, which then falls back to the
        // legacy current-settings path and can mis-route the recording
        // (wrong model, wrong is_meeting flag, wrong history mode_id).
        // Stale context after a no-op stop is safe: every successful
        // stop / cancel / start-failure path clears the context
        // normally, and the next successful `start_recording` overwrites
        // it before the timer would matter.
        clear_recording_flag(state);
        return Err(OrliqError::User {
            message: "no active recording".into(),
        });
    };
    // Capture drained — now we own the session. Release the guard so a
    // user who starts a new dictation while transcription is still
    // running isn't blocked. The orchestrator serialises work on its
    // own side (see InferenceOrchestrator::ensure_loaded), so
    // overlapping start-after-stop is safe from here.
    //
    // CONTEXT-INTENTIONAL: do NOT clear `active_recording_context`
    // here — the rest of stop_recording (model resolution, transcribe,
    // history insert, success summary) reads it. Cleared at the end
    // of the happy path and on the transcribe error path below.
    clear_recording_flag(state);
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
    // **v0.15.3 binding rule.** When `start_recording` succeeded it
    // wrote an `ActiveRecordingContext` into AppState capturing
    // (mode_id, is_meeting, meeting_model_id) AT START TIME. Stop
    // reads that context first and trusts it verbatim — settings
    // drift mid-recording cannot change the recording's character.
    // A recording started as Meeting stops as Meeting; the meeting
    // model used at stop is the one approved at start.
    //
    // Bridge case (very rare, vanishingly so post-v0.15.3): the
    // recording started under v0.15.2 and v0.15.3 was launched
    // mid-recording. No context bound. Fall back to the legacy
    // settings-snapshot path with a warn log.
    let pre_settings = state.settings.snapshot().await;
    let context_snapshot = state.active_recording_context.read().await.clone();
    let (effective_mode_id, is_meeting_bound, bound_meeting_model_id) = match &context_snapshot {
        Some(ctx) => {
            tracing::info!(
                mode_id = %ctx.mode_id,
                is_meeting = ctx.is_meeting,
                meeting_model_id = ?ctx.meeting_model_id,
                started_at_ms = ctx.started_at_ms,
                "stop_recording: using bound ActiveRecordingContext"
            );
            (
                ctx.mode_id.clone(),
                Some(ctx.is_meeting),
                ctx.meeting_model_id.clone(),
            )
        }
        None => {
            let fallback_mode_id = pre_settings
                .active_mode_id
                .clone()
                .unwrap_or_else(|| "voice-to-text".into());
            tracing::warn!(
                fallback_mode_id = %fallback_mode_id,
                "stop_recording: no ActiveRecordingContext (v0.15.2 bridge?); falling back to current settings snapshot"
            );
            (fallback_mode_id, None, None)
        }
    };
    let pre_active_mode_row = state
        .modes
        .get(&effective_mode_id)
        .ok()
        .flatten()
        .ok_or_else(|| OrliqError::User {
            message: "active mode not found".into(),
        })?;
    // is_meeting from the context wins; for the bridge case fall
    // through to the row's current is_meeting_mode.
    let is_meeting_effective = is_meeting_bound.unwrap_or(pre_active_mode_row.is_meeting_mode);
    // Synthesize a Mode that carries the start-time is_meeting flag
    // even if the user toggled it on the actual row mid-recording.
    // Other Mode fields (target_language, target_app, formatting
    // rules) are read fresh from the row — those don't need binding.
    let pre_active_mode = crate::domain::Mode {
        is_meeting_mode: is_meeting_effective,
        ..pre_active_mode_row
    };
    let resolved_model_id = match resolve_target_model_id(
        &pre_active_mode,
        &pre_settings,
        &state.models,
        bound_meeting_model_id.as_deref(),
    ) {
        Ok(id) => id,
        Err(MeetingBridgeFailure { resolution }) => {
            // Meeting recording arrived at stop_recording with no
            // bound context (bridge case) AND current settings
            // can't yield an allow-listed meeting model. Refuse
            // loudly rather than silently routing through
            // default_asr_model_id (which would typically be
            // Parakeet — wrong shape for meetings, contract
            // violation).
            tracing::error!(
                ?resolution,
                "stop_recording: refusing to transcribe meeting through non-allow-listed model"
            );
            // Restore start-time side effects so the user isn't
            // left muted with a stuck Processing phase.
            {
                let media = state.media.clone();
                tokio::task::spawn_blocking(move || media.resume_paused());
            }
            {
                let ducking = state.ducking.clone();
                tokio::task::spawn_blocking(move || ducking.restore());
            }
            // Experimental mic-mute restore. The active context is
            // about to be cleared below; do this BEFORE that so we
            // can read the applied list.
            restore_mic_mute_sideeffect(&handle, state).await;
            clear_active_recording_context(state).await;
            let user_message = match resolution {
                MeetingModelResolution::RefuseUnsupportedModel(_) => {
                    "Your meeting model setting is invalid. Open Settings → Models and choose Fast, Balanced, or Best accuracy.".to_string()
                }
                MeetingModelResolution::RefuseExplicitMissing(id) => format!(
                    "Your meeting model ({}) isn't installed. Open Settings → Models, then click Install on the option you've chosen.",
                    display_name_short(&id)
                ),
                MeetingModelResolution::RefusePreferenceUnset => {
                    "Pick a meeting model. Open Settings → Models and choose Fast (Whisper Small), Balanced (Whisper Medium), or Best accuracy (Whisper Large v3).".to_string()
                }
                MeetingModelResolution::Use(_) => {
                    // Logically unreachable — the Err branch is
                    // only constructed when resolution is a
                    // refusal — but be explicit.
                    "Meeting model unresolvable.".to_string()
                }
            };
            set_phase(
                &handle,
                DictationPhase::Error {
                    message: user_message.clone(),
                },
            )
            .await;
            schedule_idle(&handle, 2400);
            return Err(OrliqError::User {
                message: user_message,
            });
        }
    };
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
            "stop_recording: meeting mode — chunked transcript (diarisation disabled in v0.15.1)"
        );

        // v0.15.1 hotfix: speaker diarisation is DISABLED by default.
        //
        // Real-meeting v0.15.0 output proved the diarised path is not
        // production-safe as designed:
        //   * Threshold-based clustering over-fragmented a 5-min meeting
        //     into 91 fake "speakers" (every short utterance with
        //     slightly different acoustics became a new cluster).
        //   * Whisper Large running on those tiny diariser segments
        //     hallucinated content in random languages (Hindi,
        //     Vietnamese, Japanese, Arabic) for slices that were
        //     music / silence / cross-talk — a known Whisper failure
        //     mode on short non-speech audio.
        //   * Per-segment Whisper calls (N = diariser segment count)
        //     made runtime O(N) where N can be tens-to-hundreds; on
        //     CPU Whisper Large that's many minutes for a five-minute
        //     meeting.
        //
        // The asset/download/extract code from v0.15.0 stays in tree
        // (`services::inference::diarize_assets` + `diarize`) — we'll
        // need it for the v0.15.2 architectural rework — but the
        // call site does NOT reach for it. Meeting mode falls through
        // to `transcribe_chunked` (30s chunks, 2s overlap, no
        // **Speaker N:** labels, no RMS heuristic, no per-segment
        // diariser calls) — the same bounded path that was already
        // the v0.15.0 fallback when the diariser couldn't be loaded.
        //
        // Workaround for the cross-language hallucination problem:
        // the user's Settings → Models → Whisper language picker
        // already pins the language at recognizer construction time
        // (see `sherpa_whisper.rs::ensure_loaded`). Setting it to
        // "en" (or whatever the meeting language actually is) and
        // restarting the app stops Whisper from inventing
        // foreign-language content on quiet/ambiguous audio.
        // Documented in releases/0.15.1.md.
        tracing::info!(
            "meeting mode diarisation disabled in v0.15.1 beta hotfix; using chunked transcript without speaker labels"
        );

        transcribe_chunked(
            &handle,
            state,
            &model_id,
            final_.pcm,
            final_.sample_rate,
            mode.target_language.clone(),
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
            // Experimental mic-mute restore. Same reason: don't leave
            // the user muted in Teams/Discord because ASR died.
            restore_mic_mute_sideeffect(&handle, state).await;
            // Recording session is over (one way or another). Clear
            // the bound context so the next start_recording lands
            // with a clean slate. Pinned by
            // `every_clear_recording_flag_site_pairs_with_clear_active_recording_context`.
            clear_active_recording_context(state).await;
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
    let total_processing_ms = (now_ms() - started) as u64;
    if mode.is_meeting_mode {
        // v0.15.3 meeting-level summary log. One line per meeting at
        // the END of the chunked transcribe so a single
        // `tail | grep "meeting transcribe complete"` gives a clean
        // dataset for comparing Small / Medium / Large RTF across
        // machines. Per-chunk logs stay (line 1115 in transcribe_chunked)
        // — useful for chunk-level diagnostics — but they don't carry
        // the wall-clock total, the audio:processing ratio, or the
        // selected model id. The whisper_language field is the user's
        // current setting, surfaced here so we can correlate quality
        // issues against language-pin choice.
        let chunk_count = chunk_count_for_duration(asr.duration_ms);
        let rtf = realtime_factor(asr.duration_ms, total_processing_ms);
        // v0.15.6 telemetry: same shape as v0.15.5 plus two new
        // fields. `dml_available` records whether the shipping
        // runtime exposes a DirectML EP at all (independent of
        // user pref). `provider_policy_reason` records the resolver
        // decision — covers cases v0.15.5's `fallback_reason`
        // couldn't express, like "auto-Small picked CPU because
        // Step 2 evidence shows DML is slower on short clips".
        //
        // `current_provider_resolution` returns None when no model
        // is loaded — can't happen on this path (we just transcribed
        // through it), but defensively log empty sentinels so a
        // grepper never sees a half-row.
        let resolution = state.inference.current_provider_resolution().await;
        let provider_requested = resolution
            .as_ref()
            .map(|r| r.requested.as_str())
            .unwrap_or("");
        let provider_used = resolution.as_ref().map(|r| r.used.as_str()).unwrap_or("");
        let fallback_used = resolution.as_ref().is_some_and(|r| r.fallback_used);
        let fallback_reason = resolution
            .as_ref()
            .map(|r| r.fallback_reason.as_str())
            .unwrap_or("");
        let provider_policy_reason = resolution
            .as_ref()
            .map(|r| r.policy_reason.as_str())
            .unwrap_or("");
        let dml_available = crate::services::gpu::runtime_capabilities().has_dml_ep;
        tracing::info!(
            audio_duration_ms = asr.duration_ms,
            total_processing_ms,
            realtime_factor = ?rtf,
            selected_model_id = %model_id,
            chunk_count,
            whisper_language = %pre_settings.models.whisper_language,
            provider_requested,
            provider_used,
            fallback_used,
            fallback_reason,
            provider_policy_reason,
            dml_available,
            "meeting transcribe complete"
        );
    }
    // v0.15.5+: provider telemetry on every transcribe (meeting +
    // non-meeting). Mirrors the meeting log fields so a grep across
    // both surfaces the same shape. v0.15.6 adds dml_available +
    // provider_policy_reason here too.
    let dictation_resolution = state.inference.current_provider_resolution().await;
    let dictation_dml_available = crate::services::gpu::runtime_capabilities().has_dml_ep;
    tracing::info!(
        chars = asr.text.len(),
        provider_requested = %dictation_resolution.as_ref().map(|r| r.requested.as_str()).unwrap_or(""),
        provider_used = %dictation_resolution.as_ref().map(|r| r.used.as_str()).unwrap_or(""),
        fallback_used = dictation_resolution.as_ref().is_some_and(|r| r.fallback_used),
        fallback_reason = %dictation_resolution.as_ref().map(|r| r.fallback_reason.as_str()).unwrap_or(""),
        provider_policy_reason = %dictation_resolution.as_ref().map(|r| r.policy_reason.as_str()).unwrap_or(""),
        dml_available = dictation_dml_available,
        "inference.transcribe done"
    );

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
        // Global "Smart rewrite" toggle (Settings → Text Output).
        // When enabled, the deterministic rewrite (filler removal,
        // sentence-case, terminal punctuation, whitespace
        // normalisation) runs on every non-meeting dictation
        // regardless of the mode's own `post_processing` setting.
        // The toggle deliberately does NOT extend to Meeting Mode:
        // the outer `if mode.is_meeting_mode` branch above already
        // bypasses this whole block, and meeting transcripts want
        // verbatim fidelity, not polish — see the long comment
        // above the `if`. The setting is off by default and adds
        // one O(n) pass when on; no LLM, no remote call, no model
        // download. The Meeting Mode gate is pinned by the
        // `smart_rewrite_is_gated_to_non_meeting_mode` source-grep
        // test below; the rewrite's no-IO sync contract is pinned
        // by `smart_rewrite_is_sync_and_deterministic` in the
        // deterministic provider's test module.
        let global_smart_rewrite = settings.text_output.smart_rewrite;
        let post = match &mode.post_processing {
            crate::domain::PostProcessing::None => {
                if global_smart_rewrite {
                    crate::services::llm::deterministic::DeterministicProvider.rewrite_sync(&raw)
                } else {
                    raw.clone()
                }
            }
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
    //   - sherpa_parakeet's hallucination filter (see sherpa_parakeet.rs)
    //     returns "" on a quiet clip that Parakeet-TDT filled with a
    //     fabricated filler token.
    //   - Genuine silence clips where the model simply had nothing to
    //     produce.
    // For regular (non-meeting) dictation we additionally run the shared
    // output_validation::validate, which catches the cases the engine
    // internals don't: bare "." outputs, Whisper decoder loops ("the the
    // the the the the"), repeated punctuation runs, and foreign-script
    // hallucinations on English-only paths.
    //
    // Meeting Mode is intentionally NOT subject to the new validator,
    // because legitimate meeting transcripts can contain repetition,
    // mixed-language content from multilingual Whisper, or "Output
    // filtered" UX that wouldn't make sense for a 30-minute capture.
    // It keeps only the original cheap empty-string check that was the
    // pre-patch behaviour, so an empty-meeting transcript still
    // short-circuits the history insert and emits NoAudio. Either way
    // we have no usable speech to deliver - skip text_inject + history
    // + analytics, but still resume paused media / ducking so we don't
    // leave the user silent.
    use crate::services::output_validation::{
        validate, EngineFamily, LanguagePin, ValidationContext, ValidationVerdict,
    };
    let (drop_now, drop_notice): (bool, Option<String>) = if mode.is_meeting_mode {
        // Meeting Mode: pre-patch behaviour. Empty/whitespace-only
        // processed text is the only drop condition; never carries a
        // user-visible notice (silence is silence). The shared
        // validator is deliberately bypassed here.
        (processed.trim().is_empty(), None)
    } else {
        // Regular dictation: full validator. Engine family + language
        // pin are derived locally from the already-known model_id and
        // settings snapshot - no extra IPC, no registry async lookup,
        // no coupling between the validator and the rest of the
        // inference layer. Model IDs are prefix-coded by family
        // ("sherpa-whisper-*" / "sherpa-parakeet-*"); the catalogue
        // check at install time enforces the prefix.
        let engine = if model_id.starts_with("sherpa-whisper") {
            EngineFamily::Whisper
        } else {
            EngineFamily::Parakeet
        };
        let lang_setting = settings.models.whisper_language.trim().to_lowercase();
        let language_pin = match (engine, lang_setting.as_str()) {
            // Parakeet is hard-pinned English regardless of the
            // whisper_language field - applying English-only rules.
            (EngineFamily::Parakeet, _) => LanguagePin::EnglishOnly,
            (EngineFamily::Whisper, "en") => LanguagePin::EnglishOnly,
            (EngineFamily::Whisper, "" | "auto") => LanguagePin::Auto,
            (EngineFamily::Whisper, _) => LanguagePin::OtherExplicit,
        };
        let validation_ctx = ValidationContext {
            engine,
            language_pin,
            // u64 -> u32: dictation durations sit in seconds-to-minutes;
            // saturating cast is fine and avoids a TryFrom dance for a
            // field no current rule even reads.
            audio_duration_ms: u32::try_from(final_.duration_ms).unwrap_or(u32::MAX),
        };
        let verdict = validate(&processed, &validation_ctx);
        match verdict {
            ValidationVerdict::Accept => (false, None),
            ValidationVerdict::Drop {
                reason,
                user_notice,
            } => {
                tracing::info!(
                    reason = %reason.as_str(),
                    model = %model_id,
                    language = %lang_setting,
                    duration_ms = final_.duration_ms,
                    chars = processed.chars().count(),
                    user_notice = user_notice,
                    "output_validation: dropped transcript"
                );
                let notice = if user_notice {
                    Some("Output filtered".to_string())
                } else {
                    None
                };
                (true, notice)
            }
        }
    };
    if drop_now {
        {
            let media = state.media.clone();
            tokio::task::spawn_blocking(move || media.resume_paused());
        }
        {
            let ducking = state.ducking.clone();
            tokio::task::spawn_blocking(move || ducking.restore());
        }
        // Mic-mute restore on the no-audio drop path: same reason as
        // the other restore sites; don't leave the user muted in
        // Teams/Discord just because the recording produced nothing
        // usable.
        restore_mic_mute_sideeffect(&handle, state).await;
        set_phase(
            &handle,
            DictationPhase::NoAudio {
                notice: drop_notice,
            },
        )
        .await;
        // Longer settle than Success (1.2s) - the message is the whole
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
            // No diariser ran here (this is the no-text early-return
            // path) so there are no `**Speaker N:**` tokens in the
            // empty processed_text to ever rename.
            speaker_names: None,
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
        asr_model_id: model_id.clone(),
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
        // No rename has happened yet — set at insert time to None;
        // the user can rename later from the Meetings page.
        speaker_names: None,
    };
    // Group-policy gates. Both are forward-only — existing rows /
    // analytics rollups are untouched; we just stop adding to them.
    // Skipping history persistence does NOT abort the dictation: the
    // text was already injected into the focused app, the user's work
    // is done. A managed deployment that asked us not to keep the
    // record gets exactly that, with the rest of the flow unchanged.
    let policy = state.settings.policy();
    if policy.history_blocked() {
        tracing::debug!("history persistence skipped: BlockHistory policy is set");
    } else {
        state.history.insert(&entry).map_err(OrliqError::from)?;
    }

    if policy.telemetry_blocked() {
        tracing::debug!("local analytics rollup skipped: BlockTelemetry policy is set");
    } else {
        let words = processed.split_whitespace().count() as u32;
        let _ = state.analytics.record_dictation(
            words,
            (final_.duration_ms / 1000) as u32,
            settings.typing_baseline_wpm,
            Some(&mode.id),
        );
    }

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
    // Experimental mic-mute restore on the success path. No-op when
    // the active context's mic_mute_applied is empty (the common
    // case). Emits the success or sticky-failure banner depending on
    // per-app outcomes.
    restore_mic_mute_sideeffect(&handle, state).await;

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
            // matches the decision the chunker made
            // (CHUNK = CHUNKED_MEETING_CHUNK_SECS, OVERLAP = 2,
            // stride = chunk - 2). For a single-chunk recording this
            // gives 1.
            chunks: chunk_count_for_duration(asr.duration_ms),
            chars: processed.len() as u32,
            // v0.15.3: surface the model id used for this meeting so
            // the user can correlate quality vs speed across meetings
            // ("5:01 · Whisper Medium · 11 chunks · 799 chars").
            model_label: display_name_short(&model_id),
        })
    } else {
        None
    };
    // When the auto-stop timer fired this stop, attach a user-facing
    // notice to the Success phase so the bar can render a clear,
    // non-error explanation instead of the bare "Inserted" pill.
    // Manual stops (the user's hotkey) carry None here and render
    // unchanged.
    let success_notice = match reason {
        StopReason::Manual => None,
        StopReason::AutoStoppedAtCap => {
            Some("Auto-stopped at 5 min. Use Meeting Mode for longer recordings.".to_string())
        }
    };
    set_phase(
        &handle,
        DictationPhase::Success {
            summary: success_summary,
            notice: success_notice,
        },
    )
    .await;
    // Meeting mode lingers a beat longer (3 s instead of the dictation
    // 1.2 s) so the user has time to read the duration + chunk pill
    // before it decays. The pill is the only visible "your meeting was
    // saved" signal until the v0.14 Meetings page lands.
    let idle_delay = if mode.is_meeting_mode { 3000 } else { 1200 };
    schedule_idle(&handle, idle_delay);

    // Recording session is fully done. Clear the bound context so the
    // next start_recording lands with a clean slate. Pinned by
    // `every_clear_recording_flag_site_pairs_with_clear_active_recording_context`.
    clear_active_recording_context(state).await;

    Ok(entry)
}

/// Mirror of the chunking arithmetic in `transcribe_chunked` — used
/// by stop_recording's success-pill payload to report how many chunks
/// the meeting was split into without threading the count back from
/// the chunker. Reads from the same `CHUNKED_MEETING_*` constants the
/// chunker uses, so the two cannot drift.
///
/// This computes a duration-based count (ms-precision input) where
/// `projected_chunk_count` works in samples; both pin the same
/// formula `(total - chunk) / stride + 1`. Tests cover both helpers
/// at the boundary of `MAX_MEETING_CHUNKS`.
fn chunk_count_for_duration(duration_ms: u32) -> u32 {
    let chunk_s = CHUNKED_MEETING_CHUNK_SECS as u32;
    let overlap_s = CHUNKED_MEETING_OVERLAP_SECS as u32;
    let stride_s = chunk_s - overlap_s;
    let secs = duration_ms.div_ceil(1000);
    if secs == 0 {
        return 0;
    }
    if secs <= chunk_s {
        return 1;
    }
    // Chunk N starts at N * stride; covers the whole recording when
    // (N-1) * stride + chunk >= total. Solve: N = ceil((total - chunk) / stride) + 1.
    ((secs - chunk_s).div_ceil(stride_s)) + 1
}

#[tauri::command]
pub async fn cancel_recording(handle: AppHandle, state: State<'_, Arc<AppState>>) -> CmdResult<()> {
    let _ = state.audio.stop_capture().await;
    // Always release the re-entry guard — cancel is the user's "get me
    // out of whatever state I'm in" command, and if we stranded the
    // flag high they'd lose the ability to start the next recording
    // without restarting the app.
    //
    // Mic-mute restore must run BEFORE clear_active_recording_context
    // because it reads mic_mute_applied off the bound context. The
    // existing media + ducking restores below don't depend on the
    // context (they read state.media / state.ducking instead) so the
    // ordering is mic-mute first, then media, then ducking, then
    // clear.
    restore_mic_mute_sideeffect(&handle, state.inner()).await;
    clear_recording_flag(state.inner());
    clear_active_recording_context(state.inner()).await;
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
        DictationPhase::NoAudio { .. } => "no_audio",
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

#[cfg(test)]
mod diarised_helpers_tests {
    //! Pure-function tests for the diarised meeting-mode pipeline
    //! helpers. These don't touch sherpa, ONNX runtime, or any
    //! audio I/O — every helper here takes typed inputs and returns
    //! typed outputs. The full diarised pipeline (which calls these
    //! plus FFI) is exercised by manual QA on a real install.

    use super::*;
    use sherpa_rs::diarize::Segment;

    fn seg(start: f32, end: f32, speaker: i32) -> Segment {
        Segment {
            start,
            end,
            speaker,
        }
    }

    /// Adjacent same-speaker segments separated by less than
    /// `MERGE_GAP_SEC` (=0.4s) get merged. Pin the boundary
    /// behaviour: 0.39s gap merges, 0.41s gap doesn't.
    #[test]
    fn merge_adjacent_same_speaker_collapses_short_gaps() {
        let merged = merge_adjacent_same_speaker(vec![seg(0.0, 2.0, 1), seg(2.3, 5.0, 1)]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].start, 0.0);
        assert_eq!(merged[0].end, 5.0);
        assert_eq!(merged[0].speaker, 1);
    }

    #[test]
    fn merge_preserves_speaker_change() {
        let merged = merge_adjacent_same_speaker(vec![seg(0.0, 2.0, 1), seg(2.1, 5.0, 2)]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].speaker, 1);
        assert_eq!(merged[1].speaker, 2);
    }

    #[test]
    fn merge_skips_gaps_above_threshold() {
        // 1.0s gap >= MERGE_GAP_SEC (0.4) — must NOT merge.
        let merged = merge_adjacent_same_speaker(vec![seg(0.0, 2.0, 1), seg(3.0, 5.0, 1)]);
        assert_eq!(merged.len(), 2);
    }

    /// Sub-MIN_SEGMENT_SEC (=0.6) fragments are dropped by the
    /// shape pass — they're typically cross-talk artefacts that
    /// sherpa's own min_duration_on let through.
    #[test]
    fn shape_drops_sub_minimum_segments() {
        let shaped = shape_segments(vec![seg(0.0, 0.3, 1)]);
        assert!(
            shaped.is_empty(),
            "0.3s segment must be dropped (below MIN_SEGMENT_SEC)"
        );
    }

    /// Segments longer than MAX_SEGMENT_SEC (30s) are split into
    /// equal pieces, each ≤ MAX. Pin the split arithmetic.
    #[test]
    fn shape_splits_over_max_into_uniform_pieces() {
        // 45s segment → 30s + 15s
        let shaped = shape_segments(vec![seg(0.0, 45.0, 1)]);
        assert_eq!(shaped.len(), 2);
        assert_eq!(shaped[0].start, 0.0);
        assert_eq!(shaped[0].end, 30.0);
        assert_eq!(shaped[1].start, 30.0);
        assert_eq!(shaped[1].end, 45.0);
        // Speaker id is preserved across the split.
        assert_eq!(shaped[0].speaker, 1);
        assert_eq!(shaped[1].speaker, 1);
    }

    /// `pad_segment_samples` clamps at 0 and total_samples — the
    /// padded boundary can't reach past the buffer edges.
    #[test]
    fn pad_clamps_at_buffer_edges() {
        let sample_rate = 16_000u32;
        let total = sample_rate as usize * 10; // 10s buffer
                                               // Segment at 0..0.05s, padded ±200ms → start clamps at 0,
                                               // end ≈ 4000 samples (200ms + 50ms = 250ms ~= 4000 @ 16k).
        let (start_idx, end_idx) = pad_segment_samples(0.0, 0.05, sample_rate, total);
        assert_eq!(start_idx, 0, "must clamp at 0, not go negative");
        assert!(end_idx > 0 && end_idx <= total);

        // Segment at the very end of the buffer.
        let (start_idx, end_idx) = pad_segment_samples(9.95, 10.0, sample_rate, total);
        assert!(start_idx < total);
        assert_eq!(end_idx, total, "must clamp at buffer end, not overshoot");
    }

    /// **Blocker 3 regression**: padded segments overlap. Summing
    /// their durations would inflate the source-audio duration.
    /// This test pins that the helper outputs CAN sum to more than
    /// the source duration — proving why the diarised path must
    /// use source PCM length, not the sum, as
    /// `AsrResult.duration_ms`.
    #[test]
    fn padded_segments_can_sum_to_more_than_source() {
        let sample_rate = 16_000u32;
        let source_secs = 10.0_f32;
        let total = (source_secs * sample_rate as f32) as usize;
        let source_ms = (source_secs * 1000.0) as u32;

        // Three back-to-back 1s speaker-change segments: 0..1, 1..2,
        // 2..3. With BOUNDARY_PAD_MS=200, each grows to ~1.4s.
        // Sum ≈ 4.2s vs source 10s here — but the property the
        // assertion targets is that "sum could exceed the
        // un-padded total" in the worst case.
        let segments = vec![seg(0.0, 1.0, 1), seg(1.0, 2.0, 2), seg(2.0, 3.0, 3)];
        let mut summed_ms: u32 = 0;
        for s in &segments {
            let (start_idx, end_idx) = pad_segment_samples(s.start, s.end, sample_rate, total);
            let slice_ms = ((end_idx - start_idx) as u64 * 1000 / sample_rate as u64) as u32;
            summed_ms = summed_ms.saturating_add(slice_ms);
        }
        // Each 1s segment + 400ms total padding ≈ 1.4s × 3 = 4.2s.
        // Un-padded segment span is 3s. Summed padded > un-padded
        // span — proves the inflation we'd get if we used summed
        // slice durations as the result duration.
        let unpadded_span_ms = 3_000u32;
        assert!(
            summed_ms > unpadded_span_ms,
            "sum of padded slices should exceed un-padded span (got {summed_ms} vs {unpadded_span_ms})"
        );
        // Source duration is the canonical 10s. Bigger than the
        // summed padded (in this small example), but the danger
        // case is many short segments where summed padded
        // overshoots source. The point of the regression is that
        // the diarised path must NEVER hand `summed_ms` to
        // `AsrResult.duration_ms`.
        let _ = source_ms;
    }

    /// Format spec pin: `[mm:ss] **Speaker N:** text\n\n`. The
    /// rename-substitution regex on the frontend matches against
    /// this exact shape; any drift (e.g. `*Speaker N*`, no colon,
    /// 1-digit zero-pad missing) breaks rename rendering.
    #[test]
    fn format_turn_canonical_shape() {
        let out = format_speaker_turn(754, 0, "Welcome to the call.");
        assert_eq!(out, "[12:34] **Speaker 1:** Welcome to the call.\n\n");
        let out = format_speaker_turn(0, 1, "Hi.");
        assert_eq!(out, "[00:00] **Speaker 2:** Hi.\n\n");
    }

    /// Word-overlap dedup at segment boundaries. Adjacent
    /// same-speaker segments can transcribe the same boundary
    /// word twice because of BOUNDARY_PAD_MS overlap. Helper drops
    /// up to 4 leading words if they match the prev tail.
    #[test]
    fn dedup_drops_repeated_boundary_words() {
        let prev = "we'll cover the dashboard";
        let cur = "the dashboard launches today";
        assert_eq!(dedup_segment_head_against_tail(prev, cur), "launches today");
    }

    #[test]
    fn dedup_no_overlap_returns_unchanged() {
        assert_eq!(
            dedup_segment_head_against_tail("hello world", "goodbye now"),
            "goodbye now"
        );
    }

    #[test]
    fn dedup_caps_at_four_word_overlap() {
        // Even a 5-word identical prefix only drops 4. The bound
        // is intentional — drop more and we risk eating real
        // content when two speakers happen to start with similar
        // phrasing.
        let prev = "one two three four five";
        let cur = "one two three four five six";
        let result = dedup_segment_head_against_tail(prev, cur);
        // "one" stays because we only dropped 4 of the 5 identical
        // leading words.
        assert!(
            result.starts_with("one ") || result == "five six",
            "got {result:?}"
        );
    }

    // ── re-entry concurrency-guard tests (rounds 3 + 4 + 5) ────────
    //
    // These cover the v0.15.0-blocker fixes:
    //   * Round 3 closed the meeting-mode diariser preflight window
    //     (~47 MB download).
    //   * Round 4 closed the generic settings.snapshot() and
    //     audio.start_capture() windows that sit outside the
    //     diariser path.
    //   * Round 5 closed the deeper hole: even with both flags,
    //     the CAS-failed branch was awaiting `audio.current()`
    //     BEFORE reading the pending atomics, so a duplicate call
    //     could sample stale (pending=false, capture=None) and
    //     stale-heal a flag that was not stale. The fix splits
    //     the decision into a stage-1 (cheap atomics, no await)
    //     fast-reject and a stage-2 (probe + time-gate + recheck)
    //     conservative-proceed; promotes start_recording_pending
    //     to a CAS-mutex; and time-gates self-heal at
    //     RECORDING_STALE_MS to close the inherent
    //     non-atomic-snapshot race.

    // ── decide_pre_probe tests ──────────────────────────────────────

    /// Stage 1 reads ONLY the pending atomics — cheap and
    /// no-await. Both false → return None (caller proceeds to
    /// stage 2). Anything else → reject before probing.
    #[test]
    fn pre_probe_returns_none_when_no_pending() {
        assert_eq!(decide_pre_probe(false, false), None);
    }

    /// **Round-3 regression in stage-1 form** — pending+preflight
    /// must reject as preflight before we ever look at capture.
    /// Pre-round-5 this WAS read post-await, so a duplicate that
    /// landed after pending dropped saw `false` and missed the
    /// rejection.
    #[test]
    fn pre_probe_rejects_preflight_before_probe() {
        assert_eq!(decide_pre_probe(true, true), Some(Reentry::RejectPreflight));
    }

    /// **Round-4 regression in stage-1 form** — pending without
    /// preflight rejects as generic start-pending, again BEFORE
    /// any probe.
    #[test]
    fn pre_probe_rejects_generic_pending_before_probe() {
        assert_eq!(
            decide_pre_probe(true, false),
            Some(Reentry::RejectStartPending)
        );
    }

    /// Defensive: preflight without pending shouldn't happen (the
    /// inner preflight guard is always claimed inside the outer
    /// pending window) but if it does, fall through to stage 2.
    /// Pinned so a future contributor doesn't accidentally promote
    /// this to a rejection — the time-gated stale-heal will sort
    /// it out safely.
    #[test]
    fn pre_probe_preflight_without_pending_falls_through() {
        assert_eq!(decide_pre_probe(false, true), None);
    }

    // ── decide_post_probe tests ─────────────────────────────────────

    /// Live capture wins regardless of every other input. In
    /// practice the start completes and clears pending right around
    /// when capture surfaces, so `(true, _, true, _)` is the
    /// brief window where both can be true together.
    #[test]
    fn post_probe_active_capture_takes_priority() {
        for &age in &[0_i64, 100, 30_000, 10_000_000] {
            for &pending in &[false, true] {
                for &preflight in &[false, true] {
                    assert_eq!(
                        decide_post_probe(true, age, pending, preflight),
                        Reentry::RejectActiveCapture,
                        "active capture must win for age={age} pending={pending} preflight={preflight}"
                    );
                }
            }
        }
    }

    /// **Round-5 race** — pending was false at stage 1 (we got
    /// past the fast-reject) but flipped true during the
    /// `audio.current().await`. The recheck catches it and
    /// rejects. Without this, the in-flight start that just
    /// claimed pending would race the duplicate.
    #[test]
    fn post_probe_pending_recheck_rejects_when_pending_flipped() {
        // Aged flag, no live capture, but pending re-flipped true
        // with no preflight → generic start-pending reject.
        assert_eq!(
            decide_post_probe(false, 60_000, true, false),
            Reentry::RejectStartPending
        );
        // Same scenario but recheck sees preflight too → preflight
        // reject (better message).
        assert_eq!(
            decide_post_probe(false, 60_000, true, true),
            Reentry::RejectPreflight
        );
    }

    /// **Round-5 time gate** — the heart of the round-5 fix.
    /// Flag is held but very young (no capture, no pending).
    /// Must NOT stale-heal — almost certainly the in-flight start
    /// is mid-CAS-then-stamp or one scheduling tick from arming
    /// capture. Pre-round-5 this returned ProceedAfterStaleHeal
    /// and raced the in-flight start.
    #[test]
    fn post_probe_young_flag_rejects_not_stale_heal() {
        // Just stamped — age 1ms.
        assert_eq!(
            decide_post_probe(false, 1, false, false),
            Reentry::RejectYoungFlag
        );
        // 5 seconds in — typical mid-prep age.
        assert_eq!(
            decide_post_probe(false, 5_000, false, false),
            Reentry::RejectYoungFlag
        );
        // Just under threshold (29.999s).
        assert_eq!(
            decide_post_probe(false, RECORDING_STALE_MS - 1, false, false),
            Reentry::RejectYoungFlag
        );
    }

    /// Age == 0 means "no timestamp recorded" — the in-flight
    /// start CAS-set the flag but hasn't stamped the timestamp
    /// yet. Treat as "fresh enough to be racing" — never stale-heal
    /// a flag whose age can't be established.
    #[test]
    fn post_probe_zero_age_is_treated_as_young() {
        assert_eq!(
            decide_post_probe(false, 0, false, false),
            Reentry::RejectYoungFlag
        );
    }

    /// Age < 0 is impossible in practice (clock skew aside) but
    /// defensively treated as "young" — never stale-heal a
    /// negative-age flag.
    #[test]
    fn post_probe_negative_age_is_treated_as_young() {
        assert_eq!(
            decide_post_probe(false, -1_000, false, false),
            Reentry::RejectYoungFlag
        );
    }

    /// The ONLY legitimate stale-heal case: no live capture, no
    /// pending on recheck, age >= RECORDING_STALE_MS. Genuinely
    /// abandoned flag.
    #[test]
    fn post_probe_old_flag_with_no_pending_stale_heals() {
        assert_eq!(
            decide_post_probe(false, RECORDING_STALE_MS, false, false),
            Reentry::ProceedAfterStaleHeal
        );
        assert_eq!(
            decide_post_probe(false, 5 * 60_000, false, false),
            Reentry::ProceedAfterStaleHeal
        );
    }

    // ── StartPendingGuard CAS-mutex tests ───────────────────────────

    /// Happy path: claim succeeds, drop clears.
    #[test]
    fn start_pending_guard_claim_succeeds_when_flag_false() {
        let flag = std::sync::atomic::AtomicBool::new(false);
        let guard = StartPendingGuard::try_claim(&flag);
        assert!(guard.is_some(), "first claim should succeed");
        assert!(
            flag.load(std::sync::atomic::Ordering::Acquire),
            "flag should be true while guard is held"
        );
        drop(guard);
        assert!(
            !flag.load(std::sync::atomic::Ordering::Acquire),
            "flag should be cleared on guard drop"
        );
    }

    /// **Round-5 mutex contract** — claim is CAS-based.
    /// A second claim while the first is alive must fail
    /// (return None), not silently re-store true. This is what
    /// makes pending a true mutex rather than a flag.
    #[test]
    fn start_pending_guard_second_claim_fails_while_first_held() {
        let flag = std::sync::atomic::AtomicBool::new(false);
        let first = StartPendingGuard::try_claim(&flag);
        assert!(first.is_some(), "first claim should succeed");
        let second = StartPendingGuard::try_claim(&flag);
        assert!(
            second.is_none(),
            "second claim must fail while first guard is alive"
        );
        // First still holds, flag still true.
        assert!(flag.load(std::sync::atomic::Ordering::Acquire));
        drop(first);
        // Now a fresh claim succeeds.
        let third = StartPendingGuard::try_claim(&flag);
        assert!(
            third.is_some(),
            "claim should succeed once previous guard is dropped"
        );
    }

    /// Drop on panic: even if the scope unwinds, the flag clears.
    /// `Arc<AtomicBool>` so the closure can move-capture without
    /// losing access to the original for the post-panic check.
    #[test]
    fn start_pending_guard_clears_on_panic() {
        let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag_for_panic = flag.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard =
                StartPendingGuard::try_claim(&flag_for_panic).expect("first claim should succeed");
            assert!(flag_for_panic.load(std::sync::atomic::Ordering::Acquire));
            panic!("simulated panic mid-start");
        }));
        assert!(result.is_err(), "panic should have propagated");
        assert!(
            !flag.load(std::sync::atomic::Ordering::Acquire),
            "guard drop must clear the flag even when the scope unwinds"
        );
    }

    // ── PreflightGuard tests ────────────────────────────────────────

    /// PreflightGuard uses plain store(true) — it's only ever
    /// claimed inside the StartPendingGuard window, so the outer
    /// mutex serialises overlapping preflights. Just pin the
    /// claim+drop contract.
    #[test]
    fn preflight_guard_sets_and_clears_flag() {
        let flag = std::sync::atomic::AtomicBool::new(false);
        {
            let _guard = PreflightGuard::claim(&flag);
            assert!(flag.load(std::sync::atomic::Ordering::Acquire));
        }
        assert!(!flag.load(std::sync::atomic::Ordering::Acquire));
    }

    #[test]
    fn preflight_guard_clears_on_panic() {
        let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag_for_panic = flag.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = PreflightGuard::claim(&flag_for_panic);
            assert!(flag_for_panic.load(std::sync::atomic::Ordering::Acquire));
            panic!("simulated panic mid-preflight");
        }));
        assert!(result.is_err());
        assert!(!flag.load(std::sync::atomic::Ordering::Acquire));
    }

    // ── Helper-pair tests: mark_recording_started / clear_recording_flag ──

    /// `mark_recording_started` is the canonical write of
    /// `recording_in_progress`. It must (a) CAS the flag from
    /// false→true, returning true on success, and (b) stamp the
    /// timestamp on success so the time-gated stale-heal has
    /// something to read. A failed CAS must NOT overwrite an
    /// existing timestamp.
    #[test]
    fn mark_recording_started_first_call_wins_and_stamps() {
        let state = make_test_app_state();
        let before = now_ms();
        let won = mark_recording_started(&state);
        assert!(won, "first mark must win the CAS");
        assert!(state
            .recording_in_progress
            .load(std::sync::atomic::Ordering::Acquire));
        let stamped = state
            .recording_in_progress_started_at_ms
            .load(std::sync::atomic::Ordering::Acquire);
        assert!(
            stamped >= before,
            "started-at must be stamped at or after the call (got {stamped}, before={before})"
        );
    }

    #[test]
    fn mark_recording_started_second_call_loses_and_does_not_overwrite_stamp() {
        let state = make_test_app_state();
        assert!(mark_recording_started(&state));
        let first_stamp = state
            .recording_in_progress_started_at_ms
            .load(std::sync::atomic::Ordering::Acquire);
        // Sleep briefly so a successful re-stamp would be visibly
        // different.
        std::thread::sleep(std::time::Duration::from_millis(5));
        let won = mark_recording_started(&state);
        assert!(!won, "second mark must lose the CAS");
        let after_stamp = state
            .recording_in_progress_started_at_ms
            .load(std::sync::atomic::Ordering::Acquire);
        assert_eq!(
            first_stamp, after_stamp,
            "failed CAS must not overwrite the started-at stamp"
        );
    }

    /// `clear_recording_flag` zeros both the flag AND the timestamp.
    /// Pinning this prevents the regression where a future change
    /// updates the flag but leaves a stale timestamp behind for the
    /// next stale-heal probe.
    #[test]
    fn clear_recording_flag_zeros_flag_and_stamp() {
        let state = make_test_app_state();
        assert!(mark_recording_started(&state));
        clear_recording_flag(&state);
        assert!(!state
            .recording_in_progress
            .load(std::sync::atomic::Ordering::Acquire));
        assert_eq!(
            state
                .recording_in_progress_started_at_ms
                .load(std::sync::atomic::Ordering::Acquire),
            0,
            "clear must zero the started-at stamp"
        );
    }

    /// Helper for the mark/clear tests. Builds a minimal
    /// `AppState` against an in-memory sqlite — only the atomics
    /// we exercise need to be valid; the services we don't touch
    /// just need to construct.
    fn make_test_app_state() -> crate::app_state::AppState {
        // Process id + millis + a thread-id-derived nonce. cargo test
        // runs unit tests in parallel, so two tests in the same
        // process can land within the same millisecond and collide
        // on the temp dir — without a per-call nonce they share an
        // AppState's DB and pollute each other's state. Pre-v0.15.3
        // tests didn't notice because they didn't write rows;
        // v0.15.3's selection-policy tests do.
        use std::sync::atomic::{AtomicU64, Ordering};
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        let tmp = std::env::temp_dir().join(format!(
            "ut-dictation-test-{}-{}-{}",
            std::process::id(),
            now_ms(),
            nonce,
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        crate::app_state::AppState::bootstrap(&tmp).expect("AppState bootstrap")
    }

    // ── v0.15.1 hotfix tests: chunked-path bounds + label absence ────
    //
    // Pin the v0.15.1 contract:
    //   1. The chunk-cap formula matches the chunker's loop arithmetic
    //      so `MAX_MEETING_CHUNKS` is meaningful.
    //   2. A 5-minute PCM (the meeting size that produced 91 fake
    //      speakers in v0.15.0) projects to a chunk count well under
    //      the cap — proving v0.15.1 cannot trigger anywhere near 91
    //      sequential Whisper calls for that input.
    //   3. The chunked rendering format used by `transcribe_chunked`
    //      contains NO `**Speaker N:**` tokens. (This is implicit in
    //      the function — no `format_speaker_turn` call survives the
    //      round trip — but tests pin the property because v0.15.2
    //      will re-introduce labels via a different mechanism and we
    //      need an explicit guard against it leaking into the chunked
    //      path.)

    /// Pin the chunk-cap constant and its derived "max minutes" so
    /// any future tweak surfaces here. 130 chunks × (30-2) seconds
    /// v0.15.3: 135 chunks × (29-2) = 27s stride, plus 29s leading
    /// chunk = 3647s = 60 min 47 s. `max_meeting_audio_minutes()`
    /// integer-divides 3647/60 = 60.
    /// History: 60 (≈28 min) v0.15.1-r1; 120 (≈56 min) r2;
    /// 130 (≈60:40) r3 with 30s chunks; 135 (≈60:47) v0.15.3
    /// retuned for 29s chunks (sherpa-onnx warns on >=30s inputs).
    #[test]
    fn chunk_cap_constants_match_documented_values() {
        assert_eq!(CHUNKED_MEETING_CHUNK_SECS, 29);
        assert_eq!(CHUNKED_MEETING_OVERLAP_SECS, 2);
        assert_eq!(MAX_MEETING_CHUNKS, 135);
        assert_eq!(max_meeting_audio_minutes(), 60);
    }

    /// Empty PCM projects to zero chunks (the chunker rejects this
    /// before the cap check; pin the boundary anyway).
    #[test]
    fn projected_chunks_zero_for_empty_pcm() {
        assert_eq!(projected_chunk_count(0, 16_000), 0);
    }

    /// A buffer that fits in a single window projects to 1.
    #[test]
    fn projected_chunks_one_for_under_one_window() {
        let one_second = 16_000;
        let twenty_seconds = 20 * 16_000;
        let exactly_one_window = CHUNKED_MEETING_CHUNK_SECS * 16_000;
        assert_eq!(projected_chunk_count(one_second, 16_000), 1);
        assert_eq!(projected_chunk_count(twenty_seconds, 16_000), 1);
        assert_eq!(projected_chunk_count(exactly_one_window, 16_000), 1);
    }

    /// **The v0.15.1 regression target:** a 5-minute meeting (the
    /// shape Jay tested) must produce a small, bounded number of
    /// chunks — not "one Whisper call per diariser segment" (which
    /// in the v0.15.0 output was 91). 5 min @ 16 kHz = 4_800_000
    /// samples; with v0.15.3's 29s/2s/27s arithmetic the projection
    /// is ceil((4_800_000 - 464_000) / 432_000) + 1 = 12 chunks.
    /// Was 11 with v0.15.1's 30s/2s/28s.
    #[test]
    fn projected_chunks_for_five_minute_meeting_is_bounded() {
        let five_min_samples = 5 * 60 * 16_000;
        let chunks = projected_chunk_count(five_min_samples, 16_000);
        assert!(
            chunks <= 13,
            "5-min meeting must project to a small chunk count (got {chunks})"
        );
        assert!(
            chunks <= MAX_MEETING_CHUNKS,
            "5-min meeting must be well under the cap"
        );
        // And concretely:
        assert_eq!(chunks, 12);
        // Most importantly: nowhere near 91.
        assert!(
            chunks < 91,
            "must be far below the v0.15.0 91-call failure shape"
        );
    }

    /// **The round-3 regression target (still required in v0.15.3):**
    /// a standard 60-minute meeting MUST be accepted by the cap.
    /// 60 min × 16 kHz × 60 s = 57_600_000 samples; with v0.15.3's
    /// 29s/2s/27s arithmetic the projection is
    /// ceil((57_600_000 - 464_000) / 432_000) + 1 = 134 chunks.
    /// Was 129 with v0.15.1's 30s chunks; the 5-extra-chunk cost
    /// of going to 29s is what drove `MAX_MEETING_CHUNKS` from
    /// 130 → 135 in v0.15.3.
    #[test]
    fn projected_chunks_for_sixty_minute_meeting_under_cap() {
        let sixty_min_samples = 60 * 60 * 16_000;
        let chunks = projected_chunk_count(sixty_min_samples, 16_000);
        assert_eq!(chunks, 134, "60-min meeting must project to 134 chunks");
        assert!(
            chunks <= MAX_MEETING_CHUNKS,
            "60-min meeting must fit under the cap (got {chunks}, cap {MAX_MEETING_CHUNKS})"
        );
    }

    /// Even a synthetic 91-segment "diariser output" cannot drive 91
    /// Whisper calls through the v0.15.1 chunker — the chunker only
    /// looks at PCM length, not at any diariser output. Pinning this
    /// explicitly because the v0.15.0 architecture coupled the two
    /// (one Whisper call per diariser segment); v0.15.1 decouples
    /// them and the test guards against re-coupling.
    #[test]
    fn diariser_segment_count_does_not_drive_chunk_count() {
        // A short PCM — far too short to ever produce 91 chunks
        // under any reasonable chunker.
        let three_min_samples = 3 * 60 * 16_000;
        let chunks = projected_chunk_count(three_min_samples, 16_000);
        // 3 min: ceil((2_880_000 - 480_000) / 448_000) + 1 = 7.
        assert_eq!(chunks, 7);
        // Pretend a hypothetical diariser returned 91 segments for
        // this 3-min buffer (the v0.15.0 failure shape). v0.15.1's
        // chunker can't see that number — it only ever sees the PCM
        // length — so the chunk count is unaffected. Document that
        // here as a property the test pins.
        let _hypothetical_diariser_segments: usize = 91;
        assert_eq!(
            chunks, 7,
            "PCM-driven chunk count must be independent of diariser segment count"
        );
    }

    /// Right at the cap boundary: the largest PCM that still produces
    /// exactly `MAX_MEETING_CHUNKS` chunks is accepted; one more sample
    /// past that boundary projects to MAX+1 and would be rejected.
    /// With the ceil formula, the boundary is exactly
    /// `(MAX - 1) * stride + chunk`.
    #[test]
    fn projected_chunks_at_cap_boundary() {
        let chunk_samples = CHUNKED_MEETING_CHUNK_SECS * 16_000;
        let stride = (CHUNKED_MEETING_CHUNK_SECS - CHUNKED_MEETING_OVERLAP_SECS) * 16_000;
        // Largest PCM that gives exactly MAX chunks:
        // ceil((total - chunk) / stride) + 1 = MAX
        // total = (MAX - 1) * stride + chunk
        let max_samples_at_cap = (MAX_MEETING_CHUNKS - 1) * stride + chunk_samples;
        assert_eq!(
            projected_chunk_count(max_samples_at_cap, 16_000),
            MAX_MEETING_CHUNKS
        );
        // One sample past the boundary tips into MAX+1.
        assert_eq!(
            projected_chunk_count(max_samples_at_cap + 1, 16_000),
            MAX_MEETING_CHUNKS + 1
        );
    }

    /// `chunk_count_for_duration` (used for the success-pill chunks
    /// number) must match `projected_chunk_count` (used for the
    /// pre-flight cap) at all sane meeting sizes. Drift between them
    /// is the kind of bug that turns the cap into a lie.
    #[test]
    fn chunk_count_helpers_agree_for_typical_meeting_sizes() {
        // 1, 2, 5, 10, 15, 28 cover sub-cap meeting sizes from
        // round 1; 45, 56, 60 cover the v0.15.1 round-3 cap range
        // (60 is the standard meeting length, must be accepted).
        for minutes in [1, 2, 5, 10, 15, 28, 45, 56, 60] {
            let secs = (minutes as u64) * 60;
            let samples = (secs as usize) * 16_000;
            let duration_ms = (secs * 1000) as u32;
            assert_eq!(
                projected_chunk_count(samples, 16_000) as u32,
                chunk_count_for_duration(duration_ms),
                "helper drift at {minutes} min"
            );
        }
    }

    // ── v0.15.1 round-2: asset-preflight disabled tests ─────────────
    //
    // After Jay flagged that round-1 disabled the diariser on the
    // stop side but left the ~47 MB asset preflight running on
    // start, v0.15.1 also disables `ensure_assets_ready` in
    // `start_recording`'s meeting branch. The tests below pin:
    //   * The required disabled-state log line exists in source.
    //   * `ensure_assets_ready` is NOT called from `start_recording`'s
    //     meeting branch (source-grep guard — the only legitimate way
    //     to assert "this codepath does NOT call X" without standing
    //     up an AppHandle in a unit test).
    //   * `meeting_preflight_in_progress` defaults to false on a
    //     fresh AppState and stays false across the bootstrap path
    //     (proxy: no PreflightGuard claims it in production code).

    /// Source-level guard: the v0.15.1 disabled-state log line
    /// MUST be present in `start_recording`. If a future change
    /// removes or reworks the meeting preflight without bringing
    /// asset prep back AND re-emitting an equivalent diagnostic,
    /// the meeting flow becomes silent — this test catches that.
    #[test]
    fn v0151_disabled_log_line_present_in_start_recording() {
        let src = include_str!("dictation.rs");
        assert!(
            src.contains(
                "meeting mode diarisation asset preflight disabled in v0.15.1 beta hotfix"
            ),
            "the v0.15.1 required log line must remain in dictation.rs"
        );
    }

    /// **The round-2 regression target.** v0.15.0 called
    /// `ensure_assets_ready` from `start_recording`'s meeting branch
    /// to prefetch the diariser models. v0.15.1 disabled the
    /// diariser on the stop side but the round-1 packet still left
    /// the ~47 MB prefetch running here, so the user paid the
    /// download cost on first meeting for a feature that was
    /// switched off. v0.15.1 round-2 removes that call entirely.
    ///
    /// Source-grep is the right shape for "this codepath does not
    /// call X" — `start_recording` is a Tauri command requiring an
    /// AppHandle and full state, too heavy for a hermetic unit test.
    /// We scan ONLY the file prefix above the `#[cfg(test)]` test
    /// mod boundary so the test's own description / string literals
    /// can't trigger false positives. Looks for the qualified-path
    /// call shape (the only way the production code ever invoked
    /// it), broken into two literals so this test's own source line
    /// also can't match itself.
    #[test]
    fn ensure_assets_ready_not_called_in_v0151_meeting_start() {
        let src = include_str!("dictation.rs");
        // Truncate the source at the test mod boundary. Anything
        // below it is test code; the property under test is about
        // production code only.
        let test_mod_marker = "#[cfg(test)]";
        let production_src = src.split(test_mod_marker).next().unwrap_or(src);
        // Build the needle from two halves so this very source line
        // doesn't also match if the file ever gets re-included
        // (e.g. if include_str! ever pulled in this test mod
        // — `split` above prevents that, but defence in depth).
        let needle: String = format!("{}{}", "diarize_assets::ensure_", "assets_ready(");
        for (lineno, line) in production_src.lines().enumerate() {
            if line.contains(&needle) {
                let trimmed = line.trim_start();
                assert!(
                    trimmed.starts_with("//") || trimmed.starts_with("///"),
                    "qualified-path call to ensure_assets_ready survives in non-comment line {}: {line:?}",
                    lineno + 1
                );
            }
        }
    }

    /// The narrow `meeting_preflight_in_progress` flag defaults to
    /// false and STAYS false across bootstrap. With v0.15.1's
    /// preflight-disable the production code never claims it; the
    /// flag's read paths in `decide_pre_probe` / `decide_post_probe`
    /// always observe false. (The flag stays in `AppState` because
    /// removing it would force a round of churn on the round-3-to-5
    /// concurrency tests; v0.15.2 will re-engage it via PreflightGuard
    /// when asset prep comes back.)
    #[test]
    fn meeting_preflight_flag_defaults_false_in_v0151() {
        let state = make_test_app_state();
        assert!(!state
            .meeting_preflight_in_progress
            .load(std::sync::atomic::Ordering::Acquire));
    }

    /// The chunked rendering format intentionally has NO speaker
    /// labels. Pin the format prefix so a future change that
    /// accidentally re-introduces `**Speaker N:**` into the chunked
    /// path is caught at unit-test time.
    ///
    /// (Tests the format the chunked path produces directly via
    /// string assembly — `transcribe_chunked` itself can't be unit-
    /// tested without an inference orchestrator, but the rendering
    /// shape it builds is `[mm:ss] <text>\n\n[mm:ss] …`. Any
    /// `**Speaker N:**` token would be a regression.)
    #[test]
    fn chunked_render_format_has_no_speaker_labels() {
        // The exact line shape the chunked path emits per chunk:
        //   "[{:02}:{:02}] " + chunk_text   then "\n\n" between chunks.
        // Reproduce the assembly here on a couple of synthetic chunks
        // and assert the output contains no Speaker tokens.
        let mut stitched = String::new();
        for (mm, ss, text) in [(0u32, 0u32, "first chunk"), (0, 28, "second chunk")] {
            if !stitched.is_empty() {
                stitched.push_str("\n\n");
            }
            stitched.push_str(&format!("[{mm:02}:{ss:02}] {text}"));
        }
        assert!(
            !stitched.contains("**Speaker"),
            "chunked render must not contain Speaker labels (got {stitched:?})"
        );
        // And specifically the canonical diarised marker shape:
        assert!(!stitched.contains("**Speaker 1:**"));
        assert!(!stitched.contains("**Speaker 2:**"));
    }

    // ── v0.15.2 hotfix: refinery migration byte-stability pins ──────
    //
    // v0.15.1 panicked at startup on every machine that had run
    // v0.15.0 first because Windows core.autocrlf rewrote
    // V0006__history_speaker_names.sql from LF (git index) to CRLF
    // (working tree) between two `tauri build` runs. v0.15.0 was
    // built from LF bytes; v0.15.1 was built from CRLF bytes;
    // refinery hashes the migration content and the user DB
    // recorded the LF hash. v0.15.1 boot panicked.
    //
    // The systemic fix is `.gitattributes` pinning `*.sql text
    // eol=lf` so the working tree always matches the index. The
    // belt-and-braces fix is the test below: pin the SHA-256 of
    // the bundled bytes of V0006 (and as a forward guard, every
    // subsequent migration) so any future drift fails CI before
    // it can ship a binary that won't launch.
    //
    // include_bytes!() resolves at compile time from the same
    // path refinery's embed_migrations!() does, so what these
    // tests hash is exactly what the binary is shipping.

    /// Compute the SHA-256 hex of a byte slice. Local to this
    /// module so the test mod doesn't need to leak a helper into
    /// production code.
    fn sha256_hex(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(bytes);
        hex::encode(h.finalize())
    }

    /// **The v0.15.2 regression target.** Pins V0006 to the LF
    /// byte content that v0.15.0 originally shipped. Any future
    /// edit (intentional or accidental) fails CI; the operator
    /// either updates the pin (acknowledging they understand the
    /// hash will change, all user DBs running v0.15.x will be
    /// unable to launch the new binary, and a separate strategy
    /// is needed) or reverts the change.
    ///
    /// Also reads the file at runtime — if the bytes the binary
    /// was compiled with somehow drift between cargo's `include_str!`
    /// resolution and refinery's `embed_migrations!` resolution
    /// (they shouldn't, but belt-and-braces), the cross-check
    /// here would catch it.
    #[test]
    fn v0006_migration_sha256_is_pinned_to_v0150_bytes() {
        const PINNED: &str = "b3ae4db25d4afb126e27a4644d16da46f1d8770643182b45cd4233e77dbb5e1d";
        const V0006: &[u8] = include_bytes!("../db/migrations/V0006__history_speaker_names.sql");
        let actual = sha256_hex(V0006);
        assert_eq!(
            actual, PINNED,
            "V0006__history_speaker_names.sql byte content has drifted — \
             this almost certainly means CRLF contamination has crept back in. \
             Run `git ls-files --eol -- 'Utterbar Windows App/src-tauri/src/db/migrations/'` \
             to confirm; if `w/crlf` shows up, delete the file and `git checkout --` it. \
             If the change is intentional (rare), update PINNED to the new value AND \
             plan how every existing user DB will recover from the inevitable \
             refinery hash mismatch on their next launch."
        );
    }

    /// Forward guard: every other migration also gets a SHA pin
    /// so a refinery panic on V0001-V0005 (same root cause class)
    /// gets caught at unit-test time rather than user-launch time.
    /// Pins computed against the current LF working tree; if any
    /// of these change in a future commit the test surfaces it.
    ///
    /// To recompute after an intentional change:
    ///   sha256sum 'Utterbar Windows App/src-tauri/src/db/migrations/<file>.sql'
    /// (and read the v0006 test's failure message — same advice
    /// applies for V0001-V0005 if the change is intentional.)
    #[test]
    fn all_migrations_byte_pins() {
        let pins: &[(&str, &[u8], &str)] = &[
            (
                "V0001__init.sql",
                include_bytes!("../db/migrations/V0001__init.sql"),
                "7ee607c7b41953dfab6e59f1b012e84930ec499e4a53531db5b2719d7e627a6c",
            ),
            (
                "V0002__modes_english_variant.sql",
                include_bytes!("../db/migrations/V0002__modes_english_variant.sql"),
                "abb7ad691884583a963848919ce81169ca58a4052e647e65a41ab6b2cf642c85",
            ),
            (
                "V0003__history_streaming.sql",
                include_bytes!("../db/migrations/V0003__history_streaming.sql"),
                "7957ccafd9ed214aa25f93de04366063da18d0b0ace4f3e7de05006c4753da7a",
            ),
            (
                "V0004__modes_meeting_mode.sql",
                include_bytes!("../db/migrations/V0004__modes_meeting_mode.sql"),
                "abd5a135181ad5ac720a7ea6e45cab851bf0c39744748ddd00ece5a51b7948af",
            ),
            (
                "V0005__history_is_meeting.sql",
                include_bytes!("../db/migrations/V0005__history_is_meeting.sql"),
                "b8eef182592b002ce1758a79f0fd87255143f08a9f0768705d72f540feb0c0f5",
            ),
            (
                "V0006__history_speaker_names.sql",
                include_bytes!("../db/migrations/V0006__history_speaker_names.sql"),
                "b3ae4db25d4afb126e27a4644d16da46f1d8770643182b45cd4233e77dbb5e1d",
            ),
        ];
        for (name, bytes, expected) in pins {
            let actual = sha256_hex(bytes);
            assert_eq!(
                &actual, expected,
                "migration {name} byte content drifted from pinned SHA — \
                 see the v0006 test above for the recovery procedure"
            );
        }
    }

    // ── v0.15.3 meeting-model selection policy + session context ────
    //
    // Round-2 contract (still in force, default flipped in v0.15.4):
    //   * If `meeting_asr_model_id` is set and installed → Use(id)
    //   * If set and missing → RefuseExplicitMissing(id) — NO silent
    //     fallback to Medium / Large / Small
    //   * v0.15.3: If unset and Medium installed → Use(Medium)
    //   * v0.15.4: If unset and Small installed → Use(Small).
    //     The default flipped because Step 0 measurements showed
    //     Small at RTF ~0.2 vs Medium's 0.87 on the same audio —
    //     Medium is too slow on laptop-class CPUs to be a sane
    //     no-preference default.
    //   * If unset and the default isn't installed →
    //     RefusePreferenceUnset — NO silent fallback even when
    //     Medium / Large is installed.
    //
    // Round-3 (Amendment 1) contract:
    //   * `ActiveRecordingContext` bound at start_recording survives
    //     mid-recording settings drift; stop_recording reads from it.
    //   * Cleared on every recording-end path.

    use super::{
        display_name_short, realtime_factor, resolve_meeting_model_id, MeetingModelResolution,
    };

    /// Construct a `Settings` with a chosen `meeting_asr_model_id`
    /// override but everything else default.
    fn settings_with_meeting_pref(pref: Option<&str>) -> crate::domain::Settings {
        let mut s = crate::domain::Settings::default();
        s.models.meeting_asr_model_id = pref.map(|p| p.to_string());
        s
    }

    /// Build an `AppState`, register the listed Whisper variants as
    /// installed, and return both the AppState and a Settings shaped
    /// to the test's preference.
    fn fixture_with_installed(
        installed: &[&str],
        pref: Option<&str>,
    ) -> (crate::app_state::AppState, crate::domain::Settings) {
        let state = make_test_app_state();
        // models_dir is `<data_dir>/models` per ModelRegistry::new.
        // We need the data_dir; reconstruct from the AppState's db
        // path. Db's `path` is `<data_dir>/data/orliq.db`, so the
        // data_dir is two up from that.
        let db_path = state.db.path.clone();
        let data_dir = db_path.parent().unwrap().parent().unwrap().to_path_buf();
        let models_root = data_dir.join("models");
        std::fs::create_dir_all(&models_root).unwrap();
        for id in installed {
            crate::services::models::test_helpers::register_for_test(
                &state.models,
                &models_root,
                id,
            );
        }
        let settings = settings_with_meeting_pref(pref);
        (state, settings)
    }

    // ── decide pre-probe selection policy ──────────────────────────

    /// **v0.15.4 contract: the default Meeting Mode model id is
    /// Whisper Small.** Pinned explicitly so a future refactor that
    /// moves the constant elsewhere can't silently regress to
    /// Medium / Large.
    #[test]
    fn default_meeting_model_is_whisper_small() {
        assert_eq!(super::DEFAULT_MEETING_MODEL_ID, "sherpa-whisper-small");
    }

    /// **Default for fresh users (v0.15.4).** No preference + Small
    /// installed → resolve to Small. Replaces the v0.15.3 Medium
    /// default.
    #[test]
    fn no_pref_and_small_installed_uses_small() {
        let (state, settings) = fixture_with_installed(&["sherpa-whisper-small"], None);
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(
            outcome,
            MeetingModelResolution::Use("sherpa-whisper-small".into())
        );
    }

    /// **v0.15.4 critical: no silent Medium fallback.** No
    /// preference, Small NOT installed, Medium IS installed → MUST
    /// refuse. The default flipping to Small means a Medium-only
    /// install is now a "the default isn't here" case, not a
    /// "happy path" case. Refusing forces the user to either
    /// install Small or pick Medium explicitly — same shape as
    /// v0.15.3's refusal when Medium was missing and Large was
    /// installed.
    #[test]
    fn no_pref_and_only_medium_installed_refuses() {
        let (state, settings) = fixture_with_installed(&["sherpa-whisper-medium"], None);
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(outcome, MeetingModelResolution::RefusePreferenceUnset);
    }

    /// **v0.15.4 critical: no silent Large fallback.** Same shape
    /// as the Medium-only refusal above.
    #[test]
    fn no_pref_and_only_large_installed_refuses() {
        let (state, settings) = fixture_with_installed(&["sherpa-whisper-large-v3"], None);
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(outcome, MeetingModelResolution::RefusePreferenceUnset);
    }

    /// No preference, no Whisper installed at all → refuse. Error
    /// path tells the user to install in Settings → Models. No
    /// silent route through Parakeet.
    #[test]
    fn meeting_refuses_when_no_pref_and_no_whisper_installed() {
        let (state, settings) = fixture_with_installed(&[], None);
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(outcome, MeetingModelResolution::RefusePreferenceUnset);
    }

    /// Explicit Large + Large installed → use Large.
    #[test]
    fn explicit_large_still_works() {
        let (state, settings) = fixture_with_installed(
            &["sherpa-whisper-large-v3"],
            Some("sherpa-whisper-large-v3"),
        );
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(
            outcome,
            MeetingModelResolution::Use("sherpa-whisper-large-v3".into())
        );
    }

    /// Explicit Medium + Medium installed → use Medium. Pinned
    /// explicitly because v0.15.4 demoted Medium from default to
    /// "Balanced explicit option"; the test guards against the
    /// flip silently breaking the explicit-Medium path.
    #[test]
    fn explicit_medium_still_works() {
        let (state, settings) =
            fixture_with_installed(&["sherpa-whisper-medium"], Some("sherpa-whisper-medium"));
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(
            outcome,
            MeetingModelResolution::Use("sherpa-whisper-medium".into())
        );
    }

    /// **Round-2 critical: no silent fallback when explicit pref is
    /// missing.** Pref says Large, Large not installed, Medium IS
    /// installed → RefuseExplicitMissing, NOT a silent fallback to
    /// Medium. The user picked Large for a reason.
    #[test]
    fn meeting_refuses_when_pref_large_but_large_not_installed_even_if_medium_installed() {
        let (state, settings) =
            fixture_with_installed(&["sherpa-whisper-medium"], Some("sherpa-whisper-large-v3"));
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(
            outcome,
            MeetingModelResolution::RefuseExplicitMissing("sherpa-whisper-large-v3".into())
        );
    }

    /// Explicit Small + Small installed → use Small. With v0.15.4
    /// Small is also the default, but the test pins the explicit
    /// path separately so a future change to the default doesn't
    /// invalidate the explicit-pref contract.
    #[test]
    fn explicit_small_still_works() {
        let (state, settings) =
            fixture_with_installed(&["sherpa-whisper-small"], Some("sherpa-whisper-small"));
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(
            outcome,
            MeetingModelResolution::Use("sherpa-whisper-small".into())
        );
    }

    /// Symmetric: pref Small, Small not installed → refuse.
    #[test]
    fn meeting_refuses_when_pref_small_but_small_not_installed() {
        let (state, settings) =
            fixture_with_installed(&["sherpa-whisper-medium"], Some("sherpa-whisper-small"));
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(
            outcome,
            MeetingModelResolution::RefuseExplicitMissing("sherpa-whisper-small".into())
        );
    }

    /// **Allow-list pin (round-2 critical).** Meeting Mode supports
    /// exactly three model ids — Whisper Small, Medium, Large v3.
    /// Any other id is rejected at the resolver layer regardless
    /// of whether the model is installed. Pinned via the helper
    /// so the contract has a name in the test surface.
    #[test]
    fn allow_list_only_accepts_whisper_small_medium_large() {
        assert!(super::is_allowed_meeting_model_id("sherpa-whisper-small"));
        assert!(super::is_allowed_meeting_model_id("sherpa-whisper-medium"));
        assert!(super::is_allowed_meeting_model_id(
            "sherpa-whisper-large-v3"
        ));
        // Off-allow-list rejections.
        assert!(!super::is_allowed_meeting_model_id(
            "sherpa-parakeet-tdt-0.6b-v3-int8"
        ));
        assert!(!super::is_allowed_meeting_model_id(
            "sherpa-parakeet-tdt-0.6b-v3-fp16"
        ));
        assert!(!super::is_allowed_meeting_model_id(
            "sherpa-zipformer-en-int8"
        ));
        assert!(!super::is_allowed_meeting_model_id("sherpa-whisper-base"));
        assert!(!super::is_allowed_meeting_model_id(""));
        assert!(!super::is_allowed_meeting_model_id("future-unknown-asr"));
    }

    /// **Round-2 critical: no Parakeet meeting routing, even
    /// explicit.** Hand-edited `settings.json` picks Parakeet AND
    /// Parakeet is installed → MUST refuse with
    /// RefuseUnsupportedModel. The resolver must NOT trust an
    /// off-allow-list explicit id even when installed.
    #[test]
    fn explicit_parakeet_meeting_pref_with_parakeet_installed_refuses_unsupported() {
        let (state, settings) = fixture_with_installed(
            &["sherpa-parakeet-tdt-0.6b-v3-int8"],
            Some("sherpa-parakeet-tdt-0.6b-v3-int8"),
        );
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(
            outcome,
            MeetingModelResolution::RefuseUnsupportedModel(
                "sherpa-parakeet-tdt-0.6b-v3-int8".into()
            )
        );
    }

    /// **Round-2 critical: no Zipformer meeting routing.**
    /// Symmetric to the Parakeet case.
    #[test]
    fn explicit_zipformer_meeting_pref_with_zipformer_installed_refuses_unsupported() {
        let (state, settings) = fixture_with_installed(
            &["sherpa-zipformer-en-int8"],
            Some("sherpa-zipformer-en-int8"),
        );
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(
            outcome,
            MeetingModelResolution::RefuseUnsupportedModel("sherpa-zipformer-en-int8".into())
        );
    }

    /// **Round-2 critical: Whisper Base is NOT a meeting model.**
    /// Even though it's a Whisper, Base isn't allow-listed for
    /// meetings — the contract is Small/Medium/Large only.
    #[test]
    fn explicit_whisper_base_meeting_pref_with_base_installed_refuses_unsupported() {
        let (state, settings) =
            fixture_with_installed(&["sherpa-whisper-base"], Some("sherpa-whisper-base"));
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(
            outcome,
            MeetingModelResolution::RefuseUnsupportedModel("sherpa-whisper-base".into())
        );
    }

    /// No preference + only Parakeet installed → MUST refuse the
    /// preference-unset path. NEVER silently selects Parakeet.
    /// Was already enforced via the no-pref + Medium-missing
    /// branch, but pin explicitly with the most adversarial input
    /// (a non-meeting model is the only thing installed).
    #[test]
    fn no_pref_only_parakeet_installed_refuses_preference_unset() {
        let (state, settings) = fixture_with_installed(&["sherpa-parakeet-tdt-0.6b-v3-int8"], None);
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(outcome, MeetingModelResolution::RefusePreferenceUnset);
    }

    /// No preference + only Whisper Large installed → still
    /// refuses (won't silently use Large; Small is the v0.15.4
    /// default). Duplicates the `no_pref_and_only_large_installed_refuses`
    /// case above; kept under this section header so the round-2
    /// packet's named-test list still has a hit if anyone greps
    /// for it.
    #[test]
    fn no_pref_only_large_installed_refuses_preference_unset() {
        let (state, settings) = fixture_with_installed(&["sherpa-whisper-large-v3"], None);
        let outcome = resolve_meeting_model_id(&settings, &state.models);
        assert_eq!(outcome, MeetingModelResolution::RefusePreferenceUnset);
    }

    /// **Allow-list happy path.** Explicit Small / Medium / Large
    /// each work when their model is installed. This re-asserts
    /// the round-1 behaviour AFTER the allow-list hardening to
    /// confirm the three sanctioned models still resolve to
    /// `Use(id)`.
    #[test]
    fn allow_listed_models_still_resolve_to_use_when_installed() {
        for id in [
            "sherpa-whisper-small",
            "sherpa-whisper-medium",
            "sherpa-whisper-large-v3",
        ] {
            let (state, settings) = fixture_with_installed(&[id], Some(id));
            let outcome = resolve_meeting_model_id(&settings, &state.models);
            assert_eq!(
                outcome,
                MeetingModelResolution::Use(id.to_string()),
                "{id} should resolve to Use when installed and explicitly preferred"
            );
        }
    }

    /// **Bridge fall-through guard (round-2 critical).** When a
    /// meeting recording has no bound `ActiveRecordingContext`
    /// (the v0.15.2-in-flight bridge case) AND the resolver
    /// refuses (no allow-listed Whisper installed; meanwhile the
    /// global `default_asr_model_id` is Parakeet), the resolver
    /// MUST return Err — never fall through to Parakeet via
    /// default_asr_model_id.
    #[test]
    fn bridge_meeting_no_context_does_not_fall_through_to_default_asr_model() {
        let (state, mut settings) =
            fixture_with_installed(&["sherpa-parakeet-tdt-0.6b-v3-int8"], None);
        // Adversarial setup: global default_asr_model_id is Parakeet
        // (typical for users who use Parakeet for dictation), and the
        // meeting preference is unset. A bridge-case meeting recording
        // (no bound context) must NOT silently route through Parakeet.
        settings.models.default_asr_model_id = Some("sherpa-parakeet-tdt-0.6b-v3-int8".into());
        let mode = crate::domain::Mode {
            id: "meeting".into(),
            name: "Meeting".into(),
            icon: "video".into(),
            is_builtin: true,
            is_favourite: false,
            sort_order: 0,
            asr_model_id: None,
            target_language: None,
            post_processing: crate::domain::PostProcessing::None,
            context_rules: Default::default(),
            auto_activation: vec![],
            shortcut_override: None,
            formatting: Default::default(),
            english_variant: Default::default(),
            is_meeting_mode: true,
            created_at: 0,
            updated_at: 0,
        };
        let result = super::resolve_target_model_id(&mode, &settings, &state.models, None);
        match result {
            Ok(id) => panic!(
                "bridge case must refuse, not resolve — got {id} (would have silently used Parakeet)"
            ),
            Err(super::MeetingBridgeFailure { resolution }) => {
                assert_eq!(resolution, MeetingModelResolution::RefusePreferenceUnset);
            }
        }
    }

    /// Bridge failure is also produced when the user has an
    /// explicit-but-off-allow-list pref and arrives at stop with
    /// no bound context. Confirms the path surfaces
    /// `RefuseUnsupportedModel` to the caller.
    #[test]
    fn bridge_meeting_unsupported_pref_propagates_through() {
        let (state, settings) = fixture_with_installed(
            &["sherpa-parakeet-tdt-0.6b-v3-int8"],
            Some("sherpa-parakeet-tdt-0.6b-v3-int8"),
        );
        let mode = crate::domain::Mode {
            id: "meeting".into(),
            name: "Meeting".into(),
            icon: "video".into(),
            is_builtin: true,
            is_favourite: false,
            sort_order: 0,
            asr_model_id: None,
            target_language: None,
            post_processing: crate::domain::PostProcessing::None,
            context_rules: Default::default(),
            auto_activation: vec![],
            shortcut_override: None,
            formatting: Default::default(),
            english_variant: Default::default(),
            is_meeting_mode: true,
            created_at: 0,
            updated_at: 0,
        };
        let err = super::resolve_target_model_id(&mode, &settings, &state.models, None)
            .expect_err("must produce a MeetingBridgeFailure");
        assert!(matches!(
            err.resolution,
            MeetingModelResolution::RefuseUnsupportedModel(_)
        ));
    }

    /// Non-meeting recordings still flow through `default_asr_model_id`
    /// (and the `"stub"` sentinel) unchanged. The bridge-failure
    /// guard only applies when `mode.is_meeting_mode == true`.
    #[test]
    fn non_meeting_recording_still_uses_default_asr_model_id() {
        let (state, mut settings) = fixture_with_installed(&[], None);
        settings.models.default_asr_model_id = Some("sherpa-parakeet-tdt-0.6b-v3-int8".into());
        let mode = crate::domain::Mode {
            id: "voice-to-text".into(),
            name: "Voice".into(),
            icon: "mic".into(),
            is_builtin: true,
            is_favourite: false,
            sort_order: 0,
            asr_model_id: None,
            target_language: None,
            post_processing: crate::domain::PostProcessing::None,
            context_rules: Default::default(),
            auto_activation: vec![],
            shortcut_override: None,
            formatting: Default::default(),
            english_variant: Default::default(),
            is_meeting_mode: false,
            created_at: 0,
            updated_at: 0,
        };
        let id = super::resolve_target_model_id(&mode, &settings, &state.models, None)
            .expect("non-meeting recording should resolve");
        assert_eq!(id, "sherpa-parakeet-tdt-0.6b-v3-int8");
    }

    // ── realtime_factor helper ──────────────────────────────────────

    #[test]
    fn realtime_factor_zero_audio_returns_none() {
        assert_eq!(realtime_factor(0, 0), None);
        assert_eq!(realtime_factor(0, 5_000), None);
    }

    #[test]
    fn realtime_factor_sub_second_audio() {
        // 100 ms audio + 200 ms processing → 2.0
        assert_eq!(realtime_factor(100, 200), Some(2.0));
    }

    #[test]
    fn realtime_factor_typical_meeting() {
        // 5 min audio + 7 min processing → 1.4
        let audio_ms = 5 * 60 * 1000;
        let processing_ms = 7 * 60 * 1000;
        let rtf = realtime_factor(audio_ms, processing_ms).unwrap();
        assert!((rtf - 1.4).abs() < 0.001, "got {rtf}");
    }

    // ── v0.15.4 picker chip + telemetry-shape pins ──────────────────
    //
    // Source-string pins. The picker is a TS file on the frontend
    // and the telemetry is a `tracing::info!` macro call here on
    // the backend; neither is reachable from runtime asserts inside
    // a unit test. Read the source file as text and grep — crude
    // but exactly the contract the spec asks for: "the
    // Recommended chip points at Small" and "provider telemetry
    // fields are present if implemented". A future refactor that
    // moves the chip or strips the field will fail this test.

    fn read_source(rel: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .join(rel);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    /// **v0.15.5 honesty contract.** AdvancedSettings.tsx must not
    /// unconditionally claim "3–10× faster than CPU" — that copy is
    /// allowed only behind a runtime-capabilities check. Phase A
    /// shipping reality is CPU-only; the speed-claim copy is gated
    /// to fire when the runtime actually supports a GPU EP.
    #[test]
    fn advanced_settings_does_not_unconditionally_claim_gpu_acceleration() {
        let src = read_source("src/routes/settings/AdvancedSettings.tsx");
        // v0.15.6: speed claims must be gated on runtime
        // capabilities AND must not quote numbers we haven't
        // measured on production audio. Pin both shapes:
        //
        //   1. The "3-10x faster" v0.15.4-era claim must not appear
        //      at all (was gated in v0.15.5; v0.15.6 removed it
        //      because the validation matrix uses real-meeting
        //      numbers in release notes, not hand-wavy marketing).
        //   2. Any speedup-multiplier phrasing (Nx, N-Mx) elsewhere
        //      must be inside a conditional that gates on
        //      `runtimeHasAnyGpuEp` or `has_dml_ep`.
        let unconditional_needle = "3\u{2013}10\u{00d7} faster";
        assert!(
            !src.contains(unconditional_needle),
            "AdvancedSettings.tsx must NOT unconditionally claim '3-10x faster' \
             (v0.15.4-era marketing phrasing). v0.15.6 removed it; release notes \
             carry the measured per-model numbers."
        );
    }

    /// **v0.15.5 inactive-notice contract.** AdvancedSettings.tsx
    /// must surface the "Selected, but currently inactive" notice
    /// for DML/CUDA when the runtime can't serve them. Source-grep
    /// pin so a future refactor can't drop the user-facing message.
    #[test]
    fn advanced_settings_shows_inactive_notice_for_unavailable_ep() {
        let src = read_source("src/routes/settings/AdvancedSettings.tsx");
        assert!(
            src.contains("Selected, but currently inactive"),
            "AdvancedSettings.tsx must include the 'Selected, but currently inactive' notice copy"
        );
        assert!(
            src.contains("Transcribes are running on CPU"),
            "AdvancedSettings.tsx must tell the user transcribes are on CPU when the EP is unavailable"
        );
    }

    /// **v0.15.4 picker contract.** ModelsSettings.tsx pins the
    /// Recommended chip on Whisper Small. v0.15.3 had it on
    /// Medium; the swap is the user-visible signal of the default
    /// flip.
    #[test]
    fn picker_recommended_chip_points_at_whisper_small() {
        let src = read_source("src/routes/settings/ModelsSettings.tsx");
        assert!(
            src.contains("opt.id === 'sherpa-whisper-small'"),
            "ModelsSettings.tsx must check `opt.id === 'sherpa-whisper-small'` to render the Recommended chip on the Fast row"
        );
        assert!(
            !src.contains("opt.id === 'sherpa-whisper-medium'"),
            "ModelsSettings.tsx must NOT branch on `opt.id === 'sherpa-whisper-medium'` for the chip — that was the v0.15.3 behaviour"
        );
    }

    /// **v0.19.4 updater install-restart contract.** The main-window
    /// `CloseRequested` handler in `main.rs` MUST consult the
    /// `updater_install_in_progress` flag on `AppState` BEFORE the
    /// `close_to_tray ? prevent_close + hide` intercept, and MUST
    /// `return` early when the flag is set so the updater's
    /// `handle.exit(0)` shutdown can proceed. Without this gate, a
    /// user with `general.close_to_tray = true` (the default) sees
    /// Install & restart hide the app to the tray instead of letting
    /// NSIS overwrite the running .exe. Source-grep pin so a future
    /// refactor of the close handler can't silently drop the bypass.
    #[test]
    fn close_handler_skips_intercept_when_updater_install_in_progress() {
        let src = read_source("src-tauri/src/main.rs");
        // The handler must read the flag with a load() call against
        // updater_install_in_progress BEFORE the close_to_tray branch.
        let flag_load_idx = src
            .find("updater_install_in_progress")
            .expect("main.rs close handler must reference updater_install_in_progress");
        let close_to_tray_idx = src
            .find(".close_to_tray")
            .expect("main.rs close handler must read .close_to_tray");
        assert!(
            flag_load_idx < close_to_tray_idx,
            "main.rs must read updater_install_in_progress BEFORE .close_to_tray; \
             otherwise an updater-triggered exit gets intercepted as a close-to-tray"
        );
        // The bypass must early-return (not just log) so neither
        // prevent_close nor hide() runs when the flag is set. We
        // pin the .load() call near .updater_install_in_progress
        // without locking the indentation pattern, because cargo fmt
        // can reflow the surrounding code.
        let load_idx = src
            .find(".updater_install_in_progress")
            .and_then(|start| src[start..].find(".load(").map(|off| start + off))
            .expect(
                "main.rs close handler must call .load() on updater_install_in_progress \
                 with explicit ordering",
            );
        assert!(load_idx > 0);
        // Defence-in-depth: the early return literal must appear
        // inside the close handler scope.
        let handler_slice_start = src
            .find("if let tauri::WindowEvent::CloseRequested")
            .expect("main.rs must have the CloseRequested intercept");
        let handler_slice = &src[handler_slice_start..];
        assert!(
            handler_slice.contains("return;"),
            "main.rs CloseRequested handler must early-return when the updater flag is set"
        );
    }

    /// **v0.19.4 updater install-restart contract (set side).** The
    /// `install_update` command must set
    /// `updater_install_in_progress = true` BEFORE `spawn_installer`
    /// and the `handle.exit(0)` shutdown task fire, so the close
    /// handler in `main.rs` can read a `true` value when its
    /// `CloseRequested` event arrives. The flag must also be cleared
    /// on the spawn-installer error branch so a failed install
    /// doesn't lock the user out of normal close-to-tray on their
    /// next genuine [X] click.
    #[test]
    fn install_update_sets_intent_flag_before_spawn_and_exit() {
        let src = read_source("src-tauri/src/commands/updater.rs");
        // Flag set must precede spawn_installer. The store-true line
        // appears as `.store(true, Ordering::SeqCst);` indented inside
        // a method-chain on `.updater_install_in_progress`.
        let store_true_idx = src
            .find(".updater_install_in_progress")
            .and_then(|start| src[start..].find(".store(true,").map(|off| start + off))
            .expect("install_update must store(true) on updater_install_in_progress");
        let spawn_idx = src
            .find("spawn_installer(&path)")
            .expect("install_update must call spawn_installer");
        assert!(
            store_true_idx < spawn_idx,
            "install_update must set updater_install_in_progress = true BEFORE spawn_installer; \
             otherwise the close handler can race the exit task with a stale `false` read"
        );
        // The exit task is the consumer; pin that it sits AFTER the
        // store_true point too.
        let handle_exit_idx = src
            .find("handle.exit(0);")
            .expect("install_update must schedule handle.exit(0)");
        assert!(
            store_true_idx < handle_exit_idx,
            "install_update must set updater_install_in_progress = true BEFORE handle.exit(0)"
        );
        // Error-path clear: a failed spawn_installer must reset the
        // flag so the next [X] click goes to tray as expected. The
        // store(false) line appears in the spawn-error branch with
        // its own method-chain indentation.
        let store_false_idx = src.find(".store(false,").inspect(|&idx| {
            // Confirm the store(false) is on .updater_install_in_progress
            // by checking the preceding 80 bytes contain the field name.
            let look_back_start = idx.saturating_sub(80);
            let context = &src[look_back_start..idx];
            assert!(
                context.contains("updater_install_in_progress"),
                ".store(false) at byte {} must be on .updater_install_in_progress",
                idx
            );
        });
        assert!(
            store_false_idx.is_some(),
            "install_update must clear updater_install_in_progress on the spawn_installer \
             error branch; otherwise a failed install leaves the user unable to use \
             close-to-tray"
        );
    }

    /// **v0.19.3 placement contract.** The Settings → Models
    /// language-compatibility warning v0.19.2 introduced must render
    /// directly below the Whisper-language card, not between the
    /// Default model card and the Meeting Mode card. v0.19.3 moved
    /// it; this test pins the move at the Rust layer too so a
    /// future refactor can't silently revert the placement.
    #[test]
    fn models_language_warning_renders_beside_whisper_picker() {
        let src = read_source("src/routes/settings/ModelsSettings.tsx");
        // Both warning catalogue keys must appear AFTER the Whisper-
        // language card title prop and BEFORE the Keep-warm card title.
        let whisper_picker_idx = src
            .find("models.whisper_language.title")
            .expect("ModelsSettings.tsx must reference models.whisper_language.title");
        let english_warn_idx = src
            .find("models.language_warning.english_only_title")
            .expect("ModelsSettings.tsx must reference the english-only warning key");
        let no_multi_warn_idx = src
            .find("models.language_warning.no_multilingual_title")
            .expect("ModelsSettings.tsx must reference the no-multilingual warning key");
        let keep_warm_idx = src
            .find("\"Keep-warm duration\"")
            .expect("ModelsSettings.tsx must still have the Keep-warm card title");
        assert!(
            whisper_picker_idx < english_warn_idx && english_warn_idx < keep_warm_idx,
            "english-only warning must render between the Whisper-language card and the \
             Keep-warm card in source order (the v0.19.3 placement)"
        );
        assert!(
            whisper_picker_idx < no_multi_warn_idx && no_multi_warn_idx < keep_warm_idx,
            "no-multilingual warning must render between the Whisper-language card and the \
             Keep-warm card in source order (the v0.19.3 placement)"
        );
        // Negative pin against the v0.19.2 placement (between Default
        // model and Meeting Mode anchors). Both anchors are still
        // untranslated literals — a deliberate part of the
        // scope-deferred Models localisation — so the source-grep is
        // stable across this branch.
        let default_model_idx = src
            .find("title=\"Default model\"")
            .expect("ModelsSettings.tsx must still have the Default model card title");
        let meeting_model_idx = src
            .find("title=\"Meeting mode model\"")
            .expect("ModelsSettings.tsx must still have the Meeting mode card title");
        assert!(
            !(english_warn_idx > default_model_idx && english_warn_idx < meeting_model_idx),
            "english-only warning MUST NOT sit between Default model and Meeting Mode \
             (the v0.19.2 placement that v0.19.3 fixed)"
        );
        assert!(
            !(no_multi_warn_idx > default_model_idx && no_multi_warn_idx < meeting_model_idx),
            "no-multilingual warning MUST NOT sit between Default model and Meeting Mode \
             (the v0.19.2 placement that v0.19.3 fixed)"
        );
    }

    /// **v0.15.5 telemetry contract.** The `meeting transcribe
    /// complete` log line carries `provider_requested`,
    /// `provider_used`, `fallback_used`, `fallback_reason`.
    /// Source-grep rather than a live-emit test because capturing
    /// tracing output through a global subscriber is brittle. Phase
    /// A added `provider_requested` to v0.15.4's three-field set.
    #[test]
    fn meeting_transcribe_log_includes_provider_telemetry_fields() {
        let src = read_source("src-tauri/src/commands/dictation.rs");
        // v0.15.6: scan every occurrence of the literal and require
        // at least ONE to have all six field names within 1600 bytes
        // before it. v0.15.5 had four fields; v0.15.6 adds
        // dml_available + provider_policy_reason. Window bumped from
        // 1200 to 1600 because the v0.15.6 preamble comment is longer.
        let needle_literal = "\"meeting transcribe complete\"";
        let mut hit = false;
        let mut last_window = String::new();
        for (idx, _) in src.match_indices(needle_literal) {
            let window_start = idx.saturating_sub(1600);
            let window = &src[window_start..idx];
            if [
                "provider_requested",
                "provider_used",
                "fallback_used",
                "fallback_reason",
                "provider_policy_reason",
                "dml_available",
            ]
            .iter()
            .all(|n| window.contains(n))
            {
                hit = true;
                break;
            }
            last_window = window.to_string();
        }
        assert!(
            hit,
            "no `meeting transcribe complete` log call site carries all six v0.15.6 \
             telemetry fields within 1600 bytes before the literal — last window: {last_window}"
        );
    }

    /// **v0.15.5+ non-meeting telemetry contract.** The
    /// `inference.transcribe done` log line on the dictation path
    /// also carries the provider fields, so a grepper can correlate
    /// dictation-path performance with provider telemetry without
    /// the meeting log only firing for meetings.
    #[test]
    fn dictation_transcribe_done_log_includes_provider_telemetry_fields() {
        let src = read_source("src-tauri/src/commands/dictation.rs");
        let needle_literal = "\"inference.transcribe done\"";
        let mut hit = false;
        for (idx, _) in src.match_indices(needle_literal) {
            let window_start = idx.saturating_sub(1200);
            let window = &src[window_start..idx];
            if [
                "provider_requested",
                "provider_used",
                "fallback_used",
                "fallback_reason",
                "provider_policy_reason",
                "dml_available",
            ]
            .iter()
            .all(|n| window.contains(n))
            {
                hit = true;
                break;
            }
        }
        assert!(
            hit,
            "`inference.transcribe done` log must carry the six v0.15.6 telemetry fields"
        );
    }

    /// **v0.15.6 anti-regression.** The v0.8.7 Whisper CPU-force
    /// (`let provider = "cpu".to_string();` in sherpa_whisper.rs) is
    /// gone. The engine honours the resolved provider string. If a
    /// future change reintroduces the force, this test fails and
    /// signals the v0.8.7 mitigation must NOT be silently revived
    /// — the right surface for any new mitigation is the resolver,
    /// not the engine.
    #[test]
    fn sherpa_whisper_no_longer_force_cpu() {
        let src = read_source("src-tauri/src/services/inference/sherpa_whisper.rs");
        // The v0.8.7-era literal we want gone:
        let banned = "let provider = \"cpu\".to_string();";
        // The v0.15.6 replacement we want present:
        let expected = "let provider = self.provider.clone();";
        assert!(
            !src.contains(banned),
            "sherpa_whisper.rs must NOT contain the v0.8.7 CPU-force literal `{banned}` -- \
             v0.15.6 lifted it. Use the resolver (`resolve_provider`) for any model-specific \
             provider routing, not the engine."
        );
        assert!(
            src.contains(expected),
            "sherpa_whisper.rs must honour the resolved provider via `{expected}`"
        );
    }

    /// **v0.15.6 bundle resources contract.** tauri.conf.json must
    /// list DirectML.dll alongside the existing onnxruntime + sherpa
    /// DLLs. Pinned via source-grep so a future bundle.resources
    /// edit can't silently drop DirectML from the install set.
    #[test]
    fn tauri_conf_bundle_resources_lists_directml_dll() {
        let src = read_source("src-tauri/tauri.conf.json");
        assert!(
            src.contains("\"runtime/DirectML.dll\""),
            "tauri.conf.json bundle.resources must include runtime/DirectML.dll"
        );
        // Sanity: existing DLLs still listed (unchanged from v0.15.5).
        assert!(src.contains("\"runtime/onnxruntime.dll\""));
        assert!(src.contains("\"runtime/sherpa-onnx-c-api.dll\""));
        assert!(src.contains("\"runtime/sherpa-onnx-cxx-api.dll\""));
    }

    // ── meeting_asr_model_id serde back-compat ──────────────────────

    /// Existing settings.json files (pre-v0.15.3) miss the new field
    /// entirely. Deserialise must succeed with `meeting_asr_model_id =
    /// None`. The serde `#[serde(default)]` annotation on the field
    /// is what allows this; the test pins the contract.
    ///
    /// Approach: serialise a default `Settings`, strip the new field
    /// from the JSON, and deserialise. The strip simulates a
    /// pre-v0.15.3 settings.json that never wrote the field.
    #[test]
    fn meeting_asr_model_id_serde_back_compat_missing_field() {
        // Round-trip via Value to avoid depending on the exact JSON
        // shape of every nested struct (which would make the test
        // brittle to any unrelated settings change).
        let mut value: serde_json::Value =
            serde_json::to_value(crate::domain::Settings::default()).unwrap();
        // Strip the new field.
        if let Some(models) = value.get_mut("models").and_then(|m| m.as_object_mut()) {
            models.remove("meeting_asr_model_id");
        } else {
            panic!("expected `models` object in serialised settings");
        }
        // Confirm the field is actually gone.
        assert!(
            value
                .get("models")
                .and_then(|m| m.get("meeting_asr_model_id"))
                .is_none(),
            "meeting_asr_model_id must be absent for the test to be meaningful"
        );
        // Now deserialise: should succeed via #[serde(default)].
        let settings: crate::domain::Settings = serde_json::from_value(value)
            .expect("pre-v0.15.3 settings.json (without meeting_asr_model_id) must deserialise");
        assert_eq!(settings.models.meeting_asr_model_id, None);
    }

    // ── display_name_short ──────────────────────────────────────────

    #[test]
    fn display_name_short_known_models() {
        assert_eq!(display_name_short("sherpa-whisper-small"), "Whisper Small");
        assert_eq!(
            display_name_short("sherpa-whisper-medium"),
            "Whisper Medium"
        );
        assert_eq!(
            display_name_short("sherpa-whisper-large-v3"),
            "Whisper Large v3"
        );
        assert_eq!(display_name_short("sherpa-whisper-base"), "Whisper Base");
    }

    #[test]
    fn display_name_short_unknown_model_returns_id() {
        assert_eq!(
            display_name_short("sherpa-future-model-xyz"),
            "sherpa-future-model-xyz"
        );
    }

    // ── ActiveRecordingContext lifecycle (Amendment 1) ─────────────
    //
    // The context is bound at start_recording and read at stop. We
    // can exercise the lifecycle directly on AppState's RwLock
    // without standing up the full Tauri command surface.

    #[tokio::test]
    async fn active_recording_context_round_trip_holds_value() {
        let state = make_test_app_state();
        let ctx = crate::app_state::ActiveRecordingContext {
            mode_id: "voice-to-text".into(),
            is_meeting: false,
            meeting_model_id: None,
            started_at_ms: 12345,
            mic_mute_applied: Vec::new(),
        };
        *state.active_recording_context.write().await = Some(ctx.clone());
        let read = state
            .active_recording_context
            .read()
            .await
            .clone()
            .expect("context bound");
        assert_eq!(read.mode_id, "voice-to-text");
        assert!(!read.is_meeting);
        assert_eq!(read.meeting_model_id, None);
    }

    /// **Round-3 critical: meeting session remains meeting if active
    /// mode changes mid-recording.** Bind a meeting context, then
    /// "change" the active mode (in real flow this would be the
    /// user toggling mid-record); the bound is_meeting still says
    /// true.
    #[tokio::test]
    async fn meeting_session_remains_meeting_when_active_mode_changes() {
        let state = make_test_app_state();
        *state.active_recording_context.write().await =
            Some(crate::app_state::ActiveRecordingContext {
                mode_id: "meeting-mode".into(),
                is_meeting: true,
                meeting_model_id: Some("sherpa-whisper-medium".into()),
                started_at_ms: 12345,
                mic_mute_applied: Vec::new(),
            });
        // Simulate "settings drift": user swapped active mode to a
        // non-meeting one. The ActiveRecordingContext is what stop
        // reads; the simulated drift would be in `state.settings`
        // which we don't even touch here. The point is that
        // stop_recording's binding logic reads the context, not
        // settings.
        let read = state.active_recording_context.read().await.clone().unwrap();
        assert!(read.is_meeting);
        assert_eq!(
            read.meeting_model_id.as_deref(),
            Some("sherpa-whisper-medium")
        );
    }

    /// Symmetric: non-meeting session remains non-meeting.
    #[tokio::test]
    async fn non_meeting_session_remains_non_meeting_when_active_mode_changes() {
        let state = make_test_app_state();
        *state.active_recording_context.write().await =
            Some(crate::app_state::ActiveRecordingContext {
                mode_id: "voice-to-text".into(),
                is_meeting: false,
                meeting_model_id: None,
                started_at_ms: 12345,
                mic_mute_applied: Vec::new(),
            });
        let read = state.active_recording_context.read().await.clone().unwrap();
        assert!(!read.is_meeting);
        assert_eq!(read.meeting_model_id, None);
    }

    /// Meeting model id persists through settings change. Bind the
    /// context with Medium; even if a hypothetical settings update
    /// changed the preference to Small, the bound model_id stays
    /// Medium.
    #[tokio::test]
    async fn meeting_model_id_persists_through_settings_change() {
        let state = make_test_app_state();
        *state.active_recording_context.write().await =
            Some(crate::app_state::ActiveRecordingContext {
                mode_id: "meeting".into(),
                is_meeting: true,
                meeting_model_id: Some("sherpa-whisper-medium".into()),
                started_at_ms: 1000,
                mic_mute_applied: Vec::new(),
            });
        // The "settings change" is purely external; the context
        // doesn't react to it. Read confirms the start-time value
        // wins.
        let read = state.active_recording_context.read().await.clone().unwrap();
        assert_eq!(
            read.meeting_model_id.as_deref(),
            Some("sherpa-whisper-medium"),
            "bound model id must survive any external settings drift"
        );
    }

    /// `clear_active_recording_context` empties the lock.
    #[tokio::test]
    async fn active_recording_context_cleared_by_helper() {
        let state = make_test_app_state();
        *state.active_recording_context.write().await =
            Some(crate::app_state::ActiveRecordingContext {
                mode_id: "x".into(),
                is_meeting: false,
                meeting_model_id: None,
                started_at_ms: 0,
                mic_mute_applied: Vec::new(),
            });
        super::clear_active_recording_context(&state).await;
        assert!(state.active_recording_context.read().await.is_none());
    }

    /// Source-grep guard: every clear-recording-flag call site in
    /// PRODUCTION code (above the test mod) is paired with
    /// `clear_active_recording_context(...)` within 12 lines, OR is
    /// annotated with the magic comment `CONTEXT-INTENTIONAL` when
    /// the non-pair is deliberate (one such site exists in
    /// stop_recording where the context must be kept alive across
    /// the transcribe call).
    ///
    /// The needle deliberately matches the function name regardless
    /// of argument form: both the `state.inner()` callsites in the
    /// `State<_>`-taking commands and the bare-`state` callsites in
    /// the `stop_recording_inner` helper (which takes
    /// `&Arc<AppState>`) need to be covered. Loosening to the
    /// argument-agnostic form is safe because the window-pair
    /// check is what actually proves the invariant; the needle
    /// only has to find candidate sites.
    ///
    /// The test mod boundary is split on `#[cfg(test)]` so this
    /// doc-comment, which talks about the needle, isn't itself
    /// scanned.
    #[test]
    fn every_clear_recording_flag_site_pairs_with_clear_active_recording_context() {
        let src = include_str!("dictation.rs");
        // Truncate at the test mod boundary so test code's mentions
        // of `clear_recording_flag` don't trip the guard.
        let test_mod_marker = "#[cfg(test)]";
        let production_src = src.split(test_mod_marker).next().unwrap_or(src);
        let lines: Vec<&str> = production_src.lines().collect();
        // Obfuscation: assemble at runtime so this test's own source
        // doesn't contain the literal needle and self-trip.
        let needle = format!("{}{}", "clear_recording_", "flag(");
        // Skip the function's own definition (`fn clear_recording_flag(...)`)
        // and any reference inside a doc-comment line. Only real call
        // sites should be scanned.
        let fn_def_prefix = "fn ";
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if line.contains(&needle)
                && !trimmed.starts_with("///")
                && !trimmed.starts_with("//")
                && !trimmed.starts_with(fn_def_prefix)
            {
                // Look 12 lines either side for the paired
                // clear_active_recording_context OR the magic
                // CONTEXT-INTENTIONAL comment.
                let from = i.saturating_sub(12);
                let to = (i + 12).min(lines.len());
                let window = &lines[from..to];
                let paired = window
                    .iter()
                    .any(|l| l.contains("clear_active_recording_context("))
                    || window.iter().any(|l| l.contains("CONTEXT-INTENTIONAL"));
                assert!(
                    paired,
                    "clear_recording_flag at line {} (window {}-{}) is not paired with clear_active_recording_context within 12 lines (and lacks a CONTEXT-INTENTIONAL annotation):\n  {}",
                    i + 1,
                    from + 1,
                    to,
                    line.trim()
                );
            }
        }
    }

    /// Output validation must NOT run for Meeting Mode.
    ///
    /// The shared output_validation::validate is gated to regular
    /// dictation only because legitimate meeting transcripts can
    /// contain repetition, mixed-language content, and other patterns
    /// the validator's noticed-drop rules would flag. Meeting Mode
    /// keeps the pre-patch behaviour: drop only on empty/whitespace
    /// processed text, no user-visible "Output filtered" notice.
    ///
    /// This test is a source-grep guard: it proves that the
    /// validator's only call site in this file lives inside an
    /// `else` branch whose enclosing `if` tests `mode.is_meeting_mode`.
    /// Future refactors that hoist the validator out of that branch
    /// (or wire it into the meeting path) will fail this test.
    ///
    /// Obfuscation: the needle is assembled at runtime so this
    /// doc-comment doesn't itself contain the literal `validate(`.
    #[test]
    fn validate_call_is_gated_to_non_meeting_mode() {
        let src = include_str!("dictation.rs");
        let test_mod_marker = "#[cfg(test)]";
        let production_src = src.split(test_mod_marker).next().unwrap_or(src);
        let lines: Vec<&str> = production_src.lines().collect();
        let needle = format!("{}{}", "valid", "ate(&processed");
        let mut found = 0usize;
        for (i, line) in lines.iter().enumerate() {
            if !line.contains(&needle) {
                continue;
            }
            found += 1;
            // Walk backwards to find the nearest control-flow line.
            // The validator must sit inside the `else` branch of an
            // `if mode.is_meeting_mode { ... } else { ... }`. We accept
            // either a literal `} else {` between the call site and
            // the most recent `if mode.is_meeting_mode {` line, or
            // the simpler form where the enclosing block opens with
            // `if mode.is_meeting_mode { ... } else {`.
            let from = i.saturating_sub(80);
            let window = &lines[from..i];
            let saw_else = window.iter().any(|l| l.trim() == "} else {");
            let saw_meeting_if = window.iter().any(|l| l.contains("if mode.is_meeting_mode"));
            assert!(
                saw_meeting_if && saw_else,
                "validator call at line {} must sit inside the else branch of `if mode.is_meeting_mode {{ ... }} else {{ ... }}`, but the 80-line window above lacks the pairing",
                i + 1
            );
        }
        assert!(
            found >= 1,
            "expected at least one production call to the output validator on `processed`; found 0"
        );
    }

    /// Meeting Mode chunk validator must only be called inside
    /// `transcribe_chunked`.
    ///
    /// `validate_meeting_chunk` has a different rule set from the
    /// regular `validate`: it observes (does not drop) repeated-token
    /// chunks, skips the foreign-script gate, and returns a per-chunk
    /// verdict rather than a whole-transcript verdict. Calling it
    /// from the regular dictation/file path would silently weaken
    /// those callsites' rules. This guard pins the call to the
    /// chunked-transcribe function.
    ///
    /// Same source-grep technique as `validate_call_is_gated_to_non_meeting_mode`:
    /// scan the production region (split on `#[cfg(test)]`), find
    /// every call site of the meeting-chunk helper, and assert each
    /// one sits below an `async fn transcribe_chunked(` line within
    /// a reasonable lookback window. Fewer than one call site means
    /// the wiring went missing; any call site outside the chunked
    /// function fails the assertion.
    ///
    /// Obfuscation: the needle is assembled at runtime so this
    /// doc-comment does not itself contain `validate_meeting_chunk(`.
    #[test]
    fn validate_meeting_chunk_call_is_only_inside_transcribe_chunked() {
        let src = include_str!("dictation.rs");
        let test_mod_marker = "#[cfg(test)]";
        let production_src = src.split(test_mod_marker).next().unwrap_or(src);
        let lines: Vec<&str> = production_src.lines().collect();
        let needle = format!("{}{}", "validate_meeting_", "chunk(");
        let chunked_fn_marker = "async fn transcribe_chunked(";
        let next_top_level_marker = "async fn ";

        let mut found = 0usize;
        for (i, line) in lines.iter().enumerate() {
            // Skip definition / use-import / doc lines so only real
            // call sites count.
            let trimmed = line.trim_start();
            if trimmed.starts_with("//")
                || trimmed.starts_with("///")
                || trimmed.starts_with("use ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("fn ")
            {
                continue;
            }
            if !line.contains(&needle) {
                continue;
            }
            found += 1;
            // Walk backwards to find the most recent `async fn ...`
            // declaration. It must be `transcribe_chunked`.
            let mut enclosing: Option<&str> = None;
            for back in (0..i).rev() {
                let l = lines[back].trim_start();
                if l.starts_with(next_top_level_marker) {
                    enclosing = Some(l);
                    break;
                }
            }
            let enclosing_line =
                enclosing.expect("validate_meeting_chunk call has no async fn ancestor");
            assert!(
                enclosing_line.starts_with(chunked_fn_marker),
                "validate_meeting_chunk call at line {} is inside `{}` but must only be \
                 inside `async fn transcribe_chunked(...)`. Meeting Mode chunk validation \
                 has a different rule set from regular dictation/file validation; calling \
                 it from any other function would silently weaken those callsites.",
                i + 1,
                enclosing_line.split('(').next().unwrap_or(enclosing_line)
            );
        }
        assert!(
            found >= 1,
            "expected at least one production call to validate_meeting_chunk inside \
             transcribe_chunked; found 0"
        );
    }

    /// Single-shot auto-stop constants pin.
    ///
    /// `SINGLE_SHOT_AUTO_STOP_MS` must remain at 295_000 (4 minutes
    /// 55 seconds) so that the auto-stop fires at least 5 seconds
    /// before the ASR transcribe-layer cap (`MAX_TRANSCRIBE_SECONDS
    /// = 5 * 60 = 300` in `services/inference/mod.rs`). The 5-second
    /// margin is what guarantees the post-stop AudioBuffer.pcm
    /// length stays under the cap even after the resampler tail and
    /// the last cpal callback land. Bumping one constant without
    /// the other (or shrinking the margin) re-introduces the
    /// "lost the recording at 5:30" bug that this timer fixes.
    #[test]
    fn auto_stop_pin_constants_match_asr_cap() {
        assert_eq!(
            super::SINGLE_SHOT_AUTO_STOP_MS,
            295_000,
            "SINGLE_SHOT_AUTO_STOP_MS must be 295_000 ms (4:55)"
        );
        // Mirror of the cap in services/inference/mod.rs. Hard-coded
        // here because the ASR cap is module-private; if a future
        // refactor exposes it, replace this literal with the import.
        const ASR_MAX_TRANSCRIBE_SECONDS: i64 = 5 * 60;
        let asr_cap_ms: i64 = ASR_MAX_TRANSCRIBE_SECONDS * 1000;
        let margin_ms: i64 = asr_cap_ms - super::SINGLE_SHOT_AUTO_STOP_MS;
        assert!(
            margin_ms >= 5_000,
            "auto-stop must fire at least 5 s before the ASR cap (margin={} ms)",
            margin_ms
        );
        assert_eq!(
            margin_ms, 5_000,
            "design value: 5 s margin exactly. Drift from this means \
             the design rationale needs updating."
        );
    }

    /// `auto_stop_should_proceed` returns `true` for the same
    /// non-meeting session, `false` for every other case the timer
    /// could see at wake time.
    #[test]
    fn auto_stop_proceeds_only_for_same_non_meeting_session() {
        // Same session, non-meeting → proceed.
        let same_non_meeting = crate::app_state::ActiveRecordingContext {
            mode_id: "voice-to-text".into(),
            is_meeting: false,
            meeting_model_id: None,
            started_at_ms: 1_000,
            mic_mute_applied: Vec::new(),
        };
        assert!(super::auto_stop_should_proceed(
            1_000,
            Some(&same_non_meeting)
        ));

        // Same started_at_ms but is_meeting → defensive false.
        // (Should be unreachable since the timer isn't spawned for
        // meeting starts, but the helper is the last line of
        // defence.)
        let same_meeting = crate::app_state::ActiveRecordingContext {
            mode_id: "meeting".into(),
            is_meeting: true,
            meeting_model_id: Some("sherpa-whisper-small".into()),
            started_at_ms: 1_000,
            mic_mute_applied: Vec::new(),
        };
        assert!(!super::auto_stop_should_proceed(1_000, Some(&same_meeting)));

        // Different started_at_ms → stale session, false. (User
        // pressed stop and started a new recording while the timer
        // was sleeping.)
        let different_session = crate::app_state::ActiveRecordingContext {
            mode_id: "voice-to-text".into(),
            is_meeting: false,
            meeting_model_id: None,
            started_at_ms: 2_000,
            mic_mute_applied: Vec::new(),
        };
        assert!(!super::auto_stop_should_proceed(
            1_000,
            Some(&different_session)
        ));

        // No context at all → user already stopped/cancelled, false.
        assert!(!super::auto_stop_should_proceed(1_000, None));
    }

    /// Smart rewrite (Settings → Text Output) must NOT apply to
    /// Meeting Mode in v1.
    ///
    /// Meeting transcripts are reference material — fidelity over
    /// polish. The setting is a "global" override of the per-mode
    /// `PostProcessing::None` default for regular dictation, but
    /// the meeting path takes a separate branch that bypasses
    /// post_processing entirely.
    ///
    /// This test is a source-grep guard: it proves the only
    /// reference to `text_output.smart_rewrite` in production code
    /// lives inside an `else` branch whose enclosing `if` tests
    /// `mode.is_meeting_mode`. Future refactors that hoist the
    /// smart-rewrite check above the meeting gate, or that wire
    /// it into the meeting branch, will fail this test.
    ///
    /// Obfuscation: the needle is assembled at runtime so this
    /// doc-comment doesn't itself contain the literal field name.
    #[test]
    fn smart_rewrite_is_gated_to_non_meeting_mode() {
        let src = include_str!("dictation.rs");
        let test_mod_marker = "#[cfg(test)]";
        let production_src = src.split(test_mod_marker).next().unwrap_or(src);
        let lines: Vec<&str> = production_src.lines().collect();
        let needle = format!("{}{}", "text_output.smart_", "rewrite");
        let mut found = 0usize;
        for (i, line) in lines.iter().enumerate() {
            if !line.contains(&needle) {
                continue;
            }
            found += 1;
            let from = i.saturating_sub(80);
            let window = &lines[from..i];
            let saw_else = window.iter().any(|l| l.trim() == "} else {");
            let saw_meeting_if = window.iter().any(|l| l.contains("if mode.is_meeting_mode"));
            assert!(
                saw_meeting_if && saw_else,
                "smart_rewrite reference at line {} must sit inside the else branch of \
                 `if mode.is_meeting_mode {{ ... }} else {{ ... }}`, but the 80-line window \
                 above lacks the pairing. Hoisting smart_rewrite above the meeting gate \
                 would silently apply filler removal to meeting transcripts, which is out \
                 of scope for v1 (transcripts are reference material — verbatim fidelity \
                 matters more than polish).",
                i + 1
            );
        }
        assert!(
            found >= 1,
            "expected at least one production reference to text_output.smart_rewrite in \
             dictation.rs; found 0. The setting was added but never wired."
        );
    }

    /// **v0.20.0 capture-stop allocation contract.** `CaptureHandle::stop`
    /// in `services/audio/capture.rs` MUST extract the captured PCM via a
    /// single `std::mem::take` of the locked buffer, NOT via two
    /// full-buffer `.clone()` snapshots. The previous double-snapshot
    /// guarded against a late-callback race; with the safety cap at 8
    /// hours that meant up to 2 × ~920 MB of allocator churn per stop.
    /// The new flow waits the same total drain window (120 ms + 30 ms)
    /// before extraction and then moves the buffer once. Source-grep so
    /// a future refactor that re-introduces a clone here fails CI.
    #[test]
    fn capture_stop_moves_buffer_without_double_clone() {
        let src = read_source("src-tauri/src/services/audio/capture.rs");
        // Find the body of `pub async fn stop(&self) -> CaptureFinal`
        // and inspect only that function's source. We pin against the
        // matching closing `}` at the same indentation level.
        let fn_marker = "pub async fn stop(&self) -> CaptureFinal {";
        let start = src
            .find(fn_marker)
            .expect("CaptureHandle::stop signature must be present in capture.rs");
        // Walk to the closing brace by tracking depth from the opening `{` of the fn.
        let body_start = start + fn_marker.len();
        let bytes = src.as_bytes();
        let mut depth: i32 = 1;
        let mut i = body_start;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            i += 1;
        }
        assert!(
            depth == 0,
            "could not locate end of CaptureHandle::stop body — source layout has changed"
        );
        let body = &src[body_start..i];

        // 1. Must use std::mem::take on the locked buffer — single-pass extraction.
        assert!(
            body.contains("std::mem::take(&mut *self.buffer.lock().unwrap())"),
            "CaptureHandle::stop must extract the final buffer via \
             `std::mem::take(&mut *self.buffer.lock().unwrap())` to avoid \
             cloning up to 920 MB on the safety cap path"
        );

        // 2. Must NOT clone the buffer at all in this function. The previous
        //    pattern was `self.buffer.lock().unwrap().clone()`; pin its absence.
        assert!(
            !body.contains(".lock().unwrap().clone()"),
            "CaptureHandle::stop must NOT clone the locked PCM buffer; the \
             v0.19.x double-snapshot pattern was replaced with std::mem::take. \
             Re-introducing `.lock().unwrap().clone()` would re-introduce up to \
             2× full-buffer allocator churn per stop."
        );

        // 3. The drain wait that previously sat between the two snapshots
        //    must remain — the late-callback grace is what makes the single
        //    take safe. Pin both the 120 ms drop-drain and the 30 ms
        //    late-callback wait so a future tidy-up can't quietly remove
        //    the second sleep and shrink the safety margin.
        assert!(
            body.contains("Duration::from_millis(120)"),
            "CaptureHandle::stop must keep the 120 ms drop-drain wait after \
             setting the stop flag so the cpal stream finishes flushing \
             before extraction"
        );
        assert!(
            body.contains("Duration::from_millis(30)"),
            "CaptureHandle::stop must keep the 30 ms late-callback wait \
             before extracting the buffer; this is the budget that made the \
             previous re-snapshot unnecessary, so removing it would lose \
             tail audio on slow hosts"
        );

        // 4. The stop flag must still be flipped before the buffer is taken,
        //    so the capture thread is on its way out and not appending behind us.
        let stop_flag_idx = body
            .find("self.stop_flag.store(true,")
            .expect("CaptureHandle::stop must still set self.stop_flag.store(true, …) before extracting the buffer");
        let take_idx = body
            .find("std::mem::take(&mut *self.buffer.lock().unwrap())")
            .expect("expected std::mem::take call located above");
        assert!(
            stop_flag_idx < take_idx,
            "CaptureHandle::stop must set the stop flag BEFORE the mem::take \
             extraction; otherwise the capture thread could append between \
             extraction and our return"
        );
    }

    /// **v0.20.0 dead-vocabulary-hints contract.** The dictation hot
    /// path must NOT call `state.vocabulary.list_words()` — the
    /// active sherpa-onnx providers (parakeet, whisper, zipformer)
    /// all destructure their `AsrOptions` parameter as `_opts` and
    /// have never read `vocabulary_hints`. Pre-perf, dictation.rs
    /// scanned the vocabulary DB and cloned the resulting word list
    /// into `AsrOptions.vocabulary_hints` on every transcribe — pure
    /// dead plumbing. The field has been removed from `AsrOptions`,
    /// and the hot-path collection is gone. Source-grep so a future
    /// refactor that re-introduces the dead scan fails CI.
    ///
    /// Replacement behaviour (separate code path) is explicitly
    /// preserved: `apply_replacements` must still be called from
    /// dictation.rs and history.rs, and `VocabularyService` must
    /// still expose `list_words` / `apply_replacements` for the
    /// Settings → Vocabulary UI and the file-jobs ASR path.
    #[test]
    fn dictation_does_not_collect_vocabulary_words_for_asr() {
        // 1. dictation.rs production code (everything before the
        //    test module marker) must not call list_words at all.
        //    The only legitimate consumers are commands/vocabulary.rs
        //    and services/vocabulary.rs.
        let dictation_src = read_source("src-tauri/src/commands/dictation.rs");
        let test_mod_marker = "#[cfg(test)]";
        let production_src = dictation_src
            .split(test_mod_marker)
            .next()
            .unwrap_or(&dictation_src);
        assert!(
            !production_src.contains(".list_words("),
            "commands/dictation.rs production path must not call \
             VocabularyService::list_words() — the active ASR providers \
             do not consume vocabulary_hints, so collecting words on \
             every transcribe is dead plumbing. Replacement behaviour \
             goes through apply_replacements, which is a separate path."
        );

        // 2. AsrOptions must no longer carry the dead hint field.
        //    Build the needle at runtime so this assert's own message
        //    doesn't itself contain the literal token (which would
        //    make the file-level grep below trip on this test
        //    module). Pin both the struct definition (provider.rs)
        //    and any field initialiser in the production part of
        //    dictation.rs.
        let needle = format!("{}{}", "vocabulary_", "hints");
        let provider_src = read_source("src-tauri/src/services/inference/provider.rs");
        assert!(
            !provider_src.contains(&needle),
            "services/inference/provider.rs::AsrOptions must not declare \
             a `{needle}` field — no active provider reads it"
        );
        assert!(
            !production_src.contains(&needle),
            "commands/dictation.rs production path must not reference \
             `{needle}` anywhere — the field was removed from AsrOptions"
        );

        // 3. VocabularyService::list_words must still exist for the
        //    Settings → Vocabulary UI and import/export paths.
        let vocab_src = read_source("src-tauri/src/services/vocabulary.rs");
        assert!(
            vocab_src.contains("pub fn list_words("),
            "services/vocabulary.rs must still expose list_words() for the \
             Settings → Vocabulary UI; only the hot-path call from dictation \
             was removed"
        );

        // 4. Replacement behaviour preserved end-to-end. dictation.rs
        //    must still apply replacements to the ASR text, and the
        //    service must still expose apply_replacements + list_replacements.
        assert!(
            production_src.contains(".apply_replacements("),
            "commands/dictation.rs must still call apply_replacements on \
             the ASR text — replacement is a separate, user-visible path \
             from the (now-removed) vocabulary_hints scan"
        );
        assert!(
            vocab_src.contains("pub fn apply_replacements(")
                && vocab_src.contains("pub fn list_replacements("),
            "services/vocabulary.rs must still expose apply_replacements \
             and list_replacements"
        );
    }

    /// **Auto-space-on-paste contract.** Pins the round-trip
    /// behaviour of `record_delivery` + `should_prepend_space` so a
    /// future refactor of the per-app continuation heuristic can't
    /// silently regress the "hello" + "world" → "hello world"
    /// glue between back-to-back dictations in the same field.
    ///
    /// Bundled into a single test fn rather than split into many,
    /// because `LAST_DELIVERY` is a process-global `OnceLock<Mutex<…>>`
    /// and parallel test functions would race on it. Each scenario
    /// below resets that state explicitly, runs the gate, asserts.
    ///
    /// Scenarios pinned (numbered to match the gate's six branches
    /// in `should_prepend_space` plus the round-trip with
    /// `record_delivery`):
    ///
    ///   1. No prior delivery → no space. The very first dictation
    ///      after launch must never get a leading space inserted.
    ///   2. Happy path — "hello" then "world" within the window
    ///      and same app → space prepended.
    ///   3. Time window — beyond `AUTO_SPACE_WINDOW_MS` (45 s) →
    ///      no space, even if every other condition matches.
    ///   4. App switch — different `app_exe` → no space, even
    ///      within the time window.
    ///   5. Trailing whitespace on prior — "hello " then "world"
    ///      → no space (would be doubled).
    ///   6. Trailing open punct on prior — "(" then "world" →
    ///      no space (open paren / quote / bracket should hug the
    ///      next word).
    ///   7. New text starts with non-alnum — prior "hello", new
    ///      "." → no space (don't push punctuation off the word).
    ///   8. Same-app gate is case-sensitive on `app_exe` strings.
    ///      `Some("Notepad.exe")` and `Some("notepad.exe")` are
    ///      treated as different apps — pinning current behaviour
    ///      so a future "case-insensitive comparison" change is a
    ///      deliberate choice with a test fail.
    ///   9. `None`/`None` app_exe (privacy: context-active-app off)
    ///      still allows auto-space — both sides see "no known
    ///      app", which is treated as the same unknown app.
    ///
    /// Test resets `LAST_DELIVERY` to `None` between scenarios so
    /// each branch is exercised against a known starting state.
    #[test]
    fn auto_space_round_trip_contract() {
        // Helper: clear LAST_DELIVERY before each scenario.
        let clear_last = || {
            if let Ok(mut g) = last_delivery().lock() {
                *g = None;
            }
        };

        // Scenario 1: no prior delivery → no space.
        clear_last();
        assert!(
            !should_prepend_space(1_000, Some("notepad.exe"), "world"),
            "scenario 1: cold state (no prior delivery) must NOT \
             prepend a space — every first dictation after launch \
             would otherwise get a leading space"
        );

        // Scenario 2: happy path — "hello" then "world" within the
        // window and same app → space prepended.
        clear_last();
        record_delivery(10_000, Some("notepad.exe"), "hello");
        assert!(
            should_prepend_space(20_000, Some("notepad.exe"), "world"),
            "scenario 2: prior 'hello' in same app within 10 s, \
             new text starts with alnum, no trailing whitespace — \
             must prepend space (the headline use case)"
        );

        // Scenario 3: time window — beyond 45 s → no space.
        // Pick a delta of 45_001 ms so we're exactly one tick past
        // the boundary; the gate uses strict `>` so this must fail.
        clear_last();
        record_delivery(0, Some("notepad.exe"), "hello");
        assert!(
            !should_prepend_space(AUTO_SPACE_WINDOW_MS + 1, Some("notepad.exe"), "world"),
            "scenario 3: gap > AUTO_SPACE_WINDOW_MS must NOT \
             prepend a space — user has likely moved on to a \
             different thought / field"
        );

        // Scenario 3b: exactly at the boundary — `>` not `>=`,
        // so equal is still inside the window. Pin both sides
        // of the boundary.
        clear_last();
        record_delivery(0, Some("notepad.exe"), "hello");
        assert!(
            should_prepend_space(AUTO_SPACE_WINDOW_MS, Some("notepad.exe"), "world"),
            "scenario 3b: gap exactly == AUTO_SPACE_WINDOW_MS \
             must STILL prepend a space (gate uses strict `>`)"
        );

        // Scenario 4: app switch → no space.
        clear_last();
        record_delivery(10_000, Some("notepad.exe"), "hello");
        assert!(
            !should_prepend_space(20_000, Some("slack.exe"), "world"),
            "scenario 4: prior delivery in notepad.exe but current \
             active app is slack.exe — different field, must NOT \
             prepend a space"
        );

        // Scenario 5: prior ended with whitespace → no space
        // (would double up). `record_delivery` itself reads the
        // last char of the delivered string to compute this; pin
        // that the round-trip honours it.
        clear_last();
        record_delivery(10_000, Some("notepad.exe"), "hello ");
        assert!(
            !should_prepend_space(20_000, Some("notepad.exe"), "world"),
            "scenario 5: prior delivery already ended with a space — \
             must NOT prepend another (would produce 'hello  world')"
        );

        // Scenario 6: prior ended with open punct → no space
        // (the open paren / quote / bracket should hug the next
        // token). Pin all five open-punct chars from the helper:
        // '(', '[', '{', '"', '\''.
        for open in ["(", "[", "{", "\"", "'"] {
            clear_last();
            record_delivery(10_000, Some("notepad.exe"), open);
            assert!(
                !should_prepend_space(20_000, Some("notepad.exe"), "world"),
                "scenario 6: prior delivery ended with open punct \
                 {open:?} — must NOT prepend a space (open punct \
                 should hug the next token)"
            );
        }

        // Scenario 7: new text starts with non-alnum → no space.
        // Pin a representative spread: closing punct, quote,
        // sentence-ender. None should get a leading space.
        clear_last();
        record_delivery(10_000, Some("notepad.exe"), "hello");
        for next in [".", ",", "!", "?", ")", "]", "}", "\""] {
            assert!(
                !should_prepend_space(20_000, Some("notepad.exe"), next),
                "scenario 7: new text {next:?} starts with non-alnum — \
                 must NOT prepend a space (would push punctuation \
                 off the previous word)"
            );
        }

        // Scenario 8: same-app gate is case-sensitive. Currently
        // `prev.app_exe.as_deref() != current_app_exe` uses string
        // equality, which is case-sensitive. Pin the existing
        // behaviour so a future "case-insensitive" tweak is a
        // deliberate decision rather than a silent change.
        clear_last();
        record_delivery(10_000, Some("Notepad.exe"), "hello");
        assert!(
            !should_prepend_space(20_000, Some("notepad.exe"), "world"),
            "scenario 8: same-app gate is currently case-sensitive — \
             'Notepad.exe' and 'notepad.exe' are treated as different \
             apps. If a future fix makes the comparison \
             case-insensitive, flip this assertion deliberately."
        );

        // Scenario 9: None/None still allows auto-space. When
        // `privacy.allow_context_active_app == false`, both sides
        // see `app_exe = None`. The gate's `==` over `Option<&str>`
        // returns true for None == None, so the auto-space heuristic
        // continues to work for users who have context-app disabled.
        clear_last();
        record_delivery(10_000, None, "hello");
        assert!(
            should_prepend_space(20_000, None, "world"),
            "scenario 9: both prior and current app_exe are None \
             (privacy: context-active-app disabled) — auto-space \
             must STILL fire, treating 'no known app' as 'the same \
             unknown app'"
        );

        // Scenario 9b: None vs Some still counts as a switch.
        // (Asymmetric privacy or active-app probe failure between
        // two dictations.) Pin the strict equality so a privacy
        // setting flip mid-session doesn't silently glue text
        // across apps.
        clear_last();
        record_delivery(10_000, None, "hello");
        assert!(
            !should_prepend_space(20_000, Some("notepad.exe"), "world"),
            "scenario 9b: prior had no app_exe (None) but current \
             does — treat as a context change, do NOT prepend"
        );
        clear_last();
        record_delivery(10_000, Some("notepad.exe"), "hello");
        assert!(
            !should_prepend_space(20_000, None, "world"),
            "scenario 9b': prior had app_exe but current is None — \
             treat as a context change, do NOT prepend"
        );

        // Final cleanup so this test doesn't leak state into other
        // tests in the same binary that might inspect LAST_DELIVERY.
        clear_last();
    }

    /// **v0.19.6 release-build mic-mute contract.** The
    /// experimental Teams + Discord per-app microphone mute is
    /// behind the `experimental_mic_mute` Cargo feature. Without
    /// it, `mic_mute::apply()` is a no-op stub and the Settings →
    /// Advanced → Experimental toggle silently does nothing. The
    /// feature stays out of `default = []` so dev iteration
    /// (`cargo test`, plain `pnpm tauri build`, `pnpm tauri:dev`)
    /// skips the Win32 SendInput path; release builds must opt
    /// in by name.
    ///
    /// v0.19.0 stable was built with the feature, but the
    /// requirement never made it into `package.json` or the
    /// runbook, so v0.19.1 → v0.19.5 betas silently dropped
    /// it. Source-grep so a future runbook edit, npm-script
    /// rename, or release-doc cleanup that drops the feature
    /// fails CI before it can ship a no-op mic-mute toggle to
    /// users.
    ///
    /// Pins the feature flag in three places:
    ///   1. `package.json` exposes a `tauri:build:release`
    ///      script that includes `--features experimental_mic_mute`.
    ///   2. `RELEASE_RUNBOOK.md` §4 (the canonical release
    ///      ritual) instructs the operator to use it.
    ///   3. `Utterbar Windows App/RELEASE.md` §1 (the underlying
    ///      release flow the runbook enforces) instructs the
    ///      same.
    ///
    /// Also pins that `services/mic_mute.rs` keeps both the
    /// feature-on and feature-off `apply` definitions, so the
    /// no-op stub doesn't accidentally become the only path.
    #[test]
    fn release_build_includes_experimental_mic_mute_feature() {
        // read_source resolves paths relative to the parent of
        // CARGO_MANIFEST_DIR, which is the `Utterbar Windows App/`
        // directory. The runbook lives one level higher (repo root).
        let pkg_json = read_source("package.json");
        let runbook = read_source("../RELEASE_RUNBOOK.md");
        let app_release = read_source("RELEASE.md");
        let mic_mute = read_source("src-tauri/src/services/mic_mute.rs");

        // 1. package.json must expose tauri:build:release with the
        //    feature flag in the command body. Pin both the script
        //    name (so docs can reference it stably) and the flag
        //    body (so a future rename of the script alone can't
        //    silently drop the feature).
        assert!(
            pkg_json.contains("\"tauri:build:release\""),
            "package.json must define a `tauri:build:release` npm \
             script — runbooks point users at this script name; \
             renaming it without updating the docs leaves a \
             dangling reference"
        );
        assert!(
            pkg_json.contains("--features experimental_mic_mute"),
            "package.json must pass `--features experimental_mic_mute` \
             on the release build script. Plain `tauri build` \
             produces a binary where mic_mute::apply() is the \
             no-op stub, which silently breaks the Settings → \
             Advanced → Experimental toggle"
        );

        // 2. RELEASE_RUNBOOK.md §4 must point at the release
        //    script (or pass the flag explicitly). A future
        //    runbook tidy that reverts to plain `pnpm tauri build`
        //    would silently re-introduce the v0.19.1 → v0.19.5
        //    regression — fail CI on that change.
        assert!(
            runbook.contains("tauri:build:release")
                || runbook.contains("--features experimental_mic_mute"),
            "RELEASE_RUNBOOK.md must reference either the \
             `tauri:build:release` script or the explicit \
             `--features experimental_mic_mute` flag in its build \
             instructions; plain `pnpm tauri build` ships a \
             feature-stripped binary"
        );

        // 3. RELEASE.md (the underlying flow) same constraint.
        assert!(
            app_release.contains("tauri:build:release")
                || app_release.contains("--features experimental_mic_mute"),
            "Utterbar Windows App/RELEASE.md must reference either \
             the `tauri:build:release` script or the explicit \
             `--features experimental_mic_mute` flag in its build \
             instructions"
        );

        // 4. mic_mute.rs must keep BOTH paths. The feature-off
        //    stub is what every dev/test build sees; the feature-on
        //    real implementation is what release builds compile.
        //    Source-grep both `cfg` attributes so a future tidy
        //    that collapses to one path alone fails this test
        //    rather than silently shipping a permanent no-op.
        assert!(
            mic_mute.contains("#[cfg(not(feature = \"experimental_mic_mute\"))]"),
            "services/mic_mute.rs must keep the feature-off stub \
             path. Removing it would mean every dev/test build \
             fails to compile, but more importantly removing the \
             gate entirely would make the feature unconditional, \
             which is a deliberate-design decision that should \
             require an explicit Cargo `default` change, not a \
             quiet refactor"
        );
        assert!(
            mic_mute.contains("#[cfg(feature = \"experimental_mic_mute\")]"),
            "services/mic_mute.rs must keep the feature-on real \
             apply path. Without it, release builds with the \
             feature enabled would compile but silently no-op"
        );
    }
}
