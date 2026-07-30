# Utterbar Engineering Rules

## Product constraints

- Utterbar is free, open source, and MIT-licensed.
- Do not add payments, accounts, licences, telemetry, or cloud transcription.
- Use British English and write `Utterbar` as one word.
- Keep app interface language separate from dictation language.

## Engineering workflow

- Read `AGENTS.md`, `MONOREPO.md`, and `TODO.md` before broad changes.
- Prefer small, focused changes with tests that cover the changed behaviour.
- Do not revert unrelated working-tree changes.
- Do not commit, push, tag, publish a GitHub Release, deploy the website, or
  rewrite remote history without Jay's explicit approval.
- Never use `git reset --hard`, `git commit --amend`, or force-push without
  explicit approval.

## Security and releases

- Never read, print, commit, or generate secret values. Keep configuration in
  environment variables or operator-local credential stores.
- The desktop updater uses GitHub Releases directly. Stable releases are normal
  published releases; beta releases are GitHub prereleases. The exact installer
  asset name is `Utterbar_<version>_x64-setup.exe`.
- Before a desktop release, build with `pnpm tauri:build:release` and verify the
  GitHub asset digest and installer smoke test. See `RELEASE_RUNBOOK.md`.
- Website deployment is separate from app release publication. Do not deploy it
  merely because a desktop build changed.

## Testing

- Desktop Rust: run `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, and `cargo test` from `Utterbar Windows App/src-tauri`.
- Desktop frontend: run `pnpm typecheck`, `pnpm lint`, and `pnpm vitest run`
  from `Utterbar Windows App`.
- Website: run `php artisan test` from `Utterbar Website` when website code
  changes.
