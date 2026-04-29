# TODO.md — Utterbar work tracker

One place to see what's done, what's staged, what's live, and what's
still open. Kept at the monorepo root so both sub-projects are visible
from one glance. Update at the end of each session.

**Last updated: 2026-04-29 (v0.15.5 implemented in tree — Phase A of the GPU split. Provider truthfulness only: AdvancedSettings card stops claiming "3-10× faster than CPU" when the runtime is CPU-only; new inactive notice on DML/CUDA rows when selected; provider downgrade actually passes "cpu" to the engine when DML/CUDA can't be served (no more false "provider_used=cpu while passing directml" telemetry); meeting + dictation logs gain provider_requested + provider_used + fallback_used + fallback_reason; runtime_capabilities probe is file-presence based. v0.15.4 catalogue audit corrected an earlier mistake — there are NO fp16 entries in tree, the legacy fp16-related code is migration machinery for v2-era ids only. v0.15.6 = real DirectML (DLL bundle + sherpa source build + sentinel containment). Beta only. Stable still on v0.12.0.)**

---

## v0.15.5 (in tree, awaiting commit + publish) — Phase A: provider truthfulness

Make GPU-acceleration controls honest about what's actually
running. No DirectML enablement. No runtime DLL changes. No
behaviour change to transcripts.

- `services/gpu.rs::runtime_capabilities()` probes the
  shipping runtime for the DML/CUDA satellite DLLs. File-
  presence check; both `false` today.
- `services/inference/mod.rs::resolve_provider` (new pure
  helper) returns `ProviderResolution { requested, used,
  fallback_used, fallback_reason }`. When DML/CUDA is
  requested but the runtime can't serve it, `used` is `"cpu"`
  AND that's the string actually passed to the engine. Old
  behaviour passed `"directml"` to sherpa-onnx and lied in
  the log line — that's gone.
- `Active.resolution` field on the orchestrator. Exposed via
  `current_provider_resolution()` so dictation can read it
  for the transcribe-complete log.
- `meeting transcribe complete` log gains `provider_requested`
  alongside the v0.15.4 fields. `inference.transcribe done`
  (the dictation path) gains all four fields.
- `commands::diagnostics::runtime_capabilities` Tauri command,
  registered in main.rs invoke_handler.
- AdvancedSettings.tsx: card description gates the "3-10×
  faster" speed claim behind a runtime-capabilities check;
  inactive notice on DML/CUDA rows when selected against a
  runtime that can't serve them; user's preference is preserved
  (rows stay selectable so v0.15.6 picks them up automatically).

## v0.15.6 priorities — Phase B: DirectML / GPU runtime work

Hardest deferred item. Doing this properly means runtime-infra
work, not just toggling the provider string.

**Why deferred from v0.15.5:** the vendored ONNX Runtime in
`Utterbar Windows App/src-tauri/runtime/` ships only
`onnxruntime.dll` + `onnxruntime_providers_shared.dll` —
**no `onnxruntime_providers_dml.dll`, no `DirectML.dll`.**
DirectML is structurally unavailable in current builds
regardless of the `provider` config string. Re-enabling it is
not a one-line change.

**Plan (paper feasibility from the v0.15.5 packet):**
1. Switch sherpa-rs features from `download-binaries` to
   `directml` (mutually exclusive per upstream `compile_error!`).
   This triggers a CMake source build of sherpa-onnx — fetches
   `microsoft.ml.onnxruntime.directml.1.14.1.nupkg` and
   `Microsoft.AI.DirectML.1.15.0.nupkg`. New build dependency:
   CMake on the build machine. Sherpa-rs prebuilt binaries do
   not have DirectML support — this is documented upstream.
2. Bundle the resulting DLL set into `src-tauri/runtime/` and
   `tauri.conf.json` `bundle.resources`. Installer payload
   grows ~25-40 MB on the wire (estimate; see v0.15.5
   feasibility packet for methodology).
3. Wire CPU fallback at session-create / transcribe failure.
   Sentinel-file design (write-before / clear-after, scan at
   boot, quarantine-after-crash) — design only in v0.15.5;
   implement as production code paths in v0.15.6 alongside
   the actual DML runtime.
4. Per-model validation on Jay's machine pre-publish:
   Parakeet TDT v3 int8, Whisper Small, Whisper Medium,
   Whisper Large v3. Three of four were never tested against
   the new sherpa+ORT build.
5. Wire the v0.15.5-reserved telemetry to start reporting
   `provider_used="directml"` for real, plus new fallback
   reasons (`"native_crash_quarantine"`, `"output_validation_failed"`,
   `"session_create_failed"`).

**Risk: high.** Mitigation is the validation matrix and the
sentinel-based crash containment. If DML lands cleanly, the
expected speedup combined with v0.15.4's Small default takes
typical-laptop Meeting Mode RTF below 0.1.

**Phase B design items not shipped in v0.15.5:**
- Sentinel module (write-before-transcribe / clear-after /
  scan-at-boot / quarantine list). Spec'd in the v0.15.5
  feasibility packet; implementation lands when DML actually
  ships.
- Output validation (foreign-script-when-English-pinned,
  empty-output detection). Same shape — designed, not coded
  yet. Must NOT break legitimate non-English dictation.

## Future hardening — Parakeet fp16

**Correction (v0.15.5):** the v0.15.4 packet claimed fp16
catalogue entries existed and were broken. They don't exist.
The catalogue has six built-in models, none of which are
fp16: Parakeet TDT v3 int8, Whisper Base, Small, Medium,
Large v3, and the retired Zipformer. The fp16 mentions in
code are legacy machinery: a session-create CPU-rejection
guard in `sherpa_parakeet.rs:78` (defensive — no shipping
bundle triggers it), and migration code in `services/settings.rs`
+ `app_state.rs` that cleans up references to the v0.8.0-
retired `sherpa-parakeet-tdt-0.6b-v2-fp16` id. Nothing to
clean up in the live catalogue.

If a future fp16 entry is added (post-Phase B, when GPU is
real), the existing guard machinery is what surfaces a
useful error rather than a crash. No work needed today.

## Diarisation — still deferred

Same shape as before. Per the v0.15.1 review notes, the
design is "transcribe whole buffer once, diarise separately,
assign speaker labels by interval-overlap on word/segment
timestamps". Don't start until v0.15.6 lands and the perf
ceiling is gone.

**Out of scope for v0.15.5:** DirectML runtime (now v0.15.6),
sentinel/quarantine production code (Phase B design item),
output validation that drops text (Phase B design item),
real-time meeting transcription (streaming Whisper),
per-mode meeting-model override (the dormant
`Mode.asr_model_id` field stays unused), GPU-via-CUDA
(non-NVIDIA users would be left out), stable promotion of
any 0.15.x (waiting on real-meeting QA + the 2026-05-04
v0.15.4 readiness recheck).

---

---

## Next session pickup — read this first

**v0.15.2 hotfix status (in working tree, not committed):**

- v0.15.1 panicked at startup on Jay's machine after auto-
  update from v0.15.0:
  ```
  bootstrap AppState: applied migration V6__history_speaker_names
  is different than filesystem one V6__history_speaker_names
  ```
- **Root cause:** `core.autocrlf=true` on Jay's Windows machine
  rewrote V0006__history_speaker_names.sql from LF (git index)
  to CRLF (working tree) between the v0.15.0 and v0.15.1
  `tauri build` invocations. v0.15.0 binary baked LF bytes,
  recorded the LF SHA in the user DB. v0.15.1 binary baked
  CRLF bytes, computed a different SHA, refused to start —
  refinery's intentional "migration content drifted under us"
  safety guard.
- **v0.15.2 fix (in tree, not committed):**
  1. New `.gitattributes` at repo root pinning `*.sql text
     eol=lf` and `**/releases/*.md text eol=lf`. Closes the
     class of bug.
  2. V0006__history_speaker_names.sql renormalised to LF
     (1110 bytes, matches git-stored). v0.15.2 binary will
     compile the same bytes the v0.15.0 binary did →
     produces the same LF checksum → DBs that recorded the
     LF hash launch cleanly.
  3. **NEW round-2 piece:** `db::repair` module with
     `repair_known_v6_history_speaker_names_line_ending_checksum`
     called BEFORE `embedded::migrations::runner().run`.
     Detects the v0.15.1 CRLF checksum on V6's row,
     confirms `history.speaker_names` column is present
     (proof the migration's effect is on disk), and rewrites
     ONLY that row's checksum to the LF value via a
     constrained UPDATE (matching `version=6 AND
     name='history_speaker_names' AND checksum=<crlf>`).
     Without this, the LF v0.15.2 binary would still fail
     symmetrically on machines where v0.15.1 was the first
     0.15.x launch.
  4. Refinery's checksum algorithm is `SipHasher13` over
     `(name: &str, version: i32, sql: &str)` — verified by
     reading `refinery_core::Migration::unapplied`
     (refinery-core 0.8.16 runner.rs:89-93). NOT SHA-256.
     Helper recomputes via `siphasher::sip::SipHasher13` to
     reproduce the value byte-perfect.
  5. **Concrete checksum values for the packet:**
     - LF  (v0.15.0 / v0.15.2): `6176249412974396076`
     - CRLF (v0.15.1):           `7360584664322190360`
     Both stored in `refinery_schema_history.checksum` as
     decimal `Display` of the `u64`.
  6. New unit test pins V0006's SHA-256 in source so any
     future drift fails CI before publish.
  7. 10 new tests on `db::repair` covering: missing-table
     no-op, missing-row no-op, LF unchanged, CRLF+column
     repaired to LF, CRLF without column NOT repaired,
     CRLF without history-table NOT repaired, unknown
     checksum NOT repaired, LF/CRLF distinct, decimal-u64
     shape, end-to-end (synthetic CRLF DB → repair →
     refinery runs cleanly).
  8. `releases/*.md` for v0.14.13/v0.15.0/v0.15.1 also
     renormalised to LF (release notes are bundled into
     the binary via include_str! and rendered in Settings →
     Updates; line-ending drift wouldn't crash but would
     mangle rendering).
- v0.15.2 ships ONLY the line-ending fix. Speaker
  diarisation stays disabled. Meeting mode behaviour
  unchanged from v0.15.1.
- The diarisation rework that was previously labelled
  "v0.15.2 rework planning" is now **v0.15.3**.
- v0.15.1 users CAN'T auto-update because v0.15.1 doesn't
  reach the updater — the panic happens at AppState
  bootstrap, before any UI. The release notes tell users
  to either wait for v0.15.2 to land + manually download,
  or reinstall v0.15.0 to get back to a working state and
  let auto-update pick up v0.15.2 from there.

**To continue from here:**

1. **After Jay re-approves the v0.15.2 hotfix:** run gates,
   commit on `hotfix/v0.15.2-line-endings`, push, build,
   `./scripts/publish-release.sh 0.15.2 ... --beta` (no
   `--announce`, no `latest` tag), verify SHAs + feeds,
   merge `--no-ff` into main, delete branch.
2. **After v0.15.2 ships and Jay confirms it launches:**
   start the v0.15.3 diarisation rework. The architecture
   sketch + open verification questions for the rework
   moved to the renamed "v0.15.3 rework planning" section
   below.
3. **Stable promotion of any 0.15.x is still on hold** —
   v0.15.2 is just an emergency unblock so users can
   actually launch the app. Stable still serves v0.12.0.

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

## v0.15.3 Meeting Mode performance (2026-04-29) — beta only

What v0.15.3 changes:

- **Whisper Medium added to the catalogue.** SHA-pinned via Step 0
  verification (download → sha256sum → tar tjf → smoke-load through
  `SherpaWhisper::ensure_loaded` → transcribe a bundled wav and
  compare against the bundle's `trans.txt`).
  - URL: `https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-whisper-medium.tar.bz2`
  - SHA-256: `614b1172557049069d846c29d9399640bce83a4dd6c580decebd9ce2a4f32c33`
  - tarball size: 1,931,372,882 bytes (~1.93 GB; ~3.5 GB on disk
    after extract — Medium ships fp32 + int8, the resolver picks
    int8 at runtime)
  - `requires_pro: true`, `recommended: true`
- **Meeting Mode model is now an explicit user choice.** New
  `meeting_asr_model_id: Option<String>` setting on `ModelSettings`,
  serde back-compat (pre-v0.15.3 settings.json files miss the field,
  deserialise with `None`). New "Meeting mode model" card in
  Settings → Models with three radios: Fast / Balanced (recommended)
  / Best accuracy.
- **Selection policy — allow-list + no silent fallback:**
  Meeting Mode supports EXACTLY three models — Whisper Small,
  Medium, Large v3. Pinned via `is_allowed_meeting_model_id`
  helper.
  - pref off-allow-list (Parakeet, Zipformer, Whisper Base, future
    unknown ASR id) → RefuseUnsupportedModel(id) — even if the
    model is installed. User error: "Your meeting model setting
    is invalid. Open Settings → Models and choose Fast,
    Balanced, or Best accuracy."
  - pref on-allow-list + installed → Use(id)
  - pref on-allow-list + missing → RefuseExplicitMissing(id)
    (refusal copy names the model + says open Settings → Models)
  - pref unset + Medium installed → Use("sherpa-whisper-medium")
  - pref unset + Medium missing → RefusePreferenceUnset (refusal
    copy lists the three options)
  - never silently routes to ANY model — explicit user choice
    or refusal, no fallback chain.
- **Bridge fall-through hardened.** When a meeting recording
  arrives at stop_recording with no `ActiveRecordingContext`
  bound (the v0.15.2-in-flight bridge case) AND current settings
  can't yield an allow-listed meeting model,
  `resolve_target_model_id` returns
  `Err(MeetingBridgeFailure)`. stop_recording surfaces a clear
  user-facing error rather than silently routing through
  `default_asr_model_id` (which is typically Parakeet).
- **`ActiveRecordingContext`** struct on `AppState` (pub(crate))
  binds the recording session at start-time. Stop reads
  `(mode_id, is_meeting, meeting_model_id)` from the context, not
  from current settings. A meeting started as Meeting stops as
  Meeting; the meeting model used at stop is the one approved at
  start. Cleared on every recording-end path (success, cancel,
  no-active-stop, transcribe error, every start-side error).
  Source-grep guard test pins the pairing.
- **Chunk size 30s → 29s.** sherpa-onnx silently truncates inputs
  ≥30s with a stderr warning ("Only waves less than 30 seconds are
  supported"). Step 0e surfaced the warning. Tightened to 29s gives
  the recogniser the same input every chunk and removes the
  truncation. Existing v0.15.2 transcripts may have lost up to 1s
  per boundary — release notes call this out.
- **Cap retuned 130 → 135 chunks.** With 27s stride (was 28s),
  preserving "60 minutes accepted" requires 134 chunks plus a
  small shoulder. Result: 135 chunks × 27s + 29s = 3647s = 60 min
  47 s of audio.
- **Success pill** now reads `Saved · 5:01 · Whisper Medium · 11
  chunks · 799 chars`. New `model_label: String` field on
  `DictationSuccessSummary`. Frontend type mirror at
  `src/state/dictation.ts` and the auto-generated TS file both
  carry the field.
- **End-of-meeting summary log** on every meeting:
  `audio_duration_ms`, `total_processing_ms`, `realtime_factor`,
  `selected_model_id`, `chunk_count`, `whisper_language`. Lets us
  collect honest RTF data across the beta soak.
- **Pro-gating** in the new card mirrors the existing model-row UI
  (locked rows show `<ProNudge>`, Install button disabled). The
  `install_model` IPC already enforces Pro server-side; the lock is
  UX polish.
- **Whisper Large preflight rewritten** as the resolver-driven
  flow. `start_recording` no longer hard-codes
  `WHISPER_LARGE` — it calls `resolve_meeting_model_id` and
  branches on the outcome.
- Files modified: package.json, Cargo.toml, tauri.conf.json,
  Cargo.lock; domain/settings.rs, domain/events.rs, app_state.rs,
  services/models.rs, commands/dictation.rs;
  src/state/dictation.ts, src/components/bar/StatePill.tsx,
  src/routes/settings/ModelsSettings.tsx;
  releases/0.15.3.md (new); TODO.md.
- Auto-regenerated by ts-rs: src/generated/ModelSettings.ts,
  src/generated/Settings.ts, src/generated/DictationSuccessSummary.ts.

Selection-policy + lifecycle tests added to
`commands::dictation::diarised_helpers_tests`:

Round-1 (selection happy paths + base refusals):
- meeting_resolves_to_medium_when_no_pref_and_medium_installed
- meeting_refuses_when_no_pref_and_medium_not_installed_even_if_large_installed
- meeting_refuses_when_no_pref_and_no_whisper_installed
- meeting_resolves_to_large_when_pref_large_and_large_installed
- meeting_refuses_when_pref_large_but_large_not_installed_even_if_medium_installed
- meeting_resolves_to_small_when_pref_small_and_small_installed
- meeting_refuses_when_pref_small_but_small_not_installed
- meeting_asr_model_id_serde_back_compat_missing_field

Round-2 (allow-list hardening + bridge-failure path):
- allow_list_only_accepts_whisper_small_medium_large
- explicit_parakeet_meeting_pref_with_parakeet_installed_refuses_unsupported
- explicit_zipformer_meeting_pref_with_zipformer_installed_refuses_unsupported
- explicit_whisper_base_meeting_pref_with_base_installed_refuses_unsupported
- no_pref_only_parakeet_installed_refuses_preference_unset
- no_pref_only_large_installed_refuses_preference_unset
- allow_listed_models_still_resolve_to_use_when_installed
- bridge_meeting_no_context_does_not_fall_through_to_default_asr_model
- bridge_meeting_unsupported_pref_propagates_through
- non_meeting_recording_still_uses_default_asr_model_id

Helpers:
- realtime_factor_zero_audio_returns_none
- realtime_factor_sub_second_audio
- realtime_factor_typical_meeting
- display_name_short_known_models
- display_name_short_unknown_model_returns_id

ActiveRecordingContext lifecycle:
- active_recording_context_round_trip_holds_value
- meeting_session_remains_meeting_when_active_mode_changes
- non_meeting_session_remains_non_meeting_when_active_mode_changes
- meeting_model_id_persists_through_settings_change
- active_recording_context_cleared_by_helper
- every_clear_recording_flag_site_pairs_with_clear_active_recording_context

Plus the existing chunk-cap pin tests updated for 29s/27s/135 cap:
- chunk_cap_constants_match_documented_values (now 29/2/135/60)
- projected_chunks_for_five_minute_meeting_is_bounded (now 12, was 11)
- projected_chunks_for_sixty_minute_meeting_under_cap (now 134, was 129)

Selection-policy tests use a new `services::models::test_helpers::register_for_test`
helper that inserts a row + creates a directory under `models_dir`
so `ModelRegistry::is_installed` returns true without going through
the real download flow.

Out of scope for v0.15.3 (deferred to v0.15.4):
- Speaker diarisation re-introduction (architectural rework).
- DirectML re-enable for Whisper.
- Per-mode meeting-model override (the dormant
  `Mode.asr_model_id` field stays unused).
- Stripping the fp32 variants from the Medium bundle on install
  (would save ~2 GB on disk; orthogonal to v0.15.3's behaviour).

**Awaiting Jay's "approved, code"-then-"ship it" cycle: nothing
committed, nothing pushed, no build, no publish in v0.15.3 yet.**

---

## v0.15.2 Migration line-ending hotfix (2026-04-28) — beta only

What v0.15.2 changes:

- **One file restored, three new files added.**
  Files modified or created:
  1. `Utterbar Windows App/src-tauri/src/db/migrations/V0006__history_speaker_names.sql`
     — re-checked-out from git index to get rid of the
     CRLF rewrite that core.autocrlf had applied. Working
     tree now matches index (1110 bytes, LF). Same bytes
     v0.15.0 was built from.
  2. `.gitattributes` (NEW, repo root) — pins
     `*.sql text eol=lf` and `**/releases/*.md text eol=lf`.
     Stops the same class of bug from biting any future
     migration or release-note bundle. Doesn't touch any
     other file types — keeps the rule surface minimal.
  3. **NEW round-2:** `Utterbar Windows App/src-tauri/src/db/repair.rs`
     — pre-migration-runner V6 checksum repair. See the
     "Next session pickup" block at the top for the full
     contract; tl;dr: detects v0.15.1 CRLF checksum on V6
     row, requires `history.speaker_names` column as proof
     the migration's effect ran, rewrites only that one
     row's checksum to LF. Uses
     `siphasher::sip::SipHasher13` (refinery's actual
     algorithm, not SHA-256). 10 unit tests including end-
     to-end "synthetic CRLF DB → repair → refinery runs
     cleanly". `Utterbar Windows App/src-tauri/src/db/mod.rs`
     wires the helper into `Db::run_migrations` BEFORE
     the embedded refinery runner.
  4. **NEW round-2:** `siphasher = "1"` direct dep in
     `Utterbar Windows App/src-tauri/Cargo.toml`. Was
     transitively present via refinery; promoted to direct
     so the repair helper compiles cleanly without
     extern-crate gymnastics.
  5. `Utterbar Windows App/releases/0.15.2.md` (NEW) —
     release notes documenting both the byte-content
     restoration AND the round-2 checksum repair, plus
     the "reinstall v0.15.0 if you can't launch right now"
     workaround.
  6. New unit test in `commands::dictation::diarised_helpers_tests`
     pinning the SHA-256 of V0006's bundled bytes (round-1
     forward guard). Any future change fails CI.
  7. `releases/*.md` for v0.14.13/v0.15.0/v0.15.1 also
     re-checked-out (their working-tree CRLF didn't break
     anything but the new gitattributes rule covers them
     and the renormalise removed the noise).
  8. Version bump 0.15.1 → 0.15.2 in package.json,
     Cargo.toml, tauri.conf.json.

- **No app behaviour change.** Meeting Mode still uses the
  v0.15.1 chunked-no-labels path. Speaker diarisation
  stays disabled. Whisper Large preflight unchanged.
  Chunk cap still ~60 minutes.

- **Required log line** — there isn't one for v0.15.2; the
  observable success criterion is "app launches, get_phase
  returns Idle, no PANIC line in crash.log".

- **Why the bug:** refinery's `embed_migrations!`
  compile-time macro hashes each .sql file's bytes and
  compares the hash on app start against what's recorded
  in `__refinery_schema_history`. v0.15.0 was built from
  V0006 with LF (the git-stored form). Between the v0.15.0
  build and the v0.15.1 build, Windows' core.autocrlf=true
  rewrote V0006 to CRLF in the working tree. The v0.15.1
  binary baked CRLF bytes; refinery saw a different hash
  than the LF hash recorded in user DBs; raised
  ApplyMigrationFailure; main.rs `expect("bootstrap
  AppState")` panicked.

- **Why this couldn't have been caught earlier:** the
  migration file isn't touched at all in the v0.15.1
  diff. `git diff` showed clean. `git ls-files --eol`
  would have shown `i/lf w/crlf` for V0006 but that
  command isn't part of the publish ritual. Now it
  effectively is, via gitattributes — any future drift
  is corrected on next checkout.

**Diagnosis trail (preserved here as the artefact):**

> [2026-04-28T22:03:53.3816434Z] PANIC: panicked at src\main.rs:67:57:
> bootstrap AppState: applied migration V6__history_speaker_names
> is different than filesystem one V6__history_speaker_names

Hash diff confirmed by:
```
$ wc -c V0006*.sql                         # 1131 (CRLF)
$ git show HEAD:.../V0006*.sql | wc -c     # 1110 (LF)
$ git ls-files --eol -- .../V0006*.sql
i/lf    w/crlf  attr/                      # smoking gun
```

---

## v0.15.1 Diarisation rollback hotfix (2026-04-28) — beta only

What v0.15.1 changes:

- **Meeting mode no longer attempts speaker diarisation.**
  The branch in `commands::dictation::stop_recording` that
  used to try `transcribe_meeting_diarised` and fall through
  to `transcribe_chunked` on failure now goes straight to
  `transcribe_chunked`. Real-meeting v0.15.0 output proved
  the diarised path is not production-safe at any tunable
  threshold.
- **No diariser asset preflight on meeting start.** Round-2
  blocker fix: round-1 disabled the diariser on the stop
  side but left `ensure_assets_ready` running in the
  start-side meeting preflight, so users still paid the
  ~47 MB download/extract cost on first meeting for a
  feature that was switched off. Round-2 removes the
  preflight call too. First meeting on a fresh install is
  instant.
- **Whisper Large preflight stays.** The chunked transcribe
  path uses Whisper Large, so the start-side check that
  refuses meeting mode with a clear "install Whisper Large"
  message is unchanged.
- **Defence-in-depth chunk cap ≈ 60 minutes (130 chunks).**
  Round-1 had 28 min (60 chunks); round-2 raised to 56 min
  (120 chunks) after Jay flagged 28 cuts into normal 30+
  min meetings; round-3 raised again to ~60 min (130
  chunks = 60 min 40 s exact) because round-2's 56-min cap
  still rejected standard 60-min meetings on the boundary.
  Sized to comfortably accept the full 30/45/60-min meeting
  cadence with a small "I forgot to stop" shoulder, and to
  keep worst-case CPU Whisper Large transcribe under ~1
  hour (Whisper Large on CPU runs at roughly real-time).
  Documented as an explicit beta limitation in
  releases/0.15.1.md.
- **Two required log lines** on every meeting so the
  disabled state is diagnosable from a log tail:
  - on start: `meeting mode diarisation asset preflight
    disabled in v0.15.1 beta hotfix`
  - on stop: `meeting mode diarisation disabled in v0.15.1
    beta hotfix; using chunked transcript without speaker
    labels`
- Diariser code stays in tree behind `#[allow(dead_code)]`
  on `transcribe_meeting_diarised` and on `PreflightGuard`.
  Asset download/extract/wrapper code (`services::inference::
  diarize` + `diarize_assets`) is unreachable from the
  v0.15.1 meeting flow (no caller in `start_recording`,
  no caller in `stop_recording`). The whole chain stays
  in tree because v0.15.2 will reuse pieces.
- `meeting_preflight_in_progress` flag stays in `AppState`
  because the round-3-to-5 re-entry helper still loads it
  in `decide_pre_probe` / `decide_post_probe`. In v0.15.1
  it's always false; v0.15.2 will re-engage it via
  `PreflightGuard` when asset prep comes back.
- Speaker rename UI on the Meetings page stays — it's a
  pure transcript substitution and a no-op when no
  `**Speaker N:**` tokens are in the text. Old v0.14.x
  meetings render unchanged.
- **No frontend changes** beyond the version bump.
  `MeetingsRoute.tsx` and `commands::meetings.rs` stay
  exactly as v0.15.0 shipped.

What real v0.15.0 produced (preserved here as the artefact
that drove this hotfix — first 91 speakers from a 5-min
meeting):

> ```
> [00:07] **Speaker 1:** Like a fan
> [00:17] **Speaker 2:** How long you going home for?
> [00:19] **Speaker 6:** Uh, I'll get back tomorrow…
> …
> [02:39] **Speaker 51:** नल े                          ← Hindi hallucination
> [02:40] **Speaker 52:** ♪ On the day ♪
> [05:03] **Speaker 75:** Dạy dạy dạ                   ← Vietnamese
> [05:09] **Speaker 87:** やいやいやいや                   ← Japanese
> [05:15] **Speaker 91:** يا يا يا يا يا                ← Arabic
> ```

User-facing workaround for cross-language Whisper
hallucinations (documented in the release notes):

> Settings → Models → Whisper language: change from
> *Auto-detect* to **English** (or whatever you actually
> transcribe). Restart. The recogniser pins the language
> at construction time and stops auto-detect inventing
> foreign-script content for quiet/ambiguous audio.

---

## v0.15.4 rework planning — DO NOT IMPLEMENT YET

(Was labelled "v0.15.2 rework planning" → "v0.15.3 rework planning"
across the v0.15.1/v0.15.2/v0.15.3 hotfix and performance releases
that slotted in ahead of the rework. Architecture targets and
verification questions are unchanged.)

Goals (from the v0.15.1 review packet):

1. Run Whisper once per bounded long chunk, not once per
   diariser segment. One pass over the meeting buffer
   eliminates BOTH the per-segment runtime explosion AND
   the short-clip hallucination problem (Whisper sees full
   context).
2. Get word- or segment-level timestamps from Whisper.
3. Run diarisation separately on the same buffer.
4. Assign speaker labels post-transcription by mapping
   diariser segments onto the transcript's word/segment
   timestamps.
5. Never feed sub-second/noisy/music-only diariser slices
   directly into Whisper.
6. Hard guardrails: max displayed speaker count unless
   user opts into raw clusters, post-diarisation
   merge/remap of clusters, reject/treat-as-unlabelled if
   diariser returns absurd cluster counts (e.g. >10 on a
   short meeting), language/hallucination filtering based
   on actual provider capabilities, processing-time logs
   (`diarise_ms`, `whisper_ms`, `chunk_count`,
   `segment_count`, `distinct_speaker_count`).

**Open verification questions — answer THESE against the
real provider before any code is written:**

- Does `sherpa_rs::whisper::WhisperRecognizer` expose
  segment or word timestamps on its `transcribe` result?
  (Today's `AsrResult.words` is hardcoded `None` because
  the provider doesn't surface them. v0.15.2's "assign
  labels by timestamp" depends on the answer here.)
- Does `sherpa_rs::whisper` allow language to be set
  per-call, or only at recognizer construction? (Today
  `AsrOptions.language_hint` is `_opts: AsrOptions` —
  ignored. The user-facing workaround above hinges on the
  construction-time setting being correct.)
- Can the current Whisper provider be reused for
  timestamped output, or does v0.15.2 require a provider
  trait/API change?
- What's the fastest safe meeting model already installed
  / available? Whisper Medium / Small acceptable as
  optional meeting defaults — but only as **explicit user
  choice**, never as a silent forced downgrade.

**Architecture sketch (NOT a commitment):**

```
stop_recording (meeting):
  (1) transcribe_meeting_chunked()   ← bounded, single-Whisper-per-chunk
        → returns Vec<{ time_range, text, words?[] }>
  (2) diarise_full_buffer()          ← pyannote + TitaNet, threshold-based
        → returns Vec<{ time_range, cluster_id }>
  (3) post_process_clusters()        ← merge close clusters, cap to N,
                                       reject if cluster_count > 10 etc.
        → returns Vec<{ time_range, speaker_id }> (≤ N speakers)
  (4) assign_speaker_labels()        ← interval-overlap each transcript
                                       chunk against the diariser
                                       segments; label by majority overlap
        → returns final transcript with **Speaker N:** prefixes
```

(1) and (2) can run in parallel — they're independent. (3)
and (4) are post-processing on their outputs.

**Tests required for v0.15.2 (from the review packet):**

- Diariser returning N=91 clusters on a 5-min synthetic
  meeting: post-process MUST cap to ≤ N_max (configurable,
  default ~6) OR mark transcript as "unlabelled" if the
  cluster count is absurd.
- Single-pass Whisper output remains unchanged when speaker
  labels are layered on top (rendered transcript text ≡
  Whisper output text, just prefixed by `**Speaker N:**`).
- Processing-time log lines emit on every meeting.
- Chunk-cap from v0.15.1 still bounds the chunked-Whisper
  pass.
- Existing v0.15.0 unit tests for the diariser helpers
  (merge / shape / pad / dedup) still apply if the helpers
  are reused; otherwise mark obsolete.

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
