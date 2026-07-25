# Utterbar

**Windows-first, local-only dictation. Press a hotkey, speak, and your words land in whatever app is focused.**

Utterbar is free and open source. Every feature is on for everyone — no account, no cloud, no subscription. Speech recognition runs entirely on your PC; your audio never leaves the machine.

> Download the installer for Windows 10 / 11 at **[utterbar.com/download](https://utterbar.com/download)**.

---

## What it does

- **Press. Speak. Done.** Hold a hotkey, talk, let go. The text appears in the focused window — any app, no copy-paste.
- **Local speech recognition.** English runs on NVIDIA Parakeet-TDT; 30+ other languages run on OpenAI Whisper. All on-device.
- **Modes.** Voice-to-text, message, and meeting presets, plus unlimited custom modes with their own formatting and post-processing.
- **Meeting transcription.** Two-channel, live transcription of a call using system-audio (WASAPI loopback) capture.
- **File transcription.** Drop a recording (or a whole folder) and get plain text plus `.srt` / `.vtt`.
- **Custom vocabulary and replacements**, full history with optional audio retention, themes and accents, and detailed privacy controls.
- **No telemetry.** The app makes only a handful of clearly-enumerated network calls (model download and update check); nothing about your speech is ever sent anywhere.

## Repository layout

This is a single repository containing two sub-projects:

| Directory | What it is | Stack |
| --- | --- | --- |
| [`Utterbar Windows App/`](Utterbar%20Windows%20App/) | The desktop dictation app | Tauri v2 + React + Rust |
| [`Utterbar Website/`](Utterbar%20Website/) | Marketing site + self-hosted release/update feed | Laravel 12 + Blade + Tailwind |

## Build the desktop app from source

**Prerequisites (Windows):**

- [Rust](https://rustup.rs/) (stable) with `cargo`
- [Node.js](https://nodejs.org/) with `pnpm` (`npm i -g pnpm`)
- Visual Studio Build Tools (the MSVC C++ toolchain)
- LLVM / libclang — `winget install LLVM.LLVM`, then set a user environment variable `LIBCLANG_PATH=C:\Program Files\LLVM\bin` (the sherpa-onnx bindgen step needs it)

sherpa-onnx and the Silero VAD model are vendored in the repo, so a clean checkout builds without any extra downloads.

**Commands (from `Utterbar Windows App/`):**

```bash
pnpm install
pnpm tauri:dev            # dev loop (Vite HMR + Tauri shell)
pnpm tauri build         # release bundle → src-tauri/target/release/bundle/
```

**Checks (CI parity):**

```bash
pnpm typecheck && pnpm lint && pnpm vitest run && pnpm build
cd src-tauri && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
```

## Build the website

**Prerequisites:** PHP 8.3+, Composer, Node.js.

**Commands (from `Utterbar Website/`):**

```bash
composer install
php artisan migrate
npm run build
php artisan serve
```

```bash
php artisan test         # test suite
```

## Contributing

Issues and pull requests are welcome. Please run the checks above before opening a PR, and keep to the existing conventions (British English in user-facing copy; the code follows the surrounding style).

## Licence

Utterbar is released under the [MIT Licence](LICENSE).

It bundles and downloads third-party components under their own licences — most notably the **NVIDIA Parakeet / NeMo speech models, which are CC-BY-4.0 and require attribution**. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for the full list and the attributions you must preserve if you redistribute Utterbar.
