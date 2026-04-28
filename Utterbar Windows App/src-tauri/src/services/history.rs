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
            "INSERT INTO history(id,created_at,mode_id,mode_name_snapshot,asr_model_id,asr_model_version,audio_path,audio_duration_ms,raw_text,processed_text,language,processing_ms,status,error_message,app_exe,app_title,context_payload,word_timings,is_meeting,speaker_names)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
            params![
                entry.id, entry.created_at, entry.mode_id, entry.mode_name_snapshot,
                entry.asr_model_id, entry.asr_model_version, entry.audio_path, entry.audio_duration_ms,
                entry.raw_text, entry.processed_text, entry.language, entry.processing_ms,
                serde_json::to_string(&entry.status)?,
                entry.error_message, entry.app_exe, entry.app_title,
                entry.context_payload, entry.word_timings,
                entry.is_meeting as i32,
                entry.speaker_names,
            ],
        )?;
        Ok(())
    }

    /// Update the rename map on a meeting row. `names_json` is the
    /// serialised JSON object — caller is responsible for validation
    /// (see `commands::meetings::set_meeting_speaker_names` for the
    /// rules: integer keys ≥1 ≤32, non-empty trimmed names ≤64
    /// chars, no control chars). Pass `None` to clear back to the
    /// raw `**Speaker N:**` labels.
    ///
    /// Scoped to `is_meeting = 1` rows. Trying to set speaker names
    /// on a non-meeting (dictation) row, or on a row that doesn't
    /// exist at all, returns an error rather than silently writing
    /// nothing — the rename map has no rendering effect on a
    /// dictation row (no `**Speaker N:**` tokens to substitute) and
    /// silently accepting the call would mask UI bugs.
    pub fn update_speaker_names(&self, id: &str, names_json: Option<&str>) -> Result<()> {
        let conn = self.db.conn()?;
        let affected = conn.execute(
            "UPDATE history SET speaker_names=?1 WHERE id=?2 AND is_meeting=1",
            params![names_json, id],
        )?;
        if affected == 0 {
            anyhow::bail!("meeting transcript not found");
        }
        Ok(())
    }

    pub fn list(&self, filter: HistoryFilter) -> Result<Vec<HistoryEntry>> {
        let mut sql = String::from(
            "SELECT id,created_at,mode_id,mode_name_snapshot,asr_model_id,asr_model_version,audio_path,audio_duration_ms,raw_text,processed_text,language,processing_ms,status,error_message,app_exe,app_title,context_payload,word_timings,is_meeting,speaker_names FROM history WHERE 1=1"
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
            args.push(Box::new(
                serde_json::to_string(&st).unwrap_or_else(|_| "\"success\"".into()),
            ));
        }
        if let Some(meeting) = filter.is_meeting {
            sql.push_str(" AND is_meeting = ?");
            args.push(Box::new(meeting as i32));
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
            "SELECT id,created_at,mode_id,mode_name_snapshot,asr_model_id,asr_model_version,audio_path,audio_duration_ms,raw_text,processed_text,language,processing_ms,status,error_message,app_exe,app_title,context_payload,word_timings,is_meeting,speaker_names FROM history WHERE id=?1"
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
        is_meeting: row.get::<_, i32>(18)? != 0,
        speaker_names: row.get(19)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    /// Construct a fresh `HistoryService` backed by a real on-disk
    /// SQLite db with all migrations applied. We can't use `:memory:`
    /// because `Db::open` always lays out under `<data_dir>/data/`;
    /// using a temp data_dir keeps each test isolated. Cleaned up by
    /// the harness when the temp dir is dropped at process exit; for
    /// now we don't bother with explicit teardown — the temp paths
    /// are tiny and the Windows build agent reaps them between runs.
    fn fresh_service() -> HistoryService {
        let n = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let temp =
            std::env::temp_dir().join(format!("ut-history-test-{}-{}", std::process::id(), n,));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).expect("create temp data_dir");
        let db = Db::open(&temp).expect("open db");
        db.run_migrations().expect("migrations");
        HistoryService::new(std::sync::Arc::new(db))
    }

    fn make_entry(id: &str, is_meeting: bool, speaker_names: Option<&str>) -> HistoryEntry {
        HistoryEntry {
            id: id.into(),
            created_at: 1_700_000_000_000,
            // Tests don't seed the modes table, so mode_id stays
            // null to avoid a foreign-key constraint failure on
            // history.mode_id → modes.id.
            mode_id: None,
            mode_name_snapshot: "Voice-to-text".into(),
            asr_model_id: "stub".into(),
            asr_model_version: "stub".into(),
            audio_path: None,
            audio_duration_ms: 1000,
            raw_text: "hello world".into(),
            processed_text: "hello world".into(),
            language: None,
            processing_ms: 50,
            status: HistoryStatus::Success,
            error_message: None,
            app_exe: None,
            app_title: None,
            context_payload: None,
            word_timings: None,
            is_meeting,
            speaker_names: speaker_names.map(|s| s.to_string()),
        }
    }

    /// The blocker that prompted these tests: pre-fix, the INSERT
    /// column list contained `speaker_names` twice, which would
    /// have failed at runtime on every history insert. This pins
    /// that the column appears exactly once in the SQL by
    /// successfully inserting + reading back a row.
    #[test]
    fn insert_with_speaker_names_none_round_trips() {
        let svc = fresh_service();
        let entry = make_entry("01HX-NONE", true, None);
        svc.insert(&entry).expect("insert");
        let read = svc.get(&entry.id).expect("get").expect("present");
        assert_eq!(read.speaker_names, None);
        assert_eq!(read.id, entry.id);
        assert!(read.is_meeting);
    }

    #[test]
    fn insert_with_speaker_names_some_round_trips() {
        let svc = fresh_service();
        let json = r#"{"1":"Jay","2":"Customer"}"#;
        let entry = make_entry("01HX-SOME", true, Some(json));
        svc.insert(&entry).expect("insert");
        let read = svc.get(&entry.id).expect("get").expect("present");
        assert_eq!(read.speaker_names.as_deref(), Some(json));
    }

    /// list() must return the speaker_names column too — the
    /// /meetings page reads this surface, not the per-id get().
    #[test]
    fn list_includes_speaker_names() {
        let svc = fresh_service();
        let json = r#"{"1":"Jay"}"#;
        svc.insert(&make_entry("01HX-LIST-1", true, Some(json)))
            .expect("insert");
        let rows = svc
            .list(crate::domain::HistoryFilter {
                is_meeting: Some(true),
                ..Default::default()
            })
            .expect("list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].speaker_names.as_deref(), Some(json));
    }

    /// `update_speaker_names` ONLY touches meeting rows. A
    /// dictation row with the same id isn't legal in practice
    /// (ids are ULIDs) but the predicate must defend against
    /// callers that mis-target any row.
    #[test]
    fn update_speaker_names_succeeds_for_meeting_row() {
        let svc = fresh_service();
        svc.insert(&make_entry("01HX-OK", true, None))
            .expect("insert");
        svc.update_speaker_names("01HX-OK", Some(r#"{"1":"Jay"}"#))
            .expect("update should succeed for meeting row");
        let read = svc.get("01HX-OK").expect("get").expect("present");
        assert_eq!(read.speaker_names.as_deref(), Some(r#"{"1":"Jay"}"#));
    }

    #[test]
    fn update_speaker_names_clears_when_passed_none() {
        let svc = fresh_service();
        svc.insert(&make_entry("01HX-CLEAR", true, Some(r#"{"1":"Jay"}"#)))
            .expect("insert");
        svc.update_speaker_names("01HX-CLEAR", None)
            .expect("clear should succeed");
        let read = svc.get("01HX-CLEAR").expect("get").expect("present");
        assert_eq!(read.speaker_names, None);
    }

    /// Missing id → user-visible error. The frontend rolls back its
    /// optimistic state on this path.
    #[test]
    fn update_speaker_names_errors_on_missing_id() {
        let svc = fresh_service();
        let err = svc
            .update_speaker_names("01HX-NOPE", Some(r#"{"1":"Jay"}"#))
            .expect_err("must error for missing id");
        let msg = err.to_string();
        assert!(
            msg.contains("not found"),
            "expected 'not found' in error, got: {msg}"
        );
    }

    /// Dictation rows (is_meeting = 0) must NOT be silently
    /// updated. Pre-fix, `update_speaker_names` ignored the
    /// `is_meeting` flag entirely; a confused frontend (or future
    /// rename surface widening) could have stamped a rename map
    /// onto a dictation row that has no `**Speaker N:**` tokens
    /// to render against.
    #[test]
    fn update_speaker_names_does_not_touch_dictation_row() {
        let svc = fresh_service();
        svc.insert(&make_entry("01HX-DICT", false, None))
            .expect("insert dictation row");
        let err = svc
            .update_speaker_names("01HX-DICT", Some(r#"{"1":"Jay"}"#))
            .expect_err("must error for dictation row");
        assert!(err.to_string().contains("not found"));
        // And the row must still have null speaker_names — the
        // failed UPDATE didn't mutate it.
        let read = svc.get("01HX-DICT").expect("get").expect("present");
        assert_eq!(read.speaker_names, None);
    }
}
