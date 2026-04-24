//! Export / import user data.
//!
//! Goal: let a user move their Utterbar between machines or roll back a broken
//! state without hand-copying files from `%APPDATA%/Utterbar`. Tray "Reset bar
//! position" covers one-click window recovery; this is the data-preservation
//! counterpart.
//!
//! Format: uncompressed tarball containing:
//!
//!   manifest.json          — metadata (schema version, app version, timestamp)
//!   config/settings.json   — the full settings document
//!   data/<db-files>        — SQLite db + any WAL/SHM sidecars that exist
//!   recordings/<id>.wav    — optional (off by default — recordings are the
//!                            bulk of the archive and many users won't want to
//!                            move them when they're just migrating preferences)
//!
//! We intentionally skip the `models/` directory: model files are tens to
//! hundreds of MB, are re-downloadable from the catalogue, and a user's
//! GPU/CPU preference may differ on the target machine. Shipping them would
//! bloat the archive and add no real value.
//!
//! Import strategy is overwrite-in-place followed by an app restart. The
//! settings service holds its snapshot in memory, and the DB connection pool
//! caches prepared statements — hot-swapping these without a full process
//! relaunch would require deep per-service invalidation for very little gain.
//! A clean restart is simpler, safer, and the user asked to import *because*
//! they expected the app to "become" the backup.
//!
//! Format version is bumped whenever the on-disk layout of settings.json or
//! the DB schema changes in a way an old import would misread. The importer
//! refuses an archive whose format_version is newer than what this build
//! understands, rather than silently dropping fields.

use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::util::paths;
use crate::util::result::{CmdResult, OrliqError};

const MANIFEST_NAME: &str = "manifest.json";
const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct BackupManifest {
    /// Bumps whenever the on-disk shape changes. Imports refuse to proceed
    /// if this is higher than FORMAT_VERSION — "backup from a newer app".
    format_version: u32,
    /// The CARGO_PKG_VERSION string that wrote the archive. Informational —
    /// the importer doesn't gate on this, it's just for the UI to show
    /// "Archive created by Utterbar 0.3.10".
    utterbar_version: String,
    /// UNIX seconds for the "archive created at <relative time>" string.
    created_at_unix: u64,
    /// Which top-level folders exist inside the archive. Saves the importer
    /// from doing an exhaustive dirwalk just to tell the user "no recordings
    /// in this archive".
    included: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportSummary {
    pub path: String,
    pub bytes: u64,
    pub included: Vec<String>,
}

/// Write a tarball of the user's config + DB (+ optionally recordings) to `dest`.
///
/// `dest` must be an absolute path the UI already obtained via the Tauri dialog
/// plugin's save-file picker — we don't open a picker from here so this command
/// stays pure and testable.
#[tauri::command]
pub async fn export_user_data(dest: String, include_recordings: bool) -> CmdResult<ExportSummary> {
    let dest = PathBuf::from(dest);
    // Creating the parent dir is polite — users may type a path with a new
    // folder component in the save dialog. Ignoring AlreadyExists is fine.
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| OrliqError::Internal {
                message: format!("create parent dir: {e}"),
            })?;
        }
    }

    let out = fs::File::create(&dest).map_err(|e| OrliqError::Internal {
        message: format!("create {}: {e}", dest.display()),
    })?;
    let mut builder = tar::Builder::new(BufWriter::new(out));
    // Don't follow symlinks — unlikely in %APPDATA% but a defensive choice
    // against the user ever putting one there deliberately.
    builder.follow_symlinks(false);

    let mut included = Vec::new();

    // 1. settings.json (always included; it's tiny and represents the whole
    //    of the user's preferences).
    let cfg = paths::config_dir().map_err(io_err)?;
    let settings_file = cfg.join("settings.json");
    if settings_file.exists() {
        append_file(&mut builder, &settings_file, "config/settings.json")?;
        included.push("config".into());
    }

    // 2. DB files — include the main db plus any -wal / -shm sidecars. We
    //    bundle WAL/SHM because a running Utterbar with WAL-mode sqlite may
    //    have un-checkpointed pages there. Without them the target machine
    //    would be restoring a stale view.
    let db = paths::db_dir().map_err(io_err)?;
    let db_had_contents = append_dir_shallow(&mut builder, &db, "data")?;
    if db_had_contents {
        included.push("data".into());
    }

    // 3. Recordings (optional). These are the heaviest assets in the archive
    //    (a handful of WAVs can easily outweigh the rest combined), so the
    //    user opts in explicitly from the Advanced settings toggle.
    if include_recordings {
        let rec = paths::recordings_dir().map_err(io_err)?;
        let rec_had_contents = append_dir_shallow(&mut builder, &rec, "recordings")?;
        if rec_had_contents {
            included.push("recordings".into());
        }
    }

    // 4. Manifest last, so if the writer is interrupted half-way we at
    //    least don't claim the archive is complete.
    let manifest = BackupManifest {
        format_version: FORMAT_VERSION,
        utterbar_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        included: included.clone(),
    };
    let manifest_bytes =
        serde_json::to_vec_pretty(&manifest).map_err(|e| OrliqError::Internal {
            message: format!("manifest: {e}"),
        })?;
    append_bytes(&mut builder, MANIFEST_NAME, &manifest_bytes)?;

    builder
        .into_inner()
        .and_then(|w| w.into_inner().map_err(|e| e.into_error()))
        .map_err(|e| OrliqError::Internal {
            message: format!("finalise tar: {e}"),
        })?
        .sync_all()
        .ok();

    let bytes = fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    tracing::info!(path = %dest.display(), bytes, ?included, "exported user data");
    Ok(ExportSummary {
        path: dest.to_string_lossy().to_string(),
        bytes,
        included,
    })
}

