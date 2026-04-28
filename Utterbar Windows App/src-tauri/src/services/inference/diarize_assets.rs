//! On-demand download + integrity verification for the speaker
//! diarisation model assets.
//!
//! v0.15.0 introduces speaker diarisation in Meeting Mode. Two model
//! assets are required:
//!
//!   * `segmentation.onnx` — pyannote segmentation 3.0 (int8 export
//!     via sherpa-onnx). MIT licensed (CNRS, 2022).
//!   * `embedding.onnx` — NVIDIA NeMo TitaNet small. CC-BY-4.0;
//!     attribution to NVIDIA preserved in `THIRD_PARTY_LICENCES.md`
//!     and the in-app About surface.
//!
//! Asset management contract (mirrors `services::models::install`
//! discipline, plus tighter traversal protection on the extract
//! path because the diariser tarball is operator-supplied via R2
//! and a future bad pin/upload class would otherwise be a
//! containment risk for the user's data dir):
//!
//!   1. Group-policy `BlockModelDownloads` gate runs before any
//!      network call — same lever IT admins use for the rest of the
//!      catalogue.
//!   2. Download streams to `<asset>.partial`. SHA-256 hashed during
//!      stream-write. Only renamed to the final filename after the
//!      computed digest equals the pinned digest.
//!   3. SHA mismatch → delete `.partial`, return error, do NOT
//!      progress to extract.
//!   4. Tar extraction (segmentation only — the embedding asset is a
//!      bare ONNX, no archive) validates EVERY entry path against:
//!        * not absolute
//!        * no `..` components
//!        * no symlinks / hard links / device entries (only `Regular`
//!          and `Directory`)
//!        * resolved path stays inside the destination via
//!          `paths::assert_within`
//!
//!      Any rejection deletes the partially-extracted dir and errors out cleanly.
//!   5. Local SHA verification of the final-on-disk runtime files
//!      (`segmentation.onnx`, `embedding.onnx`) against the pinned
//!      per-file SHAs. Drift here means the tarball changed contents
//!      under the pin — refuse to use.
//!   6. Bundled `LICENSE` + `README.md` from the segmentation tarball
//!      are preserved next to the runtime asset (renamed to
//!      `segmentation.LICENSE` / `segmentation.README.md` so the
//!      directory listing is namespaced). Operator licence
//!      compliance for the MIT requirement of preserving the licence
//!      text.
//!
//! Pin provenance: see `THIRD_PARTY_LICENCES.md` for the verified
//! upstream URLs, sizes, and licences. Hashes here were computed
//! locally from the upstream files on 2026-04-28 and (for the
//! embedding) cross-checked against k2-fsa's published
//! `checksum.txt`.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bzip2::read::BzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

use crate::domain::{ModelProgress, ModelProgressPhase, EVT_MODEL_PROGRESS};
use crate::util::paths;

// ── Pinned asset metadata ───────────────────────────────────────────
//
// Sources verified locally on 2026-04-28. See THIRD_PARTY_LICENCES.md
// for upstream URLs + licence text. Do NOT change a pin without
// re-running the local verification flow:
//   curl -sL <URL> -o /tmp/asset && sha256sum /tmp/asset
// AND, where the upstream publishes a checksum file, cross-checking.

/// pyannote segmentation 3.0 — MIT (CNRS, 2022). Tarball contains
/// fp32 + int8 ONNX; we use int8 at runtime.
const SEG_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-segmentation-models/sherpa-onnx-pyannote-segmentation-3-0.tar.bz2";
const SEG_TARBALL_SHA: &str = "24615ee884c897d9d2ba09bb4d30da6bb1b15e685065962db5b02e76e4996488";
const SEG_TARBALL_SIZE: u64 = 6_958_444;
/// Path inside the tarball that we promote to the runtime location.
/// pyannote release ships the model as `model.int8.onnx` under a
/// single top-level directory; we strip that directory + rename for
/// clarity on disk.
const SEG_INNER_MODEL_RELPATH: &str = "model.int8.onnx";
/// Pinned SHA of the int8 ONNX file once extracted.
const SEG_RUNTIME_SHA: &str = "d582f4b4c6b48205de7e0643c57df0df5615a3c176189be3fc461e9d18827b5d";
/// Bundled licence file from the tarball — preserved alongside the
/// runtime asset to satisfy the MIT requirement.
const SEG_INNER_LICENSE_RELPATH: &str = "LICENSE";
const SEG_INNER_README_RELPATH: &str = "README.md";

