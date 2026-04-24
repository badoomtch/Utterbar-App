//! Global hotkey service. Backed by `tauri-plugin-global-shortcut`.
//!
//! Flow:
//!   1. User binds a shortcut → `HotkeyService::bind()` persists it.
//!   2. On settings load or bind, `arm_from_settings()` registers accelerators with the
//!      plugin and installs a handler that translates each event into a frontend event.
//!   3. The frontend's dictation store listens for `shortcut:triggered` and invokes the
//!      appropriate command.

use std::collections::HashMap;

use anyhow::Result;
use std::str::FromStr;
use tauri::{AppHandle, Emitter};

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut as GhShortcut, ShortcutState};
use tokio::sync::RwLock;

use crate::app_state::AppState;
use crate::domain::{Shortcut, ShortcutConflict, ShortcutId, EVT_SHORTCUT_TRIGGERED};

#[derive(Default)]
pub struct HotkeyService {
    bindings: RwLock<HashMap<ShortcutIdKey, Shortcut>>,
}

type ShortcutIdKey = String;

impl HotkeyService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn bind(&self, id: ShortcutId, shortcut: Shortcut) -> Result<()> {
        let key = key_of(&id);
        self.bindings.write().await.insert(key, shortcut);
        Ok(())
    }

    pub async fn unbind(&self, id: ShortcutId) -> Result<()> {
        let key = key_of(&id);
        self.bindings.write().await.remove(&key);
        Ok(())
    }

    pub async fn conflicts(&self) -> Vec<ShortcutConflict> {
        let map = self.bindings.read().await;
        let mut seen: HashMap<&str, &str> = HashMap::new();
        let mut out = Vec::new();
        for (k, v) in map.iter() {
            if let Some(prev) = seen.insert(&v.accelerator, k) {
                out.push(ShortcutConflict {
                    slot_a: id_from_key(prev),
                    slot_b: id_from_key(k),
                    accelerator: v.accelerator.clone(),
                });
            }
        }
        out
    }

    /// Dump every currently-bound shortcut. Unused by the active command set
    /// (shortcuts are read from settings, not from the live OS registration) but
    /// kept as the diagnostic probe for "what does the OS actually think is bound".
    #[allow(dead_code)]
    pub async fn snapshot(&self) -> Vec<(ShortcutId, Shortcut)> {
        self.bindings
            .read()
            .await
            .iter()
            .map(|(k, v)| (id_from_key(k), v.clone()))
            .collect()
    }
}

fn key_of(id: &ShortcutId) -> String {
    serde_json::to_string(id).unwrap_or_default()
}

fn id_from_key(key: &str) -> ShortcutId {
    serde_json::from_str(key).unwrap_or(ShortcutId::ToggleRecording)
}

/// Read all configured accelerators from settings and register them with the plugin.
/// Called on startup and whenever a binding changes.
pub async fn arm_from_settings(handle: &AppHandle, state: &AppState) -> Result<()> {
    let s = state.settings.snapshot().await;
    let shortcuts = handle.global_shortcut();

    // Unregister everything, then re-register from scratch. Simpler than diffing.
    let _ = shortcuts.unregister_all();

    // Build (id, accelerator) pairs, skipping unbound slots.
    let pairs: Vec<(ShortcutId, String)> = [
        (
            ShortcutId::ToggleRecording,
            s.shortcuts.toggle_recording.as_ref(),
        ),
        (ShortcutId::PushToTalk, s.shortcuts.push_to_talk.as_ref()),
        (ShortcutId::HoldToTalk, s.shortcuts.hold_to_talk.as_ref()),
        (
            ShortcutId::CancelRecording,
            s.shortcuts.cancel_recording.as_ref(),
        ),
        (ShortcutId::CycleMode, s.shortcuts.cycle_mode.as_ref()),
        (ShortcutId::OpenSettings, s.shortcuts.open_settings.as_ref()),
        (ShortcutId::OpenHistory, s.shortcuts.open_history.as_ref()),
    ]
    .into_iter()
    .filter_map(|(id, maybe)| maybe.map(|sh| (id, sh.accelerator.clone())))
    .collect();

    for (id, accel) in pairs {
        let parsed = match GhShortcut::from_str(&accel) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(?e, %accel, "invalid accelerator");
                continue;
            }
        };
        let handle = handle.clone();
        let id_for_handler = id.clone();
        if let Err(e) = shortcuts.on_shortcut(parsed, move |_, _, event| {
            // Only fire on key-down; key-up will arrive for hold-to-talk flows separately.
            if event.state == ShortcutState::Pressed {
                let _ = handle.emit(EVT_SHORTCUT_TRIGGERED, id_for_handler.clone());
            }
        }) {
            tracing::warn!(?e, %accel, "failed to register global shortcut");
        }
    }
    Ok(())
}
