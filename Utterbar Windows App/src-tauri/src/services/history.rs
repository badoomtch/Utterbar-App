//! History queries.

use std::sync::Arc;

use anyhow::Result;
use rusqlite::params;

use crate::db::Db;
use crate::domain::{HistoryEntry, HistoryFilter, HistoryStatus};

pub struct HistoryService {
    db: Arc<Db>,
}

impl HistoryService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub fn insert(&self, entry: &HistoryEntry) -> Result<()> {
        let conn = self.db.conn()?;
        conn.execute(
            "INSERT INTO history(id,created_at,mode_id,mode_name_snapshot,asr_model_id,asr_model_version,audio_path,audio_duration_ms,raw_text,processed_text,language,processing_ms,status,error_message,app_exe,app_title,context_payload,word_timings)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
            params![
                entry.id, entry.created_at, entry.mode_id, entry.mode_name_snapshot,
                entry.asr_model_id, entry.asr_model_version, entry.audio_path, entry.audio_duration_ms,
                entry.raw_text, entry.processed_text, entry.language, entry.processing_ms,
                serde_json::to_string(&entry.status)?,
                entry.error_message, entry.app_exe, entry.app_title,
                entry.context_payload, entry.word_timings,
            ],
        )?;
        Ok(())
    }

    pub fn list(&self, filter: HistoryFilter) -> Result<Vec<HistoryEntry>> {
        let mut sql = String::from(
            "SELECT id,created_at,mode_id,mode_name_snapshot,asr_model_id,asr_model_version,audio_path,audio_duration_ms,raw_text,processed_text,language,processing_ms,status,error_message,app_exe,app_title,context_payload,word_timings FROM history WHERE 1=1"
        );
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(m) = filter.mode_id {
            sql.push_str(" AND mode_id=?");
            args.push(Box::new(m));
        }
        if let Some(a) = filter.app_exe {
            sql.push_str(" AND app_exe=?");
            args.push(Box::new(a));
        }
        if let Some(s) = filter.search {
            sql.push_str(" AND (raw_text LIKE ? OR processed_text LIKE ?)");
            let pat = format!("%{s}%");
            args.push(Box::new(pat.clone()));
            args.push(Box::new(pat));
        }
        if let Some(f) = filter.from_ms {
            sql.push_str(" AND created_at >= ?");
            args.push(Box::new(f));
        }
        if let Some(t) = filter.to_ms {
            sql.push_str(" AND created_at <= ?");
            args.push(Box::new(t));
        }
        if let Some(st) = filter.status {
            sql.push_str(" AND status = ?");
            // Status is serialised the same way as on insert — a JSON
            // string of the enum variant ("\"success\"" / "\"error\"" /
            // "\"cancelled\"").
            args.push(Box::new(serde_json::to_string(&st).unwrap_or_else(|_| "\"success\"".into())));
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        args.push(Box::new(filter.limit.unwrap_or(200)));
        args.push(Box::new(filter.offset.unwrap_or(0)));

        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = args
            .iter()
            .map(|b| b.as_ref() as &dyn rusqlite::ToSql)
            .collect();
        let rows = stmt.query_map(rusqlite::params_from_iter(param_refs), row_to_history)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get(&self, id: &str) -> Result<Option<HistoryEntry>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id,created_at,mode_id,mode_name_snapshot,asr_model_id,asr_model_version,audio_path,audio_duration_ms,raw_text,processed_text,language,processing_ms,status,error_message,app_exe,app_title,context_payload,word_timings FROM history WHERE id=?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row_to_history(row)?)),
            None => Ok(None),
        }
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.db.conn()?;
        conn.execute("DELETE FROM history WHERE id=?1", params![id])?;
        Ok(())
    }

    /// Return the distinct non-null `app_exe` values across all history
    /// rows, ordered by recency-of-use (most recent first). Used to
    /// populate the History page's "App" filter dropdown without having
    /// the frontend re-derive the set on every list render.
    pub fn distinct_app_exes(&self) -> Result<Vec<String>> {
        let conn = self.db.conn()?;
        // Subquery picks the max(created_at) per app_exe so we can
        // order the outer SELECT by that — the user's "most-recently-
        // used" app surfaces at the top of the dropdown.
        let mut stmt = conn.prepare(
            "SELECT app_exe FROM history
             WHERE app_exe IS NOT NULL AND app_exe != ''
             GROUP BY app_exe
             ORDER BY MAX(created_at) DESC
             LIMIT 50",
        )?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

fn row_to_history(row: &rusqlite::Row) -> rusqlite::Result<HistoryEntry> {
    let status: String = row.get(12)?;
    Ok(HistoryEntry {
        id: row.get(0)?,
        created_at: row.get(1)?,
        mode_id: row.get(2)?,
        mode_name_snapshot: row.get(3)?,
        asr_model_id: row.get(4)?,
        asr_model_version: row.get(5)?,
        audio_path: row.get(6)?,
        audio_duration_ms: row.get(7)?,
        raw_text: row.get(8)?,
        processed_text: row.get(9)?,
        language: row.get(10)?,
        processing_ms: row.get(11)?,
        status: serde_json::from_str(&status).unwrap_or(HistoryStatus::Success),
        error_message: row.get(13)?,
        app_exe: row.get(14)?,
        app_title: row.get(15)?,
        context_payload: row.get(16)?,
        word_timings: row.get(17)?,
    })
}
