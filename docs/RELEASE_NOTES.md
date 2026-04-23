# Utterbar — Release notes

## 0.3.0 — Phase 5 implementations

Phase-3 filled in the Windows-native wiring behind the scaffold; Phase-5 closes out the remaining v1 behaviours so the only deferred work is the real Parakeet ONNX inference and the llama.cpp LLM provider (both feature-gated).

**New in 0.3.0**

- **File transcription is real**: Symphonia-backed decode across MP3, AAC, MP4/MOV/MKV, OGG, FLAC, WAV and PCM; mono downmix; 16 kHz resample; fed directly into the active ASR provider through a single-flight worker that survives process restart. Results land as a `.utterbar.txt` sidecar next to the source.
- **UIAutomation selection capture**: the context service now reads the currently focused element's selected text via `IUIAutomationTextPattern`. Gated by per-mode and global privacy toggles; capped at 4k chars.
- **Silence trim + normalisation**: optional pre-ASR conditioning wired into the dictation pipeline, driven by Settings → Audio.
- **DB snapshot on startup**: the SQLite file is backed up to `orliq.db.backup-<ts>` before migrations run; the last three snapshots are kept; `PRAGMA integrity_check` runs after migrations.
- **Streak detection**: Dashboard's "streak days" is now a real walk over `analytics_daily` — yesterday counts so midnight doesn't reset it.
- **Full mode editor**: name, icon, ASR model, target language, post-processing (None / Deterministic / LLM with prompt), context rules, formatting, and auto-activation rules — all round-tripped through `upsert_mode`.
- **Dynamic tray submenu**: the tray's "Switch Mode" list is rebuilt on startup and whenever modes are added, deleted, or duplicated. Clicking a mode entry sets it active immediately.
- **Installer + manifest polish**: Windows app manifest with per-monitor-v2 DPI awareness, UTF-8 active code page, long-path support and common-controls v6. `LICENSE.txt` at the repo root. Bundle config now lists resources, publisher, homepage, and an installer icon.
- **Release dry-run CI**: unsigned MSI + NSIS build on every push to `release/*`, with artifact upload, catches packaging regressions before a real signed release.

**Deferred to post-v1**

- Real ONNX Parakeet inference (`--features onnx-asr`) — scaffold compiles; the mel-spectrogram + decoder graph is the next big piece.
- `llama.cpp` LLM post-processor (`--features llama`) — deterministic fallback remains the default.
- Mouse-button hotkeys (H-003).
- Signed release pipeline (PK-003).

## 0.2.0 — Phase 3 implementations

Real Windows text injection, global hotkey registration, Windows clipboard + context capture, streaming model downloads with sha256 verification, reprocess-from-history, updater command + toast, diagnostics panel, sound cues. See previous entry for detail.

## 0.1.0 — scaffold

Initial Tauri v2 + React + Rust scaffold with PRD, architecture, schema, and implementation plan in `docs/`.
