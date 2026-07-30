//! Release-notes lookup for the in-app Updates page.
//!
//! The notes for the currently-running version are baked into the
//! binary at compile time via `include_str!` against
//! `releases/<CARGO_PKG_VERSION>.md` in the workspace.
//!
//! Compile-time enforcement is the point of doing it this way: bumping
//! the version in `Cargo.toml` without also creating the matching
//! `releases/<version>.md` file fails the build with a clear
//! "no such file" error from rustc. There is no silent runtime
//! fallback. A binary that compiled has notes for its own version,
//! end of story.
//!
//! Older versions' notes still live on disk under `releases/` as a
//! historical record. They are not bundled
//! into older or newer binaries; each binary only carries the notes
//! for its own `CARGO_PKG_VERSION`.

/// Markdown body of the release notes for the version embedded in
/// this binary. Always returns `Some` for a built binary because the
/// matching markdown file is required at compile time.
///
/// Returns `Option<String>` rather than `String` so the IPC contract
/// stays compatible with a future state where the binary might not
/// have notes (e.g. a hypothetical "skip notes for snapshot builds"
/// flag). The frontend already handles `null` as "no notes for this
/// version" without restructuring.
#[tauri::command]
pub fn current_release_notes() -> Option<String> {
    Some(CURRENT_RELEASE_NOTES.to_string())
}

/// Compile-time-bundled markdown for this binary's exact version.
/// `concat!` + `include_str!` resolves the path against `CARGO_PKG_VERSION`.
/// If the file is missing, the build fails with E0463 (file not found).
const CURRENT_RELEASE_NOTES: &str = include_str!(concat!(
    "../../../releases/",
    env!("CARGO_PKG_VERSION"),
    ".md"
));
