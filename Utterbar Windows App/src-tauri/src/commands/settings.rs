use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::app_state::AppState;
use crate::domain::{DataPaths, Settings, EVT_SETTINGS_CHANGED};
use crate::util::paths;
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> CmdResult<Settings> {
    Ok(state.settings.snapshot().await)
}

#[tauri::command]
pub async fn update_settings(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    patch: serde_json::Value,
) -> CmdResult<Settings> {
    let updated = state
        .settings
        .update(|s| merge_patch(s, &patch))
        .await
        .map_err(OrliqError::from)?;
    apply_side_effects(&updated);
    let _ = handle.emit(EVT_SETTINGS_CHANGED, &updated);
    Ok(updated)
}

#[tauri::command]
pub async fn reset_settings(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> CmdResult<Settings> {
    let s = state.settings.reset().await.map_err(OrliqError::from)?;
    let _ = handle.emit(EVT_SETTINGS_CHANGED, &s);
    Ok(s)
}

#[tauri::command]
pub async fn get_data_paths() -> CmdResult<DataPaths> {
    Ok(DataPaths {
        config: paths::config_dir()?.to_string_lossy().into(),
        data: paths::db_dir()?.to_string_lossy().into(),
        models: paths::models_dir()?.to_string_lossy().into(),
        recordings: paths::recordings_dir()?.to_string_lossy().into(),
        logs: paths::logs_dir()?.to_string_lossy().into(),
    })
}

fn merge_patch(s: &mut Settings, patch: &serde_json::Value) {
    if let Ok(current) = serde_json::to_value(&*s) {
        let merged = merge(current, patch.clone());
        if let Ok(next) = serde_json::from_value::<Settings>(merged) {
            *s = next;
        }
    }
}

fn merge(mut a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    match (&mut a, b) {
        (serde_json::Value::Object(ao), serde_json::Value::Object(bo)) => {
            for (k, v) in bo {
                let existing = ao.remove(&k).unwrap_or(serde_json::Value::Null);
                ao.insert(k, merge(existing, v));
            }
            a
        }
        (_, other) => other,
    }
}

fn apply_side_effects(s: &Settings) {
    #[cfg(windows)]
    {
        // The platform helper logs failures via tracing::warn —
        // see startup.rs. We deliberately do NOT propagate the
        // error out of update_settings: the toggle landing in
        // settings.json is the load-bearing user contract; the
        // OS-level registration is best-effort and surfaces in
        // the log + the support diagnostics route. A future
        // sprint adding a UI "registration failed" indicator
        // would read `is_launch_on_login_enabled()` against
        // `s.general.launch_on_login` and surface the
        // discrepancy.
        if let Err(e) =
            crate::platform::windows::startup::set_launch_on_login(s.general.launch_on_login)
        {
            tracing::warn!(error = ?e, enabled = s.general.launch_on_login,
                "settings: launch-at-login toggle wrote settings.json but \
                 registry update failed");
        }
    }
    let _ = s; // no-op on non-Windows
}

/// Return the built-in default list of conferencing-app EXE basenames
/// (Teams / Slack / Discord / Zoom / Webex variants). The settings UI
/// uses this to render an "effective list" preview when the user has
/// no custom override set, and to power the "Reset to defaults" button.
///
/// This is a thin command so the UI can stay in sync with whatever the
/// service module decides the defaults should be — adding a new vendor
/// to `DEFAULT_CONFERENCING_APPS` automatically appears in the picker.
#[tauri::command]
pub async fn get_default_conferencing_apps() -> CmdResult<Vec<String>> {
    Ok(crate::services::audio_ducking::default_conferencing_apps())
}

/// Validate a user-typed EXE basename against the existing override
/// list. Returns the canonical (sanitised) form, or a structured
/// reason string the UI can map to a localised message.
///
/// Reason strings (stable, used as match arms in the React layer):
///   * `"empty"` — input was whitespace-only.
///   * `"invalid_characters"` — input contained a path separator,
///     drive letter, quote, wildcard, or any character outside the
///     conservative `[A-Za-z0-9._-]` set Windows EXE basenames can
///     carry.
///   * `"too_long"` — canonicalised name exceeds 64 characters.
///   * `"duplicate"` — the canonicalised form is already in the
///     existing list (case-insensitive).
///
/// The canonicalisation auto-appends `.exe` so users who type
/// `Teams` instead of `Teams.exe` get the right basename. Whitespace
/// is trimmed.
#[tauri::command]
pub async fn validate_conferencing_app_name(
    name: String,
    existing: Vec<String>,
) -> Result<String, String> {
    use crate::services::audio_ducking::{
        validate_conferencing_app_name as validate, ValidateAppNameError as E,
    };
    match validate(&name, &existing) {
        Ok(canonical) => Ok(canonical),
        Err(E::Empty) => Err("empty".to_string()),
        Err(E::InvalidCharacters) => Err("invalid_characters".to_string()),
        Err(E::TooLong) => Err("too_long".to_string()),
        Err(E::Duplicate) => Err("duplicate".to_string()),
    }
}

/// Validate a single mic-mute hotkey combo against the v1 spec rules.
/// See `domain::settings::validate_mic_mute_hotkey` for the rules.
///
/// Reason strings (stable, the React layer matches on these):
///   * `"bare_character_key"`: letter / digit / punctuation with no
///     modifier, would trigger inside the user's dictations.
///   * `"no_main_key"`: modifiers only, no actual key.
///   * `"conflicts_with_utterbar_shortcut"`: combo collides with one
///     of Utterbar's own registered global shortcuts.
///
/// The "Teams and Discord cannot share a combo" pair-check is the
/// React layer's responsibility: it has both combos in its store and
/// can rerun the check whenever either side changes without a round
/// trip. Surfacing it as a Tauri command would force a round trip on
/// every keystroke during hotkey capture.
#[tauri::command]
pub async fn validate_mic_mute_hotkey(
    combo: crate::domain::settings::HotkeyCombo,
    utterbar_shortcut_conflicts: Vec<u16>,
) -> Result<(), String> {
    use crate::domain::settings::{
        validate_mic_mute_hotkey as validate, HotkeyValidationError as E,
    };
    match validate(&combo, &utterbar_shortcut_conflicts) {
        Ok(()) => Ok(()),
        Err(E::BareCharacterKey) => Err("bare_character_key".to_string()),
        Err(E::NoMainKey) => Err("no_main_key".to_string()),
        Err(E::ConflictsWithUtterbarShortcut) => {
            Err("conflicts_with_utterbar_shortcut".to_string())
        }
        // Pair-equality is checked client-side, not here. If callers
        // ever pass an equal pair to this single-combo endpoint we
        // treat it as a bug rather than try to handle it.
        Err(E::SameComboOnBothApps) => Err("bare_character_key".to_string()),
    }
}
