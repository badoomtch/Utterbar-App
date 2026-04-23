//! Local-only analytics rollups. Writes to `analytics_daily`; reads return shaped summaries.

use std::sync::Arc;

use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::Db;
use crate::util::time::today_utc;

pub struct AnalyticsService {
    db: Arc<Db>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct StatsSummary {
    pub words_today: u32,
    pub words_week: u32,
    pub seconds_saved_today: u32,
    pub seconds_saved_week: u32,
    pub avg_wpm: f32,
    pub dictations_today: u32,
    pub dictations_week: u32,
    pub streak_days: u32,
    // Lifetime rollups — sums over the full analytics_daily table with no date gate.
    // Useful both for the Dashboard "all-time" strip and for milestone nudges
    // ("you just crossed 100k words").
    pub words_all_time: u64,
    pub dictations_all_time: u64,
    pub seconds_saved_all_time: u64,
    pub seconds_spoken_all_time: u64,
    /// Distinct calendar days (UTC) on which the user dictated at least once.
    pub days_active_all_time: u32,
    /// First-ever dictation date (YYYY-MM-DD), or None if the user has never dictated.
    pub first_dictation_day: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct DailyPoint {
    pub day: String,
    pub value: f64,
}

impl AnalyticsService {
    pub fn new(db: Arc<Db>) -> Self { Self { db } }

    pub fn record_dictation(&self, words: u32, seconds_spoken: u32, baseline_wpm: u32, mode_id: Option<&str>) -> Result<()> {
        let day = today_utc();
        let saved = saved_seconds(words, seconds_spoken, baseline_wpm);
        // Always bump the cross-mode total (mode_id = '') and, if present, the per-mode row.
        for metric_value in [
            ("words", words as f64),
            ("seconds_spoken", seconds_spoken as f64),
            ("seconds_saved", saved as f64),
            ("dictations", 1.0),
        ] {
            self.bump(&day, metric_value.0, "", metric_value.1)?;
            if let Some(m) = mode_id {
                self.bump(&day, metric_value.0, m, metric_value.1)?;
            }
        }
        Ok(())
    }

    fn bump(&self, day: &str, metric: &str, mode_id: &str, delta: f64) -> Result<()> {
        let conn = self.db.conn()?;
        conn.execute(
            "INSERT INTO analytics_daily(day_utc, metric, mode_id, value) VALUES(?1,?2,?3,?4)
             ON CONFLICT(day_utc, metric, mode_id) DO UPDATE SET value = value + excluded.value",
            params![day, metric, mode_id, delta],
        )?;
        Ok(())
    }

    pub fn summary(&self) -> Result<StatsSummary> {
        let conn = self.db.conn()?;
        let today = today_utc();

        let words_today: f64 = conn.query_row(
            "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE day_utc=?1 AND metric='words' AND mode_id = ''",
            params![today], |r| r.get(0)).unwrap_or(0.0);
        let words_week: f64 = conn.query_row(
            "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE day_utc >= date(?1,'-6 days') AND metric='words' AND mode_id = ''",
            params![today], |r| r.get(0)).unwrap_or(0.0);
        let saved_today: f64 = conn.query_row(
            "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE day_utc=?1 AND metric='seconds_saved' AND mode_id = ''",
            params![today], |r| r.get(0)).unwrap_or(0.0);
        let saved_week: f64 = conn.query_row(
            "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE day_utc >= date(?1,'-6 days') AND metric='seconds_saved' AND mode_id = ''",
            params![today], |r| r.get(0)).unwrap_or(0.0);
        let dict_today: f64 = conn.query_row(
            "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE day_utc=?1 AND metric='dictations' AND mode_id = ''",
            params![today], |r| r.get(0)).unwrap_or(0.0);
        let dict_week: f64 = conn.query_row(
            "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE day_utc >= date(?1,'-6 days') AND metric='dictations' AND mode_id = ''",
            params![today], |r| r.get(0)).unwrap_or(0.0);
        let words_week_val = words_week;
        let spoken_week: f64 = conn.query_row(
            "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE day_utc >= date(?1,'-6 days') AND metric='seconds_spoken' AND mode_id = ''",
            params![today], |r| r.get(0)).unwrap_or(0.0);
        let avg_wpm = if spoken_week > 0.0 { (words_week_val / (spoken_week / 60.0)) as f32 } else { 0.0 };

        let streak_days = streak_from_dictations(&conn, &today).unwrap_or(0);

        // Lifetime rollups. mode_id = '' is the cross-mode total row, so summing it
        // double-counts nothing even though per-mode rows also exist.
        let words_all_time: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE metric='words' AND mode_id = ''",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0.0);
        let dict_all_time: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE metric='dictations' AND mode_id = ''",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0.0);
        let saved_all_time: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE metric='seconds_saved' AND mode_id = ''",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0.0);
        let spoken_all_time: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(value),0) FROM analytics_daily WHERE metric='seconds_spoken' AND mode_id = ''",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0.0);
        let days_active_all_time: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT day_utc) FROM analytics_daily
                 WHERE metric='dictations' AND mode_id = '' AND value > 0",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let first_dictation_day: Option<String> = conn
            .query_row(
                "SELECT day_utc FROM analytics_daily
                 WHERE metric='dictations' AND mode_id = '' AND value > 0
                 ORDER BY day_utc ASC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .ok();

        Ok(StatsSummary {
            words_today: words_today as u32,
            words_week: words_week as u32,
            seconds_saved_today: saved_today as u32,
            seconds_saved_week: saved_week as u32,
            avg_wpm,
            dictations_today: dict_today as u32,
            dictations_week: dict_week as u32,
            streak_days,
            words_all_time: words_all_time.max(0.0) as u64,
            dictations_all_time: dict_all_time.max(0.0) as u64,
            seconds_saved_all_time: saved_all_time.max(0.0) as u64,
            seconds_spoken_all_time: spoken_all_time.max(0.0) as u64,
            days_active_all_time: days_active_all_time.max(0) as u32,
            first_dictation_day,
        })
    }

    pub fn series(&self, metric: &str, days: u32) -> Result<Vec<DailyPoint>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT day_utc, SUM(value) FROM analytics_daily
             WHERE metric=?1 AND mode_id = ''
             AND day_utc >= date('now', ?2)
             GROUP BY day_utc ORDER BY day_utc ASC"
        )?;
        let offset = format!("-{} days", days.saturating_sub(1));
        let rows = stmt.query_map(params![metric, offset], |r| {
            Ok(DailyPoint { day: r.get(0)?, value: r.get(1)? })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

fn saved_seconds(words: u32, seconds_spoken: u32, baseline_wpm: u32) -> u32 {
    if baseline_wpm == 0 { return 0; }
    let typing = (words as f32 * 60.0 / baseline_wpm as f32) as i32;
    (typing - seconds_spoken as i32).max(0) as u32
}

/// Count consecutive days (ending today or yesterday) with at least one dictation.
/// Yesterday counts as part of the streak so a user who hasn't dictated *yet today*
/// doesn't see their streak reset at midnight.
fn streak_from_dictations(conn: &rusqlite::Connection, today: &str) -> rusqlite::Result<u32> {
    let mut stmt = conn.prepare(
        "SELECT day_utc FROM analytics_daily
         WHERE metric = 'dictations' AND mode_id = '' AND value > 0
         ORDER BY day_utc DESC LIMIT 400"
    )?;
    let days: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();
    if days.is_empty() {
        return Ok(0);
    }

    // Walk backwards from today: first day may be today or yesterday; each subsequent
    // day must be exactly one day before the previous to continue the streak.
    let Some(mut cursor) = parse_day(today) else {
        return Ok(0);
    };
    let first = parse_day(&days[0]).unwrap_or(cursor);
    if days_between(cursor, first) > 1 {
        return Ok(0);
    }
    cursor = first;
    let mut streak = 1u32;
    for d in days.iter().skip(1) {
        let Some(next) = parse_day(d) else { break; };
        if days_between(next, cursor) != 1 {
            break;
        }
        cursor = next;
        streak += 1;
    }
    Ok(streak)
}

fn parse_day(s: &str) -> Option<(i32, u32, u32)> {
    let mut parts = s.split('-');
    let y: i32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    Some((y, m, d))
}

fn days_between(a: (i32, u32, u32), b: (i32, u32, u32)) -> i64 {
    // `a` is expected to be earlier than `b` when a positive result is wanted.
    (ymd_to_epoch_days(b) - ymd_to_epoch_days(a)).abs()
}

/// Howard Hinnant's civil-from-days, inverted.
fn ymd_to_epoch_days(ymd: (i32, u32, u32)) -> i64 {
    let (mut y, m, d) = (ymd.0 as i64, ymd.1 as i64, ymd.2 as i64);
    if m <= 2 {
        y -= 1;
    }
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * if m > 2 { m - 3 } else { m + 9 } + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}
