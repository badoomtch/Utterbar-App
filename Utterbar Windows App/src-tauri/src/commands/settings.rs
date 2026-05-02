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
    // Pro gate: Free users cannot enable `privacy.retain_audio`. Strip
    // that key from the incoming patch so any other changes still land.
    // The dictation command also refuses to write on Free — this is the
    // first of two guards so the stored value stays honest.
    let patch = if !crate::util::gating::is_pro(&state.licence.status().await) {
        strip_pro_only_keys(patch)
    } else {
        patch
    };

    let updated = state
        .settings
        .update(|s| merge_patch(s, &patch))
        .await
        .map_err(OrliqError::from)?;
    apply_side_effects(&updated);
    let _ = handle.emit(EVT_SETTINGS_CHANGED, &updated);
    Ok(updated)
}

/// Remove Pro-gated keys from a settings patch so a Free user can still
/// update everything else. Today we strip:
///   • `privacy.retain_audio` — Pro-only audio keep-on-disk.
///   • `general.accent == "custom"` + `general.custom_accent_hex` — the
///     UI Pro-gates the Custom accent chip, but the backend strips the
///     patch too so a crafted IPC call can't bypass the frontend guard.
///     We rewrite the variant to `teal` (the default) so the patched
///     accent field still deserializes cleanly rather than erroring out
///     of the whole update.
fn strip_pro_only_keys(mut patch: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = patch.as_object_mut() {
        if let Some(privacy) = obj.get_mut("privacy").and_then(|v| v.as_object_mut()) {
            privacy.remove("retain_audio");
        }
        if let Some(general) = obj.get_mut("general").and_then(|v| v.as_object_mut()) {
            // Drop a custom-accent attempt entirely — rewrite accent to teal
            // if the patch tried to set "custom", and remove the hex field
            // regardless so Free users can't stockpile a colour server-side.
            let tried_custom = matches!(
                general.get("accent").and_then(|v| v.as_str()),
                Some("custom")
            );
            if tried_custom {
                general.insert("accent".into(), serde_json::Value::String("teal".into()));
            }
            general.remove("custom_accent_hex");
        }
    }
    patch
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
