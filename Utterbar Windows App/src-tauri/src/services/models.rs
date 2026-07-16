//! Model registry. Tracks the catalogue (what's available) plus installed state on disk.
//!
//! Download flow:
//!   1. Stream the tarball to `<models_root>/<id>.tar.bz2.partial`.
//!   2. Verify sha256 against the catalogue (skipped when the catalogue sha is empty).
//!   3. Extract through a bzip2 → tar pipeline into `<models_root>/<id>/`.
//!   4. Register the model row and emit `model:changed`.
//!
//! Crashes or cancels mid-flight leave the `.partial` file behind, which is cleaned up
//! on the next install attempt.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use bzip2::read::BzDecoder;
use rusqlite::params;
use sha2::{Digest, Sha256};
use tar::Archive;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

use crate::db::Db;
use crate::domain::{
    ModelBackend, ModelInfo, ModelKind, ModelProgress, ModelProgressPhase, EVT_MODEL_CHANGED,
    EVT_MODEL_PROGRESS,
};
use crate::util::time::now_ms;

pub struct ModelRegistry {
    db: Arc<Db>,
    models_dir: PathBuf,
    catalogue: RwLock<Vec<ModelInfo>>,
}

impl ModelRegistry {
    pub fn new(db: Arc<Db>, data_dir: PathBuf) -> Self {
        let models_dir = data_dir.join("models");
        let _ = std::fs::create_dir_all(&models_dir);
        Self {
            db,
            models_dir,
            catalogue: RwLock::new(built_in_catalogue()),
        }
    }

    pub fn models_root(&self) -> &PathBuf {
        &self.models_dir
    }

    pub async fn list_available(&self) -> Vec<ModelInfo> {
        // Catalogue is the source of truth for display strings (description, url, recommended).
        // Installed rows in the DB only contribute the on-disk path + installed flag.
        let mut out = self.catalogue.read().await.clone();
        let installed = self.list_installed().unwrap_or_default();
        for m in out.iter_mut() {
            if let Some(i) = installed.iter().find(|i| i.id == m.id) {
                m.installed = true;
                m.path = i.path.clone();
            }
        }
        out
    }

