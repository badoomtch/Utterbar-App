# Utterbar — Implementation Plan

Five phases. Each has a clear exit bar. Do not move on until the exit bar is met.

---

## Phase 1 — Planning & scaffold spine *(week 0)*

**Exit bar:** repo compiles, empty app launches, docs are in place.

- [x] PRD, Architecture, Schema, Screen Map, Component Inventory
- [x] Tauri v2 scaffold (`pnpm create tauri-app` shape, adapted)
- [x] Frontend: Vite + React + TS + Tailwind + shadcn baseline
- [x] Rust: workspace, tracing, tokio, sqlite deps wired
- [x] CI workflow (lint, build on PR)
- [x] Design tokens + motion presets + Framer wiring
- [x] IPC type-generation via `ts-rs`

## Phase 2 — Core shell & first dictation *(weeks 1–3)*

**Exit bar:** user can launch Utterbar, go through onboarding, bind a hotkey, press it, speak, and get text inserted into Notepad using a real Parakeet model.

1. **Windowing**
   - Floating bar window: decorations-off, always-on-top, skip-taskbar, drag region.
   - Main window scaffold with routed tabs.
   - Onboarding window scaffold.
   - Single-instance plugin wired.

2. **Tray & startup**
   - `tauri-plugin-tray` icon + menu (static first, dynamic mode list later).
   - Launch-on-login registration.
   - Close-to-tray behaviour.

3. **Settings store**
   - JSON file store with versioned schema and migration runner.
   - `useSettings` hook + `settings:changed` event.

4. **Audio capture**
   - `cpal` enumerate + select device.
   - 16 kHz mono Int16 ring buffer.
   - RMS meter stream at 30 Hz.
   - Mic test harness.

5. **Model Manager**
   - SQLite `models` table + registry service.
   - Model directory layout + integrity check (sha256).
   - Download client (reqwest) with resumable HTTP range requests.
   - Model metadata JSON schema + built-in catalogue.

6. **ASR provider interface**
   - `AsrProvider` trait.
   - `onnx_parakeet` implementation using `ort`:
     - Load encoder/decoder ONNX graphs.
     - Preprocessing (log-mel, chunking).
     - CTC / RNN-T decoding depending on model export.
     - EP auto-detection.
   - `py_sidecar` fallback:
     - Bundled `scripts/asr_sidecar.py` (NeMo) with length-prefixed stdio JSON.
     - Managed lifecycle + auto-restart.

7. **Hotkeys**
   - `global-hotkey` integration, bind/unbind on settings change.
   - Conflict detection.

8. **First dictation pipeline**
   - Hotkey → start audio → stop audio → ASR → deterministic cleanup → clipboard paste (no LLM yet).
   - Floating bar reflects phase live.

9. **Onboarding flow**
   - All 12 steps wired.
   - Model download runs in background while later steps proceed where possible.

## Phase 3 — Feature breadth *(weeks 4–6)*

**Exit bar:** every checkbox in the PRD's v1 scope is functional, even if rough.

- Modes: CRUD, auto-activation matcher, per-mode shortcuts.
- Context service: active app, selection (UIA + fallback), clipboard.
- Vocabulary + replacements: apply between ASR and LLM.
- LLM provider: `llama_cpp` + deterministic fallback.
- Text output: paste + keystroke fallback + clipboard restore.
- History: write/read/reprocess/export.
- Analytics: daily rollups + dashboard charts.
- File transcription queue: ffmpeg-decode via `symphonia` (or bundled ffmpeg), feeds ASR.
- Updater plumbing.
- Sound cues.

## Phase 4 — Polish *(weeks 7–8)*

**Exit bar:** all §9 performance targets met; visual QA pass complete; no P0/P1 bugs.

- Motion refinement pass (verify all Framer springs feel right).
- Empty / loading / error states audited surface-by-surface.
- Diagnostics panel + log viewer.
- Accessibility sweep (keyboard, aria-live, contrast, reduce-motion).
- High-DPI and multi-monitor verification.
- Installer signing dry run.
- Release notes template.

## Phase 5 — Beta & ship *(weeks 9–10)*

- Internal dogfood.
- Private beta (invitation).
- Crash reports triaged; P0s closed.
- v1.0.0 cut.

---

## Task backlog (rolling)

Tracked in `docs/BACKLOG.md` — one-liner per task with a letter-tagged ID (A = audio, I = inference, H = hotkeys, U = UI, etc.) so bug reports can reference stable IDs.
