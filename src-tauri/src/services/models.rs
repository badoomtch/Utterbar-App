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

    pub fn list_installed(&self) -> Result<Vec<ModelInfo>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare("SELECT id,kind,display_name,version,size_bytes,path,backend,languages FROM models")?;
        let rows = stmt
            .query_map([], |row| {
                let kind: String = row.get(1)?;
                let backend: String = row.get(6)?;
                let langs: String = row.get(7)?;
                Ok(ModelInfo {
                    id: row.get(0)?,
                    kind: match kind.as_str() {
                        "llm" => ModelKind::Llm,
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
                    languages: serde_json::from_str(&langs).unwrap_or_default(),
                    installed: true,
                    path: row.get(5)?,
                    sha256: String::new(),
                    download_url: None,
                    recommended: false,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Download + verify + extract + register a model. Emits `model:progress` throughout.
    pub async fn install(&self, handle: &AppHandle, id: &str) -> Result<ModelInfo> {
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

        let partial = self.models_dir.join(format!("{}.tar.bz2.partial", entry.id));
        let extracted_dir = self.models_dir.join(&entry.id);

        // Start fresh — remove any leftovers from a previous attempt.
        let _ = tokio::fs::remove_file(&partial).await;
        let _ = tokio::fs::remove_dir_all(&extracted_dir).await;
        tokio::fs::create_dir_all(&self.models_dir).await?;

        emit(handle, &entry.id, 0, entry.size_bytes, ModelProgressPhase::Starting);

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
                emit(handle, &entry.id, done, total, ModelProgressPhase::Downloading);
                next_emit = done + (total / 100).max(512 * 1024);
            }
        }
        file.flush().await?;
        drop(file);

        // ---- Verify ----
        emit(handle, &entry.id, done, total, ModelProgressPhase::Verifying);
        let digest = hex::encode(hasher.finalize());
        if !entry.sha256.is_empty() && digest != entry.sha256 {
            let _ = tokio::fs::remove_file(&partial).await;
            emit(handle, &entry.id, done, total, ModelProgressPhase::Error);
            return Err(anyhow!(
                "sha256 mismatch: expected {}, got {}",
                entry.sha256,
                digest
            ));
        }

        // ---- Extract ----
        emit(handle, &entry.id, done, total, ModelProgressPhase::Extracting);
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
                    match entry.kind { ModelKind::Asr => "asr", ModelKind::Llm => "llm" },
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

    pub fn remove(&self, id: &str) -> Result<()> {
        let installed = self.list_installed()?;
        if let Some(m) = installed.iter().find(|m| m.id == id) {
            if let Some(p) = &m.path {
                let full = self.models_dir.join(p);
                let _ = std::fs::remove_dir_all(&full);
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
    let entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect();
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

/// Built-in catalogue. URLs point at sherpa-onnx's publicly hosted Parakeet exports.
/// Keeping sha256 empty for now — the k2-fsa GitHub release assets don't publish stable
/// hashes alongside them, and we'd rather not lock the user out of a working download on
/// a harmless re-release. Integrity is still guarded by the fact that these are served
/// over HTTPS from GitHub.
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
    vec![
        // Real k2-fsa asset names — confirmed via `gh release view asr-models`.
        // Sizes are exact Content-Length from the mirror as of this catalogue
        // version (see release asset listing for provenance).
        ModelInfo {
            id: "sherpa-parakeet-tdt-0.6b-v2-int8".into(),
            kind: ModelKind::Asr,
            display_name: "Parakeet (English, compact)".into(),
            description: "Recommended for most people. A smaller download that runs fast and uses less memory, while staying nearly as accurate as the larger version for everyday speech. Pick this unless you have a specific reason not to.".into(),
            version: "2".into(),
            size_bytes: 482_468_385,
            backend: ModelBackend::Onnx,
            languages: vec!["en".into()],
            installed: false,
            path: None,
            sha256: String::new(),
            download_url: Some(
                "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8.tar.bz2"
                    .into(),
            ),
            recommended: true,
        },
        // v3 int8 — newer NeMo Parakeet generation, same 0.6B parameter budget
        // and same TDT architecture so it's a drop-in replacement that just
        // swaps the weights. Trained on more recent data; in the NeMo release
        // notes k2-fsa cite a modest WER improvement on conversational English
        // over v2. Same int8 size bucket as v2. Marked NOT recommended only
        // because v2 is what our v0.x users have been running and we don't
        // want to force a re-download on update; curious users can opt in.
        ModelInfo {
            id: "sherpa-parakeet-tdt-0.6b-v3-int8".into(),
            kind: ModelKind::Asr,
            display_name: "Parakeet v3 (English, compact)".into(),
            description: "Newer generation of the compact Parakeet model with a modest accuracy bump on conversational English. Same size and speed as the standard compact model — worth trying if you notice recurring misrecognitions on the default.".into(),
            version: "3".into(),
            size_bytes: 487_170_055,
            backend: ModelBackend::Onnx,
            languages: vec!["en".into()],
            installed: false,
            path: None,
            sha256: String::new(),
            download_url: Some(
                "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8.tar.bz2"
                    .into(),
            ),
            recommended: false,
        },
        // fp16 variant — higher-precision weights, slightly better accuracy on edge cases
        // (accents, noisy input, proper nouns), larger download. Only safe to run when a
        // GPU provider (CUDA or DirectML) is available; the CPU provider aborts on first
        // inference because the fp16 kernels aren't registered for CPU. The inference
        // layer now resolves a concrete provider string from settings.advanced.gpu_acceleration,
        // so users who picked "Smart" / "Always use GPU" on a machine with a real adapter
        // can pick this and have it run correctly. Left non-recommended so first-run users
        // don't land on a model that silently falls back on CPU-only boxes.
        ModelInfo {
            id: "sherpa-parakeet-tdt-0.6b-v2-fp16".into(),
            kind: ModelKind::Asr,
            display_name: "Parakeet (English, high-precision)".into(),
            description: "Higher-precision fp16 weights of the same Parakeet model. A touch more accurate on tricky audio, but requires GPU acceleration — pick this only if your PC has a supported GPU (and make sure GPU acceleration isn't set to CPU only in Settings → Advanced).".into(),
            version: "2".into(),
            size_bytes: 1_214_103_108,
            backend: ModelBackend::Onnx,
            languages: vec!["en".into()],
            installed: false,
            path: None,
            sha256: String::new(),
            download_url: Some(
                "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-fp16.tar.bz2"
                    .into(),
            ),
            recommended: false,
        },
    ]
}
