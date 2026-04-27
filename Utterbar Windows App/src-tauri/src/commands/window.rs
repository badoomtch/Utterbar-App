use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::app_state::AppState;
use crate::domain::events::EVT_MAIN_NAVIGATE;
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn show_floating(handle: AppHandle) -> CmdResult<()> {
    if let Some(w) = handle.get_webview_window("floating") {
        // Belt-and-braces: unminimise, clamp *before* show so the bar can't
        // materialise off-screen even for one frame, then re-enforce always-
        // on-top (certain shell events — Win+D, display rearrange — can demote
        // us, and a z-buried bar looks identical to a missing bar to users).
        let _ = w.unminimize();
        if let Err(e) = clamp_to_work_area(&w) {
            tracing::warn!(?e, "show_floating: pre-show clamp failed");
        }
        w.show().map_err(|e| OrliqError::Internal {
            message: e.to_string(),
        })?;
        let _ = w.set_always_on_top(true);
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_floating(handle: AppHandle) -> CmdResult<()> {
    if let Some(w) = handle.get_webview_window("floating") {
        w.hide().map_err(|e| OrliqError::Internal {
            message: e.to_string(),
        })?;
    }
    Ok(())
}

#[tauri::command]
pub async fn show_main(handle: AppHandle) -> CmdResult<()> {
    match handle.get_webview_window("main") {
        Some(w) => {
            // Belt and braces: unminimise in case it was minimised, then show + focus.
            let _ = w.unminimize();
            w.show().map_err(|e| OrliqError::Internal {
                message: e.to_string(),
            })?;
            let _ = w.set_focus();
            tracing::info!("main window shown");
            Ok(())
        }
        None => {
            tracing::error!("show_main: main window handle missing");
            Err(OrliqError::Internal {
                message: "Main window not found. Please restart Utterbar.".into(),
            })
        }
    }
}

/// Show the main window and ask the frontend to navigate to `path`.
///
/// The floating bar's History / Settings buttons call this so the main window
/// always lands on the page the user asked for — previously they shared a plain
/// `show_main` which just surfaced whatever route was last open, which felt
/// broken ("I clicked History, why am I on Dashboard?").
///
/// The navigation intent is delivered via the `main:navigate` event rather
/// than a URL param so the React Router history stays clean — no extra junk
/// entries from opening the bar repeatedly.
#[tauri::command]
pub async fn show_main_at(handle: AppHandle, path: String) -> CmdResult<()> {
    match handle.get_webview_window("main") {
        Some(w) => {
            let _ = w.unminimize();
            w.show().map_err(|e| OrliqError::Internal {
                message: e.to_string(),
            })?;
            let _ = w.set_focus();
            // Emit after show() so MainShell's listener is guaranteed to be
            // mounted — React Router is ready by the time the window is shown.
            let _ = handle.emit(EVT_MAIN_NAVIGATE, &path);
            tracing::info!(target = %path, "main window shown + navigated");
            Ok(())
        }
        None => {
            tracing::error!("show_main_at: main window handle missing");
            Err(OrliqError::Internal {
                message: "Main window not found. Please restart Utterbar.".into(),
            })
        }
    }
}

#[tauri::command]
pub async fn show_onboarding(handle: AppHandle) -> CmdResult<()> {
    if let Some(w) = handle.get_webview_window("onboarding") {
        w.show().map_err(|e| OrliqError::Internal {
            message: e.to_string(),
        })?;
        w.set_focus().ok();
    }
    Ok(())
}

#[tauri::command]
pub async fn set_floating_position(handle: AppHandle, x: i32, y: i32) -> CmdResult<()> {
    if let Some(w) = handle.get_webview_window("floating") {
        w.set_position(tauri::PhysicalPosition::new(x, y))
            .map_err(|e| OrliqError::Internal {
                message: e.to_string(),
            })?;
    }
    Ok(())
}

/// Pull the floating bar back into the current monitor's visible area. Called after
/// any drag (via the window's `onMoved` listener) so a user can't strand the pill
/// behind the Windows taskbar or off-screen entirely. Also surfaced as a tray menu
/// entry and a Settings → Utterbar button for when the bar has already gone missing.
// v0.6.6 crash-investigation experiment: removed `async` on this command.
//
// `respond_async_serialized_inner` (seen in every v0.6.4/v0.6.5 crash
// stack) is the tokio-worker path that drops an `AppHandle` clone on a
// non-main thread on command completion. AppHandle carries an
// `Arc<Runtime::Context>` whose inner type in tauri-runtime-wry 2.10.x
// is NOT actually `Send` (it holds `Rc<EventLoopRunner>`), even though
// Tauri unsafely impls `Send` on the outer wrapper.
//
// `clamp_floating_position` is fired on every `onMoved` event from the
// FloatingBar drag handler — dozens of times per second during a drag.
// That's the single highest-frequency async-command path in the app.
// Making it sync routes it through Tauri's synchronous IPC path which
// does NOT go through `respond_async_serialized_inner` — the command
// runs on the main UI thread and returns, no tokio task involved.
//
// This is a narrow, revertible experiment: if the crash stops recurring
// after this change, our code was the amplifier. If it continues, the
// Tauri built-in window plugin (also visible in the stack as
// `window::plugin::init::closure$72`) is the independent culprit and
// we need the `tauri-runtime-wry` fork.
#[tauri::command]
pub fn clamp_floating_position(handle: AppHandle) -> CmdResult<()> {
    if let Some(w) = handle.get_webview_window("floating") {
        if let Err(e) = clamp_to_work_area(&w) {
            tracing::warn!(?e, "clamp_floating_position failed");
        }
        // v0.6.7 persist-on-move. `clamp_floating_position` is the
        // natural "the user stopped dragging" signal (FloatingBar.tsx
        // debounces onMoved and invokes this 200ms after the last
        // move). We piggy-back on that to persist the new position —
        // no extra round-trip needed, and the persist fires on the
        // same terminal-drag event the clamp does. Post-clamp position
        // is authoritative: if the drop would have stranded the bar
        // off-screen, clamp already corrected it and we save the
        // corrected coords, not the stranded ones. Best-effort; any
        // disk write failure logs a warning and is ignored.
        if let Err(e) = save_floating_position(&w) {
            tracing::warn!(?e, "save_floating_position failed");
        }
    }
    Ok(())
}

/// Snap the floating bar back to a known-good spot: top-centre of the PRIMARY
/// monitor, 8 px below the top edge. Use when the bar has already wandered out
/// of reach — this is the "I can't find my Utterbar" escape hatch wired to the
/// tray and the Settings → Utterbar page.
///
/// We deliberately target `primary_monitor` rather than `current_monitor` here:
/// if a user is invoking reset, the current position is presumptively broken
/// (off-screen on a disconnected display, dragged past the edge of a wide
/// monitor, DPI flip stranded it, etc.), and Tauri's `current_monitor` can
/// still happily report that broken monitor as "current". Primary is the one
/// monitor we can trust to be visible right now. Size and minimised state are
/// also force-reset so a corrupted outer_size or a Win+D cycle can't leave the
/// pill invisible after the move.
#[tauri::command]
pub async fn reset_floating_position(handle: AppHandle) -> CmdResult<()> {
    if let Some(w) = handle.get_webview_window("floating") {
        // Belt-and-braces: unminimise first in case the window was minimised,
        // then pin to the default logical size so a stale outer_size can't skew
        // the centring maths below.
        let _ = w.unminimize();
        // Must match tauri.conf.json floating window dimensions, otherwise
        // reset leaves the bar undersized (clipping the expanded right half)
        // or oversized (punching past the screen edge the clamp then fights).
        let _ = w.set_size(tauri::LogicalSize::new(520.0, 64.0));
        if let Err(e) = reset_to_primary_top_centre(&w) {
            tracing::warn!(?e, "reset_floating_position failed");
        }
        // v0.6.7: reset is the user's explicit "start over" — wipe the
        // persisted position so the next launch doesn't resurrect the
        // broken location. The user can still drag the bar to a
        // preferred spot after the reset and that new position will be
        // saved by the next clamp_floating_position call.
        clear_saved_floating_position();
        // If the bar was behind the taskbar it was effectively invisible; a
        // focused show() after the move guarantees it redraws on top.
        let _ = w.show();
        let _ = w.set_always_on_top(true);
        let _ = w.set_focus();
    }
    Ok(())
}

fn resolve_monitor(w: &tauri::WebviewWindow) -> Option<tauri::Monitor> {
    // `current_monitor` can be None when the window is entirely off-screen (e.g.
    // previously dragged onto a monitor that's since been unplugged). Falling
    // back to the primary monitor gives us a sane target in that case.
    w.current_monitor()
        .ok()
        .flatten()
        .or_else(|| w.primary_monitor().ok().flatten())
}

fn clamp_to_work_area(w: &tauri::WebviewWindow) -> anyhow::Result<()> {
    // Prefer the real Windows work-area (monitor minus taskbar and any docked
    // app bars). The old 80 px hardcode caused the bar to "ping-pong" because
    // the clamp would shove it 80 px above the taskbar even when the user had
    // placed it flush against the taskbar edge. With the real rect the bar
    // settles wherever the user drops it, as long as that's inside the visible
    // rectangle — no more bounce.
    let (min_x, min_y, max_right, max_bottom) = match work_area(w) {
        Some(rect) => rect,
        None => {
            // Fallback: whole monitor with no inset. If we can't query the OS
            // we'd rather let the bar sit flush against any edge than refuse
            // to clamp at all.
            let monitor = resolve_monitor(w).ok_or_else(|| anyhow::anyhow!("no monitor"))?;
            let mp = monitor.position();
            let ms = monitor.size();
            (mp.x, mp.y, mp.x + ms.width as i32, mp.y + ms.height as i32)
        }
    };
    let wp = w.outer_position()?;
    let ws = w.outer_size()?;
    let max_x = (max_right - ws.width as i32).max(min_x);
    let max_y = (max_bottom - ws.height as i32).max(min_y);
    let cx = wp.x.clamp(min_x, max_x);
    let cy = wp.y.clamp(min_y, max_y);
    if cx != wp.x || cy != wp.y {
        w.set_position(tauri::PhysicalPosition::new(cx, cy))?;
    }
    Ok(())
}

/// Query the real Windows work-area (`rcWork`) for the monitor containing the
/// given window. This is the screen rectangle after the taskbar and any
/// always-on-top app bars are subtracted — i.e. exactly the region the bar is
/// allowed to occupy without being z-order-buried beneath the shell.
#[cfg(windows)]
fn work_area(w: &tauri::WebviewWindow) -> Option<(i32, i32, i32, i32)> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    let raw = w.hwnd().ok()?;
    let hwnd = HWND(raw.0 as *mut _);
    unsafe {
        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmon, &mut info).as_bool() {
            Some((
                info.rcWork.left,
                info.rcWork.top,
                info.rcWork.right,
                info.rcWork.bottom,
            ))
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
fn work_area(_w: &tauri::WebviewWindow) -> Option<(i32, i32, i32, i32)> {
    None
}

fn reset_to_primary_top_centre(w: &tauri::WebviewWindow) -> anyhow::Result<()> {
    // Always target the *primary* monitor — see `reset_floating_position` for
    // why we ignore current_monitor here.
    let monitor = w
        .primary_monitor()?
        .or_else(|| resolve_monitor(w))
        .ok_or_else(|| anyhow::anyhow!("no monitor"))?;
    let mp = monitor.position();
    let ms = monitor.size();
    // outer_size can be stale or zero if the window never rendered on this
    // session; fall back to the factory default so centring still produces a
    // sensible x, rather than dumping the bar flush against the left edge.
    let ws = w.outer_size().unwrap_or(tauri::PhysicalSize::new(520, 64));
    // Top-centre of the monitor, matching the factory default in
    // tauri.conf.json — the spot users expect the bar to be on first run.
    let x = mp.x + ((ms.width as i32 - ws.width as i32) / 2).max(0);
    let y = mp.y + 8;
    w.set_position(tauri::PhysicalPosition::new(x, y))?;
    Ok(())
}

/// Toggle click-passthrough for the floating bar window. When `passthrough` is true,
/// the OS routes cursor events through the transparent parts of the floating window
/// to whatever's underneath — so the user can click icons / focus other apps through
/// the 500-ish pixels of empty space either side of the collapsed pill.
///
/// The frontend drives this from a cheap cursor-position poll (see `get_cursor_pos`
/// below): as the cursor leaves the pill's bounding rect, we flip passthrough on;
/// as it re-enters, we flip it off so the pill itself is clickable again.
///
/// Without this the floating window (520 × 64 px) acted as a dead zone across the
/// entire top-centre of the desktop — users reported clicks on taskbar items or
/// background windows that sat under the bar simply not registering.
#[tauri::command]
pub async fn set_floating_passthrough(handle: AppHandle, passthrough: bool) -> CmdResult<()> {
    if let Some(w) = handle.get_webview_window("floating") {
        w.set_ignore_cursor_events(passthrough)
            .map_err(|e| OrliqError::Internal {
                message: e.to_string(),
            })?;
    }
    Ok(())
}

/// Return the current cursor position in physical screen coordinates. The floating
/// bar polls this at ~30 Hz to decide whether its window should be passing clicks
/// through (cursor outside the pill rect) or accepting them (cursor inside).
///
/// The Windows `GetCursorPos` result is already in physical pixels, which matches
/// Tauri's `outer_position()` — no DPI fiddling required at the call site.
/// Non-Windows builds return (0, 0) so the JS caller degrades to "never
/// passthrough", which is strictly safer than "always passthrough".
#[tauri::command]
pub async fn get_cursor_pos() -> CmdResult<(i32, i32)> {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        let mut p = POINT::default();
        unsafe {
            if GetCursorPos(&mut p).is_ok() {
                return Ok((p.x, p.y));
            }
        }
    }
    Ok((0, 0))
}

/// Atomically finish onboarding: mark the settings flag, hide the onboarding window,
/// and show the floating bar. Doing this in Rust guarantees an ordering the frontend
/// can't break across window contexts.
#[tauri::command]
pub async fn finish_onboarding(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> CmdResult<()> {
    state
        .settings
        .update(|s| s.onboarding_completed = true)
        .await
        .map_err(OrliqError::from)?;
    if let Some(w) = handle.get_webview_window("floating") {
        let _ = w.show();
    }
    if let Some(w) = handle.get_webview_window("onboarding") {
        let _ = w.hide();
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// v0.6.7 floating-bar position persistence
//
// The bar now reopens at the user's last position rather than always snapping
// to top-centre on launch. Pattern:
//   1. After every drag (signalled by the debounced onMoved → clamp call in
//      FloatingBar.tsx), `clamp_floating_position` persists the current
//      outer_position to `%APPDATA%\Utterbar\config\floating_position.json`.
//   2. On launch, `restore_floating_position` reads that file and validates
//      the saved coords against the current set of monitor work-areas:
//       - position must fit inside some monitor's work-area rect (i.e. the
//         saved monitor is still connected at that resolution)
//       - if it doesn't, we fall through and let Tauri's config default
//         (top-centre of primary, per tauri.conf.json) apply.
//   3. `reset_floating_position` (tray → Reset + Settings → Utterbar)
//      deletes the saved file so the user's next launch starts clean.
// -----------------------------------------------------------------------------

/// On-disk shape. Physical pixels (what Tauri returns from outer_position()).
/// Deliberately simple — we don't persist monitor id because monitor ids
/// change across Windows sessions and aren't reliable keys.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct PersistedFloatingPosition {
    x: i32,
    y: i32,
    /// Schema version. Bump if the shape ever changes; on mismatch we
    /// treat the file as absent and fall through to the default position.
    #[serde(default = "default_schema_v")]
    v: u32,
}

fn default_schema_v() -> u32 {
    1
}

fn position_file_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(crate::util::paths::config_dir()?.join("floating_position.json"))
}

/// Write the current outer_position of the floating window to disk. Best-
/// effort: any failure is logged by the caller and swallowed so a disk
/// hiccup never blocks the drag UX.
fn save_floating_position(w: &tauri::WebviewWindow) -> anyhow::Result<()> {
    let p = w.outer_position()?;
    let path = position_file_path()?;
    let rec = PersistedFloatingPosition {
        x: p.x,
        y: p.y,
        v: 1,
    };
    let json = serde_json::to_string(&rec)?;
    std::fs::write(&path, json)?;
    // v0.6.7 diagnostic: log the saved position so the persist-on-drag
    // path is visible in the log. Without this, a "bar doesn't remember
    // its position" bug is invisible — we have no signal showing whether
    // the save fired at all vs. fired with the wrong coords. Cheap
    // (once per drag-end, not per move event).
    tracing::info!(x = p.x, y = p.y, "saved floating bar position");
    Ok(())
}

/// Load + validate the persisted position, returning Some(position) only
/// when it fits inside a currently-attached monitor's **taskbar-aware
/// work area**. None means "no file, bad file, or saved position is now
/// off-screen or behind the taskbar" — all three cases should fall
/// through to the Tauri default.
///
/// v0.6.7 fix: the first version of this function validated against
/// each monitor's full screen rect, which accepted saved coords inside
/// the monitor but under the taskbar. That meant the bar briefly
/// appeared in taskbar-occluded space at startup and then snapped out
/// of it once the frontend's first onMoved fired. Now we query the
/// real Windows work-area for each attached monitor (same rect
/// `clamp_to_work_area` uses during normal drag) and reject any saved
/// position that lands outside that rectangle.
fn load_valid_floating_position(w: &tauri::WebviewWindow) -> anyhow::Result<Option<(i32, i32)>> {
    let path = position_file_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(&path)?;
    let rec: PersistedFloatingPosition = match serde_json::from_str(&json) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(?e, "floating_position.json malformed; ignoring");
            return Ok(None);
        }
    };
    if rec.v != 1 {
        return Ok(None);
    }
    // Bar size bounds for the intersects-any-monitor check. We use the
    // configured size from tauri.conf.json (520×64) rather than the live
    // window's size, because the live size may not be set yet when this
    // runs at startup.
    let bar_w: i32 = 520;
    let bar_h: i32 = 64;
    // The saved coords must land inside at least one monitor's **work
    // area**, with enough room for the whole bar. A small slack (32px on
    // each side) tolerates near-edge drops without rejecting them.
    let slack: i32 = 32;
    for monitor in w.available_monitors().unwrap_or_default() {
        let (mx, my, mr, mb) = work_area_from_monitor(&monitor);
        if rec.x + bar_w - slack >= mx
            && rec.x + slack <= mr
            && rec.y + bar_h - slack >= my
            && rec.y + slack <= mb
        {
            return Ok(Some((rec.x, rec.y)));
        }
    }
    tracing::info!(
        saved_x = rec.x,
        saved_y = rec.y,
        "saved floating position is outside all current monitor work areas; falling back to default"
    );
    Ok(None)
}

/// Taskbar-aware work area (`rcWork`) for a given monitor. Uses the
/// monitor's centre-point to look up its HMONITOR handle via
/// `MonitorFromPoint`, then queries `GetMonitorInfoW`. Falls back to
/// the full monitor rect only if both calls fail (rare, e.g. the
/// monitor was disconnected between enumeration and this lookup).
#[cfg(windows)]
fn work_area_from_monitor(monitor: &tauri::Monitor) -> (i32, i32, i32, i32) {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    let mp = monitor.position();
    let ms = monitor.size();
    let full_rect = (mp.x, mp.y, mp.x + ms.width as i32, mp.y + ms.height as i32);
    // Centre-point of the monitor — an unambiguous coordinate for
    // MonitorFromPoint to resolve to this specific monitor's HMONITOR.
    let centre = POINT {
        x: mp.x + (ms.width as i32) / 2,
        y: mp.y + (ms.height as i32) / 2,
    };
    unsafe {
        let hmon = MonitorFromPoint(centre, MONITOR_DEFAULTTONEAREST);
        if hmon.is_invalid() {
            return full_rect;
        }
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmon, &mut info).as_bool() {
            (
                info.rcWork.left,
                info.rcWork.top,
                info.rcWork.right,
                info.rcWork.bottom,
            )
        } else {
            full_rect
        }
    }
}

