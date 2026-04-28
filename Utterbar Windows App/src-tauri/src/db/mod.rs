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
