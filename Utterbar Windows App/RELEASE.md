# Desktop Release Guide

Use the repository-level [release runbook](../RELEASE_RUNBOOK.md). Releases
are published as GitHub Releases, not through the website or an object-storage
bucket.

From this directory, the normal build command is:

```sh
pnpm tauri:build:release
```

It produces the Windows installer at:

```text
src-tauri/target/release/bundle/nsis/Utterbar_<version>_x64-setup.exe
```

Upload that exact file to the GitHub Release for tag `v<version>`. Mark beta
builds as prereleases and stable builds as ordinary releases.
