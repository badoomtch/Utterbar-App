//! Pre-refinery DB repair for the v0.15.1 V6 line-ending checksum
//! drift.
//!
//! ## What this fixes
//!
//! Refinery hashes each migration's `(name, version, sql)` triple
//! with `SipHasher13` at compile time and stores the resulting `u64`
//! (Display-formatted as a decimal string) in
//! `refinery_schema_history.checksum`. On every subsequent boot,
//! refinery recomputes the checksum from the bundled SQL and
//! compares; a mismatch raises `ApplyMigrationFailure` and the
//! caller's `expect("bootstrap AppState")` panics.
//!
//! v0.15.0 was built on a Windows machine where
//! `V0006__history_speaker_names.sql` had its LF git-stored bytes
//! intact in the working tree. The v0.15.0 binary computed the LF
//! checksum and recorded it on first launch. Between the v0.15.0
//! and v0.15.1 builds, `core.autocrlf=true` rewrote the working
//! tree to CRLF; the v0.15.1 binary computed the CRLF checksum and
//! refused to start on every machine that had run v0.15.0 first.
//!
//! v0.15.2 corrects the source file back to LF and adds
//! `.gitattributes` to lock it in, so v0.15.2's compile-time
//! checksum matches v0.15.0's. That fixes Jay's machine and any
//! other DB that recorded LF.
//!
//! It does NOT help DBs that had v0.15.1 as their first 0.15.x
//! launch — those rows have the CRLF checksum recorded. Without
//! repair, the symmetric failure: LF v0.15.2 binary, CRLF
//! recorded checksum, refinery panics.
//!
//! ## What this module does
//!
//! Runs BEFORE `embedded::migrations::runner().run(conn)`. Reads
//! the row in `refinery_schema_history` for `version = 6, name =
//! 'history_speaker_names'`. Three known checksum classes:
//!
//!   * **LF checksum** (`expected_v6_checksum_lf()`) — the value
//!     v0.15.0/v0.15.2 binaries compute. Already correct, no-op.
//!   * **CRLF checksum** (`expected_v6_checksum_crlf()`) — the
//!     value v0.15.1 binaries computed. ONLY in this case AND
//!     ONLY if the `history.speaker_names` column is actually
//!     present (proof the migration's SQL ran successfully on
//!     this DB), rewrite the row's checksum to the LF value.
//!     Refinery's subsequent verification then matches and
//!     migrations proceed.
//!   * **Anything else** — leave alone. Refinery's normal
//!     divergence error path runs and the user gets a clean
//!     panic instead of silent data corruption.
//!
//! ## Why this is safe
//!
//! The repair only fires when the SQL effect (the `speaker_names`
//! column) is observably present on the DB. We're not "trusting"
//! the recorded checksum — we're inferring from the schema state
//! that V6's effect did happen, then aligning the recorded hash
//! with what the current binary expects. If the column isn't
//! there, the recorded CRLF checksum is unexplained and we
//! refuse to touch it; refinery's normal failure path runs.
//!
//! No `set_abort_divergent(false)`. No blanket "rewrite any
//! divergent migration" logic. Single-purpose, single-row,
//! checksum-narrow.

use std::hash::{Hash, Hasher};

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use siphasher::sip::SipHasher13;

/// The migration's `name` field as refinery sees it (the part after
/// `__` and before `.sql`).
const V6_NAME: &str = "history_speaker_names";
/// The migration's `version` field as refinery sees it.
const V6_VERSION: i32 = 6;

/// LF SQL bytes of `V0006__history_speaker_names.sql`. Embedded at
/// compile time from the SAME path refinery's `embed_migrations!`
/// uses, so we are guaranteed to be hashing the bytes the binary
/// is shipping. With `.gitattributes` pinning `*.sql text eol=lf`,
/// these bytes are LF on every platform.
const V6_SQL_LF: &[u8] = include_bytes!("migrations/V0006__history_speaker_names.sql");

