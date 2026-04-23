//! Vocabulary + text replacements. Replacements applied between ASR and LLM.

use std::sync::Arc;

use anyhow::Result;
use rusqlite::params;

use crate::db::Db;
use crate::domain::{Replacement, VocabEntry};
use crate::util::time::now_ms;

pub struct VocabularyService {
    db: Arc<Db>,
}

impl VocabularyService {
    pub fn new(db: Arc<Db>) -> Self { Self { db } }

    pub fn list_words(&self) -> Result<Vec<VocabEntry>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare("SELECT id,word,hint,weight,enabled,created_at FROM vocabulary ORDER BY word")?;
        let rows = stmt.query_map([], |r| Ok(VocabEntry {
            id: r.get(0)?, word: r.get(1)?, hint: r.get(2)?,
            weight: r.get(3)?, enabled: r.get::<_, i64>(4)? != 0, created_at: r.get(5)?,
        }))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upsert_word(&self, mut e: VocabEntry) -> Result<VocabEntry> {
        if e.id.is_empty() { e.id = ulid::Ulid::new().to_string(); }
        if e.created_at == 0 { e.created_at = now_ms(); }
        let conn = self.db.conn()?;
        conn.execute(
            "INSERT INTO vocabulary(id,word,hint,weight,enabled,created_at) VALUES(?1,?2,?3,?4,?5,?6)
             ON CONFLICT(word) DO UPDATE SET hint=excluded.hint, weight=excluded.weight, enabled=excluded.enabled",
            params![e.id, e.word, e.hint, e.weight, e.enabled as i64, e.created_at],
        )?;
        Ok(e)
    }

    pub fn delete_word(&self, id: &str) -> Result<()> {
        let conn = self.db.conn()?;
        conn.execute("DELETE FROM vocabulary WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn list_replacements(&self) -> Result<Vec<Replacement>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare("SELECT id,pattern,replacement,is_regex,case_sens,enabled,sort_order,created_at FROM replacements ORDER BY sort_order")?;
        let rows = stmt.query_map([], |r| Ok(Replacement {
            id: r.get(0)?, pattern: r.get(1)?, replacement: r.get(2)?,
            is_regex: r.get::<_, i64>(3)? != 0, case_sens: r.get::<_, i64>(4)? != 0,
            enabled: r.get::<_, i64>(5)? != 0, sort_order: r.get(6)?, created_at: r.get(7)?,
        }))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upsert_replacement(&self, mut r: Replacement) -> Result<Replacement> {
        if r.id.is_empty() { r.id = ulid::Ulid::new().to_string(); }
        if r.created_at == 0 { r.created_at = now_ms(); }
        let conn = self.db.conn()?;
        conn.execute(
            "INSERT INTO replacements(id,pattern,replacement,is_regex,case_sens,enabled,sort_order,created_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(id) DO UPDATE SET pattern=excluded.pattern, replacement=excluded.replacement,
                is_regex=excluded.is_regex, case_sens=excluded.case_sens, enabled=excluded.enabled,
                sort_order=excluded.sort_order",
            params![r.id, r.pattern, r.replacement, r.is_regex as i64, r.case_sens as i64, r.enabled as i64, r.sort_order, r.created_at],
        )?;
        Ok(r)
    }

    pub fn delete_replacement(&self, id: &str) -> Result<()> {
        let conn = self.db.conn()?;
        conn.execute("DELETE FROM replacements WHERE id=?1", params![id])?;
        Ok(())
    }

    /// Apply enabled literal replacements (regex deferred to v1.1).
    pub fn apply_replacements(&self, text: &str) -> Result<String> {
        let list = self.list_replacements()?;
        let mut out = text.to_string();
        for r in list.into_iter().filter(|r| r.enabled && !r.is_regex) {
            out = if r.case_sens {
                out.replace(&r.pattern, &r.replacement)
            } else {
                replace_case_insensitive(&out, &r.pattern, &r.replacement)
            };
        }
        Ok(out)
    }
}

fn replace_case_insensitive(haystack: &str, needle: &str, with: &str) -> String {
    if needle.is_empty() { return haystack.to_string(); }
    let lower_h = haystack.to_lowercase();
    let lower_n = needle.to_lowercase();
    let mut out = String::with_capacity(haystack.len());
    let mut idx = 0usize;
    while idx < haystack.len() {
        if lower_h[idx..].starts_with(&lower_n) {
            out.push_str(with);
            idx += needle.len();
        } else {
            let c = haystack[idx..].chars().next().unwrap();
            out.push(c);
            idx += c.len_utf8();
        }
    }
    out
}