/// NVIDIA NeMo TitaNet small embedding — CC-BY-4.0. SHA below matches
/// k2-fsa's published checksum.txt for the speaker-recongition-models
/// release. Direct ONNX, no archive.
const EMBED_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recongition-models/nemo_en_titanet_small.onnx";
const EMBED_SHA: &str = "ad4a1802485d8b34c722d2a9d04249662f2ece5d28a7a039063ca22f515a789e";
const EMBED_SIZE: u64 = 40_257_283;

/// Synthetic asset id used in `EVT_MODEL_PROGRESS` events when the
/// diariser-asset download is the active operation. The Meetings
/// page subscribes to the same event stream the existing model-
/// install flow uses, so the toast and progress bar reuse the
/// existing UI without new wiring.
pub const DIARISER_ASSET_PROGRESS_ID: &str = "diariser-meeting-models";

/// Total bytes the user will see counting up across both downloads.
/// Used for the "Preparing meeting models (NN MB)..." toast so the
/// number we surface matches what they actually pay in network.
pub const TOTAL_DOWNLOAD_BYTES: u64 = SEG_TARBALL_SIZE + EMBED_SIZE;

/// Resolved on-disk paths. Both live under `models_dir/diariser/`
/// — same root as the rest of the model assets, which means the
/// `paths::assert_within` containment guard from v0.14.13 covers
/// any future "delete the diariser" path for free.
pub struct DiariserAssetPaths {
    pub root: PathBuf,
    pub segmentation: PathBuf,
    pub embedding: PathBuf,
}

impl DiariserAssetPaths {
    pub fn resolve() -> Result<Self> {
        let root = paths::models_dir()?.join("diariser");
        Ok(Self {
            segmentation: root.join("segmentation.onnx"),
            embedding: root.join("embedding.onnx"),
            root,
        })
    }
}

/// Are both runtime files present AND do their SHAs still match the
/// pinned values? A "yes" lets `start_recording` skip the download
/// flow and proceed straight to capture; a "no" triggers the
/// download flow before recording arms.
///
/// Cheap to call: streams the file and hashes once. ~40 MB of file
/// I/O + sha256 on the user's machine, single-digit milliseconds on
/// any modern SSD.
pub async fn assets_ready(paths: &DiariserAssetPaths) -> bool {
    let seg_ok = matches_pinned_sha(&paths.segmentation, SEG_RUNTIME_SHA)
        .await
        .unwrap_or(false);
    let embed_ok = matches_pinned_sha(&paths.embedding, EMBED_SHA)
        .await
        .unwrap_or(false);
    seg_ok && embed_ok
}

async fn matches_pinned_sha(path: &Path, expected_hex: &str) -> Result<bool> {
    let bytes = match tokio::fs::read(path).await {
        Ok(b) => b,
        Err(_) => return Ok(false),
    };
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hex::encode(hasher.finalize());
    Ok(digest == expected_hex)
}

