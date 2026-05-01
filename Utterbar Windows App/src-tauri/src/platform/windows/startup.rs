//! Launch-on-login via the HKCU Run key.
//!
//! Windows reads `HKEY_CURRENT_USER\Software\Microsoft\Windows\
//! CurrentVersion\Run` at sign-in and starts every value as a
//! command line. Each value is `name = "<command line>"` where
//! the command line is parsed the way `CreateProcessW` parses
//! its `lpCommandLine` argument: the first whitespace-separated
//! token is the executable, the rest are arguments. **Paths
//! containing spaces must be quoted**, or Windows silently
//! tries to launch the wrong file (e.g. `C:\Program.exe`).
//! That's the chief reason this module went from a one-liner
//! `set_value(name, &exe_str)` to the slightly more careful
//! shape below.
//!
//! Three operations:
//!   * `set_launch_on_login(enabled)` — write the toggle. Logs
//!     warnings on failure rather than silently dropping the
//!     error.
//!   * `is_launch_on_login_enabled()` — read the registry value
//!     (presence-only). Used by the diagnostics page and the
//!     repair-on-launch flow.
//!   * `repair_launch_on_login(intended)` — called from the
//!     setup hook in `main.rs` at every launch. If the stored
//!     setting says startup should be on, this re-asserts the
//!     registry entry against the *current* exe path so a
//!     stale entry from a prior install (or one stripped by a
//!     third-party cleaner) is healed without the user having
//!     to toggle the setting off and back on.
//!
//! The module is structured around a `RegistryWriter` trait so
//! the decision logic in `set_launch_on_login` and
//! `repair_launch_on_login` is testable against an in-memory
//! double — Windows registry calls themselves are not unit-
//! testable cleanly without elevation and per-test cleanup.

use anyhow::{Context, Result};

const REG_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "Utterbar";

/// Format the registry value Windows will execute. Always
/// double-quotes the executable path. The Win32 command-line
/// parser accepts (and strips) the surrounding quotes, so
/// `"C:\Program Files\Utterbar\utterbar.exe"` resolves
/// correctly. Without quotes, paths containing spaces silently
/// fail at sign-in because Windows splits at the first space
/// and tries to start `C:\Program.exe` with `Files\…` as args.
///
/// Public so tests can pin the exact byte sequence written.
pub fn format_run_command(exe: &std::path::Path) -> String {
    let s = exe.to_string_lossy();
    format!("\"{s}\"")
}

/// Tiny abstraction over the HKCU\\Run subkey so the
/// decision logic in this module can be exercised without
/// touching the real registry. Production calls land on
/// `WinregWriter`; tests use the `MockWriter` defined in the
/// test module below.
pub trait RegistryWriter {
    /// Read the current value of `name` under the Run key.
    /// Returns `None` if the key/value is missing — distinct
    /// from a read error.
    fn read_value(&self, name: &str) -> Option<String>;

    /// Write `name = value` under the Run key. Creates the key
    /// if needed.
    fn write_value(&mut self, name: &str, value: &str) -> Result<()>;

    /// Delete `name` under the Run key. Missing-value is NOT
    /// an error — disabling a never-enabled toggle should
    /// succeed.
    fn delete_value(&mut self, name: &str) -> Result<()>;
}

/// Real `winreg`-backed writer. Wraps `winreg::RegKey` calls
/// behind the `RegistryWriter` trait so the rest of the module
/// is portable to tests.
#[cfg(windows)]
pub struct WinregWriter;

#[cfg(windows)]
impl RegistryWriter for WinregWriter {
    fn read_value(&self, name: &str) -> Option<String> {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        hkcu.open_subkey(REG_PATH)
            .and_then(|key| key.get_value::<String, _>(name))
            .ok()
    }

    fn write_value(&mut self, name: &str, value: &str) -> Result<()> {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(REG_PATH)
            .with_context(|| format!("create HKCU\\{REG_PATH}"))?;
        key.set_value(name, &value.to_string())
            .with_context(|| format!("set HKCU\\{REG_PATH}\\{name}"))?;
        Ok(())
    }

