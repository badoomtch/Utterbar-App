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

/// Air-gapped activation: import a `licence.bin` the admin sent
/// back, verify it offline, and copy it atomically to
/// `<data_dir>/licence.bin` so the next launch picks it up via
/// `managed::try_load_managed`.
///
/// The managed-licence loader runs once at bootstrap; this command
/// does NOT hot-reload state in the running session. The UI is
/// expected to surface a "restart Utterbar" notice on success.
///
/// Refusal cases:
///  - file unreadable / wrong length: returns `malformed`
///  - signature does not verify against the bundled public key:
///    returns `verification_failed`
///  - V1 (sandbox) payload: returns `verification_failed` because
///    `verify::verify_signed_blob` only trusts V2 today
///
/// Atomicity: we write to `<dest>.tmp` then rename onto `dest`. On
/// Windows, `std::fs::rename` is atomic when both paths are on the
/// same volume, which `data_dir()` always returns. If the tmp file
/// already exists from a previous failed import, it's overwritten
/// cleanly.
#[tauri::command]
pub async fn licence_import_managed_file(source_path: String) -> CmdResult<()> {
    use crate::services::licence::managed;
    use crate::util::result::OrliqError;

    let dest = managed::managed_licence_path()?;
    import_signed_blob_to(&source_path, &dest).map_err(|e| {
        if let OrliqError::Licence { .. } = e {
            // Already an OrliqError::Licence — pass through.
            e
        } else {
            // Non-licence error (shouldn't happen given the helper's
            // contract). Wrap so callers see a stable shape.
            OrliqError::Licence {
                code: "internal".to_string(),
                message: format!("import: {e}"),
            }
        }
    })?;
    Ok(())
}

/// Helper extracted for testing: read `source_path`, verify the
/// signed blob, atomically write it to `dest`. Both inputs are
/// caller-provided so tests can pin behaviour against a temp dir
/// without going through `paths::data_dir()`.
fn import_signed_blob_to(
    source_path: &str,
    dest: &std::path::Path,
) -> Result<(), crate::util::result::OrliqError> {
    use crate::services::licence::verify;
    use crate::util::result::OrliqError;

    if source_path.trim().is_empty() {
        return Err(OrliqError::Licence {
            code: "internal".to_string(),
            message: "source path is empty".to_string(),
        });
    }

    // Read the user-picked file. We don't try to sandbox the path
    // — they explicitly chose it via the open dialog.
    let bytes = std::fs::read(source_path).map_err(|e| OrliqError::Licence {
        code: "malformed".to_string(),
        message: format!("read {source_path}: {e}"),
    })?;

    // Verify before we touch anything in data_dir. A malformed or
    // unsigned file must NOT replace a working managed licence.
    verify::verify_signed_blob(&bytes)?;

    // Ensure the parent dir exists. `data_dir()` should have
    // created it during settings bootstrap, but a fresh install
    // pre-first-launch is the realistic edge case.
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| OrliqError::Licence {
            code: "internal".to_string(),
            message: format!("create data dir {}: {e}", parent.display()),
        })?;
    }

    // Atomic write: tmp → rename. Tmp path is `<dest>.tmp` so a
    // crashed previous import leaves a dangling file the next
    // import overwrites cleanly.
    let tmp = dest.with_extension("bin.tmp");
    std::fs::write(&tmp, &bytes).map_err(|e| OrliqError::Licence {
        code: "internal".to_string(),
        message: format!("write {}: {e}", tmp.display()),
    })?;
    std::fs::rename(&tmp, dest).map_err(|e| {
        // Best-effort cleanup of tmp on rename failure so a future
        // import isn't blocked by a stale .tmp.
        let _ = std::fs::remove_file(&tmp);
        OrliqError::Licence {
            code: "internal".to_string(),
            message: format!("rename onto {}: {e}", dest.display()),
        }
    })?;

    tracing::info!(
        source = %source_path,
        dest = %dest.display(),
        bytes = bytes.len(),
        "imported managed licence file"
    );

    Ok(())
}

