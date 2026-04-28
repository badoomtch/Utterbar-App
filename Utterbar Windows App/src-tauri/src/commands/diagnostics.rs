//! Read-only diagnostics surfaced in Settings → Advanced. Everything here is local;
//! nothing is phoned home.

use serde::Serialize;
use ts_rs::TS;

use crate::services::gpu::{self, GpuProbe};
use crate::util::paths;
use crate::util::result::{CmdResult, OrliqError};

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct Diagnostics {
    pub version: String,
    pub os: String,
    pub cpu_count: usize,
    pub log_path: String,
    pub db_path: String,
    pub models_path: String,
    /// Whether the Parakeet ONNX backend is compiled in. Kept as a boolean
    /// rather than dropped because the Advanced diagnostics panel surfaces it,
    /// and because earlier dev builds shipped it as a Cargo feature. The
    /// backend is now mandatory (sherpa-rs is a hard dependency), so this is
    /// always `true` on a production build — the field stays so older
    /// frontends keep deserialising the payload unchanged.
    pub onnx_feature: bool,
    pub llama_feature: bool,
}

#[tauri::command]
pub async fn get_diagnostics() -> CmdResult<Diagnostics> {
    Ok(Diagnostics {
        version: env!("CARGO_PKG_VERSION").into(),
        os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        cpu_count: num_cpus(),
        log_path: paths::logs_dir()?.to_string_lossy().into(),
        // `orliq.db` is the on-disk filename committed in v0.x installs —
        // renaming would orphan every user's history + modes. Left as-is.
        db_path: paths::db_dir()?.join("orliq.db").to_string_lossy().into(),
        models_path: paths::models_dir()?.to_string_lossy().into(),
        onnx_feature: true,
        llama_feature: cfg!(feature = "llama"),
    })
}

/// Probe the local machine for GPU adapters via DXGI. Used by onboarding and the
/// Advanced settings pane to decide whether GPU acceleration is on the table.
#[tauri::command]
pub async fn probe_gpu() -> CmdResult<GpuProbe> {
    Ok(gpu::probe())
}

/// Open a directory or file in the OS's native file explorer.
///
/// Reachable from any front-end webview via Tauri IPC. Pre-fix this
/// shelled out via `cmd /C start "" <path>` on Windows, which made
/// the path a `cmd.exe` argument — characters like `&`, `|`, `^`,
/// `>`, `"` were shell metacharacters, turning the command into a
/// command-injection vector for any caller who controlled the path.
/// The fix has two parts:
///
///   1. Restrict `path` to the diagnostic data roots (see
///      `paths::diagnostics_open_roots`). The current UI only opens
///      logs, the DB, recordings, models, and the v0.14.11
///      diagnostic audio dump — anything else is a misuse and we
///      reject it explicitly.
///   2. On Windows, call `ShellExecuteW` directly with the path as
///      a wide-string argument. `ShellExecuteW` doesn't go through
///      `cmd.exe` so shell metacharacters in the filename are no
///      longer special.
///
/// Caller-supplied URLs are explicitly NOT supported. The argument
/// must be a path on disk that exists and lives inside one of the
/// allowed roots.
#[tauri::command]
pub async fn open_path(path: String) -> CmdResult<()> {
    let candidate = std::path::PathBuf::from(&path);

    // The path must exist before any containment check — `canonicalize`
    // requires it on Windows and we don't want to silently open a
    // user-supplied non-path with the explorer.
    if !candidate.exists() {
        return Err(OrliqError::User {
            message: format!("path does not exist: {path}"),
        });
    }

    let allowed = paths::diagnostics_open_roots().map_err(|e| OrliqError::Io {
        message: e.to_string(),
    })?;
    let canonical_candidate = candidate.canonicalize().map_err(|e| OrliqError::Io {
        message: format!("canonicalise {path}: {e}"),
    })?;

    let mut allowed_match = false;
    for root in &allowed {
        if let Ok(canonical_root) = root.canonicalize() {
            if canonical_candidate.starts_with(&canonical_root) {
                allowed_match = true;
                break;
            }
        }
    }
    if !allowed_match {
        tracing::warn!(
            requested = %path,
            canonical = %canonical_candidate.display(),
            "open_path refused: target is outside the Utterbar data directory"
        );
        return Err(OrliqError::User {
            message: "open_path is restricted to the Utterbar data directory.".into(),
        });
    }

    opener_open(&canonical_candidate).map_err(|e| OrliqError::Io { message: e })
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Open `path` with the OS's native file explorer. Path is assumed
/// pre-validated by `open_path` (must exist + must live under one
/// of the diagnostics_open_roots).
#[cfg(windows)]
fn opener_open(path: &std::path::Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    // ShellExecuteW takes a NUL-terminated wide string. Build it
    // here directly — going through cmd.exe + the `start` builtin
    // (the pre-fix shape) was the injection vector. Wide strings
    // don't shell-interpret.
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);

    // SAFETY: `wide` is a properly NUL-terminated UTF-16 buffer that
    // outlives the call. The other PCWSTR arguments (`verb`, `params`,
    // `dir`) are NULL via PCWSTR::null(); ShellExecuteW handles NULL
    // for those parameters by using its defaults (open verb, no extra
    // params, current process directory).
    let result = unsafe {
        ShellExecuteW(
            None,           // hwnd — no parent
            PCWSTR::null(), // verb — defaults to "open"
            PCWSTR::from_raw(wide.as_ptr()),
            PCWSTR::null(), // parameters — none
            PCWSTR::null(), // working dir — leave as caller's
            SW_SHOWNORMAL,
        )
    };

    // ShellExecuteW returns an HINSTANCE-shaped value where any value
    // > 32 means success. <= 32 are documented error codes (see
    // ShellExecuteW MSDN). Treat anything <= 32 as a failure with
    // the numeric code surfaced for diagnosis — the user's IT can
    // map it back via the MSDN table.
    let code = result.0 as isize;
    if code > 32 {
        Ok(())
    } else {
        Err(format!("ShellExecuteW failed (code {code})"))
    }
}