    /// Cheap "is this model on disk?" check. One DB read against the
    /// `models` table; the path is verified to exist on disk so a stale
    /// row pointing at a manually-deleted directory returns false.
    /// Used by `dictation.rs` to decide whether Meeting Mode can
    /// auto-route to Whisper Large without falling back to the global
    /// default.
    pub fn is_installed(&self, id: &str) -> bool {
        let conn = match self.db.conn() {
            Ok(c) => c,
            Err(_) => return false,
        };
        let path: Option<String> = conn
            .query_row(
                "SELECT path FROM models WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .ok();
        match path {
            Some(rel) => self.models_dir.join(rel).exists(),
            None => false,
        }
    }

    pub fn list_installed(&self) -> Result<Vec<ModelInfo>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id,kind,display_name,version,size_bytes,path,backend,languages FROM models",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let kind: String = row.get(1)?;
                let backend: String = row.get(6)?;
                let langs: String = row.get(7)?;
                Ok(ModelInfo {
                    id: row.get(0)?,
                    kind: match kind.as_str() {
                        "llm" => ModelKind::Llm,
                        "punct" => ModelKind::Punct,
                        _ => ModelKind::Asr,
                    },
                    display_name: row.get(2)?,
                    description: String::new(),
                    version: row.get(3)?,
                    size_bytes: row.get::<_, i64>(4)? as u64,
                    backend: match backend.as_str() {
                        "onnx" => ModelBackend::Onnx,
                        "py-sidecar" => ModelBackend::PySidecar,
                        _ => ModelBackend::LlamaCpp,
                    },
                    // Engine isn't persisted in the DB — derived at
                    // merge time from the catalogue. Default keeps
                    // installed-only paths working before the merge.
                    engine: crate::domain::ModelEngine::default(),
                    languages: serde_json::from_str(&langs).unwrap_or_default(),
                    installed: true,
                    path: row.get(5)?,
                    sha256: String::new(),
                    download_url: None,
                    recommended: false,
                    // DB row on its own can't supply comparison scores — those
                    // live only in the catalogue. `list_available` merges in the
                    // real values; `list_installed` callers that need them
                    // should go through `list_available` + filter, not here.
                    accuracy_score: 0,
                    speed_score: 0,
                    requires_pro: false,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Download + verify + extract + register a model. Emits `model:progress` throughout.
    pub async fn install(&self, handle: &AppHandle, id: &str) -> Result<ModelInfo> {
        // Group-policy gate. An IT admin can pin
        // `BlockModelDownloads=1` under HKLM\SOFTWARE\Policies\Utterbar
        // for environments where models must come from an internal
        // mirror, not the public release URLs. The catalogue UI shows
        // the override so the user understands why the button refused.
        if crate::services::policy::Policy::load().model_downloads_blocked() {
            return Err(anyhow!(
                "model downloads are disabled by your IT administrator (group policy: BlockModelDownloads)"
            ));
        }

        let catalogue = self.catalogue.read().await.clone();
        let entry = catalogue
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| anyhow!("unknown model id: {id}"))?
            .clone();
        let url = entry
            .download_url
            .clone()
            .ok_or_else(|| anyhow!("model has no download_url: {id}"))?;

        let partial = self
            .models_dir
            .join(format!("{}.tar.bz2.partial", entry.id));
        let extracted_dir = self.models_dir.join(&entry.id);

        // Start fresh — remove any leftovers from a previous attempt.
        let _ = tokio::fs::remove_file(&partial).await;
        let _ = tokio::fs::remove_dir_all(&extracted_dir).await;
        tokio::fs::create_dir_all(&self.models_dir).await?;

        emit(
            handle,
            &entry.id,
            0,
            entry.size_bytes,
            ModelProgressPhase::Starting,
        );

        // ---- Download ----
        let client = reqwest::Client::builder()
            .user_agent(format!("Utterbar/{}", env!("CARGO_PKG_VERSION")))
            .build()?;
        let mut resp = client.get(&url).send().await?.error_for_status()?;
        let total = resp.content_length().unwrap_or(entry.size_bytes);

        let mut file = tokio::fs::File::create(&partial).await?;
        let mut hasher = Sha256::new();
        let mut done: u64 = 0;
        let mut next_emit = 0u64;

        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
            hasher.update(&chunk);
            done += chunk.len() as u64;
            if done >= next_emit {
                emit(
                    handle,
                    &entry.id,
                    done,
                    total,
                    ModelProgressPhase::Downloading,
                );
                next_emit = done + (total / 100).max(512 * 1024);
            }
        }
        file.flush().await?;
        drop(file);

        // ---- Verify ----
        emit(
            handle,
            &entry.id,
            done,
            total,
            ModelProgressPhase::Verifying,
        );
        let digest = hex::encode(hasher.finalize());
        // Fail-closed integrity check (v0.9.0 hardening).
        //
        // Earlier code skipped verification when the catalogue SHA was
        // empty — that meant a forgotten pin (or any new entry added
        // without one) silently accepted whatever bytes the URL
        // served. The catalogue is now the single trust root for
        // model identity; an entry with no pin is a misconfiguration,
        // not "trust everything". We refuse to install in that case
        // and surface a clear operator-side error.
        if entry.sha256.is_empty() {
            let _ = tokio::fs::remove_file(&partial).await;
            emit(handle, &entry.id, done, total, ModelProgressPhase::Error);
            return Err(anyhow!(
                "model {} has no pinned sha256 in the catalogue — refusing to install",
                entry.id
            ));
        }
        if digest != entry.sha256 {
            let _ = tokio::fs::remove_file(&partial).await;
            emit(handle, &entry.id, done, total, ModelProgressPhase::Error);
            return Err(anyhow!(
                "sha256 mismatch: expected {}, got {}",
                entry.sha256,
                digest
            ));
        }

        // ---- Extract ----
        emit(
            handle,
            &entry.id,
            done,
            total,
            ModelProgressPhase::Extracting,
        );
        let partial_for_blocking = partial.clone();
        let extracted_for_blocking = extracted_dir.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            extract_tar_bz2(&partial_for_blocking, &extracted_for_blocking)
        })
        .await
        .map_err(|e| anyhow!("extract join: {e}"))??;

