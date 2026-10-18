# Contributing to Utterbar

Bug reports, focused fixes, documentation and translation corrections are
welcome. Open an issue before substantial feature or architecture changes.

## Development Setup

The supported installer requires Windows x64. Install Node.js LTS, pnpm 9,
Rust stable with the MSVC target, Visual Studio 2022 Build Tools with Desktop
development with C++, and WebView2. Install LLVM/libclang and set
`LIBCLANG_PATH` to its `bin` directory, normally `C:\Program Files\LLVM\bin`.

Native speech-runtime libraries and the Silero model are vendored. Package
installation and Rust builds still download dependencies; installing speech
models is separate. Never commit credentials, recordings or generated installers.

```powershell
git clone https://github.com/badoomtch/Utterbar-App.git
cd "Utterbar-App/Utterbar Windows App"
pnpm install --frozen-lockfile
pnpm tauri:dev
```

To exercise experimental mic mute in development:

```powershell
pnpm tauri:dev --features experimental_mic_mute
```

For the full release feature set:

```powershell
pnpm tauri:build:release
```

NSIS output is under `src-tauri/target/release/bundle/nsis/`. Plain `tauri build`
omits experimental mic mute. See [the release runbook](RELEASE_RUNBOOK.md).

## Checks

From `Utterbar Windows App`:

```powershell
pnpm typecheck
pnpm lint
pnpm vitest run
pnpm build
```

From `Utterbar Windows App/src-tauri`:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Keep changes focused, add regression coverage, and report the checks and manual
tests actually run. Do not commit unrelated generated-type line-ending churn.

## Conventions

- British English in copy; `Utterbar` as one word.
- Free, open source and local-first. No paid tiers, accounts, telemetry or
  cloud transcription.
- App language and dictation language remain independent.
- Keep audio, transcripts and clipboard contents private, including in logs.
- Match existing patterns and preserve third-party notices and attributions.

Translations live in `Utterbar Windows App/src/i18n/locales/`. Preserve keys
and `{{placeholders}}`. Include the screen/context for wording corrections.
Do not translate model names, paths or settings identifiers.

## Help

Use [Issues](https://github.com/badoomtch/Utterbar-App/issues/new/choose) or
email **[jay@jayparry.me](mailto:jay@jayparry.me)** for private support.
Follow [SECURITY.md](SECURITY.md) for vulnerabilities.
