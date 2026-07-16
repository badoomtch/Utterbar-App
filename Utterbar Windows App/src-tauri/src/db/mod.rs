//! SQLite pool + migrations.

use std::path::{Path, PathBuf};

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

// Made `pub(crate)` so `db::repair`'s end-to-end test can run the
// migrations runner against an in-memory SQLite. The macro itself
// stays inside the inner module so production code paths still go
// through `Db::run_migrations`.
pub(crate) mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/db/migrations");
}

pub mod repair;

pub struct Db {
    pub pool: Pool<SqliteConnectionManager>,
    pub path: PathBuf,
}

impl Db {
    pub fn open(data_dir: &Path) -> Result<Self> {
        let dir = data_dir.join("data");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("orliq.db");
        // Self-heal corrupted DB before the pool tries to open it.
        // Without this, every `r2d2::Pool::get()` returns "file is not a
        // database" and `AppState::bootstrap` panics, leaving the user
        // unable to launch the app at all. Restoring from a recent
        // backup is far less destructive than the panic — the user
        // loses at most the work between the last backup and the
        // corruption, vs. losing access entirely.
        match recover_if_corrupted(&path) {
            Ok(outcome) => match outcome {
                RecoveryOutcome::Valid | RecoveryOutcome::NoFile => {}
                RecoveryOutcome::RecoveredFromBackup { from, asided } => {
                    tracing::warn!(
                        from = %from.display(),
                        asided = %asided.display(),
                        "db: orliq.db was corrupted on boot; restored from latest valid backup"
                    );
                }
                RecoveryOutcome::BrokenAsidedNoBackup { asided } => {
                    tracing::warn!(
                        asided = %asided.display(),
                        "db: orliq.db was corrupted on boot and no valid backup exists; starting fresh"
                    );
                }
            },
            Err(e) => {
                // Surface but proceed — letting r2d2 try its luck is
                // strictly better than refusing to launch. If r2d2 also
                // fails, the existing panic path runs.
                tracing::warn!(
                    ?e,
                    "db: corruption check failed; proceeding without recovery"
                );
            }
        }
        let manager = SqliteConnectionManager::file(&path).with_init(|c| {
            c.pragma_update(None, "journal_mode", "WAL")?;
            c.pragma_update(None, "synchronous", "NORMAL")?;
            c.pragma_update(None, "foreign_keys", "ON")?;
            c.pragma_update(None, "busy_timeout", 5000)?;
            Ok(())
        });
        let pool = Pool::builder().max_size(8).build(manager)?;
        Ok(Self { pool, path })
    }

    pub fn run_migrations(&self) -> Result<()> {
        let mut conn = self.pool.get()?;
        let conn: &mut Connection = &mut conn;

        // v0.15.2 pre-refinery repair: corrects the V6 checksum
        // on DBs that had v0.15.1 as their first 0.15.x launch
        // (line-ending drift between v0.15.0 LF and v0.15.1 CRLF
        // builds — see services/db/repair.rs for the full story).
        // Single-purpose, single-row, single-checksum-pair. If the
        // repair surfaces an unknown / unexpected state it leaves
        // the row alone; refinery's normal divergence panic then
        // fires with the correct error rather than being masked.
        let outcome = repair::repair_known_v6_history_speaker_names_line_ending_checksum(conn)?;
        tracing::info!(?outcome, "db::run_migrations v6-checksum repair");

        embedded::migrations::runner().run(conn)?;
        Ok(())
    }

    /// Verify that the DB is readable. On failure the caller can decide whether to
    /// restore from the most recent backup — we never silently replace a live DB.
    pub fn integrity_check(&self) -> Result<bool> {
        let conn = self.pool.get()?;
        let result: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        Ok(result == "ok")
    }

    /// Snapshot the DB file to `orliq.db.backup-<ts>` next to the live file and keep only
    /// the most recent 3 backups. Runs at startup before migrations so a corrupt DB can
    /// still be inspected out-of-band.
    pub fn backup(&self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }
        let parent = self
            .path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("db path has no parent"))?;
        let ts = crate::util::time::now_ms();
        let backup_path = parent.join(format!("orliq.db.backup-{ts}"));
        std::fs::copy(&self.path, &backup_path)?;
        prune_old_backups(parent, 3)?;
        Ok(())
    }

    pub fn conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        Ok(self.pool.get()?)
    }
}

fn prune_old_backups(dir: &std::path::Path, keep: usize) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("orliq.db.backup-")
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());
    entries.reverse();
    for old in entries.into_iter().skip(keep) {
        let _ = std::fs::remove_file(old.path());
    }
    Ok(())
}