        let _ = tokio::fs::remove_file(&partial).await;

        // The sherpa-onnx tarballs usually contain a single top-level directory; if our
        // target dir ended up with exactly one subdir, flatten it so downstream can find
        // `encoder.onnx` / `decoder.onnx` / `joiner.onnx` / `tokens.txt` directly.
        flatten_single_subdir(&extracted_dir)?;

        // ---- Register ----
        let path_rel = entry.id.clone();
        {
            let conn = self.db.conn()?;
            conn.execute(
                "INSERT OR REPLACE INTO models(id,kind,display_name,version,size_bytes,path,backend,languages,installed_at,last_used_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,NULL)",
                params![
                    entry.id,
                    match entry.kind { ModelKind::Asr => "asr", ModelKind::Llm => "llm", ModelKind::Punct => "punct" },
                    entry.display_name,
                    entry.version,
                    done as i64,
                    &path_rel,
                    match entry.backend { ModelBackend::Onnx => "onnx", ModelBackend::PySidecar => "py-sidecar", ModelBackend::LlamaCpp => "llama-cpp" },
                    serde_json::to_string(&entry.languages)?,
                    now_ms(),
                ],
            )?;
        }

        emit(handle, &entry.id, total, total, ModelProgressPhase::Done);
        let _ = handle.emit(EVT_MODEL_CHANGED, &entry.id);

        let mut info = entry.clone();
        info.installed = true;
        info.path = Some(path_rel);
        Ok(info)
    }

    /// Remove a model from disk and from the catalogue table.
    ///
    /// `m.path` is a relative directory under `self.models_dir` —
    /// historically (catalogue-only-installs) every install path
    /// was a sibling directory of `models_dir`, but the column is
    /// a plain `TEXT` field so a corrupted DB or future migration
    /// could plausibly stuff an absolute path or `..` traversal in
    /// there. `Path::join` has surprising rules (absolute path
    /// discards the base; `..` is preserved until you hit the
    /// filesystem) and the next line was a `remove_dir_all` — so a
    /// bad row would have happily nuked anything outside the models
    /// directory. The fix has three guards:
    ///
    ///   1. Reject absolute paths and explicit `..` components in
    ///      the DB-supplied component before any join.
    ///   2. Canonicalise both `models_dir` and the joined target,
    ///      then assert the target is contained under `models_dir`.
    ///   3. Only after both checks pass do we call `remove_dir_all`.
    ///
    /// On any guard failure we log loudly and return an error so
    /// the user sees a real failure instead of an empty success
    /// (which they got pre-fix on every successful row regardless).
    pub fn remove(&self, id: &str) -> Result<()> {
        let installed = self.list_installed()?;
        if let Some(m) = installed.iter().find(|m| m.id == id) {
            if let Some(p) = &m.path {
                let suspect = std::path::Path::new(p);
                // Refuse absolute paths and `..` components up
                // front — these are never valid model directory
                // components and signalling them clearly beats
                // letting the canonicalise step trip on whatever
                // they resolve to.
                if suspect.is_absolute()
                    || suspect
                        .components()
                        .any(|c| matches!(c, std::path::Component::ParentDir))
                {
                    tracing::warn!(
                        model_id = id,
                        path = %suspect.display(),
                        "model.remove refused: stored path is absolute or contains traversal"
                    );
                    return Err(anyhow::anyhow!(
                        "stored model path is not relative: {}",
                        suspect.display()
                    ));
                }
                let full = self.models_dir.join(suspect);
                if full.exists() {
                    match crate::util::paths::assert_within(&self.models_dir, &full) {
                        Ok(canonical) => {
                            // Surface deletion failure rather than
                            // silently swallowing it. Pre-fix this
                            // was `let _ = remove_dir_all(...)`,
                            // which meant a busy-file / permission /
                            // anti-virus-lock failure looked like a
                            // successful uninstall to the caller —
                            // we'd then DELETE the DB row, orphan
                            // the bytes on disk, and the next
                            // install couldn't cleanly overwrite.
                            // Now: log + return the error and leave
                            // the DB row in place so the user (or
                            // ModelRegistry::remove retry) sees the
                            // model is still installed.
                            if let Err(e) = std::fs::remove_dir_all(&canonical) {
                                tracing::error!(
                                    model_id = id,
                                    path = %canonical.display(),
                                    error = %e,
                                    "model.remove failed: remove_dir_all on the model directory failed; leaving DB row in place"
                                );
                                return Err(anyhow::Error::new(e).context(format!(
                                    "remove_dir_all failed for model '{id}' at {}",
                                    canonical.display()
                                )));
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                model_id = id,
                                path = %full.display(),
                                error = %e,
                                "model.remove refused: target is outside models_dir after canonicalisation"
                            );
                            return Err(e);
                        }
                    }
                }
            }
            let conn = self.db.conn()?;
            conn.execute("DELETE FROM models WHERE id=?1", params![id])?;
        }
        Ok(())
    }
}

