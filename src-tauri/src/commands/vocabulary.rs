use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::domain::{Replacement, VocabEntry, VocabularyImport};
use crate::util::gating::{
    cap_reached, is_pro, FREE_REPLACEMENTS_CAP, FREE_VOCAB_WORDS_CAP,
};
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
    // Free-tier cap: 50 words. Updates to existing rows (same `word`) pass
    // through to preserve grandfathering when a Pro user deactivates and
    // their list exceeds the cap — only NEW rows are rejected.
    let status = state.licence.status().await;
    if !is_pro(&status) {
        let current = state.vocabulary.list_words().map_err(OrliqError::from)?;
        let is_existing = current
            .iter()
            .any(|w| w.word.eq_ignore_ascii_case(&entry.word));
        if !is_existing && current.len() >= FREE_VOCAB_WORDS_CAP {
            return Err(cap_reached("Vocabulary", FREE_VOCAB_WORDS_CAP));
        }
    }
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
    // Free-tier cap: 25 replacements. Updates (non-empty `id` that already
    // exists) are allowed so grandfathered Pro rows stay editable.
    let status = state.licence.status().await;
    if !is_pro(&status) {
        let current = state
            .vocabulary
            .list_replacements()
            .map_err(OrliqError::from)?;
        let is_existing = !r.id.is_empty() && current.iter().any(|x| x.id == r.id);
        if !is_existing && current.len() >= FREE_REPLACEMENTS_CAP {
            return Err(cap_reached("Replacements", FREE_REPLACEMENTS_CAP));
        }
    }
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
    // Free-tier caps apply to bulk import too, else import is a trivial
    // cap bypass. We count only rows that would be new inserts (case-
    // insensitive word match / id match) so re-importing the same file
    // is idempotent.
    let status = state.licence.status().await;
    if !is_pro(&status) {
        let current_words = state.vocabulary.list_words().map_err(OrliqError::from)?;
        let new_words = payload
            .words
            .iter()
            .filter(|w| {
                !current_words
                    .iter()
                    .any(|c| c.word.eq_ignore_ascii_case(&w.word))
            })
            .count();
        if current_words.len() + new_words > FREE_VOCAB_WORDS_CAP {
            return Err(cap_reached("Vocabulary", FREE_VOCAB_WORDS_CAP));
        }

        let current_reps = state
            .vocabulary
            .list_replacements()
            .map_err(OrliqError::from)?;
        let new_reps = payload
            .replacements
            .iter()
            .filter(|r| r.id.is_empty() || !current_reps.iter().any(|c| c.id == r.id))
            .count();
        if current_reps.len() + new_reps > FREE_REPLACEMENTS_CAP {
            return Err(cap_reached("Replacements", FREE_REPLACEMENTS_CAP));
        }
    }

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
