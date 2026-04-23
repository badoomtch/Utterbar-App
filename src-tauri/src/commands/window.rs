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
#[tauri::command]
pub async fn clamp_floating_position(handle: AppHandle) -> CmdResult<()> {
    if let Some(w) = handle.get_webview_window("floating") {
        if let Err(e) = clamp_to_work_area(&w) {
            tracing::warn!(?e, "clamp_floating_position failed");
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
