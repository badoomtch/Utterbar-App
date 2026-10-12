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
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub fn list_words(&self) -> Result<Vec<VocabEntry>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id,word,hint,weight,enabled,created_at FROM vocabulary ORDER BY word",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(VocabEntry {
                id: r.get(0)?,
                word: r.get(1)?,
                hint: r.get(2)?,
                weight: r.get(3)?,
                enabled: r.get::<_, i64>(4)? != 0,
                created_at: r.get(5)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upsert_word(&self, mut e: VocabEntry) -> Result<VocabEntry> {
        if e.id.is_empty() {
            e.id = ulid::Ulid::new().to_string();
        }
        if e.created_at == 0 {
            e.created_at = now_ms();
        }
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
        let rows = stmt.query_map([], |r| {
            Ok(Replacement {
                id: r.get(0)?,
                pattern: r.get(1)?,
                replacement: r.get(2)?,
                is_regex: r.get::<_, i64>(3)? != 0,
                case_sens: r.get::<_, i64>(4)? != 0,
                enabled: r.get::<_, i64>(5)? != 0,
                sort_order: r.get(6)?,
                created_at: r.get(7)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upsert_replacement(&self, mut r: Replacement) -> Result<Replacement> {
        if r.id.is_empty() {
            r.id = ulid::Ulid::new().to_string();
        }
        if r.created_at == 0 {
            r.created_at = now_ms();
        }
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
    ///
    /// Word/phrase boundaries are honoured when a pattern starts and ends
    /// with word characters. This prevents a correction such as `blu -> blue`
    /// from changing the `blu` prefix inside `blurry`.
    pub fn apply_replacements(&self, text: &str) -> Result<String> {
        let list = self.list_replacements()?;
        let mut out = text.to_string();
        for r in list.into_iter().filter(|r| r.enabled && !r.is_regex) {
            out = replace_literal(&out, &r.pattern, &r.replacement, r.case_sens);
        }
        Ok(out)
    }
}

fn replace_literal(haystack: &str, needle: &str, with: &str, case_sensitive: bool) -> String {
    if needle.is_empty() {
        return haystack.to_string();
    }

    // Lowercase one source character at a time. Lowercasing the whole
    // haystack and indexing it with offsets from the original string is
    // invalid for Unicode case mappings whose byte length changes.
    let matches = if case_sensitive {
        find_literal_matches(haystack, needle)
    } else {
        find_case_insensitive_matches(haystack, needle)
    };
    if matches.is_empty() {
        return haystack.to_string();
    }

    let mut out = String::with_capacity(haystack.len());
    let mut copy_from = 0usize;
    for (start, end) in matches {
        out.push_str(&haystack[copy_from..start]);
        out.push_str(with);
        copy_from = end;
    }
    out.push_str(&haystack[copy_from..]);
    out
}

fn find_literal_matches(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    let mut scan_from = 0usize;
    while let Some(relative_start) = haystack[scan_from..].find(needle) {
        let start = scan_from + relative_start;
        let end = start + needle.len();
        if replacement_has_word_boundaries(haystack, needle, start, end) {
            matches.push((start, end));
            scan_from = end;
        } else {
            scan_from = start + haystack[start..].chars().next().unwrap().len_utf8();
        }
    }
    matches
}

fn find_case_insensitive_matches(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    let lower_needle = needle.to_lowercase();
    let segments: Vec<(usize, usize, String)> = haystack
        .char_indices()
        .map(|(start, ch)| {
            let end = start + ch.len_utf8();
            (start, end, ch.to_lowercase().collect())
        })
        .collect();

    let mut matches = Vec::new();
    let mut segment_index = 0usize;
    while segment_index < segments.len() {
        let mut lowered = String::new();
        let mut end_index = segment_index;
        while end_index < segments.len() && lowered.len() < lower_needle.len() {
            lowered.push_str(&segments[end_index].2);
            end_index += 1;
        }
        if lowered == lower_needle {
            let start = segments[segment_index].0;
            let end = segments[end_index - 1].1;
            if replacement_has_word_boundaries(haystack, needle, start, end) {
                matches.push((start, end));
                segment_index = end_index;
                continue;
            }
        }
        segment_index += 1;
    }
    matches
}

fn replacement_has_word_boundaries(haystack: &str, needle: &str, start: usize, end: usize) -> bool {
    let starts_word = needle.chars().next().is_some_and(is_word_character);
    let ends_word = needle.chars().next_back().is_some_and(is_word_character);
    let previous_is_word = haystack[..start]
        .chars()
        .next_back()
        .is_some_and(is_word_character);
    let next_is_word = haystack[end..]
        .chars()
        .next()
        .is_some_and(is_word_character);
    (!starts_word || !previous_is_word) && (!ends_word || !next_is_word)
}

fn is_word_character(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_prefix_correction_does_not_corrupt_longer_words() {
        assert_eq!(
            replace_literal("blurry blue blu", "blu", "blue", false),
            "blurry blue blue"
        );
    }

    #[test]
    fn literal_corrections_match_phrases_and_ignore_case() {
        assert_eq!(
            replace_literal("cubit TORRENT", "Cubit Torrent", "Qbittorrent", false),
            "Qbittorrent"
        );
        assert_eq!(
            replace_literal("bluee BlueE", "bluee", "blue", false),
            "blue blue"
        );
    }

    #[test]
    fn literal_matching_is_utf8_safe() {
        assert_eq!(
            replace_literal("İstanbul blu", "blu", "blue", false),
            "İstanbul blue"
        );
    }
}
