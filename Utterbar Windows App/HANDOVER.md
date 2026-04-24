# HANDOVER.md — recent session log (2026-04-23)

Raw-ish record of the last stretch of work, captured so it can be
merged with the web-side Claude's equivalent when the two repos get
unified. Filed here, not in CLAUDE.md, because this is session context
that becomes stale; CLAUDE.md is the long-lived doc.

Ordered oldest → newest. Each item is "what the user asked" → "what
was done" → "what it left behind."

---

## 1. Remotion marketing bundle: parity-driven refactor

**User (Jay, via website-Claude):** "They basically all don't look
like what the app looks like. Make sure it looks exactly like the
actual interface please."

**Done:**
- Replaced every brand port under `marketing/remotion/src/bar/` with a
  `BarDriver.tsx` wrapper that seeds the real zustand stores and
  renders `src/routes/FloatingBar.tsx` directly.
- Wired `@` path alias + Tauri API stubs in
  `marketing/remotion/webpack-override.mjs` so the real tree resolves.
- Debugged + fixed the Tailwind pipeline (the long story: `@remotion/
  bundler/css-loader` hardcodes `-!` pre-requests that disable
  postcss on CSS `@import`s; moved `globals.css` to a top-level JS
  import in `Root.tsx` to dodge it; hand-rolled the CSS loader rule
  to put postcss-loader ahead of css-loader explicitly).
- Rendered 6-state parity stills (idle/warming/listening/processing/
  success/error) into `marketing/remotion/out/parity/` as the
  sign-off gate. Rendered all 12 MP4s + 12 posters + 5 press stills.

**Left behind:**
- 400 px mobile scale pass on compositions not yet verified.
- Sign-off from Jay + website-Claude on the parity stills not yet
  logged back in this repo.

## 2. v0.6.1 release

**User:** Full verbatim release flow per `RELEASE.md`: bump versions,
build, write notes, run `publish-release.sh`, push main + tag.

**Done:**
- Committed the accumulated v0.6.1 app work (35 files, `c764b05`):
  in-app updater toast, Pro upgrade modal + `useProPrice` hook,
  custom accent colour (Pro), compact-compact bar, settings nav
  scroll, release-asset versioned filenames plumbing, RELEASE.md
  expansion.
- Committed the Remotion refactor (74 files, `c394161`): bundle
  plumbing + all rendered artefacts under `out/`.
- CI round 1 failed: cargo fmt. Ran `cargo fmt`, committed
  (`3914e9f`).
- CI round 2 failed: clippy `-D warnings` on an unused
  `tauri::Manager` import in `updater.rs`. Removed, committed
  (`76d200c`). CI green after that.
- `pnpm tauri build` produced
  `src-tauri/target/release/bundle/nsis/Utterbar_0.6.1_x64-setup.exe`
  (10.5 MiB, SHA
  `73628fb73b53b92474f24cc3fbc5ab2073e70899f7a06e8be0bb299f93c7820b`).
- Tagged `v0.6.1` at `76d200c`, pushed tag.
- Preflight discovered `aws` and `jq` were missing from the machine
  (fresh install path). Installed both via `winget install
  Amazon.AWSCLI jqlang.jq`.
- Seeded `~/.aws/config` + `~/.aws/credentials` for the
  `utterbar-r2` profile from `~/.utterbar-deploy/.env`.
- `publish-release.sh` uploaded to
  `s3://utterbar/releases/0.6.1/Utterbar-Setup-0.6.1.exe`; CDN
  returned 200. Admin API POST 422'd with "version required" — a
  curl+bash quoting issue with `-d "$PAYLOAD"` on Windows git-bash.
  Re-sent the body with `--data-binary "@$TMPFILE"`.
- Second retry 422'd with `file_not_in_bucket` — the admin API
  expected the legacy key `releases/0.6.1/Utterbar-Setup.exe`
  despite the release notes claiming versioned filenames. Uploaded
  the installer to BOTH keys.
- Third retry returned **HTTP 201** (`id: 2`, download_url:
  `https://dl.utterbar.com/releases/0.6.1/Utterbar-Setup.exe`, size
  10997755, announced_to 0). v0.6.1 is live on the download page.