    fn delete_value(&mut self, name: &str) -> Result<()> {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let key = match hkcu.open_subkey_with_flags(REG_PATH, winreg::enums::KEY_SET_VALUE) {
            Ok(k) => k,
            Err(_) => {
                // Run subkey is created on demand. If it doesn't
                // exist there's nothing to delete — the toggle is
                // already off, so this is a success.
                return Ok(());
            }
        };
        match key.delete_value(name) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(anyhow::Error::from(e))
                .with_context(|| format!("delete HKCU\\{REG_PATH}\\{name}")),
        }
    }
}

/// Pure decision logic for "is the registry currently in the
/// state the user asked for, and what should we do about it?"
/// Extracted so the tests can drive it without spinning up a
/// `WinregWriter`.
///
/// Returns `Action::None` when the registry already matches the
/// desired state. Returns `Action::Write(value)` or
/// `Action::Delete` otherwise.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Registry already matches desired state; do nothing. Used
    /// by `repair_launch_on_login` to skip a registry write on
    /// the common path where the entry is already correct.
    None,
    Write(String),
    Delete,
}

/// Decide what (if anything) to write/delete in the registry,
/// given the user's intent and the current registry value.
pub fn decide(intended_enabled: bool, exe_path: &std::path::Path, current: Option<&str>) -> Action {
    let desired = format_run_command(exe_path);
    match (intended_enabled, current) {
        (true, Some(v)) if v == desired => Action::None,
        (true, _) => Action::Write(desired),
        (false, None) => Action::None,
        (false, Some(_)) => Action::Delete,
    }
}

/// Apply the user-facing toggle. Called from
/// `commands::settings::apply_side_effects` whenever
/// `update_settings` runs.
///
/// Logs failures with `tracing::warn!` and returns the error so
/// the caller can decide whether to surface it. The previous
/// version of this function returned `Result<()>` but the
/// caller dropped the error with `let _ = ...`; the warn log
/// closes that gap without changing the call signature.
#[cfg(windows)]
pub fn set_launch_on_login(enabled: bool) -> Result<()> {
    let exe = std::env::current_exe().context("resolve current exe path")?;
    let mut writer = WinregWriter;
    apply(&mut writer, enabled, &exe).inspect_err(|e| {
        tracing::warn!(error = ?e, enabled, exe = %exe.display(),
            "launch-at-login: registry update failed");
    })
}

/// Repair-on-launch. Called from the setup hook in `main.rs` at
/// every app launch. If the stored setting says startup should
/// be on, this ensures the registry value points at the
/// **current** exe — healing stale entries from previous
/// installs or third-party registry cleaners without the user
/// having to toggle the setting off and back on.
///
/// When the setting says off, this also clears any stale
/// registry value left behind. That's defensive: a previous
/// install or a corrupted settings round-trip might have left
/// a dangling value the user has since disabled.
///
/// Failures are logged but never propagated up to the setup
/// hook — startup must not fail because the registry write
/// failed.
#[cfg(windows)]
pub fn repair_launch_on_login(intended_enabled: bool) {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error = ?e,
                "launch-at-login repair: cannot resolve current exe path");
            return;
        }
    };
    let mut writer = WinregWriter;
    let current = writer.read_value(APP_NAME);
    let action = decide(intended_enabled, &exe, current.as_deref());
    match action {
        Action::None => {
            tracing::debug!(
                intended = intended_enabled,
                "launch-at-login: registry already matches intent"
            );
        }
        Action::Write(ref desired) => match writer.write_value(APP_NAME, desired) {
            Ok(()) => tracing::info!(
                desired = %desired,
                previous = ?current,
                "launch-at-login repair: re-asserted registry entry"
            ),
            Err(e) => tracing::warn!(error = ?e, exe = %exe.display(),
                    "launch-at-login repair: write failed"),
        },
        Action::Delete => match writer.delete_value(APP_NAME) {
            Ok(()) => tracing::info!(
                previous = ?current,
                "launch-at-login repair: cleared stale registry entry"
            ),
            Err(e) => tracing::warn!(error = ?e,
                    "launch-at-login repair: delete failed"),
        },
    }
}

