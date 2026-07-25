use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::domain::{HistoryEntry, HistoryFilter, HistoryStatus, PostProcessing};
use crate::util::result::{CmdResult, OrliqError};
use crate::util::time::now_ms;

#[tauri::command]
pub async fn list_history(
    state: State<'_, Arc<AppState>>,
    filter: HistoryFilter,
) -> CmdResult<Vec<HistoryEntry>> {
    let rows = state.history.list(filter).map_err(OrliqError::from)?;
    Ok(rows)
}

#[tauri::command]
pub async fn get_history_entry(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> CmdResult<Option<HistoryEntry>> {
    state.history.get(&id).map_err(OrliqError::from)
}

#[tauri::command]
pub async fn delete_history_entry(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<()> {
    state.history.delete(&id).map_err(OrliqError::from)
}

/// Re-run post-processing on a stored raw transcript using a different mode, and insert
/// the result as a new history entry. ASR is **not** re-run — we already have the transcript.
#[tauri::command]
pub async fn reprocess_history(
    state: State<'_, Arc<AppState>>,
    id: String,
    mode_id: String,
) -> CmdResult<HistoryEntry> {
    let source = state
        .history
        .get(&id)
        .map_err(OrliqError::from)?
        .ok_or_else(|| OrliqError::User {
            message: "History entry not found".into(),
        })?;

    let mode = state
        .modes
        .get(&mode_id)
        .map_err(OrliqError::from)?
        .ok_or_else(|| OrliqError::User {
            message: "Mode not found".into(),
        })?;

    let raw = state
        .vocabulary
        .apply_replacements(&source.raw_text)
        .unwrap_or(source.raw_text.clone());

    // Rebuild the proc-context from the original entry's stored payload. We never
    // re-read the live clipboard/selection here — reprocess is a deterministic operation
    // over stored data only.
    let ctx: crate::services::llm::ProcContext = source
        .context_payload
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    let started = now_ms();
    let processed = match &mode.post_processing {
        PostProcessing::None => raw.clone(),
        PostProcessing::Deterministic { .. } => {
            crate::services::llm::deterministic::DeterministicProvider.rewrite_sync(&raw)
        }
        PostProcessing::Llm { prompt, .. } => {
            let llm = crate::services::llm::best_available().await;
            llm.rewrite(&raw, prompt, &ctx)
                .await
                .unwrap_or_else(|_| raw.clone())
        }
    };

    let entry = HistoryEntry {
        id: ulid::Ulid::new().to_string(),
        created_at: now_ms(),
        mode_id: Some(mode.id.clone()),
        mode_name_snapshot: format!("{} (reprocessed)", mode.name),
        asr_model_id: source.asr_model_id.clone(),
        asr_model_version: source.asr_model_version.clone(),
        audio_path: source.audio_path.clone(),
        audio_duration_ms: source.audio_duration_ms,
        raw_text: raw,
        processed_text: processed,
        language: source.language.clone(),
        processing_ms: now_ms() - started,
        status: HistoryStatus::Success,
        error_message: None,
        app_exe: source.app_exe.clone(),
        app_title: source.app_title.clone(),
        context_payload: source.context_payload.clone(),
        word_timings: source.word_timings.clone(),
        // Reprocess always produces a dictation row, even if the source
        // was a meeting. The point of reprocess is to re-run a different
        // mode's post-processing pipeline; meeting mode skips post-
        // processing, so a reprocessed row is by definition a dictation
        // and belongs on /history rather than /meetings.
        is_meeting: false,
        // Reprocessed rows are dictations: no `**Speaker N:**` tokens
        // for a rename map to substitute against. None.
        speaker_names: None,
    };
    // Group-policy `BlockHistory` blocks new persistence even on the
    // reprocess path — a reprocessed transcript is still a fresh row.
    // Existing rows are NOT touched. Surfaces as a user-visible error
    // so the frontend can render "Disabled by your administrator"
    // rather than silently dropping the result.
    if state.settings.policy().history_blocked() {
        return Err(OrliqError::User {
            message: "History is disabled by your administrator.".into(),
        });
    }
    state.history.insert(&entry).map_err(OrliqError::from)?;
    Ok(entry)
}

/// Distinct app executables that appear in the history. Used by the
/// History page to populate its "App" filter dropdown — far cheaper
/// than the frontend deriving the set on every reload.
#[tauri::command]
pub async fn list_history_apps(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<String>> {
    state.history.distinct_app_exes().map_err(OrliqError::from)
}

#[tauri::command]
pub async fn export_history(
    state: State<'_, Arc<AppState>>,
    filter: HistoryFilter,
) -> CmdResult<String> {
    // Group-policy gate. `history_export_blocked()` is true when
    // either `BlockHistoryExport` or the strict-superset `BlockHistory`
    // is set. The frontend disables the export button via
    // `policy_snapshot`, but a refusal here is the load-bearing
    // enforcement — a determined user can still hit the IPC directly.
    if state.settings.policy().history_export_blocked() {
        return Err(OrliqError::User {
            message: "Exporting history is disabled by your administrator.".into(),
        });
    }
    let rows = state.history.list(filter).map_err(OrliqError::from)?;
    let json = serde_json::to_string_pretty(&rows).map_err(|e| OrliqError::Internal {
        message: e.to_string(),
    })?;
    Ok(json)
}

#[cfg(test)]
mod history_policy_grep_tests {
    //! Source-grep guard. The `BlockHistory` policy is enforced at
    //! the command-callsite layer (`commands/dictation.rs` success
    //! path + `commands/history.rs::reprocess_history`) rather than
    //! inside `HistoryService::insert`. That choice keeps the
    //! service borrow graph simple — `HistoryService` doesn't need
    //! a `&Policy` plumbed through — but it puts the burden on
    //! reviewers to remember to gate every new production
    //! `state.history.insert(...)` callsite. This test makes the
    //! guard mechanical: any new production callsite must either
    //! sit near a `history_blocked()` check or carry an explicit
    //! marker comment claiming the bypass is intentional.
    //!
    //! Service-internal `HistoryService::insert(...)` calls in
    //! `services/history.rs` (and tests under it) are out of scope
    //! and not scanned. The policy applies to the dictation/repro
    //! flow that writes user transcripts, not to the SQL chokepoint
    //! itself.

    /// Prefix of every callsite shape we want to gate. Built from
    /// two literals so this test file's own source line can't match
    /// itself when included via `include_str!`.
    fn needle() -> String {
        format!("{}{}", "state.history.", "insert(")
    }

    /// How many production source lines above a `state.history.insert(`
    /// callsite count as "near" a `history_blocked()` check. Five is
    /// loose enough to cover a comment block + the `if` guard +
    /// the `} else {` arm, and tight enough that a guard left ten
    /// lines away by mistake still trips this test.
    const NEAR_LINES: usize = 8;

    /// The explicit-bypass marker. If a future callsite genuinely
    /// has to write history regardless of policy (e.g. an audit row
    /// that the IT admin themselves asked us to keep), the author
    /// puts this exact substring in a comment ON or just above the
    /// line. The marker is verbose on purpose — it should show up
    /// in code review, not slip through.
    const BYPASS_MARKER: &str = "policy-bypass: history-insert intentional";

    /// Production sources we scan. Service-internal callers
    /// (`services/history.rs`, `services/`) are intentionally not
    /// in this list; see the module-level docstring.
    const SCANNED: &[(&str, &str)] = &[
        ("commands/dictation.rs", include_str!("dictation.rs")),
        ("commands/history.rs", include_str!("history.rs")),
    ];

    /// True if any line in `window` mentions the policy gate. We
    /// match the helper name and a couple of common shapes to keep
    /// the heuristic robust against minor refactors (renaming the
    /// local binding, splitting the if/else, etc.).
    fn window_has_policy_gate(window: &[&str]) -> bool {
        window.iter().any(|l| {
            l.contains("history_blocked()")
                || l.contains("policy.history_blocked")
                || l.contains("policy().history_blocked")
        })
    }

    fn window_has_bypass_marker(window: &[&str]) -> bool {
        window.iter().any(|l| l.contains(BYPASS_MARKER))
    }

    /// Truncate at the test-mod boundary so a test's own description
    /// or string literal can't trigger a false positive. Mirrors the
    /// established convention in `commands/dictation.rs`.
    fn production_prefix(src: &str) -> &str {
        let marker = "#[cfg(test)]";
        src.split(marker).next().unwrap_or(src)
    }

    #[test]
    fn every_production_history_insert_is_policy_gated_or_marked() {
        let needle = needle();
        let mut violations: Vec<String> = Vec::new();

        for (label, src) in SCANNED {
            let prod = production_prefix(src);
            let lines: Vec<&str> = prod.lines().collect();
            for (idx, line) in lines.iter().enumerate() {
                if !line.contains(&needle) {
                    continue;
                }
                // Skip comment lines: a doc comment or `// state.history.insert(...)`
                // explanation is not a real callsite.
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") || trimmed.starts_with("*") {
                    continue;
                }
                let start = idx.saturating_sub(NEAR_LINES);
                let window = &lines[start..=idx];
                if window_has_policy_gate(window) || window_has_bypass_marker(window) {
                    continue;
                }
                violations.push(format!(
                    "{label}:{lineno} `{line}` is not within {NEAR_LINES} lines of a `history_blocked()` check or a `{BYPASS_MARKER}` marker",
                    lineno = idx + 1,
                    line = line.trim(),
                ));
            }
        }

        assert!(
            violations.is_empty(),
            "ungated production `state.history.insert(...)` callsites:\n  {}",
            violations.join("\n  "),
        );
    }

    /// Sanity check: the guard should actually be finding the two
    /// production callsites we know exist. If this hits zero either
    /// the regex / `include_str!` plumbing broke or someone removed
    /// the production callers — both worth a louder failure than a
    /// quiet "0 violations" pass.
    #[test]
    fn scanner_finds_the_known_production_callsites() {
        let needle = needle();
        let mut hits = 0usize;
        for (_, src) in SCANNED {
            let prod = production_prefix(src);
            for line in prod.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") || trimmed.starts_with("*") {
                    continue;
                }
                if line.contains(&needle) {
                    hits += 1;
                }
            }
        }
        // Two known callers as of this commit:
        //   * commands/dictation.rs success path
        //   * commands/history.rs::reprocess_history
        // If a third lands legitimately, bump this number in the
        // same change; the assert is a tripwire for "the scanner
        // stopped seeing anything", not a hard cap.
        assert!(
            hits >= 2,
            "expected at least 2 production `state.history.insert(` callsites, found {hits}"
        );
    }
}
