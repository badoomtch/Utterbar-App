# TODO.md — Utterbar work tracker

One place to see what's done, what's staged, what's live, and what's
still open. Kept under `C:\Users\jaypa\Documents\GitHub\Utterbar\` so
both sub-projects are visible from one glance. Update at the end of
each session.

Last updated: 2026-04-24.

---

## Done this session — ready to verify + deploy

These changes are in the working tree. **Not yet committed, not yet
deployed.**

### Desktop app (`Utterbar Windows App/`)

| Item | Status | Notes |
| --- | --- | --- |
| **6× audio fan-out fix** | code done, **runtime unverified** | `useSoundCues`, `useSystemNotifications`, and the shortcut dispatcher now early-return in non-main webviews. Typecheck + lint green. Needs smoke-test on the built installer: press hotkey, confirm ONE tone per phase, ONE notification per event. |
| **MainShell triple-scrollbar** | code done, runtime unverified | `overflow-hidden` on flex root + `min-h-0` on aside. Only `<nav>` scrolls. |
| **FloatingBar icons vanishing instantly** | code done, runtime unverified | Exit animation lengthened + x/scale terms so icons ease out instead of being clipped by the pill's closing `overflow-hidden` width spring. |
| **FloatingBar drag border not painting** | code done, runtime unverified | GPU-composited sibling accent ring; opacity toggle survives the OS window-drag paint stall. |
| **Transcription hallucination on silence** | code done, runtime unverified | RMS + duration gate + post-transcribe filler filter in `sherpa_parakeet.rs`. 18 pre-existing backup tests still pass. |
| **Backup import hardening** | code + **18 tests pass** | `validate_entry_path` rejects abs paths, drive/UNC prefixes, `..`, symlinks/hardlinks/specials, off-allow-list paths. Canonical-parent check. |

### Website (`Utterbar Website/`)

| Item | Status | Notes |
| --- | --- | --- |
| **Sign-in token hashing** | code done, tests written, **PHP tests not run** | SHA-256 in DB only, atomic single-use consume via `lockForUpdate()`, sibling-burn. Migration backfills live tokens. 8 tests in `SignInTokenTest`. |
| **Update-subscriber token hashing** | code done, tests written, **PHP tests not run** | Same pattern as sign-in. `rotateTokenAndSave` retained, staged `prepareRotation`/`commitRotation`/`discardRotation` preferred. 9 tests in `UpdateSubscriberTokenTest`. |
| **`/api/pricing` endpoint** | code done, tests written, **PHP tests not run** | Reads `services.stripe.pro_amount` + `pro_currency`. Formats `£39.99` / `£40` / `$29`. Shape pinned by `ApiContractTest`. |
| **`/api/updater/check` endpoint** | code done, tests written, **PHP tests not run** | Reads `releases` by channel + `published_at`. Filters incomplete rows (null/short/non-hex sha256, missing r2_key, non-X.Y.Z version) via `pickEligibleRelease()`. Shape pinned by `ApiContractTest`. |
| **Incomplete releases no longer installable (fix #1)** | code done, tests written, **PHP tests not run** | SHA-256 required at publish time in both `Admin\ReleaseController` and `Api\AdminReleaseApiController`. Read-time filter belt-and-braces. Admin Blade form relabelled. |
| **Failed send no longer burns unsubscribe link (fix #2)** | code done, tests written, **PHP tests not run** | Staged rotation: prepare in-memory → send → commit on success / discard on failure. All three blast sites (API controller, browser controller, `updates:announce` console) updated AND tested via end-to-end failure + success paths. |
| **Prerelease version consistency (fix #3)** | code done, tests written, **PHP tests not run** | Chose Option A: disallow `0.7.0-rc.1` at publish time, strict `^\d+\.\d+\.\d+$` regex in both controllers. Publishing rules + updater parser now match. |
| **Stale `UpdateAnnouncement` docstring** | code done | Comment + exception message updated to point at `prepareRotation()` + `commitRotation()`, not the deprecated `rotateTokenAndSave()`. |

### Monorepo-level

| Item | Status |
| --- | --- |
| **Root `CLAUDE.md`** (short rules file) | written |
| **Root `MONOREPO.md`** (merged brain dump) | written |
| **`RELEASE_RUNBOOK.md`** (enforced release ritual) | written |
| **Per-subdir `CLAUDE.md` files** | superseded-by banner added, content preserved |
| **`TODO.md`** (this file) | written |

---

## Do before next commit

- [ ] **Run PHP tests.** Requires `composer install` + PHP 8.2+ on a machine with both. Command:
  ```
  cd "Utterbar Website"
  composer install
  php artisan test --filter="SignInTokenTest|UpdateSubscriberTokenTest|ApiContractTest|ReleasePublishValidationTest|UnsubscribeRotationTest"
  ```
  I wrote all these tests without being able to execute them locally (no PHP here). Highest-confidence place for a failure is the throwing-mailer stub in `UnsubscribeRotationTest::installFailingMailer` — the Laravel 12 `Mail` facade internals might call a method the stub's `__call` fallback catches, but if not, adjust the stub's explicit methods.
- [ ] **Smoke-test the desktop fixes on a built installer.** Build v0.6.1-replacement locally, install, verify:
  1. Pressing the hotkey once → one audio cue (not 3–6).
  2. Small window size → two scrollbars max (left nav + right content), no body scrollbar.
  3. Holding + dragging the pill → bright cyan accent ring visible for the duration of the drag.
  4. Hovering away from the pill → icons ease out smoothly, not instant pop.
  5. Accidental short-tap record → empty result, no phantom "yeah"/"so"/"mm".

---

## Do before next release (v0.6.2)

Follow `RELEASE_RUNBOOK.md` top to bottom. Specific blockers from this session:

- [ ] **Resolve the v0.6.1 `EXCEPTION_ILLEGAL_INSTRUCTION` crash.** Deterministic address `0x7FF6722F8329` — crash.log has two matching entries. Address sits below `main:` in compiler-helper / CRT region (RVA `0x2F8329`, below `main`'s `0x4BFF70`), PDB only has public symbols so dumpbin can't name it. To get a symbol:
  1. `winget install --id Microsoft.WindowsSDK.Debugging` (Windows SDK Debuggers, ships `cdb.exe`).
  2. `cdb -z "target/release/utterbar.exe" -y "target/release" -c "ln 0x1402F8329; u 0x1402F8329 L20; q"` — cdb reads the full PDB and names the function.
  3. Based on the name, locate the Rust source that's being compiled into a `ud2` or a broken indirect call.
  4. Most likely cause given the disassembly shape (pointer moves through `rdi/rsi/rbx`, two nested calls, enum-discriminant-style compares): a trait-object vtable or async executor drop path. Working hypothesis in `memory/project_v061_investigation.md`.
- [ ] **Fix in-app pricing fallback.** `useProPrice.ts:31` and `pricing.rs:59` still hard-code `$29 USD`. The `/api/pricing` endpoint shipped this session delivers `£39.99 GBP` at runtime; the fallback flash is now offline-only, but worth fixing for consistency.
- [ ] **Decide on v0.6.2 scope.** Candidate bullet list for `releases/0.6.2.md`:
  - Fixes the illegal-instruction crash from 0.6.1.
  - Fixes the duplicate-sound bug (one tone per hotkey, not six).
  - Fixes in-app pricing display to show £39.99.
  - Fixes floating bar icons disappearing instantly.
  - Fixes floating bar drag border not lighting up.
  - Fixes transcription fabricating filler words on silent input.
  - Fixes MainShell triple-scrollbar at small widths.

---

## Website deploy steps (when merging the PHP fixes)

Website has no git-based deploy; everything goes via `pscp` + `plink`. Order matters:

1. `pscp` the changed PHP files: models, controllers, mail classes, console command, routes, blade view, migrations, tests dir.
2. SSH: `php artisan migrate --force` (picks up the two new `2026_04_24_*` hash migrations).
3. SSH: `php artisan view:clear && php artisan config:clear && php artisan route:clear`.
4. Seed `STRIPE_PRO_AMOUNT=3999` and `STRIPE_PRO_CURRENCY=gbp` in server `.env` if not already matching (pricing endpoint reads these).
5. Verify live:
   - `curl https://utterbar.com/api/pricing` → `{"amount_usd":39.99,"currency":"GBP","display":"£39.99",...}`
   - `curl "https://utterbar.com/api/updater/check?current=0.5.0&channel=stable"` → `available: true` with a real sha256 + download_url for the current latest stable, OR `available: false` if nothing eligible exists.
   - Pre-existing release rows without sha256 are hidden — if the operator wants one surfaced, they need to update it manually.

