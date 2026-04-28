# TODO.md — Utterbar work tracker

One place to see what's done, what's staged, what's live, and what's
still open. Kept at the monorepo root so both sub-projects are visible
from one glance. Update at the end of each session.

**Last updated: 2026-04-28 (v0.14.13 security patch merged + shipped to beta; v0.15.0 speaker-diarisation feature branch fully implemented through five review rounds, all gates green, **approved for beta-only publish**, mid-publish at session end — see "v0.15.0 status" below for the live state).**

---

## Next session pickup — read this first

**v0.15.0 status (as of session-end 2026-04-28):**

- Branch: `feature/v0.15.0-diarisation`, off `main` after the
  `ba5e923` security-patch merge.
- **APPROVED FOR BETA-ONLY PUBLISH.** Beta channel only —
  no `--announce`, no `latest` tag, no stable promotion.
- All gates green at publish time: `cargo check`, `cargo
  test` (207/207 pass, 2 ignored), `cargo clippy
  --all-targets -- -D warnings`, `cargo fmt --check`,
  `pnpm typecheck`, `pnpm lint`, `pnpm build`.
- Five blocker-review rounds with Jay have landed all
  corrections — see "Round 2/3/4/5 fixes" in the v0.15.0
  section below for the full inventory.

**To continue from here:**

1. **If publish has completed**: read the v0.15.0 section's
   "Residuals / pending operator action" subsection for the
   admin row id, CDN URL, and the soak/QA still required
   before stable promotion.
2. If a new blocker surfaces post-publish: pull a beta-only
   bugfix patch (`fix(diarise): v0.15.1 — …`) on a small
   branch, run all seven gates, ship to beta as a hotfix.
3. **Stable promotion**: only after real-meeting QA passes
   (two-speaker Zoom, in-room, solo, silence-heavy, 30+min,
   missing models, missing Whisper, rename + export, old
   v0.14 meeting open-and-render). Then re-publish without
   `--beta` so a signed v0.15.x stable row exists; this
   also un-blocks the silent stable updater feed (open
   item from v0.9.0).

**Other context worth carrying forward:**

