# TODO.md, Utterbar work tracker

Active working list. Keep short. When something ships, move it out of Next actions and into a one-line entry in Product backlog or remove it entirely. History lives in commits, release notes, and MONOREPO.md.

**Last updated: 2026-05-03**

---

## Current status

- **Stable channel:** v0.17.0 (admin row 70). Bundles the soaked v0.16.1–v0.16.5 beta line: launch-at-startup repair (HKCU Run entry re-asserted on every launch, heals stale paths from prior installs without the user toggling the setting), ASR prewarm + structured timing logs (cost moved off first dictation when prewarm applies), desktop UX polish (post-update relaunch opens visibly, Meeting Mode chunk progress, audio-reactive waveform, Settings sidebar search), opt-in deterministic Smart rewrite v1 (off by default, rule-based filler removal + whitespace + sentence-case + terminal full stop, Meeting Mode untouched), and the conferencing output-mute settings clarity work (clearer copy, editable Apps to silence list with validation, reset-to-defaults). Tagged `v0.17.0`. Supersedes v0.16.0.
- **Beta channel:** no current beta. The v0.16.5 row remains in admin (row 69) as a soak record for the bundle that became v0.17.0; beta channel will reopen with the next 0.17.x patch when one is cut.
- **In-tree app version:** v0.17.0 across `package.json`, `Cargo.toml`, `tauri.conf.json`.
- **Volume self-serve checkout shipped to the website** (merged into main from `feature/b2b-volume-checkout`). `/api/checkout/create-volume-session` accepts quantity 5-100, mints a licence with `max_devices = quantity`, SKU `utterbar-pro-volume`. Two-tier rate (£29 / £19 at the 25-seat boundary) lives in `App\Services\Pricing\VolumePricing`. Website-only; no desktop app change required.
- **Compare pages live** at `/compare` plus four competitor deep dives — `/compare/wispr-flow`, `/compare/dragon`, `/compare/voice-access`, `/compare/superwhisper`. Index rebuilt without the "Coming soon" placeholder. Shared `<x-compare-table>` component + per-page JSON-LD breadcrumbs and SoftwareApplication schema. Sitemap updated. Honest copy with explicit reseller pricing for Dragon (no false GBP-from-Nuance claim), soft phrasing on Voice Access vocabulary (Microsoft hasn't documented either way), and Superwhisper described as cross-platform sibling rather than Mac-only foil.
- **Homepage hero + Remotion bar parity** with the current desktop floating bar. Hero pill mock now shows three mode chips (Voice / Message / Meeting), tightened geometry closer to the real 64 px height, and demo cycle uses the real product phase strings ("● Recording" / "Inserted"). Remotion BarDriver + StatePillStatic learned `transcribing_background` so the next composition can render Meeting Mode chunk progress; the marketing pack was re-rendered from current source (12 file deltas, mostly the press stills picking up the new BarModeSwitch chip cluster). The "Comparison pages expansion" and "Website/remotion Utterbar visual refresh" backlog items are closed by this batch.
- **Website docs** updated to match v0.15.6 stable. Public copy on home, pricing, modes, troubleshooting, faq, getting-started, privacy, etc. now reflects DirectML and the 5-minute single-shot dictation cap.
- **Security headers** live: `X-Frame-Options`, `X-Content-Type-Options`, `Referrer-Policy`, `Permissions-Policy`, plus a Report-Only CSP.
- **Cloudflare Always Use HTTPS** enabled. Plain HTTP redirects to HTTPS for apex, www, and dl. subdomains.
- **TLS minimum** at the Cloudflare edge is TLS 1.2.
- **HSTS** live: `max-age=31536000; includeSubDomains` (one year). No `preload`.
- **CSP remains Report-Only.** Not enforced.
- **No active security emergency.**

---

## Next actions

In rough order. Adjust as evidence comes in.

1. **Watch for hardware-specific DirectML crashes** in the wild before building any sentinel/post-mortem detection. Step 2 cells produced zero crashes on Jay's machine; the audit-style sentinel + quarantine work is only justified if real soak data shows a crash on someone else's GPU. Until then, defence-in-depth here is overengineering.
2. **Watch v0.17.0 stable soak.** First stable bundle promoting the v0.16.1–v0.16.5 beta line. Consolidated watchpoints: (a) Smart rewrite v1 misuse — code-dictation users who enable the toggle and get keywords capitalised; AI / cloud confusion among users who read "Smart rewrite" as cloud-LLM; Meeting Mode verbatim contract on real meeting transcripts (the source-grep guard pins this in tests, but real-user soak validates the actual recorded transcripts); (b) conferencing output-mute restore correctness — output mute restores cleanly the moment dictation ends, including when a user removes a chip mid-call or changes the override list; the runtime resolver only un-mutes sessions Utterbar itself muted; (c) microphone-mute misunderstanding — watch reviews / Discord for users who read "Mute conferencing app output" and assume their microphone into the call is also muted; the v0.17.0 copy leads with "output" and explicitly says "does NOT mute your microphone inside those apps"; (d) launch-at-startup reliability on a wider install base — the repair-on-launch path heals stale Run entries from prior installs without the user toggling the setting; soak tells us whether that contract holds across the variety of real-world install paths; (e) ASR prewarm / startup regressions — confirm the background prewarm doesn't break first-dictation latency on slow boots, and that the structured timing logs surface usefully if anyone reports a regression. Hotfix to 0.17.1 beta if anything load-bearing surfaces.
3. **Pick the next product work item** from the backlog below — but note that the highest-value items (opt-in analytics, Smart rewrite local LLM, true per-app mic mute) are decision-gated, not ready-to-code. See "Decision-gated product work" below.

---

## Product backlog

One or two lines each. Expand only if the item moves to Next actions.

### Decision-gated product work

Three high-value items are blocked on a Jay decision before any code lands. None of them is a ready-to-code backlog task — investigation 2026-05-03 surfaced these as the next sensible product work, but each needs a paper-first decision so the implementation prompt is coherent.

- **Opt-in product analytics.** Needs privacy / copy / endpoint / retention decision before code. Decide what is collected (events vs counters, raw vs hashed identifiers), where it lives (utterbar.com endpoint, third-party, self-hosted), retention period, exact consent UI copy, and IT-policy interaction with the existing `BlockTelemetry` key. Today's `analytics_daily` is local-only SQLite rollups; remote analytics is a fresh channel with legal/brand surface, not just code. **Full investigation persisted at [docs/decisions/opt-in-product-analytics.md](docs/decisions/opt-in-product-analytics.md) — read that before any implementation prompt.** Branch order: copy/legal first (must merge and deploy with Jay's explicit approval), then backend, then desktop sender. Do not start Branch 1 until the approval questions in the decision doc are answered in writing.
- **Smart rewrite local LLM (future).** Needs model + bundle-vs-download + latency budget + Pro-gating + privacy-copy decision before code. The `LlamaCppProvider` stub at `services/llm/llama_cpp.rs` is a one-line selector flip away from being instantiated, but the v1 needs a chosen GGUF (1-3B class), a bundle-vs-first-run-download decision, prompt template + system prompt work, latency cancellation behaviour, and a marketing copy pivot from "no LLM rewrites of your speech" to "no cloud LLM; optional opt-in local LLM". **Full investigation persisted at [docs/decisions/smart-rewrite-local-llm.md](docs/decisions/smart-rewrite-local-llm.md) — read that before any implementation prompt.** Not approved; deterministic Smart rewrite v1 stays the default rewrite path. If approved, build after v0.17.0 stable as a parallel opt-in (never replaces v1) — beta first, promoted to stable when judgement-based readiness is met (no fixed soak duration).
- **True per-app microphone mute.** Do not build yet unless explicitly experimental with a verification / failure-warning UX. The v0.16.5 output-audio mute is shipped and honest about scope; true mic mute would require either per-app hotkey synthesis (Teams Ctrl+Shift+M, Zoom Alt+A, etc.) or UI Automation per call client. Both fail silently — the user could be left muted in a real call after a crash, which is the exact failure mode the v0.16.5 honesty pass warned against. If we ever build it, the v1 must lead with an unmissable "Utterbar may have left you muted in &lt;app&gt;" banner and ship behind an experimental flag in Settings → Advanced. **Full investigation persisted at [docs/decisions/true-per-app-microphone-mute.md](docs/decisions/true-per-app-microphone-mute.md) — read that before any implementation prompt.** Not approved; recommended default is do not build. If approved, ship as experimental beta for ONE app only (Teams or Discord), never bundle into stable without a soak window, and never reuse the v0.16.5 output-mute toggle.