/// Outcome of `repair_known_v6_history_speaker_names_line_ending_checksum`.
/// Surfaced from the function so the caller can log / metric what
/// path was taken; the production caller treats every variant as
/// success (it's refinery's job to fail noisily on the next step
/// if the repair didn't apply and the checksum is still wrong).
#[derive(Debug, PartialEq, Eq)]
pub enum RepairOutcome {
    /// `refinery_schema_history` table doesn't exist yet (fresh DB
    /// with no migrations ever applied). Nothing to repair.
    NoRepairTableMissing,
    /// Table exists but no V6 row. v0.15.x has never run on this
    /// DB. Refinery's normal apply path will write the LF checksum
    /// when it runs V6.
    NoRepairNoRow,
    /// V6 row's checksum already matches what this binary expects
    /// (the LF value). Most v0.15.0 / v0.15.2 users land here.
    NoRepairAlreadyLf,
    /// V6 row's checksum is the v0.15.1 CRLF value AND the
    /// `history.speaker_names` column is present on disk. Safe to
    /// rewrite the recorded checksum to LF — the SQL effect is
    /// already applied.
    RepairedCrlfToLf,
    /// V6 row's checksum is the v0.15.1 CRLF value but
    /// `history.speaker_names` is missing. We refuse to touch the
    /// row — there's no evidence the migration's effect is on
    /// disk, and silently rewriting would mask a real corruption.
    /// Refinery's normal divergence path runs.
    NoRepairCrlfButColumnMissing,
    /// V6 row's checksum is something other than LF or CRLF.
    /// Either a partial / interrupted apply, or someone hand-
    /// edited the table, or a third-party tool. Refuse to touch
    /// it; refinery's normal divergence path runs.
    NoRepairUnknownChecksum,
}

/// Compute refinery's checksum for V6 against the supplied SQL
/// bytes. Mirrors `refinery_core::Migration::unapplied`'s SipHasher13
/// usage exactly:
///
///   name (as `&str`) → version (as `i32`) → sql (as `&str`),
///   each fed through Rust's std `Hash` impl into a `SipHasher13`
///   with default key, then `finish()` for the `u64`.
///
/// The result is then `Display`-formatted as decimal — that's how
/// refinery stores it in `refinery_schema_history.checksum`.
fn checksum_for(sql_bytes: &[u8]) -> u64 {
    // Refinery's `embed_migrations!` reads the file via
    // `include_str!`, which requires UTF-8. SQL content is ASCII
    // here, so the `from_utf8` is total in practice; treat any
    // theoretical failure as a programming bug (a non-UTF-8
    // migration would have failed at compile time).
    let sql = std::str::from_utf8(sql_bytes).expect("V0006 migration SQL must be valid UTF-8");
    let mut hasher = SipHasher13::new();
    V6_NAME.hash(&mut hasher);
    V6_VERSION.hash(&mut hasher);
    sql.hash(&mut hasher);
    hasher.finish()
}

/// LF checksum (decimal `Display` of the `u64`). The value the
/// v0.15.2 binary computes from its bundled V6 — i.e. what should
/// be in `refinery_schema_history.checksum` for every working DB.
pub fn expected_v6_checksum_lf() -> String {
    checksum_for(V6_SQL_LF).to_string()
}

/// CRLF checksum (decimal `Display` of the `u64`). The value the
/// v0.15.1 binary computed from its CRLF-rewritten V6 — what's
/// recorded in DBs that first applied V6 under v0.15.1.
pub fn expected_v6_checksum_crlf() -> String {
    let mut crlf = Vec::with_capacity(V6_SQL_LF.len() + 32);
    for &b in V6_SQL_LF {
        if b == b'\n' {
            crlf.push(b'\r');
        }
        crlf.push(b);
    }
    checksum_for(&crlf).to_string()
}

