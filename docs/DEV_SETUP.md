# Utterbar — Dev setup

One-time setup for contributors on Windows 11.

## Prerequisites

1. **Node.js 20+** and **pnpm 9+**
   ```powershell
   winget install OpenJS.NodeJS.LTS
   npm install -g pnpm
   ```
2. **Rust stable** (via rustup)
   ```powershell
   winget install Rustlang.Rustup
   rustup default stable
   ```
3. **Visual Studio 2022 Build Tools** with the *Desktop development with C++* workload. Required by the Windows crate and `ring` (rustls).
4. **WebView2 Runtime** — already present on Windows 11 22H2+; installed on demand by Tauri otherwise.
5. *(Optional)* **Python 3.10+** if you want to exercise the `py_sidecar` ASR provider.

## Clone & install

```bash
git clone <your-fork> Orliq
cd Orliq
pnpm install
```

## Run in dev mode

```bash
pnpm tauri:dev
```

Tauri launches three windows: the floating bar (top of screen), the main window (hidden until tray-click or first-run onboarding), and the onboarding window on first run. The frontend hot-reloads on save; Rust reloads on full rebuild.

## Useful commands

| Task | Command |
|------|--------|
| Type-check frontend | `pnpm typecheck` |
| Lint frontend | `pnpm lint` |
| Rust fmt | `cargo fmt --manifest-path src-tauri/Cargo.toml` |
| Rust clippy | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` |
| Rust tests (also regenerates TS types via `ts-rs`) | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Production bundle | `pnpm tauri:build` |

## Feature flags

The Rust crate gates heavyweight backends so the default build stays fast:

- `onnx-asr` — enables the `ort`-backed Parakeet provider.
- `llama` — enables the `llama-cpp-2`-backed LLM provider.

```bash
cargo build --manifest-path src-tauri/Cargo.toml --features onnx-asr,llama
```

## Data paths

Everything Utterbar writes is under `%APPDATA%\Utterbar\`:

```
%APPDATA%\Utterbar\
├── config\settings.json
├── data\orliq.db          (SQLite)
├── models\<model-id>\...
├── recordings\YYYY\MM\... (opt-in)
└── logs\utterbar.log
```

Delete that directory to fully reset the app for a new dev session.

## Directory map

See [ARCHITECTURE.md](ARCHITECTURE.md#3-rust-module-map) for the full module layout.

## Troubleshooting

| Symptom | Likely cause | Fix |
|--------|-------------|-----|
| `link.exe not found` | MSVC build tools missing | Install VS 2022 Build Tools |
| `webview2` errors on first launch | Runtime not bootstrapped | `pnpm tauri:dev` once at admin prompt, or install WebView2 runtime manually |
| Floating bar doesn't show | Another window is on top of `always-on-top` | Right-click tray → Open Utterbar; the bar is a separate window from the main shell |
| Hotkeys not firing | Another app stole the accelerator | Check Settings → Shortcuts for the conflict indicator |