/// Reader pair for `set_launch_on_login`. Used by future
/// diagnostics surfaces; the repair-on-launch path reads the
/// value through the writer directly so it can also see the
/// stored command line, not just whether the value is present.
/// Marked `dead_code`-allowed because the diagnostics route
/// hasn't shipped yet but the function is the canonical "what
/// does the registry actually say" probe.
#[cfg(windows)]
#[allow(dead_code)]
pub fn is_launch_on_login_enabled() -> bool {
    WinregWriter.read_value(APP_NAME).is_some()
}

/// Apply the toggle against a pluggable writer. Pure logic;
/// the only side effect is a single registry mutation through
/// the trait. Production calls go through `set_launch_on_login`
/// which wraps `WinregWriter`; tests drive `MockWriter`.
fn apply(writer: &mut dyn RegistryWriter, enabled: bool, exe: &std::path::Path) -> Result<()> {
    if enabled {
        let value = format_run_command(exe);
        writer.write_value(APP_NAME, &value)
    } else {
        writer.delete_value(APP_NAME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    /// In-memory `RegistryWriter` for unit tests. Tracks
    /// reads, writes, and deletes so a test can assert which
    /// branches `apply()` and `decide()` took.
    #[derive(Default)]
    struct MockWriter {
        store: HashMap<String, String>,
        write_calls: usize,
        delete_calls: usize,
        fail_writes: bool,
        fail_deletes: bool,
    }

    impl MockWriter {
        fn with(initial: &[(&str, &str)]) -> Self {
            let mut m = Self::default();
            for (k, v) in initial {
                m.store.insert((*k).to_string(), (*v).to_string());
            }
            m
        }
    }

    impl RegistryWriter for MockWriter {
        fn read_value(&self, name: &str) -> Option<String> {
            self.store.get(name).cloned()
        }
        fn write_value(&mut self, name: &str, value: &str) -> Result<()> {
            if self.fail_writes {
                return Err(anyhow::anyhow!("simulated write failure"));
            }
            self.write_calls += 1;
            self.store.insert(name.to_string(), value.to_string());
            Ok(())
        }
        fn delete_value(&mut self, name: &str) -> Result<()> {
            if self.fail_deletes {
                return Err(anyhow::anyhow!("simulated delete failure"));
            }
            self.delete_calls += 1;
            self.store.remove(name);
            Ok(())
        }
    }

    fn exe_with_spaces() -> PathBuf {
        PathBuf::from(r"C:\Program Files\Utterbar\utterbar.exe")
    }

    fn exe_no_spaces() -> PathBuf {
        PathBuf::from(r"C:\Users\Test\AppData\Local\Utterbar\utterbar.exe")
    }

    /// Pin: paths with spaces are double-quoted. The
    /// no-quoting bug was the chief reason a Program-Files
    /// install silently failed at sign-in — Windows would try
    /// to launch `C:\Program.exe`. The format string here is
    /// the load-bearing fix.
    #[test]
    fn format_run_command_quotes_spaces() {
        assert_eq!(
            format_run_command(&exe_with_spaces()),
            r#""C:\Program Files\Utterbar\utterbar.exe""#
        );
    }

    /// Pin: a path without spaces is still quoted. We always
    /// quote so the format is one shape, not two; Windows is
    /// happy either way.
    #[test]
    fn format_run_command_quotes_no_spaces_too() {
        assert_eq!(
            format_run_command(&exe_no_spaces()),
            r#""C:\Users\Test\AppData\Local\Utterbar\utterbar.exe""#
        );
    }

    /// Enabling the toggle writes the quoted command line.
    #[test]
    fn enabling_writes_quoted_value() {
        let mut w = MockWriter::default();
        apply(&mut w, true, &exe_with_spaces()).expect("apply ok");
        assert_eq!(w.write_calls, 1);
        assert_eq!(w.delete_calls, 0);
        assert_eq!(
            w.store.get(APP_NAME).map(String::as_str),
            Some(r#""C:\Program Files\Utterbar\utterbar.exe""#)
        );
    }

    /// Disabling the toggle removes the value. Calling delete
    /// twice (or against an empty registry) is a no-op, not an
    /// error — we don't want disable to fail because the value
    /// was never present.
    #[test]
    fn disabling_deletes_value() {
        let mut w = MockWriter::with(&[(APP_NAME, "stale")]);
        apply(&mut w, false, &exe_no_spaces()).expect("apply ok");
        assert_eq!(w.delete_calls, 1);
        assert!(!w.store.contains_key(APP_NAME));
    }

    /// `decide` is the load-bearing query for the repair-on-
    /// launch path. The four cases are pinned individually so
    /// a future refactor that flips two branches doesn't pass.
    #[test]
    fn decide_intended_on_already_correct_returns_none() {
        let exe = exe_no_spaces();
        let current = format_run_command(&exe);
        assert_eq!(decide(true, &exe, Some(current.as_str())), Action::None);
    }

    #[test]
    fn decide_intended_on_missing_returns_write() {
        let exe = exe_no_spaces();
        let action = decide(true, &exe, None);
        match action {
            Action::Write(v) => assert_eq!(v, format_run_command(&exe)),
            other => panic!("expected Write, got {other:?}"),
        }
    }

    /// Stale value (e.g. previous install path) → Write the
    /// current path. This is the chief repair-on-launch case:
    /// user reinstalled from a different location, registry
    /// still points at the old binary, app should heal it.
    #[test]
    fn decide_intended_on_stale_value_returns_write_with_current_path() {
        let current_exe = exe_no_spaces();
        let stale = r#""C:\Old\Path\Utterbar\utterbar.exe""#;
        let action = decide(true, &current_exe, Some(stale));
        match action {
            Action::Write(v) => assert_eq!(v, format_run_command(&current_exe)),
            other => panic!("expected Write, got {other:?}"),
        }
    }

    #[test]
    fn decide_intended_off_with_existing_value_returns_delete() {
        let exe = exe_no_spaces();
        let stale = r#""C:\Old\Path\Utterbar\utterbar.exe""#;
        assert_eq!(decide(false, &exe, Some(stale)), Action::Delete);
    }

    /// Disabled + already absent → Action::None. Defends
    /// against a future refactor that "always tries to delete
    /// for safety" and produces noisy logs on every launch for
    /// the common case where the user has never enabled the
    /// toggle.
    #[test]
    fn decide_intended_off_with_no_value_returns_none() {
        let exe = exe_no_spaces();
        assert_eq!(decide(false, &exe, None), Action::None);
    }

    /// `apply` returns the writer's error so the caller can
    /// log it. Pre-fix the error was `let _ = ...`-dropped;
    /// the new caller surfaces it via tracing::warn. This test
    /// pins that errors propagate rather than getting silently
    /// converted to Ok.
    #[test]
    fn apply_propagates_write_failure() {
        let mut w = MockWriter {
            fail_writes: true,
            ..Default::default()
        };
        let err = apply(&mut w, true, &exe_no_spaces()).expect_err("must surface writer error");
        assert!(
            err.to_string().contains("simulated write failure"),
            "expected the simulated message in the error chain: {err:?}"
        );
        // Store untouched.
        assert!(w.store.is_empty());
    }

    #[test]
    fn apply_propagates_delete_failure() {
        let mut w = MockWriter::with(&[(APP_NAME, "anything")]);
        w.fail_deletes = true;
        let err = apply(&mut w, false, &exe_no_spaces()).expect_err("must surface writer error");
        assert!(err.to_string().contains("simulated delete failure"));
    }
}