/// SQLite file format magic — every valid SQLite 3 DB starts with these
/// 16 bytes (`SQLite format 3\0`). See
/// <https://www.sqlite.org/fileformat.html#magic_header_string>.
const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";

/// Outcome of [`recover_if_corrupted`]. Caller logs the variant so a user
/// who later reports "where did my history go?" has a breadcrumb in
/// `utterbar.log` pointing at the swap.
#[derive(Debug)]
enum RecoveryOutcome {
    /// File doesn't exist yet (genuine first launch / fresh install).
    NoFile,
    /// File starts with the SQLite header — nothing to do.
    Valid,
    /// File was corrupted; restored from `from` and the broken bytes
    /// are preserved at `asided` for forensics.
    RecoveredFromBackup {
        from: std::path::PathBuf,
        asided: std::path::PathBuf,
    },
    /// File was corrupted and no usable backup existed; broken bytes
    /// preserved at `asided` and a fresh DB will be created.
    BrokenAsidedNoBackup { asided: std::path::PathBuf },
}

/// Detect a corrupted `orliq.db` at boot and swap in the newest valid
/// backup. The corruption case Utterbar has actually hit in the wild
/// (Jay's restore-from-old-machine, 2026-05-20) is "the file is on disk
/// but its bytes are not a SQLite database" — r2d2 then loops on
/// `file is not a database` until `bootstrap AppState` panics and the
/// user is locked out of the app.
///
/// We detect that case by reading the first 16 bytes and comparing
/// against the SQLite magic header. Anything else we leave alone:
/// integrity-failures past the header are not our problem here (they
/// would only fire after r2d2 can open the file at all, by which time
/// the existing migration / refinery error paths run).
///
/// The broken file is always preserved at `orliq.db.broken-<ts>` rather
/// than deleted. Two reasons: (1) a future post-mortem can inspect what
/// the restore corrupted, and (2) overwriting state without a trail is
/// the worst-case if our header check ever false-positives.
fn recover_if_corrupted(path: &Path) -> Result<RecoveryOutcome> {
    if !path.exists() {
        return Ok(RecoveryOutcome::NoFile);
    }
    if file_has_sqlite_header(path)? {
        return Ok(RecoveryOutcome::Valid);
    }
    // Broken file: aside it and try to restore.
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("db path has no parent"))?;
    let asided = aside_path(parent);
    std::fs::rename(path, &asided)?;
    // Find the newest backup whose header is intact. We sort by name
    // descending — backup files use `orliq.db.backup-<unix_ms>` so the
    // lexicographic order matches the chronological order for any
    // fixed-width timestamp range, and the pre-seed backup (which has
    // a different suffix shape) lands at the end naturally.
    let mut entries: Vec<_> = std::fs::read_dir(parent)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("orliq.db.backup-")
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());
    entries.reverse();
    for entry in entries {
        let candidate = entry.path();
        if matches!(file_has_sqlite_header(&candidate), Ok(true)) {
            std::fs::copy(&candidate, path)?;
            return Ok(RecoveryOutcome::RecoveredFromBackup {
                from: candidate,
                asided,
            });
        }
    }
    Ok(RecoveryOutcome::BrokenAsidedNoBackup { asided })
}

/// True iff the file's first 16 bytes equal the SQLite 3 magic header.
/// A file shorter than 16 bytes is reported as non-SQLite, not as an
/// error.
fn file_has_sqlite_header(path: &Path) -> Result<bool> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut buf = [0u8; 16];
    match f.read(&mut buf)? {
        n if n == SQLITE_MAGIC.len() => Ok(&buf == SQLITE_MAGIC),
        _ => Ok(false),
    }
}

/// Build a fresh `orliq.db.broken-<unix_ms>` path next to the DB. A unix
/// millisecond suffix means repeated launches against the same broken
/// file don't clobber the earlier forensic copy.
fn aside_path(dir: &Path) -> std::path::PathBuf {
    let ts = crate::util::time::now_ms();
    dir.join(format!("orliq.db.broken-{ts}"))
}

