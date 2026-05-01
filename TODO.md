# TODO.md, Utterbar work tracker

Active working list. Keep short. When something ships, move it out of Next actions and into a one-line entry in Product backlog or remove it entirely. History lives in commits, release notes, and MONOREPO.md.

**Last updated: 2026-05-01**

---

## Current status

- **Stable channel:** v0.15.6 (admin row 56). DirectML execution provider live, Whisper Small stays on CPU on Auto.
- **Beta channel:** v0.15.10 (admin row 60). All three ASR surfaces now run the shared output validator: regular dictation (since 0.15.8), file transcription (since 0.15.9), and Meeting Mode per-chunk (new in 0.15.10). Meeting chunks drop on empty / whitespace / single-punct / 10+-repeated-punct, observe-only on repeated-token loops, and skip foreign-script + known-hallucination rules. Whole meetings still rely on the existing empty-transcript guard.
- **In-tree app version:** v0.15.10 across `package.json`, `Cargo.toml`, `tauri.conf.json`.
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

1. **B2B sprint: fleet licence delivery + IT policy controls.** Make Utterbar credible for managed deployment. Two chunks: (a) admin-downloadable signed `licence.bin` that the desktop app reads from `%APPDATA%\Utterbar\licence.bin` and treats as the active Pro licence, with managed wins over user-entered; (b) three new group policy keys (`BlockHistory`, `BlockHistoryExport`, `BlockTelemetry`) under the existing `HKLM\SOFTWARE\Policies\Utterbar` registry path. Plus IT-deployment docs. Out of scope for this sprint: volume self-serve checkout, air-gapped activation, code-signed customer builds, Stripe invoice automation.
2. **Promote the validation work to stable.** Once 0.15.8 / 0.15.9 / 0.15.10 soak shows the noticed-drop rules don't false-positive on real users (target: 1-2 weeks of beta with `output_validation: dropped` log lines reviewed), bundle as 0.16.0 stable. Single-version cut bringing all three ASR-surface validators to the stable channel.
3. **Watch for hardware-specific DirectML crashes** in the wild before building any sentinel/post-mortem detection. Step 2 cells produced zero crashes on Jay's machine; the audit-style sentinel + quarantine work is only justified if real soak data shows a crash on someone else's GPU. Until then, defence-in-depth here is overengineering.
4. **Pick the next product work item** from the backlog below.

---

## Product backlog

One or two lines each. Expand only if the item moves to Next actions.

- **Whisper Small DML policy re-test.** Auto-Small currently routes to CPU per real-meeting evidence on Jay's machine (CPU RTF 0.190 vs DML 0.222). Re-test only if future evidence on different hardware reverses it; the policy is a one-line change in `services/inference/mod.rs::resolve_provider`.

### B2B follow-throughs (post first-pass)

The 2026-04-27 B2B pass shipped the front door (`/business`, enquiry form, `/docs/it-deployment`, volume pricing, three group-policy registry keys). What's still open before the first real volume sale closes:

- **Code-signed installer at order time.** EV cert procurement, sign during `publish-release.sh`, ship to one customer without leaking into the public R2 bucket. Not a blocker for the first enquiry; only when the first one says yes.
- **Bulk licence delivery for fleets.** Single signed `licence.bin` artefact + documented drop path under `%APPDATA%\Utterbar\` for IT teams. Spec'd, not built.
- **Manual-mint invoice / VAT receipt.** First volume customer needs an invoice with VAT broken out. One-off Stripe Invoice for the first 1-3 deals.
- **Volume self-serve checkout.** Extend Stripe checkout with a quantity selector mapping to `max_devices`. Build only when there's a queue.
- **More group-policy keys** (`BlockTelemetry`, `BlockHistory`, `BlockHistoryExport`) after `Settings > Privacy` exists in the desktop app.
- **Air-gapped activation.** Build only when a real buyer asks. `licence_request.bin` writer + paired admin endpoint + one-button ingest.

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