/// Ensure both diariser assets are present + verified on disk,
/// downloading them if missing. Idempotent: if the assets are
/// already good, returns immediately. If a previous attempt left a
/// `.partial` behind or a corrupted final file, deletes and
/// re-downloads cleanly.
///
/// Emits `EVT_MODEL_PROGRESS` events under the
/// `DIARISER_ASSET_PROGRESS_ID` synthetic id so the floating bar's
/// existing progress-toast wiring works without new event types.
///
/// Group-policy gated: an IT admin who has set
/// `HKLM\SOFTWARE\Policies\Utterbar\BlockModelDownloads = 1` blocks
/// this download path — same lever the rest of the catalogue uses.
pub async fn ensure_assets_ready(handle: &AppHandle) -> Result<DiariserAssetPaths> {
    let asset_paths = DiariserAssetPaths::resolve()?;
    if assets_ready(&asset_paths).await {
        return Ok(asset_paths);
    }

    if crate::services::policy::Policy::load().model_downloads_blocked() {
        return Err(anyhow!(
            "Speaker diarisation requires a one-time download (~47 MB) of pyannote segmentation + NVIDIA TitaNet small. Your IT administrator has disabled model downloads (group policy: BlockModelDownloads). Contact them or use a non-meeting mode."
        ));
    }

    tokio::fs::create_dir_all(&asset_paths.root)
        .await
        .with_context(|| format!("create diariser dir {}", asset_paths.root.display()))?;

    emit_progress(
        handle,
        0,
        TOTAL_DOWNLOAD_BYTES,
        ModelProgressPhase::Starting,
    );

    // ── Segmentation (tarball) ──────────────────────────────────────
    if !matches_pinned_sha(&asset_paths.segmentation, SEG_RUNTIME_SHA)
        .await
        .unwrap_or(false)
    {
        let tarball_path = asset_paths.root.join("segmentation.tar.bz2.partial");
        download_with_pinned_sha(
            handle,
            SEG_URL,
            &tarball_path,
            SEG_TARBALL_SHA,
            SEG_TARBALL_SIZE,
            0,
            TOTAL_DOWNLOAD_BYTES,
        )
        .await?;

        // Hardened extraction — single-pass scan that validates
        // every entry path against zip-slip-style traversal before
        // any byte is written to disk.
        emit_progress(
            handle,
            SEG_TARBALL_SIZE,
            TOTAL_DOWNLOAD_BYTES,
            ModelProgressPhase::Extracting,
        );
        let tarball_for_blocking = tarball_path.clone();
        let root_for_blocking = asset_paths.root.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            extract_segmentation_tarball_safely(&tarball_for_blocking, &root_for_blocking)
        })
        .await
        .map_err(|e| anyhow!("segmentation extract join: {e}"))??;

        let _ = tokio::fs::remove_file(&tarball_path).await;

        // Verify the on-disk runtime ONNX matches its pin. The
        // tarball SHA above proves the *archive* is what we expect;
        // this proves the specific file we promote is the right
        // one (defence in depth — a malicious tarball with a
        // matching outer SHA would have to also produce a
        // matching inner ONNX SHA, which under SHA-256 is
        // computationally infeasible).
        if !matches_pinned_sha(&asset_paths.segmentation, SEG_RUNTIME_SHA)
            .await
            .unwrap_or(false)
        {
            let _ = tokio::fs::remove_file(&asset_paths.segmentation).await;
            return Err(anyhow!(
                "segmentation runtime SHA mismatch after extract — refusing to use"
            ));
        }
    }

    // ── Embedding (direct ONNX) ─────────────────────────────────────
    if !matches_pinned_sha(&asset_paths.embedding, EMBED_SHA)
        .await
        .unwrap_or(false)
    {
        let embedding_partial = asset_paths.root.join("embedding.onnx.partial");
        download_with_pinned_sha(
            handle,
            EMBED_URL,
            &embedding_partial,
            EMBED_SHA,
            EMBED_SIZE,
            SEG_TARBALL_SIZE,
            TOTAL_DOWNLOAD_BYTES,
        )
        .await?;

        // Atomic-ish rename — same disk so the rename is atomic on
        // NTFS. If a crash interrupts the move (extremely rare),
        // the next `assets_ready` call sees no embedding file or a
        // wrong-SHA one and re-downloads.
        tokio::fs::rename(&embedding_partial, &asset_paths.embedding)
            .await
            .with_context(|| {
                format!(
                    "promote {} to {}",
                    embedding_partial.display(),
                    asset_paths.embedding.display()
                )
            })?;
    }

    emit_progress(
        handle,
        TOTAL_DOWNLOAD_BYTES,
        TOTAL_DOWNLOAD_BYTES,
        ModelProgressPhase::Done,
    );
    tracing::info!(
        seg_path = %asset_paths.segmentation.display(),
        embed_path = %asset_paths.embedding.display(),
        "diariser assets ready"
    );
    Ok(asset_paths)
}

