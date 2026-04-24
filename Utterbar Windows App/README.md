# Utterbar Windows App

Local-only dictation for Windows 10 / 11. Press a hotkey, speak, text lands in whatever app has focus. ASR runs on-device via NVIDIA Parakeet (sherpa-onnx). No audio ever leaves the machine.

This directory holds the desktop client. The marketing website and licence backend live at [../Utterbar Website/](../Utterbar%20Website/). Both are part of a single monorepo rooted at the parent directory.

## Getting started

```powershell
pnpm install
pnpm tauri:dev
```

Full dev-machine setup: [./docs/DEV_SETUP.md](./docs/DEV_SETUP.md).

## Where to look for what

| You want | Read |
|---|---|
| Repo-wide rules for working here | [../CLAUDE.md](../CLAUDE.md) |
| Architecture + cross-repo contracts | [../MONOREPO.md](../MONOREPO.md) |
| What's done, in flight, or open | [../TODO.md](../TODO.md) |
| How a release actually ships | [../RELEASE_RUNBOOK.md](../RELEASE_RUNBOOK.md) + [./RELEASE.md](./RELEASE.md) |
| Published release notes | [./releases/](./releases/) |

## Build + packaging

```powershell
pnpm tauri build
```

NSIS installer output under `src-tauri/target/release/bundle/nsis/`. Publishing + R2 upload are handled by `scripts/publish-release.sh` — see [RELEASE.md](./RELEASE.md) for the full flow.

## Stack

Tauri 2 shell · React 18 + Vite + TypeScript frontend · Rust backend · sherpa-onnx + ONNX Runtime for ASR · SQLite (bundled rusqlite + refinery migrations) · framer-motion for motion.

## Historical note

Earlier iterations of this project were named **Orliq**. The code still surfaces that name in a handful of places (notably `OrliqError` in `src-tauri/src/util/result.rs`) — these are intentionally left for backwards compatibility with on-disk state and error-code consumers. The product and repo are `Utterbar` everywhere else. See [../MONOREPO.md](../MONOREPO.md) for the full picture.
