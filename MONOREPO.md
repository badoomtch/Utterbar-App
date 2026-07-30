# Utterbar Monorepo

## Projects

| Directory | Purpose | Stack |
| --- | --- | --- |
| `Utterbar Windows App/` | Windows desktop dictation application | Tauri v2, React, Rust |
| `Utterbar Website/` | Public product website and contact form | Laravel, Blade, Alpine, Tailwind |

## Desktop architecture

The desktop application captures microphone audio, runs local speech
recognition, and pastes the result into the previously active target window.
Speech models and inference stay on the device. The app does not require an
account, payment service, licence server, or telemetry service.

The Rust backend lives in `src-tauri/src/`. The React interface lives in `src/`.
Settings are persisted locally and exposed to the interface through Tauri
commands. Keep app interface language separate from dictation language.

## Updates and releases

GitHub Releases is the release source of truth.

- Stable channel: the latest ordinary published GitHub Release.
- Beta channel: the highest published release, including GitHub prereleases.
- The updater accepts only the expected NSIS asset name, GitHub SHA-256 digest,
  and matching downloaded byte count before installing.
- The public website's download and changelog routes redirect to GitHub.

Build and release instructions are in `RELEASE_RUNBOOK.md`.

## Website architecture

The Laravel site is a marketing and documentation surface. It has no updater
API, release admin API, payment flow, account system, licence system, or
subscriber database. Website deployment is separate from desktop releases.

## Security boundaries

- Runtime secrets belong in environment variables or operator-local credential
  stores, never tracked files.
- Do not place private keys, API tokens, passwords, or cloud access keys in
  source, documentation, tests, release notes, or commit messages.
- Do not deploy the website or publish a GitHub Release without explicit
  approval.