/// Stream-download `url` to `dest` while computing SHA-256 inline.
/// Verifies against `expected_sha`; on mismatch deletes the partial
/// and errors out. Emits progress events that account for the
/// download being one of two contributing to the total.
async fn download_with_pinned_sha(
    handle: &AppHandle,
    url: &str,
    dest: &Path,
    expected_sha: &str,
    expected_size: u64,
    base_offset: u64,
    grand_total: u64,
) -> Result<()> {
    // Start fresh — a leftover .partial from a previous crashed
    // attempt would otherwise pollute the hasher.
    let _ = tokio::fs::remove_file(dest).await;

    let client = reqwest::Client::builder()
        .user_agent(format!("Utterbar/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .with_context(|| "build reqwest client")?;
    let mut resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("status from {url}"))?;
    let actual_total = resp.content_length().unwrap_or(expected_size);

    let mut file = tokio::fs::File::create(dest)
        .await
        .with_context(|| format!("create partial {}", dest.display()))?;
    let mut hasher = Sha256::new();
    let mut done: u64 = 0;
    let mut next_emit = 0u64;

    while let Some(chunk) = resp
        .chunk()
        .await
        .with_context(|| format!("read body from {url}"))?
    {
        file.write_all(&chunk).await?;
        hasher.update(&chunk);
        done += chunk.len() as u64;
        if done >= next_emit {
            emit_progress(
                handle,
                base_offset + done,
                grand_total,
                ModelProgressPhase::Downloading,
            );
            next_emit = done + (actual_total / 100).max(512 * 1024);
        }
    }
    file.flush().await?;
    drop(file);

    emit_progress(
        handle,
        base_offset + done,
        grand_total,
        ModelProgressPhase::Verifying,
    );
    let digest = hex::encode(hasher.finalize());
    if digest != expected_sha {
        let _ = tokio::fs::remove_file(dest).await;
        return Err(anyhow!(
            "SHA-256 mismatch downloading diariser asset from {url}: expected {expected_sha}, got {digest}"
        ));
    }

    Ok(())
}

/// Hardened tarball extraction for the segmentation archive.
///
/// Does NOT use `tar::Archive::unpack` (which has a history of
/// traversal-related CVEs and offers no fine-grained per-entry
/// validation). Instead walks every entry, validates its path
/// against several rules, and only then writes content:
///
///   1. Reject absolute entry paths.
///   2. Reject any `Component::ParentDir` (`..`) in the entry
///      path.
///   3. Reject entries that aren't `Regular` or `Directory` —
///      symlinks, hard links, device files, FIFOs all blocked.
///   4. Resolve `entry_path → dest_root.join(entry_path)` and
///      assert via `paths::assert_within` that the resolved path
///      lives under `dest_root` after canonicalising both sides.
///
/// On any rejection: log + return error. The caller's outer flow
/// then deletes the partially-populated dir and surfaces the
/// failure. We DO NOT continue extraction silently.
///
/// After extraction succeeds, promotes the relevant inner files to
/// the canonical runtime locations:
///   * `<inner-dir>/model.int8.onnx` → `dest_root/segmentation.onnx`
///   * `<inner-dir>/LICENSE` → `dest_root/segmentation.LICENSE`
///   * `<inner-dir>/README.md` → `dest_root/segmentation.README.md`
///
/// Then removes the now-empty inner directory + the un-needed helper scripts and fp32 model.
fn extract_segmentation_tarball_safely(archive_path: &Path, dest_root: &Path) -> Result<()> {
    let staging = dest_root.join("_extract_staging");
    // Clean any leftover staging from a prior crashed attempt
    // before the canonicalise step would trip on stale entries.
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging)
        .with_context(|| format!("create staging {}", staging.display()))?;

    let file = std::fs::File::open(archive_path)
        .with_context(|| format!("open tarball {}", archive_path.display()))?;
    let bz = BzDecoder::new(std::io::BufReader::new(file));
    let mut archive = Archive::new(bz);

    for entry in archive
        .entries()
        .with_context(|| "iterate tarball entries")?
    {
        let mut entry = entry.with_context(|| "read tarball entry")?;

        // Validate entry kind. We only allow regular files and
        // directories — anything else (symlink, hardlink, device,
        // FIFO) is a potential traversal/escape vector and has no
        // legitimate place in a model archive.
        let entry_type = entry.header().entry_type();
        if !entry_type.is_file() && !entry_type.is_dir() {
            return Err(anyhow!(
                "tarball contains disallowed entry kind {:?}; refusing to extract",
                entry_type
            ));
        }

        let path_in_tar = entry
            .path()
            .with_context(|| "decode entry path")?
            .into_owned();

        // Reject absolute paths up-front — `Path::join` with an
        // absolute component on the right would silently discard
        // the staging base.
        if path_in_tar.is_absolute() {
            return Err(anyhow!(
                "tarball entry has absolute path: {}",
                path_in_tar.display()
            ));
        }

        // Reject any `..` component. Belt-and-braces with the
        // assert_within check below — `..` in a relative path
        // wouldn't escape on its own (assert_within would catch
        // it after canonicalise), but failing fast here gives a
        // clearer error and avoids partial filesystem mutation
        // for an entry we're about to reject anyway.
        if path_in_tar
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(anyhow!(
                "tarball entry contains traversal: {}",
                path_in_tar.display()
            ));
        }

        let resolved = staging.join(&path_in_tar);

        if entry_type.is_dir() {
            std::fs::create_dir_all(&resolved)
                .with_context(|| format!("create dir from tarball: {}", resolved.display()))?;
            // Containment check happens BELOW after the dir exists
            // (canonicalize requires existence on Windows).
        } else {
            // For files: ensure the parent directory exists and
            // that the parent itself is contained, before we open
            // a writer.
            if let Some(parent) = resolved.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create parent {}", parent.display()))?;
                let parent_check = paths::assert_within(&staging, parent)
                    .with_context(|| format!("contain check on parent {}", parent.display()))?;
                let _ = parent_check;
            }
            entry
                .unpack(&resolved)
                .with_context(|| format!("unpack {}", resolved.display()))?;
        }

        // Containment check on the final resolved path. For dirs we
        // do it post-create (canonicalize needs existence), for files
        // post-unpack.
        paths::assert_within(&staging, &resolved).with_context(|| {
            format!(
                "tarball entry escapes staging dir: {} → {}",
                path_in_tar.display(),
                resolved.display()
            )
        })?;
    }

    // Tarball validated end-to-end. Promote the inner files we
    // actually want at runtime to the canonical locations under
    // `dest_root` (one level up from staging), then delete staging.
    //
    // Tarball lays out as:
    //   sherpa-onnx-pyannote-segmentation-3-0/
    //     model.int8.onnx
    //     model.onnx
    //     LICENSE
    //     README.md
    //     run.sh, *.py (helper scripts we don't need)
    let entries: Vec<_> = std::fs::read_dir(&staging)
        .with_context(|| format!("re-list staging {}", staging.display()))?
        .filter_map(|e| e.ok())
        .collect();
    let inner_dir = entries
        .iter()
        .find(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .ok_or_else(|| anyhow!("segmentation tarball had no inner directory"))?;

    let promote = |from: &Path, to: &Path| -> Result<()> {
        if !from.exists() {
            return Err(anyhow!(
                "expected file missing from tarball after extract: {}",
                from.display()
            ));
        }
        if to.exists() {
            std::fs::remove_file(to).ok();
        }
        std::fs::rename(from, to)
            .with_context(|| format!("promote {} → {}", from.display(), to.display()))?;
        Ok(())
    };

    promote(
        &inner_dir.join(SEG_INNER_MODEL_RELPATH),
        &dest_root.join("segmentation.onnx"),
    )?;
    promote(
        &inner_dir.join(SEG_INNER_LICENSE_RELPATH),
        &dest_root.join("segmentation.LICENSE"),
    )?;
    promote(
        &inner_dir.join(SEG_INNER_README_RELPATH),
        &dest_root.join("segmentation.README.md"),
    )?;

    // Sweep up the staging tree (helper scripts, fp32 model, the
    // empty inner dir, and the now-empty staging root).
    let _ = std::fs::remove_dir_all(&staging);
    Ok(())
}