fn emit(handle: &AppHandle, model_id: &str, done: u64, total: u64, phase: ModelProgressPhase) {
    let _ = handle.emit(
        EVT_MODEL_PROGRESS,
        ModelProgress {
            model_id: model_id.into(),
            bytes_done: done,
            bytes_total: total,
            phase,
        },
    );
}

/// Stream-decompress and un-archive a `.tar.bz2` into `dest`. Blocking; call inside
/// `spawn_blocking`.
fn extract_tar_bz2(archive_path: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    let file = std::fs::File::open(archive_path)?;
    let bz = BzDecoder::new(std::io::BufReader::new(file));
    let mut tar = Archive::new(bz);
    tar.unpack(dest)?;
    Ok(())
}

/// If `dir` contains exactly one child that is itself a directory, move every entry
/// from that child into `dir` and delete the empty child.
fn flatten_single_subdir(dir: &std::path::Path) -> Result<()> {
    let entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    if entries.len() != 1 {
        return Ok(());
    }
    let only = &entries[0];
    if !only.file_type()?.is_dir() {
        return Ok(());
    }
    let inner = only.path();
    for child in std::fs::read_dir(&inner)?.filter_map(|e| e.ok()) {
        let from = child.path();
        let to = dir.join(child.file_name());
        std::fs::rename(&from, &to)?;
    }
    let _ = std::fs::remove_dir(&inner);
    Ok(())
}

