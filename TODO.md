# TODO.md, Utterbar work tracker

Active working list. Keep short. When something ships, move it out of Next actions and into a one-line entry in Product backlog or remove it entirely. History lives in commits, release notes, and MONOREPO.md.

**Last updated: 2026-05-05**

---

## Current status

- **Stable channel:** v0.16.0 (admin row 63). Bundles the v0.15.7-v0.15.12 beta line: auto-stop near the 5-minute single-shot cap, ASR output validation across regular dictation / file transcription / Meeting Mode chunks, managed fleet licence delivery via `licence.bin`, three new IT policy keys (BlockHistory, BlockHistoryExport, BlockTelemetry), volume self-serve checkout with two-tier rate, admin polish for B2B rows, honest revenue accounting (per-row amount_pence), and air-gapped activation request/import. Tagged `v0.16.0`. Supersedes v0.15.6.
- **Beta channel:** v0.16.0 (admin row 64). Same bytes as stable; published to beta in lockstep so beta users are not stranded on 0.15.12.
- **In-tree app version:** v0.16.0 across `package.json`, `Cargo.toml`, `tauri.conf.json`.
- **Volume self-serve checkout shipped to the website** (merged into main from `feature/b2b-volume-checkout`). `/api/checkout/create-volume-session` accepts quantity 5-100, mints a licence with `max_devices = quantity`, SKU `utterbar-pro-volume`. Two-tier rate (£29 / £19 at the 25-seat boundary) lives in `App\Services\Pricing\VolumePricing`. Website-only; no desktop app change required.
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
2. **Watch v0.16.0 stable soak.** First stable from a wide bundle (six betas worth of work in one promote). Particular attention on: per-chunk Meeting Mode validation false-positives, the 4:55 auto-stop interaction with mid-sentence speech, managed-licence + air-gapped flows on real customer hardware. Hotfix to 0.16.1 if anything load-bearing surfaces.
3. **Fix launch-at-startup reliability.** User reports the app does not launch after restarting Windows even though the setting is enabled. Treat as a bug: inspect startup registration, installer behaviour, and whether the setting writes the expected Windows startup entry.
4. **Updater relaunch should open the app, not hide to tray.** After an update and relaunch, bring the app/floating bar/settings surface visibly back instead of starting minimised/closed-to-tray.
5. **Pick the next product work item** from the backlog below.

---

## Product backlog

One or two lines each. Expand only if the item moves to Next actions.

- **Whisper Small DML policy re-test.** Auto-Small currently routes to CPU per real-meeting evidence on Jay's machine (CPU RTF 0.190 vs DML 0.222). Re-test only if future evidence on different hardware reverses it; the policy is a one-line change in `services/inference/mod.rs::resolve_provider`.

### Product / UX ideas from Jay

- **Per-app meeting mute controls.** Let users choose target apps (Teams, Discord, Zoom, etc.) and automatically mute/unmute themselves there while Utterbar is recording. High-value but integration-heavy: needs per-app detection, hotkey/API strategy per app, visible fail-safe state, and careful testing so we never leave someone muted.
- **Smart rewrite toggle.** Optional slower post-processing pass that lightly fixes obvious sentence issues before paste. Must be opt-in, visibly slower, and conservative enough not to rewrite meaning. Likely local deterministic first; LLM-style rewrite only if privacy/cost/product stance is explicit.
- **Meeting transcription progress bar.** Show progress while "Transcribing Meeting" so long recordings feel alive. Should report chunk progress from Rust rather than fake a timer.
- **Settings search.** Add search/filter to Settings so users can find controls by keyword. Straightforward frontend work if settings sections already have stable labels.
- **Opt-in product analytics.** Add onboarding + Settings toggle for sending usage analytics to help development, default off or explicit opt-in. Must show exactly what is collected, link to the privacy docs, and respect IT policy controls. Current `BlockTelemetry` only gates local analytics; remote analytics would need separate consent, endpoint, retention policy, and legal copy.
- **Comparison pages expansion.** `/compare` currently has one real comparison and placeholder content. Add more competitor pages and tighten the index so it is not thin/placeholder-looking.
- **Audio-reactive waveform.** Floating bar waveform should move only when audio is detected instead of playing a constant decorative animation. Use existing input level/meter data where possible.
- **Website/remotion Utterbar visual refresh.** The interactive Utterbar hero and Remotion compositions should match the current real floating bar, including the mode controls.

### B2B follow-throughs (post first-pass)

The 2026-04-27 B2B pass shipped the front door (`/business`, enquiry form, `/docs/it-deployment`, volume pricing, the original five group-policy registry keys). The 2026-05-01 v0.15.11 beta added managed fleet licence delivery via `licence.bin` and three more policy keys (`BlockHistory`, `BlockHistoryExport`, `BlockTelemetry`). The 2026-05-03 v0.15.12 beta added air-gapped activation (export request, admin upload, import licence.bin). Still open:

- **Code-signed installer at order time.** EV cert procurement, sign during `publish-release.sh`, ship to one customer without leaking into the public R2 bucket. Not a blocker for the first enquiry; only when the first one says yes.
- **Manual-mint invoice / VAT receipt.** First volume customer needs an invoice with VAT broken out. One-off Stripe Invoice for the first 1-3 deals.
- **Machine-bound licence.bin (v3 payload).** The 0.15.12 air-gapped flow records audit / `max_devices` intent, but the resulting `licence.bin` is the same bytes for every device on the licence; a user can technically copy it to a second offline machine. The 14 reserved bytes in the v2 signed payload were left there for a fingerprint hash, but the v2 verifier silently ignores them, so binding can't be load-bearing until a v3-aware desktop saturates the install base. Build only when a buyer needs hard binding and is willing to soak through the version-skew window. Spec'd on the v0.15.12 docs page; not built.
- ~~**Admin dashboard revenue counters under-report volume revenue.**~~ Shipped 2026-05-02 (`feature/admin-volume-revenue-accounting`). Migration adds nullable `amount_pence` + `currency` to `licences`; the Stripe webhook (single-user and volume) populates them at mint time, manual admin mints stay null. The Gross / Net tiles now SUM the column, and a new "Untracked rows" tile flags how many rows lack an amount (admin manual mints + any pre-2026-05-02 Stripe row, which we deliberately do not backfill).

### Pre-existing open issues

- **Ed25519 sandbox key sunset.** Live key is v2; v1 sandbox key still accepted transitionally. Sunset planned ~6 weeks after v0.9.0 saturates.
- **Legal pages refresh.** `legal/privacy.blade.php` and `legal/terms.blade.php` still need a final review against the live Stripe / licence / device-fingerprint reality.
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