**Left behind (now tracked in CLAUDE.md §12):**
- `publish-release.sh` still uses `-d "$PAYLOAD"`. Patch to
  `--data-binary @file` next time we ship.
- Admin API `file_not_in_bucket` check hardcoded to the legacy key
  → we're double-uploading. Web side needs to accept the versioned
  name.
- The in-app updater endpoint `/api/updater/check` returns 404 on
  `utterbar.com`. Also tried `/api/updates/check`,
  `/api/update/check`, `/api/version/check` — all 404. Users on
  0.6.0 will not auto-upgrade to 0.6.1 until the web side exposes
  this route. Critical for the updater feature to actually land.

## 3. 4.9 GB idle memory (user flagged mid-session)

Screenshot showed Task Manager with two Utterbar processes totalling
4888 MB committed, 3.9% CPU, idle. Previous baseline was much lower.

Not investigated this session. Hypotheses and first diagnostic step
recorded in CLAUDE.md §12a. Top suspect: a `llama.cpp` context
loaded globally at boot (size matches a 7B quant). Second suspect:
sherpa-onnx / Parakeet session allocated non-lazily.

## 4. History page redesign (user flagged at end)

Screenshot of `src/routes/HistoryRoute.tsx` showed the 2-column
layout collapsing at the current window width: list column squeezed
to 6-char-wide vertical words, detail pane showing "Nothing
selected" at near-half the viewport.

User: two columns don't scale; should be a single-column list with
click-to-expand detail (modal or slide-over).

I overclaimed in the v0.6.1 release notes — bullet said "responsive
padding + smarter empty state + detail pane no longer eats a second
viewport on narrow windows." That's cosmetic; the structural
problem is untouched. Filed as CLAUDE.md §12g with the correct
design direction. Not retro-patchable into a minor — needs a real
redesign.

## 5. Monorepo merge intent

User plans to merge this repo with the `utterbar-web` repo. Each
Claude (this one for desktop, one for web) writes its own rules doc;
the two get combined.

- This repo's rules: `CLAUDE.md` at root (written this session).
- Boundaries documented in CLAUDE.md §14 — this repo owns the
  desktop runtime + installer + Remotion source + release notes; the
  web repo owns pricing / licence issuance / release registry / the
  auto-update API schema / the public marketing site.

**Known contract drift (discovered this session) that the combined
doc needs to resolve:**
- Versioned installer filename: client uploads `Utterbar-Setup-
  <v>.exe`, server expects `Utterbar-Setup.exe`.
- `/api/updater/check` advertised in-app, not served on web.
- `publish-release.sh` is in this repo but tests a contract the web
  repo owns; integration tests should live in the monorepo.

---

## Quick facts to carry over

- **Current shipped version:** 0.6.1 (stable, tag=latest, as of
  this session).
- **CDN URL pattern:** `https://dl.utterbar.com/releases/<v>/
  Utterbar-Setup.exe` (legacy) + `…/Utterbar-Setup-<v>.exe` (new,
  not yet consumed by admin API).
- **R2 bucket:** `utterbar`. AWS profile name: `utterbar-r2`.
- **Admin API base:** `https://utterbar.com` + token from
  `~/.utterbar-deploy/.env`.
- **Latest commits on main:** `c764b05` (v0.6.1 app) → `c394161`
  (marketing) → `3914e9f` (fmt) → `76d200c` (clippy) → plus
  uncommitted `CLAUDE.md` + this file.

## Follow-ups I would queue if I were the next Claude in line

1. Investigate the 4.9 GB memory usage (start with §12a
   hypothesis 4, the llama.cpp context).
2. Redesign the History route as one column + detail modal.
3. Patch `publish-release.sh` curl invocation to `--data-binary
   @file`.
4. Coordinate with web-Claude on the `/api/updater/check` route so
   the in-app updater feature actually works for 0.6.0 users.
5. Hand the 6 parity stills to Jay for visual sign-off; after
   sign-off, the Remotion refactor is done.