/// Built-in catalogue. URLs point at sherpa-onnx / k2-fsa publicly hosted exports.
/// Every entry MUST carry a pinned sha256 — the install path verifies the downloaded
/// archive against this and fails closed if the hash mismatches OR is empty. To add a
/// new model: download the asset, run `sha256sum`, paste the digest here. Bumping a
/// model version requires re-pinning; HTTPS-from-GitHub alone is not sufficient
/// integrity (a compromised GH release or MITM with a forged cert would otherwise be
/// invisible).
fn built_in_catalogue() -> Vec<ModelInfo> {
    // Catalogue scope — what IS and ISN'T plug-compatible with the current
    // inference pipeline. `services::inference::sherpa_parakeet` is hard-wired
    // to `sherpa_rs::transducer::TransducerRecognizer` with `model_type =
    // "nemo_transducer"`, expects an encoder/decoder/joiner/tokens layout, and
    // is therefore Parakeet-TDT-only. The broader k2-fsa release has lots of
    // other attractive variants we COULD offer eventually:
    //
    //   • Streaming Zipformer (real-time transcription) — needs the
    //     `OnlineRecognizer` API path, a ring-buffered feeder, and a different
    //     partial-result surface in the pipeline. Tracked as a follow-up
    //     because it's a meaningful refactor, not a one-line addition.
    //   • Whisper exports — `OfflineRecognizer` with `WhisperConfig`, separate
    //     tokenizer layout. Would give us multilingual + punctuation.
    //   • NeMo Canary flash (180M, int8, EN/ES/DE/FR) — small enough to ship
    //     as "multilingual", but also its own recognizer variant.
    //
    // Adding any of those here without the runtime plumbing would just hand
    // the user a downloaded model that crashes on first use, so they stay
    // out of the catalogue until the inference layer learns to dispatch on
    // model kind. Everything below is Parakeet-TDT and genuinely runnable.
    use crate::domain::ModelEngine;
    vec![
        // Parakeet v3 int8 — the recommended default. Newer NeMo
        // generation than v2, same 0.6B parameter budget, same TDT
        // architecture, slightly better WER on conversational English.
        // Drops to int8 weights for fast CPU inference. English only;
        // multilingual support arrives via Whisper in v0.8.1.
        ModelInfo {
            id: "sherpa-parakeet-tdt-0.6b-v3-int8".into(),
            kind: ModelKind::Asr,
            display_name: "Parakeet v3 (English, compact)".into(),
            description: "Recommended for most people. Fast on any modern CPU, accurate on conversational English. Pick this unless you have a specific reason not to.".into(),
            version: "3".into(),
            size_bytes: 487_170_055,
            backend: ModelBackend::Onnx,
            engine: ModelEngine::ParakeetTdt,
            languages: vec!["en".into()],
            installed: false,
            path: None,
            // Pinned 2026-04-26 against the live k2-fsa release asset.
            // Verified by the fail-closed branch in `install_model`:
            // any byte-for-byte tampering of the upstream tarball
            // produces a hash mismatch and the install aborts.
            sha256: "5793d0fd397c5778d2cf2126994d58e9d56b1be7c04d13c7a15bb1b4eafb16bf".into(),
            download_url: Some(
                "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8.tar.bz2"
                    .into(),
            ),
            recommended: true,
            accuracy_score: 9,
            speed_score: 10,
            requires_pro: false,
        },
        // Whisper base — multilingual, free tier. ~99 languages via
        // OpenAI Whisper. Slower than Parakeet on English but the only
        // free-tier path to non-English dictation. Bundle name + size
        // confirmed against the live k2-fsa GitHub release at v0.8.6
        // build time. (v0.8.2–v0.8.5 used a guessed `.int8.tar.bz2`
        // suffix that didn't exist — those releases listed Whisper but
        // every install attempt 404'd silently. Fixed here.)
        ModelInfo {
            id: "sherpa-whisper-base".into(),
            kind: ModelKind::Asr,
            display_name: "Whisper base (multilingual)".into(),
            description: "OpenAI Whisper, ~99 languages with quality varying by language. Pick this if you dictate in anything other than English. Slower than Parakeet on English; comparable on non-English.".into(),
            version: "base".into(),
            size_bytes: 207_557_382,
            backend: ModelBackend::Onnx,
            engine: ModelEngine::Whisper,
            languages: vec!["multi".into()],
            installed: false,
            path: None,
            // Pinned 2026-04-26 against the live k2-fsa release asset.
            sha256: "911b2083efd7c0dca2ac3b358b75222660dc09fb716d64fbfc417ba6c99ff3de".into(),
            download_url: Some(
                "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-whisper-base.tar.bz2"
                    .into(),
            ),
            recommended: false,
            accuracy_score: 7,
            speed_score: 6,
            requires_pro: false,
        },
        // Whisper small — Pro tier. Multilingual. From v0.15.4 the
        // recommended Meeting Mode default: the fastest of the three
        // allow-listed meeting models, with acceptable accuracy on
        // clear speech. Step 0 measurements on Jay's machine showed
        // it transcribes a real meeting comfortably faster than
        // realtime — Medium took ~real-time on the same audio, and
        // a no-preference user wants something that returns text
        // before the next meeting starts.
        ModelInfo {
            id: "sherpa-whisper-small".into(),
            kind: ModelKind::Asr,
            display_name: "Whisper small (multilingual)".into(),
            description: "Fastest Meeting Mode option. Lower accuracy than Medium or Large v3 — proper nouns and unusual words are the weakest point — but acceptable for clear speech and the quickest of the three options.".into(),
            version: "small".into(),
            size_bytes: 639_387_718,
            backend: ModelBackend::Onnx,
            engine: ModelEngine::Whisper,
            languages: vec!["multi".into()],
            installed: false,
            path: None,
            // Pinned 2026-04-26 against the live k2-fsa release asset.
            sha256: "486a46afbb7ba798507190ffe02fea2dd726049af212e774537efac6afb210a6".into(),
            download_url: Some(
                "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-whisper-small.tar.bz2"
                    .into(),
            ),
            // v0.15.4: recommended default for Meeting Mode. The
            // picker's Recommended chip lights up on whichever
            // meeting-allow-listed model has this flag set; v0.15.3
            // had it on Medium.
            recommended: true,
            accuracy_score: 9,
            speed_score: 4,
            requires_pro: true,
        },
        // Whisper medium — Pro tier. Multilingual. The "Balanced"
        // explicit option in the Meeting Mode picker. v0.15.3 made
        // Medium the default; v0.15.4 demoted it to "explicit
        // choice" because Step 0 measurements showed it ran roughly
        // real-time on Jay's machine — too slow to be a sane
        // no-preference default on laptop-class CPUs. Still picked
        // when the user wants slightly higher accuracy than Small
        // and is willing to wait.
        //
        // Bundle ships fp32 + int8 variants in the same tarball;
        // the resolver in services/inference/sherpa_whisper.rs
        // picks int8 at runtime, so the on-disk footprint is bigger
        // than the int8-only Large but the runtime memory load is
        // similar.
        //
        // Pinned 2026-04-29 against the live k2-fsa release asset.
        // SHA + size verified locally via the v0.15.3 Step 0 ritual
        // (download → sha256sum → tar tjf → smoke-load through
        // SherpaWhisper::ensure_loaded → transcribe a bundled wav
        // and compare against the bundle's trans.txt). Do not change
        // these without re-running that flow.
        ModelInfo {
            id: "sherpa-whisper-medium".into(),
            kind: ModelKind::Asr,
            display_name: "Whisper medium (multilingual, balanced)".into(),
            description: "Balanced Meeting Mode option. Slower than Small with slightly higher quality, especially on proper nouns and unusual words. Multilingual; pin Whisper language to your meeting language for best results on quiet or music-heavy audio.".into(),
            version: "medium".into(),
            size_bytes: 1_931_372_882,
            backend: ModelBackend::Onnx,
            engine: ModelEngine::Whisper,
            languages: vec!["multi".into()],
            installed: false,
            path: None,
            sha256: "614b1172557049069d846c29d9399640bce83a4dd6c580decebd9ce2a4f32c33".into(),
            download_url: Some(
                "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-whisper-medium.tar.bz2"
                    .into(),
            ),
            // v0.15.4: NOT recommended — Small is the new Meeting
            // Mode default. v0.15.3 had this `true`.
            recommended: false,
            accuracy_score: 9,
            speed_score: 5,
            requires_pro: true,
        },
        // Whisper large v3 — Pro tier, the highest-accuracy Meeting
        // Mode option. Multilingual. Picked when the audio is very
        // noisy / multi-speaker / heavily accented and the user is
        // willing to wait for a slower transcribe. v0.15.3 stopped
        // auto-routing meetings here — the meeting model is an
        // explicit user choice in Settings → Models. v0.15.4 made
        // Small the no-preference default; Medium and Large remain
        // explicit-only choices.
        ModelInfo {
            id: "sherpa-whisper-large-v3".into(),
            kind: ModelKind::Asr,
            display_name: "Whisper Large v3 (highest accuracy)".into(),
            description: "Highest-accuracy Whisper checkpoint. Picked as the Meeting Mode \"Best accuracy\" option when audio is very noisy, multi-speaker, or accented and you don't mind a slower transcribe. Slower than Small or Medium.".into(),
            version: "large-v3".into(),
            size_bytes: 1_068_482_488,
            backend: ModelBackend::Onnx,
            engine: ModelEngine::Whisper,
            languages: vec!["multi".into()],
            installed: false,
            path: None,
            // Pinned 2026-04-28 against the live k2-fsa release asset.
            sha256: "2d0e134b3b5fc4a0533baf24a0c9d473b629aa47f030af0a165a05f461df7a03".into(),
            download_url: Some(
                "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-whisper-large-v3.tar.bz2"
                    .into(),
            ),
            recommended: false,
            accuracy_score: 10,
            speed_score: 2,
            requires_pro: true,
        },
        // Whisper large-v3 turbo — Pro tier, multilingual. A pruned
        // large-v3 (decoder trimmed 32 -> 4 layers) that keeps close to
        // large-v3 accuracy at roughly 5x the speed. Ships int8
        // encoder/decoder/tokens only and loads through the same
        // SherpaWhisper path as the other Whisper checkpoints. Being a
        // non-Small Whisper model it uses DirectML on Auto (unlike
        // Whisper Small, which the resolver keeps on CPU), so it is the
        // natural "Whisper accuracy without the wait" pick on a GPU.
        //
        // Long-audio note: like every offline Whisper, sherpa truncates
        // a single decode at ~30s, so turbo relies on the same 29s
        // chunking the provider added in v0.20.5 for dictations past
        // half a minute.
        //
        // Pinned 2026-07-16 against the live k2-fsa release asset:
        // downloaded sherpa-onnx-whisper-turbo.tar.bz2, verified the
        // asset size + SHA-256 below, and confirmed the archive ships
        // turbo-encoder.int8.onnx / turbo-decoder.int8.onnx /
        // turbo-tokens.txt (the suffixes resolve_files matches). Smoke
        // load through SherpaWhisper::ensure_loaded is still pending on
        // Jay's machine.
        ModelInfo {
            id: "sherpa-whisper-turbo".into(),
            kind: ModelKind::Asr,
            display_name: "Whisper Turbo (multilingual, fast)".into(),
            description: "Close to Whisper Large v3 accuracy but much faster, and it uses your GPU on Auto. Multilingual, around 99 languages. A strong all-round pick when you want Whisper quality without waiting as long as Large v3.".into(),
            version: "turbo".into(),
            size_bytes: 563_790_207,
            backend: ModelBackend::Onnx,
            engine: ModelEngine::Whisper,
            languages: vec!["multi".into()],
            installed: false,
            path: None,
            sha256: "b11acbbcd660b44a8e0df33724feb5aaa709cf65668f2823d59f656312544f22".into(),
            download_url: Some(
                "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-whisper-turbo.tar.bz2"
                    .into(),
            ),
            recommended: false,
            accuracy_score: 9,
            speed_score: 7,
            requires_pro: true,
        },
        // Punctuation restoration (sherpa CT-Transformer, zh-en, int8).
        // ModelKind::Punct keeps it out of the dictation-model pickers;
        // the Text Output "Restore punctuation" toggle uses it to add
        // commas / full stops / question marks to a raw transcript via a
        // token classifier (not an LLM rewrite). The zh-en vocab covers
        // English; Chinese tokens are harmless for English dictation.
        // accuracy_score / speed_score are unused for a non-ASR model.
        //
        // Pinned 2026-07-16 against the live k2-fsa punctuation-models
        // release: downloaded and hashed the int8 tarball.
        ModelInfo {
            id: "sherpa-punct-ct-transformer".into(),
            kind: ModelKind::Punct,
            display_name: "Punctuation restoration".into(),
            description: "Adds punctuation to your dictation (commas, full stops, question marks). A small on-device model, not an AI rewrite. Turn it on under Text Output once installed.".into(),
            version: "ct-transformer-2024-04-12".into(),
            size_bytes: 64_717_756,
            backend: ModelBackend::Onnx,
            engine: ModelEngine::Punct,
            languages: vec!["multi".into()],
            installed: false,
            path: None,
            sha256: "c0d5aa5f8eeb686032345e180bedf39319dc2e0556781c6264bcadba8328a6e1".into(),
            download_url: Some(
                "https://github.com/k2-fsa/sherpa-onnx/releases/download/punctuation-models/sherpa-onnx-punct-ct-transformer-zh-en-vocab272727-2024-04-12-int8.tar.bz2"
                    .into(),
            ),
            recommended: false,
            accuracy_score: 0,
            speed_score: 0,
            requires_pro: false,
        },
        // Offline Zipformer was retired in v0.14.2: Parakeet covers the
        // English fast-path comprehensively, and shipping two
        // English-only fast models was creating decision fatigue in the
        // model picker. Whisper covers multilingual; Parakeet covers
        // English-fast. The Zipformer tarball stays available in
        // k2-fsa's release archive if anyone needs it. Existing installs
        // get the entry pruned via the AppState bootstrap migration in
        // app_state.rs (same shape as the v0.8.0 Parakeet v2 cleanup).
    ]
}