/// Pre-migration-runner repair. See module-level doc-comment for
/// the contract; this function executes it.
pub fn repair_known_v6_history_speaker_names_line_ending_checksum(
    conn: &Connection,
) -> Result<RepairOutcome> {
    // 1. Does refinery's history table even exist? If not, this is
    //    a fresh DB and refinery will create it.
    let table_exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='refinery_schema_history')",
        [],
        |row| row.get::<_, i64>(0).map(|n| n != 0),
    )?;
    if !table_exists {
        return Ok(RepairOutcome::NoRepairTableMissing);
    }

    // 2. Read the V6 row.
    let recorded: Option<String> = conn
        .query_row(
            "SELECT checksum FROM refinery_schema_history \
             WHERE version = ?1 AND name = ?2",
            params![V6_VERSION, V6_NAME],
            |row| row.get(0),
        )
        .optional()?;
    let Some(recorded) = recorded else {
        return Ok(RepairOutcome::NoRepairNoRow);
    };

    let lf = expected_v6_checksum_lf();
    let crlf = expected_v6_checksum_crlf();

    // 3. Already the LF value: most users, no work needed.
    if recorded == lf {
        return Ok(RepairOutcome::NoRepairAlreadyLf);
    }

    // 4. Anything other than the v0.15.1 CRLF value is unknown.
    //    Refuse to touch.
    if recorded != crlf {
        tracing::warn!(
            recorded = %recorded,
            expected_lf = %lf,
            expected_crlf = %crlf,
            "v0.15.2 repair: V6 checksum is neither LF nor CRLF — refusing to touch (refinery will surface)"
        );
        return Ok(RepairOutcome::NoRepairUnknownChecksum);
    }

    // 5. Recorded is the CRLF value. ONLY repair if the SQL's
    //    effect (the `speaker_names` column on `history`) is
    //    actually present. Without that proof we're not sure the
    //    migration ever ran successfully against this DB.
    let column_present = history_has_speaker_names_column(conn)?;
    if !column_present {
        tracing::warn!(
            "v0.15.2 repair: V6 row has CRLF checksum but `history.speaker_names` is missing — refusing to repair (refinery will surface)"
        );
        return Ok(RepairOutcome::NoRepairCrlfButColumnMissing);
    }

    // 6. Repair: rewrite ONLY this row's checksum to LF. Single
    //    `UPDATE` with both `version` and `name` in the WHERE
    //    clause so we can't accidentally bridge to a different
    //    row, and we assert exactly one row was affected.
    let affected = conn.execute(
        "UPDATE refinery_schema_history SET checksum = ?1 \
         WHERE version = ?2 AND name = ?3 AND checksum = ?4",
        params![lf.as_str(), V6_VERSION, V6_NAME, crlf.as_str()],
    )?;
    if affected != 1 {
        // Should be impossible — we just SELECTed the same row —
        // but if somehow another process raced us, surface
        // loudly. Not a panic; refinery's verification will
        // tell us what's actually there.
        tracing::error!(
            affected,
            "v0.15.2 repair: expected 1 affected row but got {affected}; refinery will verify"
        );
    } else {
        tracing::warn!(
            from = %crlf,
            to = %lf,
            "v0.15.2 repair: rewrote V6 history_speaker_names checksum from v0.15.1 CRLF value to LF value"
        );
    }
    Ok(RepairOutcome::RepairedCrlfToLf)
}

