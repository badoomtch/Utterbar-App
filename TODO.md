# TODO.md — Utterbar work tracker

One place to see what's done, what's staged, what's live, and what's
still open. Kept at the monorepo root so both sub-projects are visible
from one glance. Update at the end of each session.

**Last updated: 2026-04-24 (v0.6.4 build-pending)**

---

## Where we are right now

**Desktop app:** v0.6.4 in-flight. Rust compiled green; installer build
is the next step. Testing + your sign-off before publishing anywhere.

**Website:** No changes deployed yet. A full commit-ready payload sits
in the tree: two hashed-token migrations, `/api/pricing`, `/api/updater/check`,
staged-rotation email flow, admin Blade form required-SHA fix, 58 PHP
tests passing. Deploy script at `Utterbar Website/scripts/deploy-website.ps1`
ready to run with `$env:CLOUDWAYS_PASSWORD`.

**Versions in-tree:**
- App: `0.6.4`
- Latest published app version: `0.6.1` (still what users see on the
  website's download page — nothing newer has been uploaded to R2 or
  announced).

---

## Done since v0.6.1, all rolled into v0.6.4

Everything below is committed to `badoomtch/Utterbar-App` main OR will
be committed as part of the v0.6.4 commit that is about to go.

### Desktop — stability

| Item | Mechanism | Status |
| --- | --- | --- |
| Heap-corruption crash (WebView fan-out → 3× start_recording) | Frontend gate: sound cues, notifications, shortcut dispatcher early-return in non-main webviews | committed v0.6.2 |
| Heap-corruption crash (inference teardown race) | Sweeper + inline-unload respect `Arc::strong_count` so native recogniser isn't dropped under a live transcribe | committed v0.6.3 |
| Heap-corruption crash (COM apartment mismatch, UIA on tokio workers) | Dedicated `utterbar-uia` thread owns STA apartment for app lifetime; `snapshot_selection` routes via crossbeam channel. Fixed `audio_ducking::ComGuard` to not over-release on `RPC_E_CHANGED_MODE`. | **staged for v0.6.4** |
| Hotkey silently stops working | `recording_in_progress` + `transcribe_in_progress` atomics self-heal when stale (no active capture OR timestamp > 30s) | committed v0.6.3 / refined v0.6.4 |
| Transcription inventing filler words | RMS + duration gate + post-filter against known hallucination tokens | committed v0.6.2 |
| 6× audio per hotkey | Frontend fan-out gate (same fix as the first crash class) | committed v0.6.2 |
| Start-recording while previous still transcribing | Rust rejects with `dictation:busy` event; FloatingBar flashes border for 900ms | committed v0.6.3 |

### Desktop — UI

| Item | Mechanism | Status |
| --- | --- | --- |
| History page 2-column squeeze | Full redesign: one column of entries, detail in a modal, ESC + click-outside dismiss | committed v0.6.3 |
| FloatingBar open/close staggered | Single 280ms timeline shared by width + opacity (no nested delays) | committed v0.6.3 / refined v0.6.4 |
| FloatingBar drag border flashing then vanishing | Replaced pointerup-clear with Tauri `onMoved`-debounced drag-end detection + 2.5s hard fallback | **staged for v0.6.4** |
| FloatingBar collapsed pill empty-space | Dropped min-w; pill hugs content | committed v0.6.3 |
| FloatingBar expanded pill — drag-handle space | `justify-between` on the outer pill keeps logo+wordmark left, action cluster right, empty drag zone between | **staged for v0.6.4** |
| Triple scrollbar on small window | `overflow: hidden` on html/body/#root + flex guards on aside | committed v0.6.3 |
| Sidebar version number clipped on short windows | `shrink-0 pb-5` on footer block | committed v0.6.3 |
| Icons vanish instantly on unhover | Exit animation with opacity-only fade on shared timeline | committed v0.6.3 |
| In-app Pro price fallback `$29 → £39.99` | Updated both `useProPrice.ts` + `pricing.rs` fallback | committed v0.6.2 |

### Website — security + correctness

| Item | Status |
| --- | --- |
| Sign-in tokens stored as SHA-256 hex, not plaintext | committed (website working tree) |
| Update-subscriber tokens hashed | committed |
| `/api/pricing` endpoint | committed |
| `/api/updater/check` endpoint | committed |
| Release publish validation: SHA-256 required, strict X.Y.Z version | committed |
| Staged token rotation (failed email send keeps old unsubscribe link alive) | committed |
| 58 PHP tests — all passing locally on PHP 8.2 + phpunit 11 | run + green |
| `deploy-website.ps1` for pscp/plink push to Cloudways | committed |

