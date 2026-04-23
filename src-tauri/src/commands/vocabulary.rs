use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::domain::{Replacement, VocabEntry, VocabularyImport};
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn list_vocabulary(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<VocabEntry>> {
    state.vocabulary.list_words().map_err(OrliqError::from)
}

#[tauri::command]
pub async fn upsert_vocabulary(
    state: State<'_, Arc<AppState>>,
    entry: VocabEntry,
) -> CmdResult<VocabEntry> {
    state
        .vocabulary
        .upsert_word(entry)
        .map_err(OrliqError::from)
}

#[tauri::command]
pub async fn delete_vocabulary(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<()> {
    state.vocabulary.delete_word(&id).map_err(OrliqError::from)
}

#[tauri::command]
pub async fn list_replacements(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<Replacement>> {
    state
        .vocabulary
        .list_replacements()
        .map_err(OrliqError::from)
}

#[tauri::command]
pub async fn upsert_replacement(
    state: State<'_, Arc<AppState>>,
    r: Replacement,
) -> CmdResult<Replacement> {
    state
        .vocabulary
        .upsert_replacement(r)
        .map_err(OrliqError::from)
}

#[tauri::command]
pub async fn delete_replacement(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<()> {
    state
        .vocabulary
        .delete_replacement(&id)
        .map_err(OrliqError::from)
}

#[tauri::command]
pub async fn import_vocabulary(
    state: State<'_, Arc<AppState>>,
    payload: VocabularyImport,
) -> CmdResult<()> {
    for w in payload.words {
        let _ = state.vocabulary.upsert_word(w);
    }
    for r in payload.replacements {
        let _ = state.vocabulary.upsert_replacement(r);
    }
    Ok(())
}

#[tauri::command]
pub async fn export_vocabulary(state: State<'_, Arc<AppState>>) -> CmdResult<VocabularyImport> {
    Ok(VocabularyImport {
        words: state.vocabulary.list_words().map_err(OrliqError::from)?,
        replacements: state
            .vocabulary
            .list_replacements()
            .map_err(OrliqError::from)?,
    })
}