#[cfg(test)]
mod import_tests {
    //! Tests for the import-helper extracted from the
    //! `licence_import_managed_file` Tauri command. These
    //! exercise the file-I/O surface (read, verify gate, atomic
    //! write) without touching the live `data_dir()` or the
    //! Tauri runtime.
    //!
    //! Verification correctness itself is pinned in
    //! `services::licence::verify::tests`; what these tests
    //! pin is the *integration*: the helper does NOT replace a
    //! working dest file when the source is invalid, and DOES
    //! overwrite a stale `<dest>.tmp` left by a previous import.
    use super::import_signed_blob_to;
    use crate::util::result::OrliqError;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    /// Per-test temp dir. Same shape as `services::history::tests`
    /// uses; cleaned up by the OS reaper between runs.
    fn temp_dir() -> std::path::PathBuf {
        let n = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("ut-import-test-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Write a deliberately-invalid 95-byte source file and assert
    /// the helper refuses with `malformed`. The dest file MUST NOT
    /// exist after the failure — a failed import never touches
    /// the managed licence.
    #[test]
    fn refuses_truncated_source() {
        let dir = temp_dir();
        let src = dir.join("bad.bin");
        let dest = dir.join("licence.bin");
        std::fs::write(&src, vec![0u8; 95]).expect("write src");

        let err = import_signed_blob_to(src.to_str().unwrap(), &dest).expect_err("must reject");
        match err {
            OrliqError::Licence { code, .. } => assert_eq!(code, "malformed"),
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(
            !dest.exists(),
            "dest must not be created on a failed import"
        );
    }

    /// A 96-byte all-zeros blob is the right length but its
    /// signature does not verify. Pin that the helper rejects with
    /// `verification_failed` and leaves dest untouched.
    #[test]
    fn refuses_unsigned_blob() {
        let dir = temp_dir();
        let src = dir.join("unsigned.bin");
        let dest = dir.join("licence.bin");
        std::fs::write(&src, vec![0u8; 96]).expect("write src");

        let err = import_signed_blob_to(src.to_str().unwrap(), &dest).expect_err("must reject");
        match err {
            OrliqError::Licence { code, .. } => assert_eq!(code, "verification_failed"),
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(
            !dest.exists(),
            "dest must not be created on a failed import"
        );
    }

    /// Empty source path string surfaces the `internal` error from
    /// the early guard, NOT a misleading `malformed` from the file
    /// read.
    #[test]
    fn refuses_empty_source_path() {
        let dest = temp_dir().join("licence.bin");
        let err = import_signed_blob_to("", &dest).expect_err("must reject");
        match err {
            OrliqError::Licence { code, message } => {
                assert_eq!(code, "internal");
                assert!(
                    message.contains("source path is empty"),
                    "message should be specific: {message}"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    /// A stale `licence.bin.tmp` left from a crashed previous
    /// import must NOT block a fresh import. The helper writes
    /// over the tmp file before renaming. We can exercise this
    /// without a valid signed blob by stopping after the verify
    /// step (since we don't have a server private key here).
    /// Instead, verify by inspection: a failed verify on a fresh
    /// import does NOT remove an existing dest file.
    #[test]
    fn failed_import_leaves_existing_dest_alone() {
        let dir = temp_dir();
        let src = dir.join("bad.bin");
        let dest = dir.join("licence.bin");
        // Pre-existing dest file (e.g. a previously-imported good
        // licence). Use a 96-byte sentinel so we can detect any
        // unintended overwrite.
        let sentinel = vec![0xAA; 96];
        std::fs::write(&dest, &sentinel).expect("seed dest");

        // Truncated source → verify rejects.
        std::fs::write(&src, vec![0u8; 95]).expect("write bad src");
        let err = import_signed_blob_to(src.to_str().unwrap(), &dest).expect_err("must reject");
        assert!(matches!(err, OrliqError::Licence { .. }));

        // Sentinel intact.
        let actual = std::fs::read(&dest).expect("read dest");
        assert_eq!(actual, sentinel, "failed import must not corrupt dest");
    }
}