### Monorepo

- Root `CLAUDE.md`, `MONOREPO.md`, `RELEASE_RUNBOOK.md`, `TODO.md`, `.gitignore`. Per-subdir `CLAUDE.md` files superseded-by-root-linked.

---

## Do before committing v0.6.4 (this session, imminent)

- [ ] **Compile check Rust** — `cargo clippy --all-targets -- -D warnings` should pass.
- [ ] **Frontend typecheck + lint** — done, both green.
- [ ] **Build the v0.6.4 installer** (`pnpm tauri build`). ~7 min.
- [ ] **Hash the installer, record size + SHA-256**.
- [ ] **Commit + push** to `badoomtch/Utterbar-App`.

## Do before publishing v0.6.4 (not this session)

- [ ] **You install + test for ~30 min.** Specific things to check:
  - No crash across continuous use.
  - Hotkey never stops responding.
  - Border stays cyan for the whole drag.
  - Dragging the empty middle of the expanded pill moves the window without closing it.
  - Open/close reads as one smooth motion.
  - Busy-flash triggers when you hotkey during transcribe.
- [ ] **You give explicit sign-off.**
- [ ] I run `publish-release.sh` (SHA-256 computed, upload to R2, POST admin API, announce). 
- [ ] Git tag `v0.6.4`, push tag.

## Website deploy — independent of app release

Can ship anytime. When you're ready:

```powershell
$env:CLOUDWAYS_PASSWORD = '<password>'
.\Utterbar Website\scripts\deploy-website.ps1
```

Requires `pscp` + `plink` (`winget install PuTTY.PuTTY`). Runs composer
install --no-dev, `php artisan migrate --force` (applies the two new
hash-token migrations), clears caches, rebuilds Vite, smoke-tests the
new endpoints.

---

## Open product issues (deferred, not blocking v0.6.4)

None of these need to ship with v0.6.4:

- [ ] **Licence not persisting across app restarts.** You reported this on v0.6.3. I haven't investigated yet.
- [ ] **Licence show/hide toggle in the settings panel.** New feature.
- [ ] **Admin page `/admin` cohesion** — no releases link, no licence-revoke UI. Website work, independent of app release.
- [ ] **Docs/app drift audit** — findings in `Utterbar Website/DOCS_AUDIT_v0.6.1.md`, needs landing.
- [ ] **Remotion video quality pass** — slow to load, zoomed out, bordered, audio waveform speed wrong, doesn't match the real app. Separate dedicated session. Plan: re-render compositions against real `FloatingBar.tsx` imports, push to R2 for CDN-backed delivery.
- [ ] **Docs-stay-in-sync-with-app rule** — meta: when app behaviour changes, docs should update in the same change. Not a ticket, a policy. Already added to the tone of how I work; leave here as reminder.

---

## Open crash risk still to watch

- **First-touch COM init on fresh tokio worker.** The fixes in v0.6.4
  should resolve the mismatch-corruption path, but we've never actually
  observed the crash under the new architecture yet. If v0.6.4 still
  crashes after 30 min of use, the next suspect is clipboard reads
  via `text_inject::read_clipboard_text` (no COM but uses `GlobalLock`
  on clipboard handles — different memory-safety family). I'll know
  what to look at from the crash.log subsystem name.
- **Drag border might still flash-then-stick on window managers that
  don't emit `onMoved`.** The hard fallback at 2.5s is the safety net;
  if you observe weirdness, tell me.

---

## Session caveats to carry forward

- **Monorepo:** `c:\Users\jaypa\Documents\GitHub\Utterbar\` is a single git repo. `Utterbar Windows App/` and `Utterbar Website/` sit inside. Remote: `badoomtch/Utterbar-App` private.
- **App version ≠ website version.** The website deploys continuously; only the desktop installer has a semver.
- **The user (Jay) tests every app build before we ship.** No installer goes to R2 until Jay explicitly approves.
- **PHP tests run locally** via the composer/php install I set up. 58 tests. Run `php artisan test --filter="SignInTokenTest|UpdateSubscriberTokenTest|ApiContractTest|ReleasePublishValidationTest|UnsubscribeRotationTest"` to re-verify.
- **Memory files** under `.claude/projects/.../memory/` reflect this session's decisions. Key ones: `feedback_versioning.md` (app vs website), `project_v061_investigation.md` (crash history), `reference_release_runbook.md` (release ritual pointer).

---

*End of TODO. Edit freely. Don't let stale entries pile up.*