#[cfg(test)]
pub(crate) mod test_helpers {
    //! Test fixtures for `ModelRegistry`. Lets cross-module tests
    //! (notably `commands::dictation`'s meeting-resolution policy)
    //! set up a registry with specific models marked installed
    //! without going through the real download flow.

    use super::{ModelBackend, ModelKind, ModelRegistry};
    use rusqlite::params;
    use std::path::Path;

    /// Mark a model as installed in the registry's DB AND create a
    /// non-empty directory at the matching `models_dir/<id>/` so
    /// `ModelRegistry::is_installed` returns true (it requires both
    /// the row AND the on-disk path to exist).
    ///
    /// Used by the v0.15.3 selection-policy tests that exercise
    /// `resolve_meeting_model_id` against various installed-model
    /// combinations. Inserts only the columns needed for the
    /// `is_installed` lookup; tests that need other ModelInfo
    /// fields read the catalogue separately.
    pub fn register_for_test(registry: &ModelRegistry, models_root: &Path, id: &str) {
        let conn = registry.db.conn().expect("test db conn");
        let path_rel = id; // matches the `path_rel = entry.id.clone()` in real install
        conn.execute(
            "INSERT OR REPLACE INTO models(id,kind,display_name,version,size_bytes,path,backend,languages,installed_at,last_used_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,NULL)",
            params![
                id,
                "asr",
                id,
                "test",
                0_i64,
                path_rel,
                match ModelBackend::Onnx {
                    ModelBackend::Onnx => "onnx",
                    _ => "onnx",
                },
                "[\"multi\"]",
                0_i64,
            ],
        )
        .expect("insert models row");
        // is_installed checks `models_dir.join(rel).exists()`, so
        // create the directory.
        let _ = std::mem::discriminant(&ModelKind::Asr); // keep ModelKind in scope
        std::fs::create_dir_all(models_root.join(path_rel)).expect("create test model dir");
    }
}

