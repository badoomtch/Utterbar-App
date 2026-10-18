# Utterbar Windows App

Local-only dictation for Windows 10 / 11. Press a hotkey, speak, text lands in whatever app has focus. ASR runs on-device via NVIDIA Parakeet (sherpa-onnx). No audio ever leaves the machine.

This directory holds the desktop client. The [project homepage](../README.md),
[user guide](../docs/GETTING_STARTED.md), downloads and support now live on
GitHub. The legacy website source is retained in the monorepo; it is not a
licence backend or a dependency of the desktop updater.

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
pnpm tauri:build:release
```

NSIS installer output is under `src-tauri/target/release/bundle/nsis/`. Publish
the installer through GitHub Releases; see [RELEASE.md](./RELEASE.md) for the
full flow.

Use the feature-safe release command above, not plain `tauri build`, which
omits experimental mic-mute support. For private support, email
[jay@jayparry.me](mailto:jay@jayparry.me).

## Stack

Tauri 2 shell · React 18 + Vite + TypeScript frontend · Rust backend · sherpa-onnx + ONNX Runtime for ASR · SQLite (bundled rusqlite + refinery migrations) · framer-motion for motion.

## Historical note

Earlier iterations of this project were named **Orliq**. The code still surfaces that name in a handful of places (notably `OrliqError` in `src-tauri/src/util/result.rs`) — these are intentionally left for backwards compatibility with on-disk state and error-code consumers. The product and repo are `Utterbar` everywhere else. See [../MONOREPO.md](../MONOREPO.md) for the full picture.