/// Read a previously-exported archive, validate its manifest, and overwrite
/// the user's config + DB in place. Recordings are restored only if present
/// in the archive. The app is expected to restart immediately after.
///
/// **This is destructive.** The caller MUST confirm with the user before
/// invoking — the UI shows a modal spelling out what gets replaced.
#[tauri::command]
pub async fn import_user_data(handle: AppHandle, src: String) -> CmdResult<()> {
    let src = PathBuf::from(src);
    let f = fs::File::open(&src).map_err(|e| OrliqError::Internal {
        message: format!("open {}: {e}", src.display()),
    })?;
    let mut archive = tar::Archive::new(BufReader::new(f));

    // First pass: read manifest only. We do a two-pass read because tar is a
    // streaming format and we want to refuse unknown-version archives BEFORE
    // we start clobbering live files. Iterating once and then re-opening is
    // the simplest way to get that guarantee without buffering the whole
    // archive in memory.
    {
        let mut found_manifest = false;
        for entry in archive.entries().map_err(io_err)? {
            let mut entry = entry.map_err(io_err)?;
            let path = entry.path().map_err(io_err)?.to_path_buf();
            if path == Path::new(MANIFEST_NAME) {
                let mut buf = Vec::new();
                use std::io::Read;
                entry.read_to_end(&mut buf).map_err(io_err)?;
                let manifest: BackupManifest =
                    serde_json::from_slice(&buf).map_err(|e| OrliqError::Internal {
                        message: format!("invalid manifest: {e}"),
                    })?;
                if manifest.format_version > FORMAT_VERSION {
                    return Err(OrliqError::Internal {
                        message: format!(
                            "Backup was created by a newer Utterbar \
                             (archive format v{}, this build understands up to v{}). \
                             Please update Utterbar before importing.",
                            manifest.format_version, FORMAT_VERSION
                        ),
                    });
                }
                tracing::info!(
                    from_version = %manifest.utterbar_version,
                    included = ?manifest.included,
                    "import manifest accepted",
                );
                found_manifest = true;
                break;
            }
        }
        if !found_manifest {
            return Err(OrliqError::Internal {
                message: "Archive is missing manifest.json — is this an Utterbar backup?".into(),
            });
        }
    }

    // Re-open for the actual extraction pass.
    let f = fs::File::open(&src).map_err(io_err)?;
    let mut archive = tar::Archive::new(BufReader::new(f));

    let data_root = paths::data_dir().map_err(io_err)?;
    // Canonical form of the data root so we can verify every destination
    // actually lands underneath it. If `data_root` doesn't exist yet we
    // create it first so canonicalize() doesn't fail.
    fs::create_dir_all(&data_root).map_err(io_err)?;
    let data_root_canon = fs::canonicalize(&data_root).map_err(io_err)?;

    for entry in archive.entries().map_err(io_err)? {
        let mut entry = entry.map_err(io_err)?;
        // Validate the entry through the hardened path-safety check. This
        // rejects absolute paths, drive prefixes, UNC/root prefixes, `..`
        // traversal, symlinks, hardlinks, and any non-regular tar entry
        // types (directories are allowed but unnecessary — we `create_dir_all`
        // on the destination parent anyway). See `validate_entry_path`.
        let safe_rel = match validate_entry_path(&entry)? {
            EntryDecision::Extract(rel) => rel,
            EntryDecision::Skip => continue,
        };
        // Join + canonicalise check. Even if `safe_rel` passed component
        // checks, we also verify the FINAL resolved destination path lies
        // underneath `data_root_canon` — a belt-and-braces guard against
        // any corner case the component check missed on Windows (e.g.
        // case-folded reparse points, short-name aliases).
        let dest = data_root.join(&safe_rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(io_err)?;
            // Canonicalise the *parent* (which exists) and assert it's
            // under data_root. The leaf doesn't exist yet so we can't
            // canonicalise `dest` itself.
            let parent_canon = fs::canonicalize(parent).map_err(io_err)?;
            if !parent_canon.starts_with(&data_root_canon) {
                return Err(OrliqError::Internal {
                    message: format!(
                        "refused to extract outside data root: resolved parent {} is not under {}",
                        parent_canon.display(),
                        data_root_canon.display()
                    ),
                });
            }
        }
        entry.unpack(&dest).map_err(io_err)?;
    }

    tracing::info!(from = %src.display(), "import complete — scheduling restart");
    // Restart so every service boots from the restored on-disk state. Doing
    // this async on the runtime lets the command response reach the UI first
    // (so the toast can say "Restarting…") before the process tears down.
    let h = handle.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        h.restart();
    });
    Ok(())
}

