//! Release-notes lookup for the in-app Updates page.
//!
//! The notes for the currently-running version are baked into the
//! binary via `include_str!` against the markdown file at
//! `releases/<version>.md` in the workspace. Bundling at compile time
//! beats bundling as a Tauri resource because:
//!   * No path resolution at runtime — the markdown is a static string.
//!   * No "missing file" failure mode — if the markdown isn't there the
//!     compile fails, which is exactly what we want for a release.
//!   * No bundle-size concern — the notes are a few KB.
//!
//! The version lookup table here grows by one entry per release. Only
//! the current version's notes are surfaced in the UI, but keeping a
//! lookup table means we can add a "previous releases" view later
//! (v0.14.2+) without changing the IPC contract.

/// Markdown body of the release notes for the version embedded in the
/// running binary. Returns `None` if the running version doesn't have a
/// bundled notes file — typically an unreleased dev build run from
/// `cargo run`. The frontend renders the markdown directly.
#[tauri::command]
pub fn current_release_notes() -> Option<String> {
    let version = env!("CARGO_PKG_VERSION");
    notes_for(version).map(|s| s.to_string())
}

/// Static version → notes table. Each released version that ships an
/// in-app Updates page entry adds a row here. Older versions can be
/// retired (set to None) once we no longer expect users to be on them.
fn notes_for(version: &str) -> Option<&'static str> {
    match version {
        "0.14.1" => Some(include_str!("../../../releases/0.14.1.md")),
        "0.14.0" => Some(include_str!("../../../releases/0.14.0.md")),
        "0.13.3" => Some(include_str!("../../../releases/0.13.3.md")),
        "0.13.2" => Some(include_str!("../../../releases/0.13.2.md")),
        "0.13.1" => Some(include_str!("../../../releases/0.13.1.md")),
        "0.13.0" => Some(include_str!("../../../releases/0.13.0.md")),
        _ => None,
    }
}
