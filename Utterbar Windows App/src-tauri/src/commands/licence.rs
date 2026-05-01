//! Tauri commands for the licence system.
//!
//! Three verbs. The UI reads status on mount via `licence_status`, listens to
//! the `licence_changed` event for subsequent updates, and calls
//! `licence_activate` / `licence_deactivate` from the Settings → Licence page.

use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::app_state::AppState;
use crate::domain::licence::LicenceStatus;
use crate::util::result::CmdResult;

#[tauri::command]
pub async fn licence_status(state: State<'_, Arc<AppState>>) -> CmdResult<LicenceStatus> {
    Ok(state.licence.status().await)
}

#[tauri::command]
pub async fn licence_activate(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    key: String,
) -> CmdResult<LicenceStatus> {
    state.licence.activate(handle, &key).await
}

#[tauri::command]
pub async fn licence_deactivate(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> CmdResult<LicenceStatus> {
    state.licence.deactivate(handle).await
}

/// True when the active Pro status came from a managed (fleet) licence
/// file at `<data_dir>/licence.bin` rather than the keyring cache or the
/// paste-flow activation. The Settings > Licence page calls this on
/// mount to swap the activation form for a "managed by your administrator"
/// notice. Frozen at startup; matches the rest of the policy/managed
/// stack - managed-licence changes require restarting Utterbar.
#[tauri::command]
pub fn licence_is_managed(state: State<'_, Arc<AppState>>) -> CmdResult<bool> {
    Ok(state.licence.is_managed())
}

/// Air-gapped activation: write a JSON activation-request file to the
/// path the user picked in the save dialog. The file is what the
/// admin uploads at /admin/licences/{id} to register an offline
/// activation. See `services::licence::request` for the format and
/// `docs/it-deployment` (Air-gapped activation) for the customer-
/// facing playbook.
///
/// Returns the suggested filename so the UI can render a "saved as
/// X" confirmation. The file at `dest_path` is written atomically:
/// we serialise to a string first, then a single write call. Partial
/// writes on disk-full leave a truncated file, which the admin
/// upload validator rejects on format magic.
///
/// Cross-platform: the underlying fingerprint is Windows-only so this
/// command is gated `#[cfg(windows)]`. On non-Windows builds (CI
/// editor / dev tooling) the command falls back to a clear error
/// rather than refusing to compile.
#[tauri::command]
pub async fn licence_export_request_file(dest_path: String) -> CmdResult<String> {
    use crate::services::licence::request::ActivationRequest;
    use crate::util::result::OrliqError;

    #[cfg(not(windows))]
    {
        let _ = dest_path;
        return Err(OrliqError::Licence {
            code: "internal".to_string(),
            message: "activation-request export requires Windows".to_string(),
        });
    }

    #[cfg(windows)]
    {
        let req = ActivationRequest::for_current_machine()?;
        let json = req.to_json_pretty()?;

        // The path comes from the user's dialog pick. We don't try to
        // sandbox it — the user explicitly chose it. We do refuse to
        // write to an empty path, which can happen if the JS wrapper
        // didn't honour the cancel-on-dismiss contract.
        if dest_path.trim().is_empty() {
            return Err(OrliqError::Licence {
                code: "internal".to_string(),
                message: "destination path is empty".to_string(),
            });
        }

        std::fs::write(&dest_path, json.as_bytes()).map_err(|e| OrliqError::Licence {
            code: "internal".to_string(),
            message: format!("write request file to {dest_path}: {e}"),
        })?;

        tracing::info!(path = %dest_path, "activation request file written");
        Ok(req.suggested_filename())
    }
}