fn append_file(
    builder: &mut tar::Builder<BufWriter<fs::File>>,
    src: &Path,
    archive_path: &str,
) -> Result<(), OrliqError> {
    let mut f = fs::File::open(src).map_err(io_err)?;
    builder
        .append_file(archive_path, &mut f)
        .map_err(|e| OrliqError::Internal {
            message: format!("archive {}: {e}", archive_path),
        })
}

fn append_bytes(
    builder: &mut tar::Builder<BufWriter<fs::File>>,
    archive_path: &str,
    bytes: &[u8],
) -> Result<(), OrliqError> {
    let mut header = tar::Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, archive_path, bytes)
        .map_err(|e| OrliqError::Internal {
            message: format!("archive {}: {e}", archive_path),
        })
}

/// Shallow-append all direct children of `src` under `archive_prefix/…`.
/// Returns true if at least one file was added. Non-recursive by design —
/// the db and recordings dirs are both one level deep for Utterbar.
fn append_dir_shallow(
    builder: &mut tar::Builder<BufWriter<fs::File>>,
    src: &Path,
    archive_prefix: &str,
) -> Result<bool, OrliqError> {
    if !src.exists() {
        return Ok(false);
    }
    let mut any = false;
    for entry in fs::read_dir(src).map_err(io_err)? {
        let entry = entry.map_err(io_err)?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let archive_path = format!("{}/{}", archive_prefix, name);
        append_file(builder, &path, &archive_path)?;
        any = true;
    }
    Ok(any)
}