#[cfg(target_os = "macos")]
fn opener_open(path: &std::path::Path) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn opener_open(path: &std::path::Path) -> Result<(), String> {
    std::process::Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    //! Path-validation tests for the `open_path` Tauri command.
    //!
    //! These cover the validation half of the fix — that the
    //! diagnostics opener refuses paths outside the allowed roots.
    //! The actual `ShellExecuteW` call isn't exercised here (it
    //! would spawn a real Explorer window mid-CI). The injection
    //! protection is purely structural: by replacing `cmd /C start`
    //! with a wide-string argument to `ShellExecuteW`, shell
    //! metacharacters in the path are no longer interpreted.

    use crate::util::paths;
    use std::fs;

    /// Helper: write a file inside the Utterbar data dir and return
    /// its absolute path. Cleans up after the test via the returned
    /// guard.
    struct TempInDataDir {
        path: std::path::PathBuf,
    }
    impl Drop for TempInDataDir {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    fn temp_in_logs(name: &str) -> TempInDataDir {
        let p = paths::logs_dir().expect("logs dir").join(name);
        fs::write(&p, b"test").expect("write temp");
        TempInDataDir { path: p }
    }

    /// Path inside `logs_dir` (one of the allowed roots) is accepted
    /// by `assert_within`. Sanity check that the helper agrees with
    /// the open_path validator on the happy case.
    #[test]
    fn assert_within_accepts_path_inside_allowed_root() {
        let f = temp_in_logs("ut-test-allowed.txt");
        let logs = paths::logs_dir().unwrap();
        let canonical = paths::assert_within(&logs, &f.path);
        assert!(
            canonical.is_ok(),
            "path inside logs_dir should pass assert_within: {:?}",
            canonical.err()
        );
    }

    /// `C:\Windows\System32` (or the platform equivalent) is not
    /// inside any Utterbar data root, so `assert_within` rejects.
    /// This is the core protection — pre-fix `cmd /C start C:\Windows\System32`
    /// would have happily opened it.
    #[test]
    fn assert_within_rejects_path_outside_allowed_root() {
        let outside = if cfg!(windows) {
            std::path::PathBuf::from("C:\\Windows\\System32")
        } else {
            std::path::PathBuf::from("/etc")
        };
        if !outside.exists() {
            // CI runner without these system paths — skip rather
            // than fail. The protection is structural and exercised
            // by the other tests too.
            return;
        }
        let logs = paths::logs_dir().unwrap();
        assert!(
            paths::assert_within(&logs, &outside).is_err(),
            "system path outside logs_dir must be rejected"
        );
    }

    /// `..` traversal in the candidate is resolved by canonicalise
    /// before the containment check, so a path like
    /// `<logs>\..\..\Windows` ends up as `C:\Windows` and fails the
    /// containment check rather than being silently followed.
    #[test]
    fn assert_within_rejects_traversal_out_of_root() {
        let logs = paths::logs_dir().unwrap();
        let traversal = logs.join("..").join("..");
        // The traversal target must exist for canonicalize to work
        // (the parent of `data_dir` is the user's APPDATA which
        // always does on Windows).
        if !traversal.exists() {
            return;
        }
        assert!(
            paths::assert_within(&logs, &traversal).is_err(),
            "traversal outside logs_dir must be rejected"
        );
    }

    /// A non-existent path can't be canonicalised on Windows, so
    /// `assert_within` errors before any containment is checked —
    /// which is fine: `open_path` checks `exists()` first and
    /// short-circuits with a clearer "path does not exist" error.
    /// This test pins that contract: a missing path is never
    /// silently treated as inside.
    #[test]
    fn assert_within_rejects_missing_path() {
        let logs = paths::logs_dir().unwrap();
        let missing = logs.join("definitely-not-a-real-file-ut.txt");
        assert!(
            paths::assert_within(&logs, &missing).is_err(),
            "missing path must error rather than silently pass"
        );
    }

    /// `diagnostics_open_roots` returns at least the directories
    /// the current UI surfaces in "show in folder" buttons. If a
    /// future PR drops one of these roots, the open_path command
    /// would refuse to reveal a legitimate target — surface that
    /// regression in CI.
    #[test]
    fn diagnostics_open_roots_includes_expected_dirs() {
        let roots = paths::diagnostics_open_roots().expect("open roots");
        assert!(
            roots.iter().any(|r| r == &paths::logs_dir().unwrap()),
            "logs_dir must be an allowed open root"
        );
        assert!(
            roots.iter().any(|r| r == &paths::models_dir().unwrap()),
            "models_dir must be an allowed open root"
        );
        assert!(
            roots
                .iter()
                .any(|r| r == &paths::diagnostics_audio_dir().unwrap()),
            "diagnostics_audio_dir (v0.14.11) must be an allowed open root"
        );
    }
}