/// True iff the `history` table has a `speaker_names` column.
/// Reads `pragma_table_info` rather than catching errors from a
/// `SELECT speaker_names FROM history LIMIT 0` because the latter
/// would consume the user's expected error path on a corrupt DB.
fn history_has_speaker_names_column(conn: &Connection) -> Result<bool> {
    let history_exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='history')",
        [],
        |row| row.get::<_, i64>(0).map(|n| n != 0),
    )?;
    if !history_exists {
        return Ok(false);
    }
    let mut stmt = conn.prepare("PRAGMA table_info(history)")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let col_name: String = row.get(1)?;
        if col_name == "speaker_names" {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    //! End-to-end coverage of the V6 checksum repair against an
    //! in-memory SQLite. Each test sets up its own DB, runs the
    //! repair, asserts the outcome, and (where applicable) runs
    //! refinery's migrations to confirm the post-repair DB
    //! actually boots.

    use super::*;
    use rusqlite::Connection;

    /// Bring up a fresh in-memory SQLite with the
    /// `refinery_schema_history` schema refinery would create on
    /// first apply, plus zero migrations. Tests that need to
    /// simulate "v0.15.x has run before on this DB" insert their
    /// own V0001-V0006 rows below.
    fn db_with_refinery_table() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE refinery_schema_history( \
                 version INT4 PRIMARY KEY, \
                 name VARCHAR(255), \
                 applied_on VARCHAR(255), \
                 checksum VARCHAR(255))",
            [],
        )
        .unwrap();
        conn
    }

    /// Insert a V6 row with a chosen checksum. Doesn't insert
    /// V0001-V0005; tests that exercise the full migration runner
    /// need to be careful about ordering.
    fn insert_v6_row(conn: &Connection, checksum: &str) {
        conn.execute(
            "INSERT INTO refinery_schema_history(version, name, applied_on, checksum) \
             VALUES (?1, ?2, ?3, ?4)",
            params![V6_VERSION, V6_NAME, "2026-04-28T22:00:00Z", checksum],
        )
        .unwrap();
    }

    /// Create a `history` table that has the `speaker_names`
    /// column (i.e. the post-V6 schema). Used to simulate a DB
    /// where V6's effect is on disk.
    fn create_history_with_speaker_names(conn: &Connection) {
        conn.execute(
            "CREATE TABLE history(id INTEGER PRIMARY KEY, speaker_names TEXT NULL)",
            [],
        )
        .unwrap();
    }

    /// Create a `history` table WITHOUT `speaker_names` (pre-V6
    /// schema). Used to simulate "V6 row is in the table but
    /// migration's SQL never actually ran".
    fn create_history_without_speaker_names(conn: &Connection) {
        conn.execute("CREATE TABLE history(id INTEGER PRIMARY KEY)", [])
            .unwrap();
    }

    /// Diagnostic: print the actual LF + CRLF u64 values. Run via
    /// `cargo test db::repair::tests::print_v6_checksums -- --ignored --nocapture`
    /// to capture them for the v0.15.2 review packet. Marked
    /// `#[ignore]` so the normal test run isn't littered with
    /// stdout noise.
    #[test]
    #[ignore]
    fn print_v6_checksums() {
        println!("LF   checksum (decimal u64): {}", expected_v6_checksum_lf());
        println!(
            "CRLF checksum (decimal u64): {}",
            expected_v6_checksum_crlf()
        );
    }

    /// Pin: the LF and CRLF checksums are different. If they
    /// somehow weren't, the entire repair is meaningless; this
    /// test catches that compile-time impossibility surviving
    /// some future Hash trait change.
    #[test]
    fn lf_and_crlf_checksums_are_distinct() {
        let lf = expected_v6_checksum_lf();
        let crlf = expected_v6_checksum_crlf();
        assert_ne!(
            lf, crlf,
            "LF and CRLF checksums must differ (lf={lf}, crlf={crlf})"
        );
    }

    /// Both checksums are non-empty decimal strings. Pin the
    /// shape so a refinery upgrade that changes the storage
    /// format is caught here.
    #[test]
    fn checksums_are_decimal_u64_strings() {
        for s in [expected_v6_checksum_lf(), expected_v6_checksum_crlf()] {
            assert!(!s.is_empty());
            assert!(s.chars().all(|c| c.is_ascii_digit()));
            // Round-trips through u64 — exactly what refinery's
            // INSERT format depends on.
            let _: u64 = s.parse().expect("checksum must parse as u64");
        }
    }

    /// **Required test 5.** Missing V6 row → no-op. Fresh DB
    /// post-bootstrap, refinery_schema_history exists but is
    /// empty.
    #[test]
    fn missing_v6_row_is_noop() {
        let conn = db_with_refinery_table();
        let outcome = repair_known_v6_history_speaker_names_line_ending_checksum(&conn).unwrap();
        assert_eq!(outcome, RepairOutcome::NoRepairNoRow);
    }

    /// Even-fresher case: refinery's table itself doesn't exist
    /// (genuinely first-ever launch). No-op.
    #[test]
    fn missing_refinery_table_is_noop() {
        let conn = Connection::open_in_memory().unwrap();
        let outcome = repair_known_v6_history_speaker_names_line_ending_checksum(&conn).unwrap();
        assert_eq!(outcome, RepairOutcome::NoRepairTableMissing);
    }

    /// **Required test 1.** LF checksum row is unchanged.
    #[test]
    fn lf_checksum_row_is_unchanged() {
        let conn = db_with_refinery_table();
        let lf = expected_v6_checksum_lf();
        insert_v6_row(&conn, &lf);
        create_history_with_speaker_names(&conn);
        let outcome = repair_known_v6_history_speaker_names_line_ending_checksum(&conn).unwrap();
        assert_eq!(outcome, RepairOutcome::NoRepairAlreadyLf);
        let after: String = conn
            .query_row(
                "SELECT checksum FROM refinery_schema_history WHERE version = 6",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(after, lf, "LF row must be left exactly as-is");
    }

    /// **Required test 2.** CRLF checksum row + `history.speaker_names`
    /// column → repair to LF.
    #[test]
    fn crlf_checksum_row_plus_column_is_repaired_to_lf() {
        let conn = db_with_refinery_table();
        let crlf = expected_v6_checksum_crlf();
        let lf = expected_v6_checksum_lf();
        insert_v6_row(&conn, &crlf);
        create_history_with_speaker_names(&conn);
        let outcome = repair_known_v6_history_speaker_names_line_ending_checksum(&conn).unwrap();
        assert_eq!(outcome, RepairOutcome::RepairedCrlfToLf);
        let after: String = conn
            .query_row(
                "SELECT checksum FROM refinery_schema_history WHERE version = 6",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(after, lf, "CRLF row must be rewritten to LF value");
    }

    /// **Required test 3.** CRLF checksum row but
    /// `history.speaker_names` is missing → refuse to repair.
    #[test]
    fn crlf_checksum_row_without_column_is_not_repaired() {
        let conn = db_with_refinery_table();
        let crlf = expected_v6_checksum_crlf();
        insert_v6_row(&conn, &crlf);
        create_history_without_speaker_names(&conn);
        let outcome = repair_known_v6_history_speaker_names_line_ending_checksum(&conn).unwrap();
        assert_eq!(outcome, RepairOutcome::NoRepairCrlfButColumnMissing);
        let after: String = conn
            .query_row(
                "SELECT checksum FROM refinery_schema_history WHERE version = 6",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            after, crlf,
            "CRLF row must be left untouched when column missing"
        );
    }

    /// Edge case of test 3: the `history` table itself doesn't
    /// exist (corruption / partial wipe). Treated the same as
    /// "column missing" — no repair.
    #[test]
    fn crlf_checksum_row_without_history_table_is_not_repaired() {
        let conn = db_with_refinery_table();
        let crlf = expected_v6_checksum_crlf();
        insert_v6_row(&conn, &crlf);
        // Don't create history at all.
        let outcome = repair_known_v6_history_speaker_names_line_ending_checksum(&conn).unwrap();
        assert_eq!(outcome, RepairOutcome::NoRepairCrlfButColumnMissing);
    }

    /// **Required test 4.** Unknown checksum → refuse to touch.
    /// "Unknown" = not LF, not CRLF. Use a plausible-looking but
    /// fabricated value to make sure the helper's "is this
    /// recognised?" check is on actual value-equality, not on
    /// shape.
    #[test]
    fn unknown_checksum_row_is_not_repaired() {
        let conn = db_with_refinery_table();
        let bogus = "1234567890123456789"; // valid u64, not LF, not CRLF
        insert_v6_row(&conn, bogus);
        create_history_with_speaker_names(&conn);
        let outcome = repair_known_v6_history_speaker_names_line_ending_checksum(&conn).unwrap();
        assert_eq!(outcome, RepairOutcome::NoRepairUnknownChecksum);
        let after: String = conn
            .query_row(
                "SELECT checksum FROM refinery_schema_history WHERE version = 6",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(after, bogus, "unknown checksum must be left untouched");
    }

    /// **Required test 6.** End-to-end: a synthetic CRLF-applied
    /// V6 DB (with all earlier migrations also recorded against
    /// the bytes the v0.15.2 binary will compute) launches
    /// successfully under the v0.15.2 migrations runner after
    /// repair.
    ///
    /// We can't easily synthesize a "v0.15.1-applied DB" with all
    /// 6 migrations because we'd need every migration's CRLF
    /// checksum, but only V6 was CRLF-contaminated in v0.15.1
    /// (V1-V5 had been LF in the working tree since v0.10.0+).
    /// So the realistic shape: V1-V5 with LF checksums, V6 with
    /// CRLF checksum + the `speaker_names` column. The repair
    /// rewrites V6 to LF; the runner then sees its embedded
    /// migrations as already-applied and exits cleanly.
    #[test]
    fn end_to_end_repair_then_refinery_run_succeeds() {
        let mut conn = Connection::open_in_memory().unwrap();
        // Apply all 6 migrations the way refinery would, so the
        // schema state matches a v0.15.1-launched DB.
        crate::db::embedded::migrations::runner()
            .run(&mut conn)
            .unwrap();
        // At this point V1-V6 all have the LF checksum (the
        // embedded migrations are LF in the bundle). Rewrite V6
        // to the CRLF value to simulate "v0.15.1 was the binary
        // that first applied V6".
        let crlf = expected_v6_checksum_crlf();
        let updated = conn
            .execute(
                "UPDATE refinery_schema_history SET checksum = ?1 \
                 WHERE version = 6 AND name = 'history_speaker_names'",
                params![crlf.as_str()],
            )
            .unwrap();
        assert_eq!(updated, 1, "test setup: expected to rewrite V6 row");

        // Now run the v0.15.2 sequence: repair, then refinery.
        let outcome = repair_known_v6_history_speaker_names_line_ending_checksum(&conn).unwrap();
        assert_eq!(outcome, RepairOutcome::RepairedCrlfToLf);
        // Refinery should be a no-op now (everything already
        // applied + checksums match).
        crate::db::embedded::migrations::runner()
            .run(&mut conn)
            .expect("post-repair refinery run must succeed");
    }
}