#[cfg(not(windows))]
fn work_area_from_monitor(monitor: &tauri::Monitor) -> (i32, i32, i32, i32) {
    // Non-Windows: no standard work-area query. We only ship on Windows
    // so this path exists for compile-on-macOS/linux in CI. Falls back
    // to the full monitor rect.
    let p = monitor.position();
    let s = monitor.size();
    (p.x, p.y, p.x + s.width as i32, p.y + s.height as i32)
}

/// Remove the saved position file so the next launch uses the default.
/// Best-effort.
fn clear_saved_floating_position() {
    if let Ok(path) = position_file_path() {
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// Public restore entry. Called from main.rs during the setup hook,
/// before `w.show()`, so the bar appears directly at the saved position
/// with no visible snap. Returns Ok even when there's nothing to
/// restore — the caller just moves on.
///
/// Two-step:
///   1. Apply the saved position if valid (validation now uses the
///      taskbar-aware work area — see `load_valid_floating_position`).
///   2. Run `clamp_to_work_area` unconditionally afterwards. On the
///      restore-applied path this is a no-op because the validator
///      already enforced the work-area bounds; on the no-restore
///      path (first launch, stale save, monitor change) it ensures
///      the default Tauri position is clamped too. Either way the
///      bar is guaranteed to be inside a visible, non-taskbar-occluded
///      rectangle before `w.show()` is called by the caller.
pub fn restore_floating_position(w: &tauri::WebviewWindow) -> anyhow::Result<()> {
    // v0.6.7 diagnostic: log before + after so the full restore + clamp
    // sequence is visible. Helps diagnose "the bar doesn't remember its
    // position" when the save path IS working (file exists with the
    // right coords) but some later show/clamp is stomping the restore.
    let before = w.outer_position().ok();
    if let Some((x, y)) = load_valid_floating_position(w)? {
        tracing::info!(
            saved_x = x,
            saved_y = y,
            before_x = before.as_ref().map(|p| p.x),
            before_y = before.as_ref().map(|p| p.y),
            "restoring floating bar position from previous session"
        );
        w.set_position(tauri::PhysicalPosition::new(x, y))?;
    } else {
        tracing::info!(
            before_x = before.as_ref().map(|p| p.x),
            before_y = before.as_ref().map(|p| p.y),
            "no saved floating position to restore; using tauri config default"
        );
    }
    // Unconditional pre-show clamp: guarantees the first paint is inside
    // the taskbar-aware work area whether we restored a saved position,
    // fell back to the Tauri-config default, or restored a saved
    // position that just happened to be right at a work-area edge.
    if let Err(e) = clamp_to_work_area(w) {
        tracing::warn!(?e, "pre-show clamp in restore_floating_position failed");
    }
    let after = w.outer_position().ok();
    tracing::info!(
        after_x = after.as_ref().map(|p| p.x),
        after_y = after.as_ref().map(|p| p.y),
        "floating bar position after restore + pre-show clamp"
    );
    Ok(())
}
