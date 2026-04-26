# RELEASE_RUNBOOK.md — the Utterbar release ritual

> Single ritual for shipping a new Utterbar version. Claude follows this top
> to bottom **every time** — no steps skipped, no order swapped, no ad-hoc
> "let's just do this one differently." If something in the runbook is wrong
> or out of date, FIX THE RUNBOOK in the same PR as the release so the next
> version inherits the fix.

**Pair file:** [CLAUDE.md](CLAUDE.md) (rules) + [MONOREPO.md](MONOREPO.md)
(reference). This runbook wins over both for release workflow.

**Authoritative release docs** linked from here:
- [Utterbar Windows App/RELEASE.md](Utterbar%20Windows%20App/RELEASE.md) — the existing canonical release flow. This runbook enforces it; it doesn't replace it.
- [Utterbar Windows App/scripts/publish-release.sh](Utterbar%20Windows%20App/scripts/publish-release.sh) — the script that actually ships the bits.

---

## 0. Preconditions (check before starting)

- [ ] `git status` in `Utterbar Windows App/` is clean (no uncommitted work) OR all uncommitted work is deliberately part of this release.
- [ ] Current branch is `main` or an explicit release branch. Not `dev`, not a WIP branch.
- [ ] Latest `main` is pulled from `origin`.
- [ ] `~/.utterbar-deploy/.env` exists with `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_ENDPOINT`, `R2_BUCKET`, `UTTERBAR_ADMIN_API_TOKEN`, `UTTERBAR_API_BASE`. If missing: stop, see [RELEASE.md](Utterbar%20Windows%20App/RELEASE.md) setup section.
- [ ] `aws`, `jq`, `curl`, `sha256sum`, `pnpm` all on PATH. Windows fresh machine: `winget install Amazon.AWSCLI jqlang.jq`.
- [ ] `aws s3 ls s3://utterbar/releases/ --profile utterbar-r2` returns the current set of releases (= R2 credentials are valid).
- [ ] `curl -sS -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $UTTERBAR_ADMIN_API_TOKEN" "$UTTERBAR_API_BASE/api/admin/releases" -X GET` returns `405` or `200` (= admin token is valid; `401` means bad token).

If any precondition fails: stop and resolve before touching anything else. Partial releases are harder to recover than delayed ones.

---

## 1. Pick the version number

Semver, pre-1.0 so MINOR bumps are normal:

- **PATCH** (`0.6.1 → 0.6.2`) — bug fixes only, no new user-visible features, no schema changes. Safe for everyone to auto-update to.
- **MINOR** (`0.6.x → 0.7.0`) — new features, new Pro-gated behaviour, new settings, or anything that would surprise a user reading the release notes. Also used for Ed25519 keypair rotation (see [LAUNCH_CHECKLIST.md §A2](Utterbar%20Website/LAUNCH_CHECKLIST.md)).
- **MAJOR** (`0.x → 1.0`) — reserved for the "we ship this to the public" launch.

Decision must be made BEFORE any file is edited. Write it down in the PR description or release notes header.

---

## 2. Bump the version in exactly three files

All three must match. The only tool we use for this is [scripts/bump-version.mjs](Utterbar%20Windows%20App/scripts/bump-version.mjs).

```bash
cd "Utterbar Windows App"
pnpm version:bump <new-version>     # or:  node scripts/bump-version.mjs <new-version>
```

The script updates:
1. [package.json](Utterbar%20Windows%20App/package.json) — `version` field
2. [src-tauri/Cargo.toml](Utterbar%20Windows%20App/src-tauri/Cargo.toml) — `[package] version`
3. [src-tauri/tauri.conf.json](Utterbar%20Windows%20App/src-tauri/tauri.conf.json) — `version` field

