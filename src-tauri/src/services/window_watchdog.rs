//! Floating-bar watchdog.
//!
//! The floating pill is a frameless, skipTaskbar=true, alwaysOnTop=true window. On
//! Windows, all three together make the pill uniquely easy to lose when something
//! system-level tells every top-level window to get out of the way:
//!
//!   * Win+D  / Win+M  — shell sends SC_MINIMIZE to everything. The `minimizable:false`
//!     Tauri flag only removes the caption-button affordance; it does NOT block the
//!     WM_SYSCOMMAND, so the pill ends up minimised and invisible (no taskbar entry to
//!     click on, either).
//!   * UAC prompt      — secure desktop hides all non-elevated windows, and their
//!     z-order isn't always fully restored on return.
//!   * Virtual-desktop swap — AOT windows sometimes stay pinned to the desktop they
//!     were created on instead of following the active one.
//!   * Display change  — unplugging a monitor or a DPI shift can strand the window
//!     at a position that's technically "valid" but behind the shell.
//!
//! The pragmatic fix is a 2 Hz repair loop. It's cheap (two syscalls per tick when
//! things are healthy, a few more when they're not) and doesn't need a wndproc
//! subclass.  We only repair when the user has `bar_visibility == Always` — the
//! WhileRecording and Hidden modes legitimately call `hide()` between phases and
//! the watchdog must not fight the frontend there.

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::app_state::AppState;
use crate::domain::settings::BarVisibility;

/// Spawn the watchdog as a background tokio task. Called once from the startup
/// hook. The task loops for the process lifetime; shutdown is implicit when the
/// tokio runtime tears down on exit.
pub fn spawn(handle: AppHandle, state: Arc<AppState>) {
    tokio::spawn(async move {
        // 2 Hz. Faster felt jumpy in testing — the pill would flicker back in a
        // frame or two after Win+D and the animation fighting Windows' own minimise
        // animation produced visible tearing. Slower (1 Hz) made "lost" feel
        // lingering. 500 ms is the sweet spot.
        let mut ticker = tokio::time::interval(Duration::from_millis(500));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            let visibility = state.settings.snapshot().await.general.bar_visibility;
            if !matches!(visibility, BarVisibility::Always) {
                // WhileRecording / Hidden — leave well alone.
                continue;
            }

            let Some(w) = handle.get_webview_window("floating") else {
                continue;
            };

            // Repair sequence, in order:
            //   1. If minimised, unminimise. Almost always the culprit.
            //   2. If not visible, show.
            //   3. Re-assert always-on-top. Cheap idempotent call; guards against
            //      any code path that silently cleared the flag (some older Tauri
            //      window APIs could).
            let was_minimized = w.is_minimized().unwrap_or(false);
            if was_minimized {
                if let Err(e) = w.unminimize() {
                    tracing::debug!(?e, "watchdog: unminimize failed");
                }
            }

            let visible = w.is_visible().unwrap_or(true);
            if !visible {
                if let Err(e) = w.show() {
                    tracing::debug!(?e, "watchdog: show failed");
                }
            }

            if was_minimized || !visible {
                // Only re-assert AOT when we just repaired something. Calling it
                // every tick is wasteful and can steal focus from the active app.
                let _ = w.set_always_on_top(true);
                tracing::info!(
                    was_minimized,
                    was_hidden = !visible,
                    "watchdog repaired floating bar"
                );
            }
        }
    });
}
