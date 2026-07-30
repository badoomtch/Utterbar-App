# Release Runbook

Utterbar publishes installers through GitHub Releases. GitHub is the only
release metadata and download source; the website does not host updater files.

## Before release

1. Confirm the working tree and intended branch are clean.
2. Bump the version with `node scripts/bump-version.mjs <version>` from
   `Utterbar Windows App`, then run `cargo update -p utterbar --precise <version>`
   from `src-tauri`.
3. Add user-facing notes at `Utterbar Windows App/releases/<version>.md`.
4. Run the desktop test suite and frontend checks.
5. Build with `pnpm tauri:build:release` from `Utterbar Windows App`.
6. Confirm both checks pass:
   ```sh
   grep -ac "send_hotkey_globally" "src-tauri/target/release/utterbar.exe"
   Test-Path "src-tauri/target/release/silero_vad.onnx"
   ```

The first command must return at least `1`. The installer asset must be named
`Utterbar_<version>_x64-setup.exe` because the in-app updater requires that
exact name.

## Publish on GitHub

Create a GitHub Release from the tag `v<version>` and upload the NSIS installer
from `src-tauri/target/release/bundle/nsis/`.

- A **stable** release is an ordinary published GitHub Release.
- A **beta** release is a published GitHub Release marked **Set as a
  pre-release**.
- Do not use drafts for updates: the desktop client deliberately ignores them.
- Include the markdown from `releases/<version>.md` as the release body.

The desktop app checks GitHub directly. Stable-channel users receive the latest
ordinary release. Beta-channel users receive the highest published version,
including prereleases. The client refuses assets without a GitHub SHA-256 digest
or with a different byte size after download.

## Verify

Before telling users the release is available:

1. Confirm the GitHub Release is published with the correct tag, prerelease
   status, notes, and installer asset.
2. Confirm the asset shows a SHA-256 digest in the GitHub API response.
3. Install the build on Windows and smoke-test the affected workflow.
4. Confirm the app offers the release on the intended update channel.

Creating a public release, pushing tags, or publishing installers is
outward-facing. Obtain Jay's explicit approval immediately before doing it.