- v0.14.12 mic-routing fix shipped to beta + verified by
  Jay in real-world recordings ("recorded from the Scarlet
  one, so that worked").
- v0.14.13 security/stability patch shipped to beta; merged
  into `main` at commit `ba5e923` with all CC-BY-4.0
  attribution surfaces.
- The v0.15 implementation plan went through Step 0 model-
  asset SHA verification (locally computed + cross-checked
  against k2-fsa upstream `checksum.txt` for the embedding
  asset) before any code landed. The pinned SHAs in
  `services::inference::diarize_assets` are the verified
  ones. **Do NOT change them without re-running the local
  fetch + sha256sum flow.**

---

## Where we are right now

**Last publicly shipped stable: v0.8.7** (admin release row id=19, 2026-04-26). Stable channel currently spans v0.7.0 → v0.8.7. Tip jar live on home + pricing + download. GA4 consent-gated. Per-page OG images, admin tip + analytics + licence + subscriber dashboards, /blog with file-backed Markdown posts, /compare/* SEO pages, multi-engine ASR (Parakeet TDT + Whisper + Zipformer) all wired and shipping.

**Beta channel:**
- **v0.8.8** (id=20, unsigned) — Whisper CPU-pin + `[Music]` filter.
- **v0.9.0** (id=21, **Ed25519-signed**) — security hardening pass: signed updater metadata, downgrade-prevention + dl.utterbar.com URL pin, DPAPI-encrypted licence cache, dual-key licence verification (V1 sandbox + V2 live), pinned model SHA-256s, admin-API audit log + tighter throttles, Settings IA (Update channel moved into General).

**In-tree app version: v0.11.0** across `package.json`, `Cargo.toml`, `tauri.conf.json`. Last publicly built version is v0.10.0 (stable). v0.11.0 is queued for a beta-channel build with: per-app auto-activation matcher wired, .srt/.vtt subtitle export for file transcription, history → vocabulary/replacement correction loop, history filters (mode/app/date range). See [releases/0.11.0.md](Utterbar%20Windows%20App/releases/0.11.0.md).

**Stable updater feed quirk:** the new server-side eligibility filter requires `updater_signature` set when `UPDATER_PUBLIC_KEY_B64` is configured. None of the legacy v0.7/v0.8.6/v0.8.7 stable rows are signed, so `/api/updater/check?channel=stable` returns `available:false` for every stable user until a signed v0.9.x stable row is published. **Plan:** soak v0.9.0 on beta for 1–2 weeks, then re-publish without `--beta`.

**Website:** deployed. All cross-side contract endpoints live:
- `GET /api/pricing` returns `£39.99 GBP`.
- `GET /api/updater/check?current=<v>` returns the latest stable
  release row.
- `POST /api/licence/{fetch,activate,deactivate,heartbeat}` working.
- Stripe live (Pro checkout + tip jar both go through it).
- Magic-link sign-in uses SHA-256-hashed tokens.
- GA4 (`G-5PQ5CNPQ38`) loads only after consent banner accept.
- Cookie consent banner + footer "Cookies" revisit link.
- Per-page OG image generator at `/og/image`.
- Download analytics + tip dashboard at `/admin/{analytics,tips}`.

**Versions in tree (verified 2026-04-27):**
- App `package.json` / `Cargo.toml` / `tauri.conf.json`: `0.9.0` (matches beta-published binary).
- Website `config/site.php`: `0.8.7` (informational fallback; real release data served from the `releases` table).

---

## In-flight (uncommitted) — do NOT count as shipped

Working-copy work staged on disk but not committed. Verified via
`git status`. Cross-check against shipped release notes before
claiming any of this is live.

**Website (Laravel):**
- Waitlist removal: `WaitlistController`, `WaitlistEntry`,
  `WaitlistConfirmation` mailable, `waitlist-form` Blade component,
  `/waitlist` route + page, waitlist confirmation email templates,
  and the `2026_04_24_110000_drop_waitlist_entries_table.php`
  migration are all deleted/staged for deletion. The migration has
  not yet run on production.
- New admin surfaces: `Admin/ContactController`,
  `Admin/LicenceController`, `Admin/SubscriberController`, plus
  views under `pages/admin/contacts/`, `pages/admin/licences/`,
  `pages/admin/subscribers/` and an `admin-nav` component.
- New artisan command: `LicenceRemint`.
- New marketing pages: `pages/compare/*` and
  `pages/what-is-local-dictation.blade.php`.
- Pricing card / prompt-cta / footer / docs / pages / contact /
  legal Blade views modified across the board.
- `config/marketing.php`, `routes/web.php`, `deploy-website.ps1`,
  `app.js`, `still.blade.php`, `video-embed.blade.php` modified.

**B2B / business pass (2026-04-27, staged):**
- New website surfaces: `/business` landing page, `/business/contact`
  typed enquiry form, `/docs/it-deployment` deployment doc, plus a
  volume-pricing block on `/pricing` (Team / Business / Enterprise).
- Backend: `BusinessEnquiryController`, `Admin/BusinessEnquiryController`,
  `BusinessEnquiry` model + `business_enquiries` table migration,
  `BusinessEnquirySubmission` mailable, admin index/show views with
  status update, `BusinessEnquiry` admin nav tab.
- Self-serve checkout stays at 1 seat (Personal). Team/Business
  flow through enquiry → Jay hand-mints a licence with the
  appropriate `max_devices`. No SAML, no SSO, no fake enterprise
  console.
- Desktop app: `src-tauri/src/services/policy.rs` — group-policy
  registry overlay reading `HKLM\SOFTWARE\Policies\Utterbar`
  (HKCU as per-user fallback). Three pinnable values shipped:
  `UpdateChannel`, `CheckUpdatesAutomatically`, `BlockModelDownloads`.
  Wired into `SettingsStore::snapshot()` (overlays on every read)
  and into `ModelRegistry::install()` (refuses downloads when
  `BlockModelDownloads=1`). 6 unit tests pass.
- Migration `2026_04_27_140000_create_business_enquiries_table.php`
  has not yet run on production.

**Desktop app:**
- v0.7.3 release notes (auto-update flow rework — toast on focus,
  manual "Check now" wiring, onboarding consent step).
- Modified: `MainShell.tsx`, `Onboarding.tsx`, `GeneralSettings.tsx`,
  `ModelsSettings.tsx`, `UpdateToast.tsx`, plus a new
  `src/state/update.ts` store.
- Rust-side: `pricing.rs`, `inference/mod.rs`, `licence/verify.rs`
  modified.
- Marketing/Remotion bundle: HowItWorks + OnboardingWalkthrough
  compositions deleted; new `StatePillStatic.tsx`; manifest +
  rendered `out/` regenerated.

---

## Cross-side contract status (verified 2026-04-26)

Reconfirmed by re-reading the script + release docs (see fault
line §5 below):

- `Utterbar Windows App/scripts/publish-release.sh` uploads to the
  versioned R2 key `releases/<v>/Utterbar-Setup-<v>.exe` only — no
  legacy second-key upload. POST body explicitly includes `r2_key`
  so the admin API does not fall back to its default. POSTs to
  `$UTTERBAR_API_BASE/api/admin/releases` via temp-file
  `--data-binary "@..."` (avoids git-bash multiline JSON mangling).
- The website-side admin API's `file_not_in_bucket` fallback now
  defaults to the versioned key (matches what the script uploads).
- `--announce` triggers email-out to confirmed `update_subscribers`.

If anything in [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md) or
[Utterbar Windows App/RELEASE.md](Utterbar%20Windows%20App/RELEASE.md)
contradicts the above, fix the docs in the same PR.

---

## Open product issues

### B2B follow-throughs (post first-pass)

The 2026-04-27 B2B pass shipped the front door (landing page,
enquiry form, IT-deployment doc, volume pricing block) and a small
group-policy surface (5 pinnable values: `UpdateChannel`,
`CheckUpdatesAutomatically`, `BlockModelDownloads`,
`BlockSystemAudio`, `RequireLicence`). What still needs doing
before the *first real volume sale* closes cleanly:

- [ ] **Code-signed installer at order time.** Public installer is
  unsigned (SmartScreen warnings on first install). For any
  Business / Enterprise order where a fleet rollout can't tolerate
  that, we sign at order time and fold the cert cost into the
  quote. Requires: a process for buying / loading an EV
  signing-cert on demand, signing during `publish-release.sh`,
  and a way to ship the signed installer to one customer without
  it leaking into the public R2 bucket. **Not a blocker for the
  first enquiry — only when the first enquiry says yes.**
- [ ] **Bulk licence delivery for fleets.** Today personal licences
  arrive via a per-buyer email + magic link. A 50-seat hand-mint
  needs *one* artefact the IT team can deploy across every
  machine — likely a signed `licence.bin` file + a documented
  drop path under `%APPDATA%\Utterbar\` (or a registry-pinned
  override). Spec'd, not built.
- [ ] **Manual-mint invoice / VAT receipt.** Stripe handles this
  for self-serve. Hand-minted Business licences currently have no
  paper trail — first volume customer will need an invoice with
  VAT broken out before they pay. Probably a one-off Stripe
  Invoice from the dashboard for the first 1–3 deals; revisit if
  it becomes routine.
- [ ] **Volume self-serve checkout (later).** Manual mint is fine
  for the first 3–5 deals; painful at 20. When the queue justifies
  it, extend Stripe checkout with a quantity selector that maps
  to `max_devices` on the minted licence. Don't build until there
  is queue.
- [ ] **Three more policy keys waiting on Settings work.**
  `BlockTelemetry`, `BlockHistory`, `BlockHistoryExport` would all
  be high-value pinnable values for compliance buyers, but the
  underlying user-facing toggles don't exist yet. Build the
  Settings → Privacy controls first, then add the policy
  overrides. Promised in the IT-deployment doc as "more keys per
  customer request" rather than as shipped today.
- [ ] **Air-gapped activation.** Documented as "on request" only.
  Build when a real buyer asks: a `licence_request.bin` writer in
  the app, a paired admin endpoint that returns a signed
  `licence_response.bin`, and a one-button ingest flow in the
  app's Account UI. Each leg is small; combined it's a
  sub-session of work.

### Pre-existing open issues

- [ ] **Waitlist tidy-up** — code is staged for deletion in-tree
  but not yet pushed/deployed. Run the drop migration on production
  after the deploy, then verify `prompt-cta` + footer + admin views
  no longer reference a waitlist anywhere.
- [ ] **Legal pages refresh** — `legal/privacy.blade.php` and
  `legal/terms.blade.php` still describe the site as storing only
  waitlist emails. With Stripe live they need to mention purchase
  records, licence keys, and device fingerprints.
- [ ] **Ed25519 key rotation.** Sandbox keypair still in use on the
  live site — see [LAUNCH_CHECKLIST.md §A2](Utterbar%20Website/LAUNCH_CHECKLIST.md).
  Plan needs to allow existing v0.7.x installs to keep validating.
- [ ] **Admin dashboard cohesion** — the staged `Admin/*`
  controllers + views (contacts, licences, subscribers) need a
  navigation pass once they ship. The `admin-nav` component is the
  starting point.
- [ ] **Remotion video quality pass.** Re-render compositions
  against current `FloatingBar.tsx` imports; consider R2-backed CDN
  delivery.
- [ ] **4.9 GB idle RAM** — never investigated. See
  [MONOREPO.md §10.6](MONOREPO.md).

---

## v0.15.0 Speaker Diarisation (feature/v0.15.0-diarisation branch, 2026-04-28) — implemented, not shipped

Real per-turn speaker labels in Meeting Mode. The v0.13.x — v0.14.x
loudness/RMS classifier was suppressed in v0.14.3 and removed
entirely in v0.15.0; replaced by sherpa-onnx pyannote 3.0
segmentation + NVIDIA NeMo TitaNet small embedding + threshold-based
clustering.

Pipeline (Meeting Mode only):
- start_recording preflight refuses without Whisper Large installed
  (was a silent fall-through to Parakeet pre-fix); ensures diariser
  assets on first meeting use, downloading them via the hardened
  asset manager (~47 MB compressed, SHA-pinned, traversal-protected
  extract).
- stop_recording: diariser runs once on the full mixed buffer →
  segments → user-space merge + shape pass → per-segment Whisper
  Large transcribe → stitched output `[mm:ss] **Speaker N:** text`.
- Hard fallback: any failure (assets missing, model load error,
  panic in compute, empty segment list) routes to the bounded
  chunked path with `[mm:ss] text` only — no labels, no RMS
  classifier, safe for 30+ minute recordings.

Rename surface:
- Meetings page lets the user rename `**Speaker N:**` labels
  inline. Persists per-row in `history.speaker_names` (JSON,
  V0006 migration). Re-rename non-destructive; clear-rename
  reverts to defaults. Substitution at render time only;
  `processed_text` stays canonical.
- Server-side validation: integer ids 1-32, non-empty trimmed
  names ≤64 chars, no control chars.

Attribution:
- New `THIRD_PARTY_LICENCES.md` at repo root + bundled copy
  rendered at Settings → About → Third-party licences.
- Covers: NVIDIA Parakeet TDT v3 (CC-BY-4.0, was previously
  uncredited), NVIDIA NeMo TitaNet small (CC-BY-4.0, new in
  v0.15.0), pyannote segmentation 3.0 (MIT, new), Whisper
  Large v3 (Apache-2), sherpa-onnx, sherpa-rs, ONNX Runtime,
  major Rust + frontend deps.

Tests: 52 new tests across the v0.15 surface, landed across
five review rounds:
- Round 1 (initial implementation, 14 tests): 5 diariser-asset
  traversal/safety + 2 diariser config + 7 speaker-rename
  validation.
- Round 2 (Jay's blocker review, +20 tests): 7 history insert/
  list/update round-trip + meeting-row-only update guard,
  2 commands::meetings error mapping, 11 diarised pipeline
  helpers (merge / shape / pad / format / dedup) including the
  `padded_segments_can_sum_to_more_than_source` regression
  pinning the source-PCM duration fix.
- Round 3 (concurrency review, +5 tests): 3 `decide_reentry`
  branch tests including the meeting-preflight reject case,
  2 RAII guard tests including the panic-clears-flag scenario.
- Round 4 (concurrency widening, +3 tests): 1 generic-start-
  pending reject case, 1 preflight-without-pending defensive
  case, 1 nested-guard footgun pin.
- Round 5 (non-atomic-snapshot race, +10 tests): 4 stage-1
  `decide_pre_probe` + 6 stage-2 `decide_post_probe` cases
  covering active-capture priority (8 input combinations),
  pending-recheck-after-probe rejection, the
  `RECORDING_STALE_MS` time-gate (young-flag-rejects /
  old-flag-stale-heals / zero-and-negative-age defensive),
  3 `StartPendingGuard` CAS-mutex contract tests, 2
  `PreflightGuard` RAII tests, 3 `mark_recording_started`
  / `clear_recording_flag` pair-write tests against a real
  `AppState` bootstrap.

Total cargo test count: 207 passing, 2 ignored, 0 failed
(was 155 pre-v0.15.0). Frontend typecheck/lint/build clean.

**Round 2 blocker fixes** (what changed between the original
implementation and the first re-review):
- `services::history::insert` SQL had `speaker_names` listed
  twice in the column list — would have failed at runtime on
  every history insert. Deduped + added round-trip tests.
- `services::history::update_speaker_names` previously
  ignored `is_meeting`, silently accepting renames on
  dictation rows. Now scoped `WHERE is_meeting=1` with
  zero-affected-rows surfacing as `OrliqError::User
  ("Meeting transcript not found.")`.
- `transcribe_meeting_diarised` previously summed padded slice
  durations into `AsrResult.duration_ms`; padded segments
  overlap by ±200ms which inflates the wall-clock past the
  recording's real length. Now `duration_ms` is computed
  once from the source PCM length; the summed slice value is
  kept as a diagnostic log only.
- Fallback `OVERLAP_SECONDS = 2` (production value since
  v0.13.x) reconciled against the implementation plan's
  loose 5s placeholder. Kept 2s, documented in the code
  comment + release notes.
- `MeetingsRoute::persistRename` clear path was missing
  try/catch + rollback. Now both set + clear share a single
  guarded path.

**Round 3 blocker fix** (meeting-mode preflight race):
- `start_recording` for meeting mode CASs
  `recording_in_progress` true BEFORE awaiting
  `ensure_assets_ready` (which can take seconds-to-minutes
  on first use). Pre-fix, a second hotkey press during
  asset prep would pass the v0.6.3 stale-self-heal — the
  guard saw `flag=true && capture=None` and proceeded,
  racing the diariser .partial files.
- New `meeting_preflight_in_progress: AtomicBool` on
  `AppState`. Set true before the asset-prep await,
  cleared on every exit path including panic via a
  RAII guard.

**Round 4 blocker fix** (generic start-pending race):
- The round-3 fix only covered the diariser preflight
  window. The same race still existed for non-meeting
  starts during `settings.snapshot().await` and
  `audio.start_capture().await` — duplicate hotkey would
  see `pending_meeting=false` and `capture=None` and
  stale-heal a flag that wasn't stale.
- New `start_recording_pending: AtomicBool` claimed
  IMMEDIATELY after the `recording_in_progress` CAS,
  before any await; cleared on every return via RAII.
  Covers the entire start lifecycle, not just the
  diariser window.
- Re-entry guard widened to a 4-variant `Reentry` enum
  with `RejectStartPending` for the generic case.

**Round 5 blocker fix** (non-atomic-snapshot race):
- Even with both flags, the CAS-failed branch was
  awaiting `audio.current()` BEFORE reading the pending
  atomics. The await is a yield point, so a duplicate
  call could sample stale `(capture=None, pending=false)`
  and stale-heal an unstale flag. **No fix that samples
  three atomics across an `await` can produce a coherent
  snapshot.**
- Three interlocking changes:
  1. `start_recording_pending` promoted to a CAS-mutex.
     `StartPendingGuard::try_claim` returns `Option<Self>`
     — `None` if already claimed by another start.
  2. New `recording_in_progress_started_at_ms: AtomicI64`
     stamped on every CAS-set, zeroed on every clear via
     centralised `mark_recording_started` /
     `clear_recording_flag` pair-write helpers (wired at
     all 7 clear sites).
  3. **Two-stage re-entry decision.** Stage 1
     (`decide_pre_probe`) reads ONLY the cheap pending
     atomics — no await — and rejects if either is held.
     Stage 2 (`decide_post_probe`) probes capture, reads
     the started-at age, then **re-reads the pending
     atomics**. Returns `ProceedAfterStaleHeal` only when
     ALL signals agree, including
     `flag_age_ms >= RECORDING_STALE_MS` (30 s, mirrors
     `TRANSCRIBE_STALE_MS`).
- New `Reentry::RejectYoungFlag` for the case "flag held
  but younger than the stale threshold" — the round-5
  regression target.

Pinned model SHAs:
- pyannote segmentation tarball
  (24615ee884c897d9d2ba09bb4d30da6bb1b15e685065962db5b02e76e4996488,
  ~6.96 MB) → extracts to model.int8.onnx
  (d582f4b4c6b48205de7e0643c57df0df5615a3c176189be3fc461e9d18827b5d,
  1.54 MB) — MIT
- nemo_en_titanet_small.onnx
  (ad4a1802485d8b34c722d2a9d04249662f2ece5d28a7a039063ca22f515a789e,
  ~40.3 MB) — CC-BY-4.0, cross-verified against k2-fsa
  upstream checksum.txt

**Publish state (filled in by the operator after each step):**

- Beta-only publish approved 2026-04-28. No `--announce`,
  no `latest` tag, no stable promotion. Stable feed
  intentionally unchanged.
- Commit hash: see the v0.15.0 commit on
  `feature/v0.15.0-diarisation`.
- Branch pushed to origin: yes.
- Installer built locally: `src-tauri/target/release/bundle/
  nsis/Utterbar_0.15.0_x64-setup.exe` — see admin row for
  the recorded SHA.
- Admin release row id + CDN URL: recorded by
  `publish-release.sh` against the website admin API; the
  script's stdout is the source of truth and the row is
  visible at `https://utterbar.com/admin`.
- Beta updater feed: `curl
  "https://utterbar.com/api/updater/check?current=0.0.0&channel=beta"`
  must report v0.15.0.
- Stable updater feed: same query with `channel=stable`
  must STILL return `available:false` (v0.7/v0.8 unsigned
  rows; v0.9.0-beta still pending stable promotion).
- Branch merged to `main` (`--no-ff`) and feature branch
  deleted locally + on origin AFTER the beta publish
  verification passes.

**Residuals / pending operator action:**

- **Real-meeting QA before stable promotion.** Run through
  the matrix Jay flagged: two-speaker Zoom, in-room,
  solo, silence-heavy, 30+min, missing models, missing
  Whisper, rename + export, old v0.14 meeting open-and-
  render. Only after all of these pass should v0.15.x
  ship to stable (re-run `publish-release.sh` without
  `--beta`) — that also un-blocks the silent stable
  updater feed (open item from v0.9.0).
- Threshold tuning. The default `DIARIZE_THRESHOLD=0.5` is
  sherpa's documented default; real-recording data during
  beta soak may push it to 0.3-0.7. The
  `diarisation complete segment_count=N distinct_speakers=M
  speakers=...` log line on every meeting is the diagnostic.
- Eventual: silero-vad pre-pass to harden against music
  intros / non-speech. Filed as v0.16 follow-up.
- `THIRD_PARTY_LICENCES.md` and `src/data/thirdPartyLicences.ts`
  are kept in sync manually; both files reference each other
  in their headers. CI lint to enforce sync is a v0.16 nicety.

---

## Security/stability patch (security/v0.14.13 branch, 2026-04-28) — implemented, not shipped

A six-priority security/stability sweep on top of the v0.14.12 mic
fix. Branch: `security/v0.14.13`. Not yet built/published — Jay's
tested-and-passed sign-off on v0.14.12 is the gate before this lands
on `main`.

What landed:

- **P1 — `open_path` injection.** `cmd /C start <path>` shell-out
  replaced with a direct `ShellExecuteW` call (no `cmd.exe`
  interpretation). Path is canonicalised and must live under one of
  the allow-listed Utterbar data roots (logs, db, models,
  recordings, diagnostics audio dump, data root). 5 unit tests in
  `commands/diagnostics.rs`.
- **P2 — Release publishing hardening.** Server now stream-reads R2
  to recompute SHA-256 and rejects pasted-vs-computed mismatches.
  The persisted `sha256` is the **computed** value, not the pasted
  one. Updater signature is verified against the computed SHA. The
  browser admin form now requires `updater_signature` on the same
  conditions as the API path. Stable + announce requires an
  explicit `confirm_stable_announce` flag. Validator extracted to
  `App\Services\Releases\ReleaseValidator`. 23 publish-validation
  tests, all passing. `publish-release.sh` updated to set
  `confirm_stable_announce=true` automatically when `--announce`
  is on a stable channel.
- **P3 — Markdown XSS.** `Markdown::renderEmbedded` now passes
  `html_input => 'strip'` and `allow_unsafe_links => false` to
  CommonMark. Raw `<script>` / `<iframe>` tags stripped;
  `javascript:`/`data:`/`vbscript:` URLs in markdown links
  neutralised. 9 absence-style tests in `MarkdownXssTest`.
- **P4 — PHPUnit.** `tests/Unit/.gitkeep` added so the suite runs.
  `UpdateSubscriberTokenTest` repaired against the GET/POST split
  (GET = render confirm-button page, POST = state mutation). All
  76 website tests passing.
- **P5 — Tauri capabilities.** Dropped `fs:default` and
  `core:path:default` (frontend doesn't import either plugin).
  Other capabilities left in place — narrowing further would be a
  larger refactor and runtime QA needs to confirm nothing breaks.
  Manual QA list: main window, floating window, onboarding,
  settings, file picker, shell links, update toast, hotkey
  recording, tray recording, diagnostics open buttons.
- **P6 — Model deletion path safety.** `ModelRegistry::remove` and
  the two retirement-prune blocks in `app_state.rs` (Parakeet v2,
  Zipformer) now reject absolute paths and `..` traversals up
  front, and use `paths::assert_within` to confirm the joined
  target canonicalises inside `models_root` before any
  `remove_dir_all`. Shared `safe_model_path` helper in
  `app_state.rs`; reusable `assert_within` in `util/paths.rs`. 4
  unit tests in `safe_model_path_tests`.

Gates run + green: `cargo check`, `cargo test` (155/155),
`cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`,
`pnpm typecheck`, `pnpm lint`, `pnpm build`, `php vendor/bin/phpunit`
(76/76).

**Residuals / pending operator action:**

- **Manual runtime QA after the v0.14.13 install** — capability
  narrowing in P5 is the only item that *can't* be proven by the
  test suite alone. Once installed, exercise: floating bar,
  onboarding, settings → audio (file picker + meter), settings →
  diagnostics ("show in folder" buttons), hotkey + tray recording,
  update toast. If anything breaks at runtime with a Tauri
  permission error, the fix is to re-add the specific permission
  to `capabilities/default.json` (don't restore `fs:default` /
  `core:path:default` wholesale).
- **Browser admin form stable-publish UX** — the new
  `confirm_stable_announce` checkbox lives in
  `pages/admin/releases/create.blade.php`. First operator to use
  it after the patch ships should confirm the button styling reads
  as "destructive" (red accent on the confirm box) and not as a
  routine option.
- **Server config sanity-check** — `UPDATER_PUBLIC_KEY_B64` is
  already set in production, so the new mandatory-signature path
  on the browser form will refuse any publish without a signature.
  Operators using the browser form (rare; the script is the
  default) need to paste the b64 signature from
  `scripts/sign-update-metadata.sh`.

---

## Open risks to watch

- **v0.7.0 runtime patch maintenance cost.** Any Tauri `cargo
  update` past 2.10.x requires rebasing the vendored
  `tauri-runtime-wry` crate. Rationale in
  [MONOREPO.md §10.9](MONOREPO.md).
- **First-run model download failure paths.** Unchanged this
  release; still a plausible support ticket source.
- **v0.7.3 in-flight onboarding consent step** — defaults to on,
  but verify the consent toggle actually gates the
  `/api/updater/check` call before shipping.

---

## Session caveats to carry forward

- **Monorepo:** `c:\Users\jaypa\Documents\GitHub\Utterbar\` is a
  single git repo. `Utterbar Windows App/` and `Utterbar Website/`
  sit inside. Remote: `badoomtch/Utterbar-App` private.
- **App version ≠ website version.** Website deploys continuously;
  only the desktop installer has a semver. The
  `config('site.version')` value is informational and tracks the
  last shipped app version only.
- **Jay tests every app build before we ship.** No installer goes
  to R2 until Jay explicitly approves.
- **PHP tests are not in CI.** Rerun on the dev machine before
  deploying the website.

---

*End of TODO. Edit freely. Don't let stale entries pile up.*