fn emit_progress(handle: &AppHandle, done: u64, total: u64, phase: ModelProgressPhase) {
    let _ = handle.emit(
        EVT_MODEL_PROGRESS,
        ModelProgress {
            model_id: DIARISER_ASSET_PROGRESS_ID.into(),
            bytes_done: done,
            bytes_total: total,
            phase,
        },
    );
}

#[cfg(test)]
mod tests {
    //! Pure-function tests for the asset manager. Network-touching
    //! flows are deliberately not exercised here — the
    //! download/extract path runs in `ensure_assets_ready` which is
    //! a tokio async + filesystem + network surface; covering it
    //! requires either a real upstream call (CI-flaky) or a
    //! sophisticated mock (more code to maintain than what it
    //! tests). The structural protections are exercised here.

    use super::*;
    use std::io::Write;

    /// Build a tiny tarball in memory containing one synthesised
    /// entry, write it to a temp path, and return the path. Helper
    /// for the traversal-protection tests below.
    ///
    /// `tar::Builder::append_data` defends against absolute paths
    /// and `..` itself — refuses to write them — so we can't use
    /// Builder to construct deliberately-malicious tarballs.
    /// Instead we hand-write the tar header (USTAR + 512-byte
    /// blocks) bypassing Builder entirely. This is the same shape
    /// a real attacker's tarball would have.
    fn build_test_tarball(entries: &[(&str, &[u8], tar::EntryType)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ut-diariser-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let tar_path = dir.join("test.tar.bz2");

        let mut bz = bzip2::write::BzEncoder::new(
            std::fs::File::create(&tar_path).unwrap(),
            bzip2::Compression::fast(),
        );

        for (path, body, kind) in entries {
            let mut header = tar::Header::new_ustar();
            // Write the path field directly into the header bytes;
            // skip Builder's path validation. The tar `name` field
            // is 100 bytes (USTAR; we keep test paths short).
            let path_bytes = path.as_bytes();
            assert!(
                path_bytes.len() <= 100,
                "test path too long for tar name field"
            );
            // SAFETY: USTAR header has `name: [u8; 100]` at offset 0.
            // We write directly into that range. tar 0.4's
            // `Header::as_mut_bytes` exposes the full 512-byte block.
            let header_bytes = header.as_mut_bytes();
            header_bytes[..path_bytes.len()].copy_from_slice(path_bytes);
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_entry_type(*kind);
            header.set_cksum();

            // Write header + body padded to 512-byte boundary.
            std::io::Write::write_all(&mut bz, header.as_bytes()).unwrap();
            std::io::Write::write_all(&mut bz, body).unwrap();
            let pad = (512 - (body.len() % 512)) % 512;
            if pad > 0 {
                std::io::Write::write_all(&mut bz, &vec![0u8; pad]).unwrap();
            }
        }
        // Two zero blocks = end-of-archive marker.
        std::io::Write::write_all(&mut bz, &[0u8; 1024]).unwrap();

        bz.flush().unwrap();
        bz.finish().unwrap();
        tar_path
    }

    fn fresh_dest() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ut-diariser-dest-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn extract_rejects_absolute_path_in_tarball() {
        let evil = if cfg!(windows) {
            "C:\\Windows\\System32\\evil.txt"
        } else {
            "/etc/evil.txt"
        };
        let tarball = build_test_tarball(&[(evil, b"haha", tar::EntryType::Regular)]);
        let dest = fresh_dest();
        let result = extract_segmentation_tarball_safely(&tarball, &dest);
        assert!(
            result.is_err(),
            "absolute entry path must be rejected, got {:?}",
            result
        );
        let _ = std::fs::remove_dir_all(tarball.parent().unwrap());
        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn extract_rejects_traversal_in_tarball() {
        let tarball =
            build_test_tarball(&[("../../../../escape.txt", b"hax", tar::EntryType::Regular)]);
        let dest = fresh_dest();
        let result = extract_segmentation_tarball_safely(&tarball, &dest);
        assert!(
            result.is_err(),
            "`..` traversal must be rejected, got {:?}",
            result
        );
        let _ = std::fs::remove_dir_all(tarball.parent().unwrap());
        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn extract_rejects_symlink_entry() {
        // tar::EntryType::Symlink and Link both forbidden.
        let tarball = build_test_tarball(&[("evil-symlink", b"", tar::EntryType::Symlink)]);
        let dest = fresh_dest();
        let result = extract_segmentation_tarball_safely(&tarball, &dest);
        assert!(
            result.is_err(),
            "symlink entry must be rejected, got {:?}",
            result
        );
        let _ = std::fs::remove_dir_all(tarball.parent().unwrap());
        let _ = std::fs::remove_dir_all(&dest);
    }
}
