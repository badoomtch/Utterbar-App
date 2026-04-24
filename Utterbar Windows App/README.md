# ORLIQ

A premium, **local-only** dictation app for Windows 11.

ORLIQ turns speech into high-quality text without ever sending your voice or text to the cloud. It lives in a movable floating bar at the top of your screen, integrates deeply with Windows, and uses NVIDIA Parakeet models running entirely on-device.

> **Status:** v0.1 scaffold. Phase 1 planning complete; Phase 2 implementation in progress.

## Pillars

1. **Entirely local.** No cloud transcription. No cloud post-processing. No telemetry by default.
2. **Premium and native.** Dark-first UI, frosted surfaces, tasteful motion, real Windows integration (tray, global hotkeys, DPI, multi-monitor).
3. **Workflow-deep.** Modes, vocabulary, text replacements, per-app auto-activation, clipboard/selection context, history with reprocessing, file transcription, local stats.
4. **Honest.** Language support is driven by the actually-installed model. "Minutes saved" is grounded in a real typing-speed baseline. Nothing is faked.

## Tech stack

- **Tauri v2** desktop shell
- **Rust** backend for native integrations
- **React + TypeScript + Vite** frontend
- **Tailwind + shadcn/ui** for settings and panels
- **Framer Motion** for transitions
- **SQLite** (via `rusqlite` + `tauri-plugin-sql`) for local metadata, history, analytics
- **ASR:** NVIDIA Parakeet (English + multilingual) via a pluggable provider interface. ONNX Runtime is the default backend; a Python sidecar provider is available as a fallback.
- **Local LLM post-processing:** optional. Clean deterministic fallbacks when absent.

## Repo layout

```
Orliq/
├── docs/                 # PRD, architecture, schema, plans
├── src/                  # React frontend (TypeScript)
├── src-tauri/            # Rust backend + Tauri config
├── resources/            # Icons, sounds, installer assets
├── scripts/              # Build, package, model-fetch helpers
└── .github/              # CI workflows
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full module map.

## Getting started (dev)

> Requires Windows 11, Node 20+, Rust stable, and the Tauri prerequisites (WebView2, MSVC build tools).

```bash
pnpm install
pnpm tauri dev
```

## Privacy

- No network requests are made for transcription or text processing.
- All recordings, transcripts, vocabulary, and analytics are stored under `%APPDATA%/ORLIQ/`.
- You can inspect, export, or wipe any data from **Settings → Privacy**.

## License

Proprietary. Original work — not affiliated with or derived from any other dictation product.