### Other backlog

- **Whisper Small DML policy re-test.** Auto-Small currently routes to CPU per real-meeting evidence on Jay's machine (CPU RTF 0.190 vs DML 0.222). Re-test only if future evidence on different hardware reverses it; the policy is a one-line change in `services/inference/mod.rs::resolve_provider`.

### Product / UX ideas from Jay

- **Per-app meeting mute controls — partially shipped.** v0.16.5 beta shipped the **output-audio** half: the existing "Mute conferencing app output while recording" toggle silences what you HEAR from Teams / Slack / Discord / Zoom / Webex while dictating, and Settings → Privacy → Media now exposes an editable list of EXE basenames with built-in defaults, validation (rejects paths / command lines / quotes / wildcards / duplicates; auto-appends `.exe`; caps at 64 chars), and reset-to-defaults. Restored cleanly when dictation ends; only un-mutes sessions Utterbar itself flipped, so a pre-existing manual mute is preserved. **Not shipped: microphone mute inside the call apps.** True per-app mic mute would require simulating each app's in-call mute shortcut (Teams Ctrl+Shift+M, Discord configurable, Zoom Alt+A, Webex Ctrl+M) or driving UI Automation app-by-app — both brittle and silently fail-prone, with the "left someone muted after a crash" failure mode that the original ask explicitly wanted to avoid. Future work, only if a real user asks for it: an experimental opt-in mic-mute toggle that surfaces an unmissable banner saying "Utterbar may have left you muted in &lt;app&gt; — click to confirm restore" because the failure mode IS silent. Not scheduled.
- ~~**Smart rewrite toggle (deterministic v1).**~~ Shipped 2026-05-02 in v0.16.4 beta (`feature/smart-rewrite-deterministic`). Off-by-default `text_output.smart_rewrite` toggle in Settings → Text Output → Formatting. When enabled, runs the existing rule-based deterministic provider (filler removal, whitespace normalisation, sentence-case, terminal full stop) on regular dictation regardless of the active mode's `post_processing` setting. Local-only, no model, no remote call. Meeting Mode is gated out by a source-grep guard test. Edge-input contract pinned by `smart_rewrite_edge_input_contract` so a future tweak to the building-block helpers can't silently change what the toggle produces. UI copy is honest about the v1 limitations (best for chat / email; turn off for code).
- **Smart rewrite (local LLM, future).** A second pass of the toggle that runs a small local LLM (llama.cpp via the existing feature-gated `LlamaCppProvider` stub) to do "fix obvious wording / punctuation / ASR weirdness" beyond what deterministic rules can handle. Out of scope for v1 because it requires picking + bundling/downloading a model, prompt template work, latency budgeting, and a marketing pivot from "no LLM rewrites of your speech" to "no cloud LLM; optional opt-in local LLM". Build only when the deterministic v1's edge-cases prove the rule-based approach is insufficient AND the model-bundling / privacy-copy decisions are made explicitly.
- ~~**Meeting transcription progress bar.**~~ Shipped 2026-05-01 in v0.16.3 beta (`feat(meetings): show transcription progress`). Chunk-based: `dictation:chunk-progress` event emits `{ chunk_index, total_chunks }` once per chunk boundary; StatePill renders "Transcribing meeting · N/M" with a thin progress track that fills as chunks complete. Word-level progress is out of scope (offline recogniser doesn't expose mid-chunk callbacks).
- ~~**Settings search.**~~ Shipped 2026-05-01 in v0.16.3 beta (`feat(settings): add settings search`). Section-level search with curated keyword aliases per section ("GPU" → Advanced, "hotkey" → Shortcuts, "system audio" → Audio); Esc / X clears; empty search restores the full list. Pure helper at `src/routes/settings_search.ts`. Per-control search inside each section is a separate item if it ever surfaces.
- **Opt-in product analytics.** Add onboarding + Settings toggle for sending usage analytics to help development, default off or explicit opt-in. Must show exactly what is collected, link to the privacy docs, and respect IT policy controls. Current `BlockTelemetry` only gates local analytics; remote analytics would need separate consent, endpoint, retention policy, and legal copy.
- ~~**Comparison pages expansion.**~~ Shipped 2026-05-01 (`feature/website-compare-hero-remotion-polish`). Three new competitor pages — Dragon, Windows Voice Access, Superwhisper — alongside the existing Wispr Flow page; index rebuilt to drop the "Coming soon" framing; shared `<x-compare-table>` component + JSON-LD; sitemap updated. Talon and MacWhisper deliberately deferred — both are mentioned in alternatives lists but full pages are not justified by current traffic.
- ~~**Audio-reactive waveform.**~~ Shipped 2026-05-01 in v0.16.3 beta (`feat(bar): make waveform react to audio level`). Floating bar's seven bars are now driven by the existing `dictation:meter` event (RMS + peak) via a single rAF loop in the new `MeterBars` component; asymmetric attack/release smoother + per-bar phase offsets give a natural ribbon silhouette; reduced-motion freezes the bars at a static 12 px row. No raw audio is stored or logged.
- ~~**Website/remotion Utterbar visual refresh.**~~ Shipped 2026-05-01 (`feature/website-compare-hero-remotion-polish`). Homepage hero pill now shows three mode chips with real product phase language; Remotion BarDriver + StatePillStatic learned `transcribing_background` and the marketing pack was re-rendered from current source (12 byte-changed outputs synced to `Utterbar Website/public/`). Out of scope, deliberately: redesigning the Remotion compositions themselves, adding a new `feature-meeting-progress` composition (the source can render it; no asset is registered yet).

### B2B follow-throughs (post first-pass)

The 2026-04-27 B2B pass shipped the front door (`/business`, enquiry form, `/docs/it-deployment`, volume pricing, the original five group-policy registry keys). The 2026-05-01 v0.15.11 beta added managed fleet licence delivery via `licence.bin` and three more policy keys (`BlockHistory`, `BlockHistoryExport`, `BlockTelemetry`). The 2026-05-03 v0.15.12 beta added air-gapped activation (export request, admin upload, import licence.bin). Still open:

- **Code-signed installer at order time.** EV cert procurement, sign during `publish-release.sh`, ship to one customer without leaking into the public R2 bucket. Not a blocker for the first enquiry; only when the first one says yes.
- **Manual-mint invoice / VAT receipt.** First volume customer needs an invoice with VAT broken out. One-off Stripe Invoice for the first 1-3 deals.
- **Machine-bound licence.bin (v3 payload).** The 0.15.12 air-gapped flow records audit / `max_devices` intent, but the resulting `licence.bin` is the same bytes for every device on the licence; a user can technically copy it to a second offline machine. The 14 reserved bytes in the v2 signed payload were left there for a fingerprint hash, but the v2 verifier silently ignores them, so binding can't be load-bearing until a v3-aware desktop saturates the install base. Build only when a buyer needs hard binding and is willing to soak through the version-skew window. Spec'd on the v0.15.12 docs page; not built.
- ~~**Admin dashboard revenue counters under-report volume revenue.**~~ Shipped 2026-05-02 (`feature/admin-volume-revenue-accounting`). Migration adds nullable `amount_pence` + `currency` to `licences`; the Stripe webhook (single-user and volume) populates them at mint time, manual admin mints stay null. The Gross / Net tiles now SUM the column, and a new "Untracked rows" tile flags how many rows lack an amount (admin manual mints + any pre-2026-05-02 Stripe row, which we deliberately do not backfill).

### Pre-existing open issues

- ~~**Launch-at-startup reliability.**~~ Resolved by the v0.16.1 repair-on-launch path; Jay confirmed 2026-05-03 the app now opens on Windows restart/startup. The repair-on-launch hook in `main.rs` re-asserts the HKCU Run entry against the current exe path on every launch, healing stale entries from prior installs without the user having to toggle the setting.
- **Ed25519 sandbox key sunset.** Live key is v2; v1 sandbox key still accepted transitionally. Sunset planned ~6 weeks after v0.9.0 saturates.
- ~~**Legal pages refresh.**~~ Shipped 2026-05-03 (`feature/legal-pages-refresh-may2026`). Privacy and Terms now match the v0.16.x B2B reality: managed `licence.bin` and air-gapped activation add no outbound calls, volume / multi-seat licences create one record per order with `max_devices = quantity`, Business enquiry form retention mirrors the Contact form, and the volume licence-grant shape is clarified. EULA placeholder untouched. Both pages stamped "Last updated: 3 May 2026".
- **Admin dashboard cohesion.** Staged `Admin/*` controllers + views need a navigation pass once they ship.
- **Remotion video quality pass.** Re-render compositions against current `FloatingBar.tsx`; consider R2-backed CDN delivery.
- **4.9 GB idle RAM.** Never reproduced on v0.9.0+. Investigate only if it surfaces again.
- **Microsoft Store cert path.** v0.12.0 submission tests whether the existing build passes Store cert as-is. If cert rejects, build a Store-specific variant that compiles the in-app updater out via a feature flag.

### Diarisation (still deferred)

Speaker labels remain off in shipping builds. The architectural rework (transcribe whole buffer once, diarise separately, assign speaker labels by interval-overlap on word/segment timestamps) is on the parking lot. **Do not start diarisation work on top of unstable ASR / runtime work.** v0.15.6 just promoted to stable; let the DirectML soak settle and the open verification questions about `sherpa_rs::whisper` timestamps + per-call language override get answered first.

---

## Security audit status

External audit ran late April 2026. Findings sorted into three buckets.

### Shipped

- Browser-hardening headers (`X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, `Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=()`).
- Content-Security-Policy in **Report-Only** mode.
- Cloudflare "Always Use HTTPS" enabled.
- TLS minimum at the edge: TLS 1.2.
- HSTS live: `Strict-Transport-Security: max-age=31536000; includeSubDomains` (one year). No `preload`.

### Deferred

- **Origin lockdown for `178.62.6.215`.** Parked hardening, not active work, not urgent. Stripe webhook HMAC, licence Ed25519, admin bearer token, and per-route Laravel throttles all hold on the direct path; nothing's currently exploitable. If we ever pick this up, it's operational work (not a repo change): Cloudways firewall allow-listing Cloudflare IPs, Cloudflare Authenticated Origin Pulls + nginx mTLS, and SSH (port 22) restriction tracked separately. Trigger to revisit: a real bypass attempt in logs, an IP-leak incident, or a quiet maintenance window.
- **CSP enforcement / inline-script nonce refactor.** Current Report-Only with `'unsafe-inline'` is hygiene-level CSP. Real win comes from refactoring 9 inline `<script>` blocks to use a per-request nonce, then dropping `'unsafe-inline'`. Moderate refactor across ~14 view files plus middleware nonce plumbing. **Do not flip CSP to enforcement until the nonce refactor is in.**
- **SSH IP restriction (port 22).**
- **CAA records.** Pin the CAs Cloudflare actually uses to prevent mis-issuance.
- **DMARC reporting address (`rua=`).** Current record is `v=DMARC1; p=quarantine` with no inbox receiving alignment-failure reports.

### Accepted / false positive

- **`XSRF-TOKEN` not HttpOnly** is intentional Laravel behaviour. The session cookie (`utterbar-session`) is `secure; httponly; samesite=lax`; flipping HttpOnly on `XSRF-TOKEN` would break every form on the site.
- **Cloudflare edge port 8080** is documented Cloudflare global behaviour (alternate-HTTPS proxy port set), not site config. Origin port 8080 is closed.
- **DKIM** exists at `resend._domainkey.utterbar.com`. Scanner only checked default selectors.
- **"No rate limiting"** is misleading: every sensitive Laravel route is throttled (`/api/contact`, `/sign-in`, `/api/licence/*`, `/api/admin/*`, `/api/checkout/*`, `/api/updater/check`, `/tip/*`, `/download/*`, etc.). See `routes/web.php`.
- **Public support email** is intentional.

---

## Cross-side contract status

Verified clean as of 2026-04-30. No outstanding drift. `publish-release.sh` uploads to the versioned R2 key, server recomputes SHA-256 and verifies updater signature, beta and stable rows live in `(version, channel)` uniqueness namespaces. Full notes in `RELEASE_RUNBOOK.md` and `MONOREPO.md §10`.

---

## Session caveats to carry forward

- **Monorepo:** `c:\Users\jaypa\Documents\GitHub\Utterbar\` is one repo. `Utterbar Windows App/` and `Utterbar Website/` sit inside. Remote: `badoomtch/Utterbar-App` private.
- **App version vs website version.** Website deploys continuously; only the desktop installer has a semver. `config('site.version')` is informational.
- **Jay tests every app build before we ship.** No installer goes to R2 without explicit approval.
- **PHP tests are not in CI.** Re-run on the dev machine before deploying the website.
- **Versioning convention** (per `CLAUDE.md §5.7`): minor (`0.x.0`) = mainline / stable, patch (`0.x.y`, y > 0) = beta channel only. Ship features through beta soak; do not skip straight to mainline.

---

*End of TODO. Edit freely. Move shipped work out, don't let it accumulate.*