---

## Open product issues (out of scope this session, noted for triage)

None of these are needed for v0.6.2 to ship:

- [ ] **Remotion marketing videos slow + ugly + waveform too fast + not like the real bar.** Bigger product-quality pass, not a tool-call fix. Needs its own session: audit `marketing/remotion/out/`, re-import real `FloatingBar.tsx` components into `BarDriver`, match audio-wave motion to the app's real ripple rate, verify each composition represents one clear idea, re-render all 12 MP4s + posters, re-upload to `public/videos/` on the website.
- [ ] **History page two-column layout doesn't scale** (`HistoryRoute.tsx`). Below ~1100 px the list column squeezes to 6-char-wide vertical words. Needs redesign as one column + modal detail pane. Queue for next polish cycle; v0.6.1 release notes already over-claimed a fix for this — correct in v0.6.2 notes.
- [ ] **Remotion 400 px mobile breakpoint not verified.** Videos render correctly at 1920×1080, unknown at website's mobile breakpoint.
- [ ] **Docs drift from `DOCS_AUDIT_v0.6.1.md`.** Shortcuts doc says all defaults are unbound (they're not). Modes doc claims six built-in modes (three ship). Vocabulary doc mentions a non-existent Team tier. Troubleshooting references a pill-visibility hotkey that doesn't exist + missing an updater section. Priority order documented in the audit file.

---

## Session-state caveats to carry forward

- **No PHP toolchain on this machine.** Tests for the website are source-complete but unrun. Every summary in this session has flagged this; do not merge without running them.
- **`cargo clean` was run.** First full `cargo build` after this session is a cold compile. Not caused by any change in this session; noting so it's not a surprise.
- **Claude permissions blanket-allowed.** `C:\Users\jaypa\.claude\settings.json` has `"defaultMode": "bypassPermissions"`. Every command runs without a prompt. Honour risky actions (destructive git, force push, etc.) with manual confirmation regardless.
- **Memory state**: `MEMORY.md` + `project_v061_*` files under `.claude/projects/.../memory/` are up to date with this session's findings. `reference_release_runbook.md` points at the new runbook file.

---

*End of TODO.* Edit freely as things ship or slip. Do not let stale entries pile up — if an item is done and verified, delete it or move it to a "Done this session" block with a date.
