//! Canonical data paths for Utterbar. Everything user-visible is under `%APPDATA%/Utterbar/`.

use std::path::PathBuf;

use anyhow::{anyhow, Result};

pub fn data_dir() -> Result<PathBuf> {
    let base = dirs::data_dir().ok_or_else(|| anyhow!("no data dir"))?;
    let path = base.join("Utterbar");
    ensure_dir(&path)?;
    Ok(path)
}

pub fn config_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("config");
    ensure_dir(&p)?;
    Ok(p)
}

pub fn db_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("data");
    ensure_dir(&p)?;
    Ok(p)
}

pub fn models_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("models");
    ensure_dir(&p)?;
    Ok(p)
}

pub fn recordings_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("recordings");
    ensure_dir(&p)?;
    Ok(p)
}

pub fn logs_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("logs");
    ensure_dir(&p)?;
    Ok(p)
}

/// Where the diagnostic audio dump (raw + post-preprocess WAV pairs)
/// lands when `audio.diagnostic_audio_dump` is enabled. Off by default
/// because it leaks private audio to disk.
pub fn diagnostics_audio_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("diagnostics").join("audio");
    ensure_dir(&p)?;
    Ok(p)
}

fn ensure_dir(p: &std::path::Path) -> Result<()> {
    if !p.exists() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}

/// Assert that `candidate` is a path inside `base` after both are
/// canonicalised. Returns `Ok(canonical)` on success; an error
/// otherwise.
///
/// Why this exists: `Path::join` has surprising behaviour. Joining
/// against an absolute path discards the base, and `..` traversals
/// in the joined component aren't resolved until you hit the
/// filesystem. A column like `model.path` that's a `TEXT` field in
/// SQLite has no DB-side guarantee that it stays inside the models
/// directory; if a corrupted DB or bad migration ever stuffs an
/// absolute path or `..\..\Windows` in there, a naive
/// `models_dir.join(p)` followed by `remove_dir_all` would happily
/// nuke whatever the DB pointed at.
///
/// Usage: callers compute `base.join(suspect)` then ask
/// `assert_within(&base, &full)?` BEFORE doing any destructive
/// operation. Both `base` and `full` are canonicalised so symlinks
/// and `..` are resolved. The returned `PathBuf` is the canonical
/// form of `full`, which the caller should use for the operation
/// rather than the un-canonicalised input.
///
/// Same canonicalisation strategy as the standard "zip slip"
/// containment check, lifted because the same shape applies to
/// model deletion (P6) and the diagnostics opener (P1).
pub fn assert_within(base: &std::path::Path, candidate: &std::path::Path) -> Result<PathBuf> {
    let canonical_base = base
        .canonicalize()
        .map_err(|e| anyhow!("base path not canonicalisable ({}): {e}", base.display()))?;
    let canonical_candidate = candidate.canonicalize().map_err(|e| {
        anyhow!(
            "candidate path not canonicalisable ({}): {e}",
            candidate.display()
        )
    })?;
    if !canonical_candidate.starts_with(&canonical_base) {
        return Err(anyhow!(
            "path escapes its containment root: {} is not inside {}",
            canonical_candidate.display(),
            canonical_base.display()
        ));
    }
    Ok(canonical_candidate)
}

/// The set of directories a user-facing "open in file explorer"
/// command is allowed to reveal. Anything outside is rejected with
/// a clear error rather than silently opening it. Pre-fix, the
/// `open_path` Tauri command shelled through `cmd /C start <path>`
/// where `<path>` was an unvalidated string from the frontend — a
/// command-injection vector via shell metacharacters (`&`, `|`,
/// `>`) and an unbounded reveal surface (could open `C:\Windows`).
///
/// This list is the union of directories any current Utterbar UI
/// surfaces a "show in folder" link for: logs, the SQLite DB,
/// recordings, models, and the v0.14.11 diagnostic audio dump.
/// Add new roots here ONLY when a real UI surface needs it; we'd
/// rather reject a legitimate path than expand the surface.
pub fn diagnostics_open_roots() -> Result<Vec<PathBuf>> {
    Ok(vec![
        data_dir()?,
        logs_dir()?,
        db_dir()?,
        models_dir()?,
        recordings_dir()?,
        diagnostics_audio_dir()?,
    ])
}

// Tiny inline replacement for `dirs` so we don't add another dep.
// %APPDATA% on Windows; XDG_DATA_HOME on Linux; ~/Library/Application Support on macOS.
mod dirs {
    use std::path::PathBuf;

    pub fn data_dir() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var_os("APPDATA").map(PathBuf::from)
        }
        #[cfg(target_os = "macos")]
        {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        }
    }
}
