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
5. **LLVM / libclang** for the speech-runtime bindings. Install LLVM and set
   `LIBCLANG_PATH=C:\Program Files\LLVM\bin` (adjust for your installation).

## Clone & install

```bash
git clone <your-fork> Utterbar
cd Utterbar/Utterbar\ Windows\ App
pnpm install
```

The desktop app lives under `Utterbar Windows App/`. The public project homepage
is the [root README](../../README.md); the website directory is legacy source,
not a licence backend. All commands below run from `Utterbar Windows App/`.

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
| Production bundle including experimental mic mute | `pnpm tauri:build:release` |

## Feature flags

`experimental_mic_mute` includes the Teams/Discord mic-mute path. The canonical
release script enables it. For a matching development run, use
`pnpm tauri:dev --features experimental_mic_mute`.

The optional `llama` feature remains in the source tree; it is not part of the
supported release workflow and does not change the product's no-LLM-rewrite
policy. Do not enable optional backends just to build the ordinary app.

## Data paths

Everything Utterbar writes is under `%APPDATA%\Utterbar\`:

```
%APPDATA%\Utterbar\
├── config\settings.json
├── config\floating_position.json   (last known floating-bar position)
├── data\utterbar.db                (SQLite; legacy installs may still use orliq.db)
├── models\<model-id>\...
├── recordings\YYYY\MM\...          (opt-in)
└── logs\utterbar.log.<date>
```

Use disposable test data for development. Back up anything important before
resetting this directory; it can contain private history and recordings.

## Directory map

See [MONOREPO.md](../../MONOREPO.md) and [Contributing](../../CONTRIBUTING.md)
for architecture and current workflow guidance.

## Troubleshooting

| Symptom | Likely cause | Fix |
|--------|-------------|-----|
| `link.exe not found` | MSVC build tools missing | Install VS 2022 Build Tools |
| `webview2` errors on first launch | Runtime missing | Install Microsoft's WebView2 Runtime; do not routinely run the app as administrator |
| Floating bar doesn't show | Another window is on top of `always-on-top` | Right-click tray → Open Utterbar; the bar is a separate window from the main shell |
| Hotkeys not firing | Another app stole the accelerator | Check Settings → Shortcuts for the conflict indicator |