**Verify** (don't skip this — the script has silently failed before):
```bash
grep -n 'version' package.json                           # line 3
grep -n '^version' src-tauri/Cargo.toml                  # line 3
grep -n '"version"' src-tauri/tauri.conf.json            # line 4
```
All three must show the new version. If they don't match, STOP and fix manually.

---

## 3. Write the release notes FIRST

Before building, create `Utterbar Windows App/releases/<version>.md`. This file is POSTed verbatim to the admin API and rendered on the website's download + changelog pages — **it is the user-facing changelog**.

### Format

```markdown
# Utterbar <version>

<one-sentence tagline — "What's new in one breath">

## New
- First user-facing addition
- Second user-facing addition

## Improved
- First refinement
- Second refinement

## Fixed
- First bug fix
- Second bug fix
```

### Voice rules (same as [CLAUDE.md §5.1](CLAUDE.md))

- **British English.** *colour*, *customise*, *licence* (noun).
- **Second person.** "You can now…" not "Users can…".
- **Concrete.** "Custom accent colours (Pro)" not "Expanded theming options".
- **No marketing fluff.** Don't ship "Completely reimagined audio experience" when the change is "lowpass filter on the start cue".
- **No invented features.** If it's not in the build you're about to ship, it's not in the notes.
- **Don't mention internals.** Users don't care that we refactored the zustand store. They care that the pill stops drifting.

### Claim-check before saving

Read each bullet and ask: *is this literally true of the binary I'm about to build?* v0.6.1 shipped with "History page detail pane no longer eats a second viewport" when the page was still a broken 2-column layout below 1100 px — that's the exact kind of bullet to catch here. If the bullet is aspirational, either cut it or downgrade it to what was actually done.

Commit the `.md` with the version bump from §2 as a single commit:
```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json releases/<version>.md
git commit -m "Release v<version>"
```

---

## 4. Build

```bash
cd "Utterbar Windows App"
pnpm install                 # idempotent; safety
pnpm tauri build
```

Expected output:
```
Finished `release` profile [optimized] target(s) in Xm Ys
Built application at: …\utterbar.exe
Finished 2 bundles at:
    …\bundle\msi\Utterbar_<version>_x64_en-US.msi
    …\bundle\nsis\Utterbar_<version>_x64-setup.exe
```

**Hard checks before moving on:**
- [ ] NSIS installer filename is `Utterbar_<version>_x64-setup.exe` (contains the version — if it doesn't, `bump-version.mjs` didn't update `tauri.conf.json`).
- [ ] SHA-256 of the `.exe` stored locally for comparison later:
  ```bash
  sha256sum "src-tauri/target/release/bundle/nsis/Utterbar_<version>_x64-setup.exe"
  ```
- [ ] Smoke-test the installer on Jay's machine: install, launch, dictate one sentence, confirm text appears in Notepad.
- [ ] Check `%APPDATA%\Utterbar\logs\crash.log` after 5 minutes of use — if any new entries appear, **STOP**. Do not ship. Investigate the crash before publishing. (v0.6.1 was shipped with an active illegal-instruction crash — don't repeat that.)

If smoke test passes, proceed. If it fails, fix the bug, re-build, re-smoke. Do NOT bump the version again — the version you picked in §1 is the version that ships.

---

## 5. Publish via `publish-release.sh`

```bash
cd "Utterbar Windows App"
./scripts/publish-release.sh <version> \
    "src-tauri/target/release/bundle/nsis/Utterbar_<version>_x64-setup.exe" \
    releases/<version>.md \
    --announce --tag "latest"
```

Flags:
- `--announce` — sends the release email to confirmed update-subscribers. Include unless you explicitly want a silent release.
- `--tag "latest"` — displays the cyan "latest" pill next to the version on `/download`. Include for stable releases.
- `--beta` — channel=beta instead of stable. Do not use for user-facing releases yet (no beta audience).

### What the script does (for debugging)

1. Computes SHA-256 of the `.exe`.
2. Uploads to R2 at `s3://utterbar/releases/<version>/Utterbar-Setup-<version>.exe` via the `utterbar-r2` AWS profile.
3. HEAD-verifies `https://dl.utterbar.com/releases/<version>/Utterbar-Setup-<version>.exe` returns HTTP 200.
4. POSTs metadata (`version`, `channel`, `sha256`, `notes_markdown`, `announce`, `tag`) to `$UTTERBAR_API_BASE/api/admin/releases` with bearer-token auth.
5. Expects `201 Created` with `id`, `download_url`, `size_bytes`, `announced_to`.

### Known drifts and workarounds (as of 2026-04-26)

See [MONOREPO.md §10](MONOREPO.md) for the full list. When fixing drift, **fix it for real** — don't work around it every release.

- **R2 key — RESOLVED.** The committed `publish-release.sh` uploads to the versioned key `releases/<v>/Utterbar-Setup-<v>.exe` only and POSTs `r2_key` explicitly. The admin API's `file_not_in_bucket` fallback now defaults to the same versioned path. No second-key upload, no legacy `Utterbar-Setup.exe`. Old R2 objects from the dual-upload era are orphans — safe to delete from the bucket.
- **`curl -d "$PAYLOAD"` quoting bug — RESOLVED.** The committed `publish-release.sh` uses `--data-binary "@$TMP_PAYLOAD"` with a temp file and explicitly sends `r2_key` in the payload. If you hit a 422 about missing fields, it's not this bug — read the actual server response body before re-running.

### If publish fails

| Response | What it means | What to do |
| --- | --- | --- |
| `201 Created` | Everything worked. | Proceed to §6. |
| `401 Unauthorized` | `UTTERBAR_ADMIN_API_TOKEN` is wrong. | Check `~/.utterbar-deploy/.env`. Fix and re-run (script is idempotent as long as the DB row wasn't created). |
| `409 version_exists` | A previous attempt partially succeeded. | See "409 recovery" in [RELEASE.md §5](Utterbar%20Windows%20App/RELEASE.md). `DELETE /api/admin/releases/<id>?delete_file=1` then re-run. |
| `422 file_not_in_bucket` | R2 upload succeeded but admin API can't find it at the key the script claimed. | Check `releases/<v>/Utterbar-Setup-<v>.exe` exists in R2. The script and the admin API now both use the versioned key — if they've drifted, fix the drift before re-running. |
| `422 version required` (or similar per-field) | curl quoting bug (§5 drifts). | Re-run with payload written to a temp file and `--data-binary "@$TMPFILE"`. |
| `503` | Admin API is not configured server-side. | Stop. Server ops issue — contact Jay. |

---

## 6. Verify end-to-end

These are non-negotiable. Skipping them is how v0.6.1 shipped with a broken updater route.

- [ ] `curl -sI https://dl.utterbar.com/releases/<version>/Utterbar-Setup-<version>.exe` → HTTP 200 with matching content-length.
- [ ] Open `https://utterbar.com/download` in a fresh incognito browser — the new version is displayed with the `latest` pill, SHA-256 matches what `publish-release.sh` computed, "Download" button points at the versioned URL.
- [ ] Open `https://utterbar.com/changelog` (or `/releases` depending on what's wired up) — the new version's notes render correctly with the same markdown you wrote in §3.
- [ ] `curl "$UTTERBAR_API_BASE/api/updater/check?current=<previous-version>&channel=stable"` should return the new version's metadata. The route IS implemented in `UpdaterApiController.php`; if it returns 404 on the live site it means the website hasn't been deployed yet — run `Utterbar Website/scripts/deploy-website.ps1` first, then retry. If it returns a 200 with `available: false` unexpectedly, check the `sha256` + `r2_key` on the admin release row; the read-time filter rejects rows with malformed either field.
- [ ] Run the OLD version (e.g. from the previous installer) on a clean machine, let it idle for ~60 s → the `UpdateToast` should appear. If not, same reason as above.

---

## 7. Tag + push

Only after §6 is green:

```bash
cd "Utterbar Windows App"
git tag v<version>
git push origin main v<version>
```

If you had to make fix-up commits during §4 or §5, push those too. Tag points at the exact commit that was built.

---

## 8. Post-release housekeeping

- [ ] Update [MONOREPO.md §12](MONOREPO.md) "Current state" section: bump "Latest shipped version" and update "Latest app-repo commits" hashes.
- [ ] If the release fixed any known drift from [MONOREPO.md §10](MONOREPO.md), delete the drift entry. Don't let stale drift pile up.
- [ ] If this release pulled a previous version (see v0.6.1), update the relevant `memory/project_v061_pulled.md`-style memory file with "superseded by v<new>".
- [ ] Close any tracker items (GitHub issues, Linear cards, checklist items) that this release resolved.

---

## 9. If the release is bad

You find out a shipped build crashes / has a regression / breaks activation. Response:

1. **Pull it from the download page first, investigate second.** Users installing right now are hitting the bug.
   ```bash
   curl -X DELETE "$UTTERBAR_API_BASE/api/admin/releases/<id>?delete_file=0" \
     -H "Authorization: Bearer $UTTERBAR_ADMIN_API_TOKEN"
   ```
   `delete_file=0` keeps the R2 object (so users mid-download don't 404) but removes the DB row (so `/download` falls back to the previous stable release).
2. **Tell users.** If `--announce` was used, some users already got an email pointing them at a broken installer. Send a follow-up. Script doesn't do this — hand-write it for now.
3. **Record the failure.** Add a memory file under `memory/project_v<ver>_pulled.md` with: what broke, when you found out, what evidence you have (crash.log, user report), and what the re-release needs to change. This is the pattern established by [project_v061_pulled.md](../.claude/projects/c--Users-jaypa-Documents-GitHub-Utterbar/memory/project_v061_pulled.md).
4. **Fix the root cause, not the symptom.** If the binary crashed, diagnose why — don't just rebuild. See [MONOREPO.md §10.6](MONOREPO.md) + [project_v061_investigation.md](../.claude/projects/c--Users-jaypa-Documents-GitHub-Utterbar/memory/project_v061_investigation.md) for the template.
5. **Re-release at the next patch.** Never re-upload under the same version — the SHA-256 would change and anyone who cached the old one gets stuck. Bump to `<version>+1` and follow this runbook from §1.

---

## 10. What this runbook does NOT cover

- **Ed25519 keypair rotation** — that's [LAUNCH_CHECKLIST.md §A2](Utterbar%20Website/LAUNCH_CHECKLIST.md). v0.7.0 shipped still trusting the sandbox keypair; rotation is outstanding.
- **Code signing** — we ship unsigned EXEs today. SmartScreen warnings are expected. Cert plan is in [LAUNCH_CHECKLIST.md §A1](Utterbar%20Website/LAUNCH_CHECKLIST.md).
- **Stripe price changes** — those don't need a new app build. Update `STRIPE_PRO_PRICE_ID` in server `.env`.
- **Website-only changes** — there's no parallel runbook yet; deploy via `pscp`/`plink` per [MONOREPO.md §2 build commands](MONOREPO.md).

---

## 11. Diff from "how it happened last time"

What the v0.6.1 release got wrong that this runbook prevents:

| v0.6.1 mistake | Runbook step that prevents it |
| --- | --- |
| Shipped with active crashes — no smoke-test on `crash.log` | §4 hard check: read `crash.log` after 5 min use |
| Release notes claimed a History fix that wasn't done | §3 claim-check: read each bullet against the actual binary |
| Double-uploaded to two R2 keys because legacy check wasn't fixed | §5 drift list — forces explicit acknowledgement each time |
| curl quoting bug blocked publish mid-flight | §5 drift list + error table — first-line troubleshooting |
| Updater endpoint missing — users on v0.6.0 couldn't auto-update | §6 verify step: actually hit `/api/updater/check` |
| No record of "what broke and why" when v0.6.1 was pulled | §9.3 memory file requirement |

If a new class of mistake hits on the next release, **add a row to this table** as part of closing out §8.
