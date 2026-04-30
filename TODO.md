# TODO.md — Utterbar work tracker

One place to see what's done, what's staged, what's live, and what's
still open. Kept at the monorepo root so both sub-projects are visible
from one glance. Update at the end of each session.

**Last updated: 2026-04-30 (post-stable-promotion)** — v0.15.6 promoted to stable on 2026-04-30 after compressed QA on Jay's machine plus a v0.12.0 → v0.15.6 upgrade test on a friend's machine. Stable row 55 was created with the original beta-only notes, then deleted with `delete_file=0` and re-published as row 56 with consolidated v0.12.0 → v0.15.6 release notes. Beta row 54 untouched. Same binary, same SHA across all three rows. Soak floor was 2026-05-05; promotion brought forward by five days based on QA results. The release-note consolidation is intentional because stable users skipped every beta from v0.13 onward.

---

## Where we are right now

### Channels

| Channel | Version | Admin row | Notes |
| --- | --- | --- | --- |
| **Stable** | **v0.15.6** | id=56 (2026-04-30, with consolidated notes) | Served on `/download` with `latest` pill. Promoted early after compressed QA. Row 55 (also stable, original beta-only notes) was deleted with `delete_file=0` and superseded by 56. No `--announce` on either stable publish (subscriber list was empty either way, but principle of least surprise). SHA-256 `39ed250fa3d0ee94f5fb7c3d695e02693cfcdbf3766c94b0c4ce9cf9424885f9`. |
| **Beta** | **v0.15.6** | id=54 (2026-04-30) | Beta row preserved. Same SHA as stable. CDN URL: https://dl.utterbar.com/releases/0.15.6/Utterbar-Setup-0.15.6.exe. |

### In-tree app version
`v0.15.6` across [package.json](Utterbar%20Windows%20App/package.json), [Cargo.toml](Utterbar%20Windows%20App/src-tauri/Cargo.toml), [tauri.conf.json](Utterbar%20Windows%20App/src-tauri/tauri.conf.json). Matches the published beta binary.

