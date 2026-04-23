//! System tray. Static top-level items plus a dynamic "Switch Mode" submenu rebuilt
//! whenever the modes list changes.

use std::sync::Arc;

use anyhow::Result;
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::app_state::AppState;

const TRAY_ID: &str = "main-tray";
const MENU_ID_OPEN: &str = "open";
const MENU_ID_START: &str = "start";
const MENU_ID_PAUSE: &str = "pause";
const MENU_ID_RESET_BAR: &str = "reset_bar";
const MENU_ID_QUIT: &str = "quit";
const MENU_PREFIX_MODE: &str = "mode:";

pub fn install(app: &AppHandle) -> Result<()> {
    let menu = build_menu(app, &[])?;
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
        wire_events(app, &tray);
    }
    Ok(())
}

/// Rebuild the tray menu from the current modes list. Safe to call on every settings
/// or mode change.
pub async fn rebuild(app: &AppHandle, state: &AppState) -> Result<()> {
    let modes: Vec<(String, String)> = state
        .modes
        .list()
        .unwrap_or_default()
        .into_iter()
        .map(|m| (m.id, m.name))
        .collect();
    let menu = build_menu(app, &modes)?;
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

fn build_menu(app: &AppHandle, modes: &[(String, String)]) -> Result<Menu<tauri::Wry>> {
    let open = MenuItemBuilder::new("Open Utterbar")
        .id(MENU_ID_OPEN)
        .build(app)?;
    let start = MenuItemBuilder::new("Start Recording")
        .id(MENU_ID_START)
        .build(app)?;
    let pause = MenuItemBuilder::new("Pause Shortcuts")
        .id(MENU_ID_PAUSE)
        .build(app)?;
    // Escape hatch for "I dragged the bar behind the taskbar and it's gone" —
    // snaps it back to the top-centre safe spot. Put directly under Start
    // Recording so users who've just lost the bar see it without scrolling the
    // menu past the Switch Mode submenu.
    let reset_bar = MenuItemBuilder::new("Reset Utterbar position")
        .id(MENU_ID_RESET_BAR)
        .build(app)?;
    let quit = MenuItemBuilder::new("Quit").id(MENU_ID_QUIT).build(app)?;

    let mut builder = MenuBuilder::new(app).items(&[&open, &start, &reset_bar]);

    if !modes.is_empty() {
        let mut submenu = SubmenuBuilder::new(app, "Switch Mode");
        for (id, name) in modes {
            let item = MenuItemBuilder::new(name)
                .id(format!("{MENU_PREFIX_MODE}{id}"))
                .build(app)?;
            submenu = submenu.item(&item);
        }
        builder = builder.item(&submenu.build()?);
    }

    builder = builder.item(&pause).separator().item(&quit);
    Ok(builder.build()?)
}

fn wire_events(app: &AppHandle, tray: &tauri::tray::TrayIcon) {
    let handle_for_menu = app.clone();
    tray.on_menu_event(move |app, event| {
        let id = event.id.as_ref();
        match id {
            MENU_ID_OPEN => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            MENU_ID_START => {
                let _ = app.emit_to("floating", "tray:start-recording", ());
            }
            MENU_ID_PAUSE => {
                let _ = app.emit_to("floating", "tray:pause-toggled", ());
            }
            MENU_ID_RESET_BAR => {
                // Run the reset from a background task so we don't block the
                // tray's UI thread on window manipulation.
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::commands::window::reset_floating_position(app).await {
                        tracing::warn!(?e, "tray: reset bar position failed");
                    }
                });
            }
            MENU_ID_QUIT => {
                handle_for_menu.exit(0);
            }
            other if other.starts_with(MENU_PREFIX_MODE) => {
                let mode_id = other.trim_start_matches(MENU_PREFIX_MODE).to_string();
                if let Some(state) = app.try_state::<Arc<AppState>>() {
                    let state = state.inner().clone();
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = state
                            .settings
                            .update(|s| s.active_mode_id = Some(mode_id.clone()))
                            .await
                        {
                            tracing::warn!(?e, "failed to set active mode from tray");
                        }
                        let _ = app.emit("mode:changed", mode_id);
                    });
                }
            }
            _ => {}
        }
    });

    tray.on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let app = tray.app_handle();
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    });
}