fn io_err(e: impl std::fmt::Display) -> OrliqError {
    OrliqError::Internal {
        message: e.to_string(),
    }
}

/// The verdict on a tar entry after path + type validation.
#[cfg_attr(test, derive(Debug))]
enum EntryDecision {
    /// Safe to extract to `data_root.join(path)`.
    Extract(PathBuf),
    /// Benign but unused — a directory entry, or the manifest (which the
    /// first pass already consumed). Caller should continue without writing.
    Skip,
}

/// Path + entry-type safety check run against every file in the import
/// archive. Defensive on five fronts:
///
/// 1. **Path literal** — raw path string must not start with `/` or `\`, or
///    match `<letter>:` (a Windows drive prefix), or be empty.
/// 2. **Path components** — must contain no `ParentDir` (`..`) or `RootDir`,
///    `Prefix` (drive letters, UNC). Every component must be `Normal`.
/// 3. **Entry type** — reject symlinks, hardlinks, block/char devices,
///    FIFOs, GNU long-link / long-name extensions, and anything else that
///    isn't a plain file or directory. These tar entry types either
///    dereference to arbitrary filesystem locations (symlink, hardlink) or
///    are meaningless for Utterbar's data and indicate a malicious archive.
/// 4. **Allow-list** — the top-level component must be one of the known
///    Utterbar data namespaces: `manifest.json`, `config/settings.json`,
///    `data/*`, `recordings/*`. Anything else is rejected rather than
///    silently extracted.
/// 5. **Path length** — reject absurd lengths (>4096 bytes) so an attacker
///    can't stall the extractor with giant paths.
fn validate_entry_path<R: std::io::Read>(
    entry: &tar::Entry<R>,
) -> Result<EntryDecision, OrliqError> {
    use std::path::Component;
    use tar::EntryType;

    // 3. Entry type first — cheap check, catches symlinks before we even
    // look at the path.
    let et = entry.header().entry_type();
    match et {
        EntryType::Regular | EntryType::Continuous => {}
        EntryType::Directory => return Ok(EntryDecision::Skip),
        EntryType::Symlink
        | EntryType::Link
        | EntryType::Char
        | EntryType::Block
        | EntryType::Fifo => {
            return Err(OrliqError::Internal {
                message: format!(
                    "refused tar entry with special type {:?}: {}",
                    et,
                    entry
                        .header()
                        .path()
                        .ok()
                        .as_deref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<unreadable>".into())
                ),
            });
        }
        // GNU long-name / long-link / sparse / xglobal / xheader / other.
        // These are metadata-only entries that the `tar` crate handles
        // internally; if one reaches us as a visible entry we treat it as
        // a malformed archive.
        _ => {
            return Err(OrliqError::Internal {
                message: format!("refused unsupported tar entry type {:?}", et),
            });
        }
    }

    // Pull the path; reject unreadable.
    let path = entry.path().map_err(io_err)?.into_owned();
    let path_str = path.to_string_lossy();

    // 5. Length guard.
    if path_str.len() > 4096 {
        return Err(OrliqError::Internal {
            message: "refused tar entry with absurdly long path".into(),
        });
    }

    // 1. Literal prefix checks. Tar paths are forward-slash by spec, but
    // a malicious writer may use backslashes or drive letters; reject all.
    if path_str.is_empty() {
        return Err(OrliqError::Internal {
            message: "refused tar entry with empty path".into(),
        });
    }
    if path_str.starts_with('/') || path_str.starts_with('\\') || path_str.starts_with("\\\\") {
        return Err(OrliqError::Internal {
            message: format!("refused rooted/UNC path: {}", path_str),
        });
    }
    if path_str.len() >= 2
        && path_str.as_bytes()[1] == b':'
        && path_str.as_bytes()[0].is_ascii_alphabetic()
    {
        return Err(OrliqError::Internal {
            message: format!("refused drive-prefixed path: {}", path_str),
        });
    }

    // 2. Component-level checks. Every component must be Normal.
    for comp in path.components() {
        match comp {
            Component::Normal(_) => {}
            Component::CurDir => {
                // `./foo` — strip silently; not malicious, just noise.
            }
            _ => {
                return Err(OrliqError::Internal {
                    message: format!(
                        "refused tar entry with non-normal path component {:?}: {}",
                        comp, path_str
                    ),
                });
            }
        }
    }

    // Manifest is handled in the first pass — skip it here.
    if path == Path::new(MANIFEST_NAME) {
        return Ok(EntryDecision::Skip);
    }

    // 4. Allow-list. Normalise by collecting only Normal components (so
    // `./foo/bar` → `foo/bar`) then check the first two.
    let comps: Vec<&std::ffi::OsStr> = path
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s),
            _ => None,
        })
        .collect();
    if comps.is_empty() {
        return Err(OrliqError::Internal {
            message: format!("refused empty/root-only path: {}", path_str),
        });
    }
    let first = comps[0].to_string_lossy();
    let ok = match first.as_ref() {
        // config/settings.json — exact match required.
        "config" => comps.len() == 2 && comps[1].to_string_lossy() == "settings.json",
        // data/<single-file> — e.g. utterbar.db, utterbar.db-wal, utterbar.db-shm.
        // No subdirectories.
        "data" => comps.len() == 2,
        // recordings/<single-file>.wav — no subdirectories.
        "recordings" => {
            comps.len() == 2
                && comps[1]
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .ends_with(".wav")
        }
        _ => false,
    };
    if !ok {
        return Err(OrliqError::Internal {
            message: format!("refused path outside allow-list: {}", path_str),
        });
    }

    // Build the safe relative path from the normalised components so the
    // caller extracts to a canonical location (no `./` prefix, no mixed
    // separators).
    let mut rel = PathBuf::new();
    for c in &comps {
        rel.push(c);
    }
    Ok(EntryDecision::Extract(rel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tar::{Builder, EntryType, Header};

    /// Build a single-entry in-memory tar archive and run `validate_entry_path`
    /// on its first entry. Returns Ok/Err so tests can assert with the
    /// standard test harness.
    fn check_single_entry(
        path: &str,
        entry_type: EntryType,
        body: &[u8],
    ) -> Result<EntryDecision, String> {
        let mut buf = Vec::new();
        {
            let mut builder = Builder::new(Cursor::new(&mut buf));
            let mut header = Header::new_gnu();
            header.set_entry_type(entry_type);
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            // append_data writes the path verbatim when given `Header` so we
            // can smuggle attacker-shaped paths without the builder
            // normalising them.
            builder
                .append_data(&mut header, path, body)
                .map_err(|e| format!("append: {e}"))?;
            builder.finish().map_err(|e| format!("finish: {e}"))?;
        }
        let mut archive = tar::Archive::new(Cursor::new(&buf));
        let mut entries = archive.entries().map_err(|e| e.to_string())?;
        let entry = entries
            .next()
            .ok_or_else(|| "no entries".to_string())?
            .map_err(|e| e.to_string())?;
        validate_entry_path(&entry).map_err(|e| format!("{e:?}"))
    }

    #[test]
    fn rejects_parent_dir_traversal() {
        let r = check_single_entry("../../etc/passwd", EntryType::Regular, b"x");
        assert!(r.is_err(), "expected ../.. rejection, got {r:?}");
    }

    #[test]
    fn rejects_absolute_unix_path() {
        let r = check_single_entry("/etc/passwd", EntryType::Regular, b"x");
        assert!(r.is_err(), "expected /abs rejection, got {r:?}");
    }

    #[test]
    fn rejects_windows_drive_prefix() {
        let r = check_single_entry("C:/Windows/System32/foo", EntryType::Regular, b"x");
        assert!(r.is_err(), "expected C: rejection, got {r:?}");
    }

    #[test]
    fn rejects_backslash_root_path() {
        let r = check_single_entry("\\Windows\\foo", EntryType::Regular, b"x");
        assert!(r.is_err(), "expected \\ root rejection, got {r:?}");
    }

    #[test]
    fn rejects_unc_path() {
        let r = check_single_entry("\\\\server\\share\\foo", EntryType::Regular, b"x");
        assert!(r.is_err(), "expected UNC rejection, got {r:?}");
    }

    #[test]
    fn rejects_symlink_entry() {
        let r = check_single_entry("config/settings.json", EntryType::Symlink, b"");
        assert!(r.is_err(), "expected symlink rejection, got {r:?}");
    }

    #[test]
    fn rejects_hardlink_entry() {
        let r = check_single_entry("config/settings.json", EntryType::Link, b"");
        assert!(r.is_err(), "expected hardlink rejection, got {r:?}");
    }

    #[test]
    fn rejects_char_device_entry() {
        let r = check_single_entry("data/utterbar.db", EntryType::Char, b"");
        assert!(r.is_err(), "expected char-device rejection, got {r:?}");
    }

    #[test]
    fn rejects_path_outside_allowlist() {
        let r = check_single_entry("models/parakeet.onnx", EntryType::Regular, b"x");
        assert!(r.is_err(), "expected allow-list rejection, got {r:?}");
    }

    #[test]
    fn rejects_config_subdirectory() {
        let r = check_single_entry("config/nested/settings.json", EntryType::Regular, b"x");
        assert!(r.is_err(), "expected config subdir rejection, got {r:?}");
    }

    #[test]
    fn rejects_data_subdirectory() {
        let r = check_single_entry("data/subdir/utterbar.db", EntryType::Regular, b"x");
        assert!(r.is_err(), "expected data subdir rejection, got {r:?}");
    }

    #[test]
    fn rejects_recordings_non_wav() {
        let r = check_single_entry("recordings/evil.exe", EntryType::Regular, b"x");
        assert!(r.is_err(), "expected non-wav rejection, got {r:?}");
    }

    #[test]
    fn allows_config_settings_json() {
        let r = check_single_entry("config/settings.json", EntryType::Regular, b"{}");
        assert!(matches!(r, Ok(EntryDecision::Extract(_))), "got {r:?}");
    }

    #[test]
    fn allows_data_db_files() {
        for name in &["utterbar.db", "utterbar.db-wal", "utterbar.db-shm"] {
            let p = format!("data/{name}");
            let r = check_single_entry(&p, EntryType::Regular, b"x");
            assert!(
                matches!(r, Ok(EntryDecision::Extract(_))),
                "path {p}: {r:?}"
            );
        }
    }

    #[test]
    fn allows_recordings_wav() {
        let r = check_single_entry("recordings/clip-01.wav", EntryType::Regular, b"x");
        assert!(matches!(r, Ok(EntryDecision::Extract(_))), "got {r:?}");
    }

    #[test]
    fn skips_manifest() {
        let r = check_single_entry(MANIFEST_NAME, EntryType::Regular, b"{}");
        assert!(matches!(r, Ok(EntryDecision::Skip)), "got {r:?}");
    }

    #[test]
    fn skips_directory_entry() {
        let r = check_single_entry("config/", EntryType::Directory, b"");
        assert!(matches!(r, Ok(EntryDecision::Skip)), "got {r:?}");
    }

    #[test]
    fn rejects_absurdly_long_path() {
        let long = "a/".repeat(3000);
        let r = check_single_entry(&long, EntryType::Regular, b"x");
        assert!(r.is_err(), "expected length rejection, got {r:?}");
    }
}
