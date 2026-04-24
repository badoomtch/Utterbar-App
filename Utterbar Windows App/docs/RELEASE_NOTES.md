# Utterbar — Release notes

## 0.5.0 — Licence system

The paid tier is live end-to-end. Utterbar now has a working licence system on both sides of the wire: the app speaks the four `/api/licence/*` endpoints on utterbar.com, verifies signed blobs offline on every launch, and surfaces a proper Settings → Licence panel. Nothing is gated yet — beta testers see the full feature set, but the tier is visible so activation can be validated in the wild before v1 flips the gates.

**New in 0.5.0**

- **Offline signature verification.** Ed25519 over sha256(payload) against a bundled sandbox public key. Every launch re-verifies the cached blob — no signature, no Pro. Fail-closed; absence of network is never treated as revocation.
- **Windows device fingerprint.** Deterministic `utterbar-fp-v1:` prefix + `MachineGuid` (registry, KEY_WOW64_64KEY view) + lowercased hostname + first non-virtual MAC. Computed once at startup; never surfaced in the UI.
- **Keyring-backed cache.** Windows Credential Manager on `Utterbar/Licence`, DPAPI at rest. 30-day heartbeat window; only an explicit `410 revoked` from the server clears it.
- **Four endpoints wired up.** `/fetch`, `/activate`, `/deactivate`, `/heartbeat` via reqwest (rustls, HTTP/2, 15s/5s timeouts). User-Agent `Utterbar/{version} (windows; x86_64)` so server logs correlate by build. Server-slug → UI-code mapping lives in one place.
- **Deep-link activation.** `utterbar://licence/{TOKEN}` wired through the Tauri deep-link plugin — the purchase email's "activate" link now hands the app a key directly.
- **Settings → Licence.** Paste UB-XXXX-… on the Free screen to activate; Pro surface shows the licence ID, SKU, copyable display key, this-device metadata, and a read-only list of other devices (with a link to manage them on utterbar.com/account). Inline errors per server slug — `malformed`, `not_found`, `device_limit`, `network`, `verification_failed`, and a rose banner for `revoked`.
- **Sidebar tier indicator.** Muted "Upgrade to Pro →" link when Free; cyan LED + "Pro" link to the panel when Pro. Visible on every page.

**v1 posture**

Feature gates are deliberately off. The tier is exposed so the licence round-trip can be exercised by real users without also shipping a buggier Pro-only codepath to a smaller audience. Gates flip on at public launch — the list is drafted but not wired.

**Deferred to post-v1**

- Real ONNX Parakeet inference (`--features onnx-asr`).
- `llama.cpp` LLM post-processor (`--features llama`).
- Mouse-button hotkeys (H-003).
- Signed release pipeline (PK-003).
- Public ed25519 key rotation (v1 → v2) once the live-launch keypair is minted.

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