#[cfg(test)]
mod catalogue_tests {
    //! Pinning tests for the v0.15.4 Meeting Mode catalogue contract.
    //! `recommended` drives the "Recommended" chip in the picker, so
    //! a silent flip would change visible UI without a code review.
    use super::built_in_catalogue;

    fn find(id: &str) -> super::ModelInfo {
        built_in_catalogue()
            .into_iter()
            .find(|m| m.id == id)
            .unwrap_or_else(|| panic!("catalogue must contain {id}"))
    }

    /// **v0.15.4 contract.** Whisper Small carries the Recommended
    /// chip; Medium does not. Pre-v0.15.4 it was the other way
    /// around — guard the flip so a future catalogue edit can't
    /// silently revert.
    #[test]
    fn whisper_small_is_recommended_and_medium_is_not() {
        assert!(
            find("sherpa-whisper-small").recommended,
            "Whisper Small must carry the Recommended chip in v0.15.4"
        );
        assert!(
            !find("sherpa-whisper-medium").recommended,
            "Whisper Medium must NOT carry the Recommended chip in v0.15.4"
        );
    }

    /// Whisper Large v3 is never the recommended option — it's
    /// the explicit "Best accuracy / slowest" choice.
    #[test]
    fn whisper_large_is_not_recommended() {
        assert!(!find("sherpa-whisper-large-v3").recommended);
    }

    /// Catalogue copy must not bake numeric performance claims into
    /// user-facing description strings. Step 0 numbers go in release
    /// notes; the catalogue surfaces relative speed/quality only.
    /// Pinned because it's tempting to add "RTF 0.2" or "5× faster"
    /// to the description and that's the kind of claim that ages
    /// badly across hardware.
    #[test]
    fn meeting_model_descriptions_avoid_numeric_performance_claims() {
        for id in [
            "sherpa-whisper-small",
            "sherpa-whisper-medium",
            "sherpa-whisper-large-v3",
        ] {
            let desc = find(id).description;
            for needle in ["RTF", "real-time", "realtime", "× faster", "x faster"] {
                assert!(
                    !desc.to_lowercase().contains(&needle.to_lowercase()),
                    "{id} description must not contain {needle:?} — keep numeric perf out of the catalogue. Got: {desc}"
                );
            }
        }
    }
}
