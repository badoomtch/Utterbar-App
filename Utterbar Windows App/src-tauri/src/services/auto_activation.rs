//! Per-app auto-activation watcher.
//!
//! Polls the foreground-window signals every ~750ms. When the window
//! changes (exe/title/class differs from the last seen), runs the
//! `match_auto_activation` matcher against the user's mode rules. If a
//! mode wins and is different from the currently-active mode, switches
//! to it — emitting a `mode_changed` event so the UI updates.
//!
//! ## Design choices
//!
//! - **Poll, not WinEventHook.** A `SetWinEventHook` listener is event-
//!   driven and tighter, but it requires a message pump on the calling
//!   thread and a careful unhook-on-drop dance. The Utterbar foreground
//!   capture path already proves polling at this cadence is fine for
//!   user-perceived responsiveness; the simpler model wins until we
//!   have a reason to upgrade.
//!
//! - **Idle-only switches.** We never change the active mode while the
//!   user is actively recording or mid-transcribe. A queued switch
//!   waits for the next idle tick. This avoids the worst-case where the
//!   user starts dictating, alt-tabs to read a reference, and the mode
//!   silently changes mid-utterance. If a user switched while
//!   recording, the post-processing pipeline that fired might use
//!   different settings than the dictation surface implies — confusing.
//!
//! - **Debouncing.** If the foreground window changes rapidly (alt-tab
//!   spam, briefly-focused popups), we still only switch when the
//!   matched mode differs from the currently active one. A user who
//!   moves between Slack and a browser, both using the same matched
//!   mode, sees no flapping.
//!
//! - **Precedence (deterministic).** First match wins, scanning modes
//!   in user-defined `sort_order`. Within a mode, rules are scanned in
//!   array order. Documented in `ModeService::match_auto_activation`.
//!
//! - **Disabled when no rules.** If no mode has any auto-activation
//!   rules, the watcher does the cheap exe/title/class read and bails.
//!   No DB queries, no settings writes — effectively free.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::app_state::AppState;

/// How often to check the foreground window. 750ms is a deliberate
/// floor — fast enough to feel reactive when the user alt-tabs, slow
/// enough that 80 polls/min on a quiet machine is invisible CPU. The
/// `GetForegroundWindow + GetWindowTextW + GetClassNameW` triad costs a
/// few dozen microseconds; we're well below the noise floor.
const POLL_INTERVAL: Duration = Duration::from_millis(750);

/// Spawn the auto-activation watcher. Runs for the lifetime of the
/// process; `JoinHandle` is dropped because there's no shutdown path
/// that needs to await it (Tokio aborts background tasks on runtime
/// shutdown).
pub fn spawn(handle: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        watch_loop(handle, state).await;
    });
}

async fn watch_loop(handle: AppHandle, state: Arc<AppState>) {
    let mut last_signal: Option<(Option<String>, Option<String>, Option<String>)> = None;
    let mut last_switched_to: Option<String> = None;

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        // 1. Hold off if the user is recording or transcribing. We
        //    don't change modes mid-utterance — the post-processing
        //    pipeline already chose its mode at recording start.
        if state.recording_in_progress.load(Ordering::Relaxed)
            || state.transcribe_in_progress.load(Ordering::Relaxed)
        {
            continue;
        }

        // 2. Snapshot the foreground window. Cheap call on Windows;
        //    a no-op default on other platforms.
        let ctx = state.context.snapshot(false, false);
        let signal = (ctx.exe.clone(), ctx.title.clone(), ctx.class.clone());

        // 3. Window unchanged since last poll → nothing to do.
        if last_signal.as_ref() == Some(&signal) {
            continue;
        }
        last_signal = Some(signal.clone());

        // 4. Match against the user's mode rules.
        let matched = match state.modes.match_auto_activation(
            ctx.exe.as_deref(),
            ctx.title.as_deref(),
            ctx.class.as_deref(),
        ) {
            Ok(Some(m)) => m,
            Ok(None) => continue,
            Err(e) => {
                tracing::warn!(error = ?e, "auto-activation match failed");
                continue;
            }
        };

        // 5. Skip if it's already the active mode. Saves a settings
        //    write + a UI event on every alt-tab between two windows
        //    that share the same matched mode.
        let snapshot = state.settings.snapshot().await;
        let current_active = snapshot.active_mode_id.clone();
        if current_active.as_deref() == Some(matched.id.as_str()) {
            last_switched_to = Some(matched.id.clone());
            continue;
        }

        // 6. Skip if WE just switched to this mode and the user has
        //    since switched away manually. Without this check, the
        //    watcher would yank the user back to the auto-activated
        //    mode every poll until they alt-tabbed. With it, a manual
        //    switch is sticky for the current foreground window — the
        //    next foreground change is what re-arms auto-activation.
        if last_switched_to.as_deref() == Some(matched.id.as_str())
            && current_active != Some(matched.id.clone())
        {
            // User overrode our last decision. Don't fight them.
            continue;
        }

        // 7. Switch. Persist via SettingsStore so the UI listener
        //    fires the `mode_changed` event on its own; we don't need
        //    to emit anything bespoke.
        let new_id = matched.id.clone();
        let label = matched.name.clone();
        if let Err(e) = state
            .settings
            .update(|s| {
                s.active_mode_id = Some(new_id.clone());
            })
            .await
        {
            tracing::warn!(error = ?e, "auto-activation: failed to persist active mode");
            continue;
        }

        last_switched_to = Some(matched.id.clone());

        // Diagnostic: log the auto-switch event with enough context
        // to debug "why did my mode change?" reports without exposing
        // PII (we log the exe basename, not the full path or title).
        let exe_basename = ctx
            .exe
            .as_deref()
            .and_then(|p| std::path::Path::new(p).file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        tracing::info!(
            mode = %label,
            mode_id = %matched.id,
            triggered_by = %exe_basename,
            "auto-activated mode"
        );

        // Frontend listens to settings_changed already; emit a
        // dedicated event too so a future toast can surface "Switched
        // to <mode> for <app>" without polling the settings store.
        let _ = handle.emit(
            "mode:auto_activated",
            AutoActivatedPayload {
                mode_id: matched.id,
                mode_name: matched.name,
                triggered_exe: ctx.exe,
            },
        );
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct AutoActivatedPayload {
    mode_id: String,
    mode_name: String,
    triggered_exe: Option<String>,
}

#[cfg(test)]
mod tests {
    // The watcher itself is tested by integration; the matcher logic
    // it depends on (`ModeService::match_auto_activation` and the
    // `glob` helper) has its own coverage in services/modes.rs.
    //
    // The interesting decision boundaries here — (a) skip when
    // recording, (b) skip if window unchanged, (c) skip if matched
    // mode is already active, (d) sticky after manual override —
    // are all simple branches over external state and would require
    // mocking five service Arcs to exercise. Punted in favour of
    // beta-channel runtime testing.
    #[test]
    fn signal_tuple_eq() {
        // Sanity: tuple-of-Option<String> equality behaves as we
        // expect — None/None match, identical Some/Some match,
        // any difference doesn't.
        type Sig = (Option<String>, Option<String>, Option<String>);
        let a: Sig = (
            Some("notepad.exe".to_string()),
            Some("Untitled".to_string()),
            None,
        );
        let b: Sig = (
            Some("notepad.exe".to_string()),
            Some("Untitled".to_string()),
            None,
        );
        assert_eq!(a, b);
        let c: Sig = (
            Some("notepad.exe".to_string()),
            Some("file.txt".to_string()),
            None,
        );
        assert_ne!(a, c);
    }
}