### What v0.15.6 actually changed
- **DirectML execution provider, embedded.** Vendored sherpa-onnx + ONNX Runtime DLL bundle at [src-tauri/vendor/sherpa-onnx/](Utterbar%20Windows%20App/src-tauri/vendor/sherpa-onnx/) rebuilt against ORT 1.14.1 + DirectML 1.15.0. `DirectML.dll` (~18.5 MB) added to the runtime payload; DML EP now exposed by `onnxruntime.dll` (no satellite `_providers_dml.dll` — ORT 1.14.1 statically embeds it).
- **Whisper CPU-force lifted.** The v0.8.7 mitigation that pinned every Whisper transcribe to CPU regardless of user pref is gone. Step 2 8/8 PASS confirmed the original DML failure mode no longer reproduces against the new bundle. Step 2 cells now live as a permanent `#[ignore]` regression suite at [src-tauri/src/services/inference/step2_dml.rs](Utterbar%20Windows%20App/src-tauri/src/services/inference/step2_dml.rs).
- **Model-aware Auto policy.** When user picks Auto and a hardware GPU is present:
  - Whisper Medium → DirectML
  - Whisper Large v3 → DirectML
  - Parakeet TDT v3 int8 → DirectML
  - Whisper Small → **CPU** (Step 2 short-clip evidence: DML 0.78× CPU; real-meeting evidence on Jay's machine: Small CPU 5:43 → RTF 0.190 vs Small DML 9:27 → RTF 0.222 — DML penalty for Small persists into long audio).
- **Explicit DirectML overrides** the auto-Small CPU rule. If user picks DirectML in Settings → Advanced, every model goes to DML.
- **Embedded-EP runtime detection.** `services/gpu.rs::runtime_capabilities()` now requires both `DirectML.dll` *and* a PE-export check on `onnxruntime.dll` for `OrtSessionOptionsAppendExecutionProvider_DML` (via `goblin`). Legacy satellite layout still detected for backwards compat.
- **Telemetry on every transcribe.** `dml_available` (bool) and `provider_policy_reason` (string, nine pinned values: `user_explicit_cpu`, `user_explicit_directml`, `user_explicit_cuda`, `dml_ep_unavailable`, `cuda_ep_unavailable`, `auto_dml_speedup`, `auto_small_prefers_cpu_short_clip`, `auto_cpu_no_dml_runtime`, `auto_cpu_no_hardware_gpu`) — pinned by resolver tests so production logs are diagnosable.
- **Settings → Advanced honest copy** when DML is actually available; v0.15.5 inactive notice still fires for CUDA (we don't bundle it).

### Pre-publish QA fixes that landed in the same commit

Two issues surfaced during real-meeting validation; both fixed before the installer was rebuilt and published:

- **`services/llm/deterministic.rs` UTF-8 panic.** `replace_regex` was byte-indexing into a precomputed `to_lowercase()` haystack. A real Parakeet dictation containing `£200` panicked at `deterministic.rs:123:31` (`start byte index N is not a char boundary; it is inside '£'`). Fix: drop the lower-case precomputation, slice the original haystack via `str::get(idx..after)`, compare with `eq_ignore_ascii_case` (filler list is ASCII). Five new tests pin the exact crash repro plus CJK / emoji / diacritic cases.
- **CUDA `provider_policy_reason` typo.** Explicit-CUDA branch reported `"user_explicit_directml"`. Fixed to `"user_explicit_cuda"`. Resolver test strengthened.

### Stable promotion (closed)

Promoted on 2026-04-30 after compressed QA. Evidence:

- A → K manual checklist passed on Jay's machine (Parakeet £-sign dictation clean, Whisper Small Auto routed to CPU as designed, Whisper Medium DirectML produced sane transcripts, Parakeet DirectML clean across 5 short dictations, settings persisted across restarts, beta/stable feeds gated correctly, no panic in crash.log, no new crash dumps).
- v0.12.0 → v0.15.6 upgrade tested on a second machine (Jay's friend); installed clean, SmartScreen warning expected for an unsigned installer.
- Stable feed verified returning v0.15.6 with matching SHA after row 56 was published; beta feed (row 54) preserved.
- Notes consolidated for stable users. Original beta-only notes (row 55) deleted with `delete_file=0` and superseded by row 56 with the v0.12.0 → v0.15.6 recap.

Caveats users may hit, all called out in the consolidated notes:

- **Cold DirectML first-run cost.** Parakeet adds roughly 20-25 s before the first transcript appears after launch; Whisper Medium / Large can take 30-60 s of graph compilation on the first meeting in a session. Subsequent recordings in the same session reuse the cached recogniser. Quitting loses the cache.
- **5-minute single-shot dictation cap** (carried forward from v0.12.1). Regular dictation refuses inputs longer than 5 minutes with a clear error. Meeting Mode chunks internally and handles up to about an hour. The streaming-recogniser rebuild that lifts this cap is a separate future work item.
- **Speaker labels remain off.** Earlier diarisation attempts produced wrong-shaped output. Transcripts come back timestamped without `**Speaker N:**` labels. The rework is planned for a future release.
- **In-app Settings > Updates page on the v0.15.6 binary still shows the original baked-in beta-only notes** because `commands/release_notes.rs` uses `include_str!` against `releases/<CARGO_PKG_VERSION>.md` at compile time. The website (`/changelog`, `/download`) shows the consolidated stable-promotion notes. The in-app notes will catch up naturally on the next release.

If a regression needs code, cut a fresh `0.15.7` beta. Never re-publish `0.15.6` with new bytes; that would invalidate the SHA on already-installed clients on every channel.

---

## Open product issues

### Follow-up — 302s transcription cap UX

Non-meeting (single-shot) dictation refuses recordings longer than 302s with audio loss. Surfaced 2026-04-29 when Jay forgot to switch to Meeting Mode. **Out of scope for v0.15.6.** Separate ticket post-release. Options to weigh: auto-stop with a "Recording auto-stopped at 5 min — switch to Meeting Mode for longer dictation" toast, or surface a visible 5:00 countdown in the floating bar. Pick one when this comes up next; both are small.

### Follow-up — Whisper Small DML on long audio

Auto-Small currently routes to CPU based on both 30-second-clip evidence (DML 0.78× CPU) and a real 9:27 meeting on Jay's machine (DML RTF 0.222 vs CPU RTF 0.190). If a future longer-audio test on different hardware shows DML overtakes CPU for Small, flip the resolver to `auto_dml_speedup` for Small. Decision not blocking; the policy is a one-line change in [services/inference/mod.rs::resolve_provider](Utterbar%20Windows%20App/src-tauri/src/services/inference/mod.rs).

### Follow-up — Sentinel + post-mortem crash detection

Step 2 produced zero crashes across 8 cells. Defence-in-depth (sentinel file + quarantine + post-mortem detection) deferred unless real soak data shows hardware-specific DML failures. If the v0.15.6 soak surfaces a crash on someone else's GPU, this becomes the v0.15.7 priority.

### Follow-up — Output validation that drops text

Designed-only since v0.15.5. Foreign-script and empty-output filters that would prevent Whisper hallucinations from reaching the user. Not blocking. Unrelated to DirectML.

### B2B follow-throughs (post first-pass)

The 2026-04-27 B2B pass shipped the front door — `/business` landing page, enquiry form, IT-deployment doc, volume-pricing block — and a small group-policy registry surface (3 pinnable values: `UpdateChannel`, `CheckUpdatesAutomatically`, `BlockModelDownloads`). What still needs doing before the first real volume sale closes cleanly:

- [ ] **Code-signed installer at order time.** Public installer is unsigned (SmartScreen warnings). Required before any Business / Enterprise rollout. Process: buy/load EV cert on demand, sign during `publish-release.sh`, ship signed installer to one customer without leaking into the public R2 bucket. **Not a blocker for the first enquiry — only when the first enquiry says yes.**
- [ ] **Bulk licence delivery for fleets.** Today's per-buyer email + magic link doesn't scale. A 50-seat hand-mint needs one artefact (signed `licence.bin` + documented drop path) the IT team can deploy across machines. Spec'd, not built.
- [ ] **Manual-mint invoice / VAT receipt.** Hand-minted Business licences currently have no paper trail. First volume customer will need an invoice with VAT broken out before they pay. One-off Stripe Invoice from the dashboard for the first 1–3 deals; revisit if it becomes routine.
- [ ] **Volume self-serve checkout (later).** Manual mint is fine for the first 3–5 deals, painful at 20. Extend Stripe checkout with a quantity selector mapping to `max_devices` when there's a queue.
- [ ] **More policy keys waiting on Settings work.** `BlockTelemetry`, `BlockHistory`, `BlockHistoryExport` would all be high-value pinnable values for compliance buyers, but the underlying user-facing toggles don't exist yet. Build Settings → Privacy first, then add overrides.
- [ ] **Air-gapped activation.** Documented as "on request" only. Build when a real buyer asks: a `licence_request.bin` writer in the app, a paired admin endpoint that returns a signed `licence_response.bin`, and a one-button ingest flow in the app's Account UI. Each leg is small; combined sub-session of work.

### Pre-existing open issues

- [ ] **Ed25519 sandbox key sunset.** Live key is v2 (`LICENCE_PAYLOAD_VERSION=2`); v1 sandbox key still accepted transitionally so pre-v0.9.0 caches keep verifying. Sunset planned ~6 weeks after v0.9.0 saturates the install base. Rollout plan needs to allow existing v0.7.x installs to keep validating.
- [ ] **Legal pages refresh.** `legal/privacy.blade.php` and `legal/terms.blade.php` need to mention purchase records, licence keys, and device fingerprints (Stripe is live; previous text described the site as storing only waitlist emails). Partial pass shipped earlier; needs final review.
- [ ] **Admin dashboard cohesion.** Staged `Admin/*` controllers + views (contacts, licences, subscribers) need a navigation pass once they ship. The `admin-nav` component is the starting point.
- [ ] **Remotion video quality pass.** Re-render compositions against current [FloatingBar.tsx](Utterbar%20Windows%20App/src/routes/FloatingBar.tsx) imports; consider R2-backed CDN delivery.
- [ ] **4.9 GB idle RAM** — never reproduced on v0.9.0+. Treat as folklore until reproduced. If it surfaces, attach VMMap to the running wry process and check heap commit by region. See [MONOREPO.md §10.6](MONOREPO.md).
- [ ] **Microsoft Store cert.** v0.12.0 submission status: testing whether the existing build passes Store cert as-is with the in-app updater intact and a "leave it alone, we know" cert note. If cert rejects, build a Store-specific variant that compiles the in-app updater out via a feature flag. Tracked separately from direct-channel releases. See [CLAUDE.md §5.7](CLAUDE.md).

---

## Diarisation — still deferred

Same shape as before. Per the v0.15.1 review notes, the design is "transcribe whole buffer once, diarise separately, assign speaker labels by interval-overlap on word/segment timestamps". Don't start until v0.15.6 has cleared the soak window AND the stable promotion has happened — diarisation work on top of an unsoaked DirectML runtime is the wrong order.

Open verification questions to answer against the real provider before any diarisation code is written:

- Does `sherpa_rs::whisper::WhisperRecognizer` expose segment or word timestamps on its `transcribe` result? (Today's `AsrResult.words` is hardcoded `None` because the provider doesn't surface them. The "assign labels by timestamp" approach depends on the answer.)
- Does `sherpa_rs::whisper` allow language to be set per-call, or only at recognizer construction?
- Can the current Whisper provider be reused for timestamped output, or does the rework require a provider trait/API change?

Architecture sketch (NOT a commitment):

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
                                       chunk against the diariser segments;
                                       label by majority overlap
        → returns final transcript with **Speaker N:** prefixes
```

(1) and (2) can run in parallel. (3) and (4) are post-processing.

---

## Recently shipped (chronological — most recent first)

Compressed history. Full detail in commit messages, release notes (`Utterbar Windows App/releases/<v>.md`), and the relevant memory files.

- **v0.15.6 (beta, 2026-04-30, admin row 54)** — DirectML execution provider, Whisper CPU-force lifted, model-aware Auto policy, embedded-EP runtime detection, `dml_available` + `provider_policy_reason` telemetry, deterministic UTF-8 £-panic fix, CUDA telemetry typo fix.
- **v0.15.5 (beta, 2026-04-29)** — Provider truthfulness pass: Settings → Advanced now says "Selected, but currently inactive" when the GPU acceleration radio is set to a provider the runtime can't actually serve. Unblocked the v0.15.6 honest-copy work.
- **v0.15.4 (beta, 2026-04-29)** — Whisper Small as Meeting Mode default. Performance improvement for the most common meeting use case (sub-30-min, English-only).
- **v0.15.3 (beta, 2026-04-29)** — Meeting Mode performance + allow-listed model picker. Whisper Medium added to the catalogue (SHA-pinned). `meeting_asr_model_id` setting with three radios (Fast / Balanced / Best accuracy). Allow-list (Small / Medium / Large v3) — meeting mode refuses any model outside the list with a clear user-facing error rather than silently falling back to Parakeet. `ActiveRecordingContext` binds the recording session at start-time. Chunk size 30s → 29s (avoid sherpa truncation), cap retuned 130 → 135 chunks (~60 min 47 s of audio accepted).
- **v0.15.2 (beta, 2026-04-28)** — Migration line-ending hotfix. `core.autocrlf=true` rewrote V0006 between v0.15.0 and v0.15.1, refinery's checksum guard refused to start v0.15.1 on machines that had run v0.15.0. Fix: `.gitattributes` pinning `*.sql text eol=lf`, V0006 renormalised, plus a one-shot `db::repair` module that recognises the v0.15.1 CRLF checksum on V6 + presence of the `history.speaker_names` column and rewrites only that row's checksum to the LF value via `siphasher::sip::SipHasher13` (refinery's actual algorithm). 10 new repair tests.
- **v0.15.1 (beta, 2026-04-28)** — Diarisation rollback hotfix. v0.15.0 produced 91 speaker clusters on a 5-min meeting with cross-language hallucinations (Hindi, Vietnamese, Japanese, Arabic — Whisper auto-detect on noisy/short slices); diariser disabled, meeting mode goes straight to chunked transcribe with a ~60-min cap. Diariser code stays in tree behind `#[allow(dead_code)]` for the rework.
- **v0.15.0 (beta, 2026-04-28)** — Speaker diarisation first attempt. Pulled by v0.15.1 hotfix (see above). 52 new tests across 5 review rounds (race conditions on meeting-preflight, generic start-pending, non-atomic-snapshot — all pinned). Pyannote + TitaNet model SHAs pinned in [services::inference::diarize_assets](Utterbar%20Windows%20App/src-tauri/src/services/inference/). Threshold tuning + silero-vad pre-pass deferred to the rework.
- **v0.14.13 (security)** — `open_path` injection fixed (ShellExecuteW + canonicalised allow-list), release publishing hardened (server-side SHA recompute, signature verification over computed SHA, `confirm_stable_announce` flag), Markdown XSS, model deletion path safety. 23 publish-validation tests + 5 `open_path` tests + 9 markdown XSS tests + 4 `safe_model_path` tests, all passing.
- **v0.14.12 (beta)** — Mic-routing fix (Scarlett audio interface verified by Jay).
- **v0.12.0 → v0.12.1 (stable + beta soak, 2026-04-27)** — v0.12.0 was the new mainline that bundled v0.11.x betas (per-app auto-activation, .srt/.vtt subtitle export, history → vocabulary correction loop, history filters) plus v0.12.0 additions (B2B surface, group-policy registry overlay). v0.12.1 beta soak shipped two safety patches (recording cap lifted to 8h, transcribe-input ceiling at 5 min returning a clear user-facing error) in response to a 40-min meeting crash where sherpa's offline decoder allocated O(duration) memory and OOM'd.

Earlier history (v0.7.0 — v0.11.x) lives in [MONOREPO.md §10](MONOREPO.md) drift entries and the per-version release notes.

---

## Cross-side contract status

Verified clean as of 2026-04-30. No drift outstanding:

- [publish-release.sh](Utterbar%20Windows%20App/scripts/publish-release.sh) uploads to the versioned R2 key `releases/<v>/Utterbar-Setup-<v>.exe` only. POST body explicitly includes `r2_key`. The website-side admin API's `file_not_in_bucket` fallback also defaults to the versioned key.
- The publish endpoint runs server-side SHA recompute (canonical hash from R2), updater-signature verification (over the computed SHA), and a `confirm_stable_announce` guard for stable+announce. Persisted SHA is the server-computed value, not the script-pasted one.
- `--announce` triggers email-out to confirmed `update_subscribers`; `--tag "latest"` displays the cyan pill; `--beta` flips channel. Beta and stable rows live in `(version, channel)` uniqueness namespaces, so the same version string can exist on both — that's the promote-beta-to-stable mechanism.

If anything in [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md) or [Utterbar Windows App/RELEASE.md](Utterbar%20Windows%20App/RELEASE.md) contradicts the above, fix the docs in the same PR.

---

## Session caveats to carry forward

- **Monorepo:** `c:\Users\jaypa\Documents\GitHub\Utterbar\` is a single git repo. `Utterbar Windows App/` and `Utterbar Website/` sit inside. Remote: `badoomtch/Utterbar-App` private.
- **App version ≠ website version.** Website deploys continuously; only the desktop installer has a semver. The `config('site.version')` value is informational and tracks the last shipped app version only.
- **Jay tests every app build before we ship.** No installer goes to R2 until Jay explicitly approves.
- **PHP tests are not in CI.** Re-run on the dev machine before deploying the website.
- **Versioning convention** (effective from v0.12.0; see [CLAUDE.md §5.7](CLAUDE.md)): minor (`0.x.0`) = mainline / stable, patch (`0.x.y`, y > 0) = beta channel only. Don't ship features straight to mainline without a beta soak.

---

*End of TODO. Edit freely. Don't let stale entries pile up.*
