use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::domain::{HistoryEntry, HistoryFilter, HistoryStatus, PostProcessing};
use crate::util::gating::{is_pro, pro_required, FREE_HISTORY_CAP};
use crate::util::result::{CmdResult, OrliqError};
use crate::util::time::now_ms;

#[tauri::command]
pub async fn list_history(
    state: State<'_, Arc<AppState>>,
    filter: HistoryFilter,
) -> CmdResult<Vec<HistoryEntry>> {
    // Free-tier cap: return at most 100 entries. The DB still holds
    // everything — we filter on read so upgrading to Pro restores the full
    // list without a restore step. List is ordered newest-first, so
    // truncating keeps the useful recent slice.
    let rows = state.history.list(filter).map_err(OrliqError::from)?;
    let status = state.licence.status().await;
    if !is_pro(&status) && rows.len() > FREE_HISTORY_CAP {
        return Ok(rows.into_iter().take(FREE_HISTORY_CAP).collect());
    }
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
    // Pro gate: re-transcribe / reprocess is Pro-only.
    let lic = state.licence.status().await;
    if !is_pro(&lic) {
        return Err(pro_required("Re-transcribe"));
    }

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
    };
    state.history.insert(&entry).map_err(OrliqError::from)?;
    Ok(entry)
}

#[tauri::command]
pub async fn export_history(
    state: State<'_, Arc<AppState>>,
    filter: HistoryFilter,
) -> CmdResult<String> {
    let rows = state.history.list(filter).map_err(OrliqError::from)?;
    let json = serde_json::to_string_pretty(&rows).map_err(|e| OrliqError::Internal {
        message: e.to_string(),
    })?;
    Ok(json)
}