#[cfg(test)]
mod recovery_tests {
    //! Coverage of [`recover_if_corrupted`] across the four shapes the
    //! caller cares about: missing file, valid DB, corrupted DB with a
    //! usable backup, corrupted DB with no usable backup.
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Pattern matching `services/history.rs` and `commands/licence.rs`:
    /// build a unique per-test directory under the system temp root so
    /// parallel cargo test runs don't trip over each other.
    fn fresh_dir(label: &str) -> std::path::PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::SeqCst);
        let d =
            std::env::temp_dir().join(format!("ut-db-recovery-{label}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn write_bytes(p: &Path, bytes: &[u8]) {
        let mut f = std::fs::File::create(p).unwrap();
        f.write_all(bytes).unwrap();
    }

    #[test]
    fn missing_file_is_noop() {
        let dir = fresh_dir("missing");
        let p = dir.join("orliq.db");
        let outcome = recover_if_corrupted(&p).unwrap();
        assert!(matches!(outcome, RecoveryOutcome::NoFile));
        assert!(!p.exists(), "must not create a file in the NoFile path");
    }

    #[test]
    fn valid_file_is_unchanged() {
        let dir = fresh_dir("valid");
        let p = dir.join("orliq.db");
        // Header + a few zero pages — looks enough like SQLite to pass
        // our cheap check. Real validity is a refinery concern.
        let mut bytes = Vec::from(&SQLITE_MAGIC[..]);
        bytes.resize(4096, 0);
        write_bytes(&p, &bytes);
        let original = std::fs::read(&p).unwrap();
        let outcome = recover_if_corrupted(&p).unwrap();
        assert!(matches!(outcome, RecoveryOutcome::Valid));
        assert_eq!(
            std::fs::read(&p).unwrap(),
            original,
            "live DB must not be touched"
        );
    }

    #[test]
    fn corrupted_file_restored_from_newest_valid_backup() {
        let dir = fresh_dir("restore");
        let p = dir.join("orliq.db");
        // Broken main DB.
        write_bytes(&p, b"this is not a sqlite file at all");
        // Two valid backups; the second is newer (larger suffix).
        let older = dir.join("orliq.db.backup-1000");
        let newer = dir.join("orliq.db.backup-2000");
        let mut older_bytes = Vec::from(&SQLITE_MAGIC[..]);
        older_bytes.extend_from_slice(b"-older");
        let mut newer_bytes = Vec::from(&SQLITE_MAGIC[..]);
        newer_bytes.extend_from_slice(b"-newer");
        write_bytes(&older, &older_bytes);
        write_bytes(&newer, &newer_bytes);

        let outcome = recover_if_corrupted(&p).unwrap();
        match outcome {
            RecoveryOutcome::RecoveredFromBackup { from, asided } => {
                assert_eq!(from, newer, "must pick the newest valid backup");
                assert!(asided.exists(), "broken bytes must be preserved");
                let asided_bytes = std::fs::read(&asided).unwrap();
                assert_eq!(&asided_bytes, b"this is not a sqlite file at all");
            }
            other => panic!("expected RecoveredFromBackup, got {other:?}"),
        }
        // The live DB now has the newer backup's bytes.
        assert_eq!(std::fs::read(&p).unwrap(), newer_bytes);
    }

    #[test]
    fn corrupted_file_skips_broken_backups_and_picks_valid_older_one() {
        let dir = fresh_dir("skip-broken");
        let p = dir.join("orliq.db");
        write_bytes(&p, b"corrupt");
        // Newer backup is itself broken — should be skipped.
        let broken_newer = dir.join("orliq.db.backup-3000");
        write_bytes(&broken_newer, b"also corrupt");
        let valid_older = dir.join("orliq.db.backup-1500");
        let mut older_bytes = Vec::from(&SQLITE_MAGIC[..]);
        older_bytes.extend_from_slice(b"-valid");
        write_bytes(&valid_older, &older_bytes);

        let outcome = recover_if_corrupted(&p).unwrap();
        match outcome {
            RecoveryOutcome::RecoveredFromBackup { from, .. } => {
                assert_eq!(from, valid_older);
            }
            other => panic!("expected RecoveredFromBackup, got {other:?}"),
        }
    }

    #[test]
    fn corrupted_file_with_no_valid_backup_is_asided_and_left_for_fresh_create() {
        let dir = fresh_dir("no-backup");
        let p = dir.join("orliq.db");
        write_bytes(&p, b"corrupt");
        // A backup is present but also corrupted.
        let broken_backup = dir.join("orliq.db.backup-1");
        write_bytes(&broken_backup, b"also corrupt");

        let outcome = recover_if_corrupted(&p).unwrap();
        match outcome {
            RecoveryOutcome::BrokenAsidedNoBackup { asided } => {
                assert!(asided.exists(), "broken bytes must be preserved");
                assert_eq!(std::fs::read(&asided).unwrap(), b"corrupt");
            }
            other => panic!("expected BrokenAsidedNoBackup, got {other:?}"),
        }
        // Live path is gone; r2d2/SQLite will create a fresh empty DB
        // on first connection. We assert absence, not contents.
        assert!(!p.exists(), "broken live DB must be moved aside");
    }
}
