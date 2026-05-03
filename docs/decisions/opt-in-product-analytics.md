# Opt-in Product Analytics Decision

**Status:** Decision-gated. **Not approved for implementation.** Investigation only.

**Recorded:** 2026-05-03 against `bfa71c2` on `main`.

**Owner of the decision:** Jay.

This document persists the conclusion of the 2026-05-03 investigation so future Claude sessions do not re-investigate or accidentally start coding it. If you are reading this and considering implementing analytics, **stop and re-read the "Approval decisions required before code" section at the bottom**. None of those approvals exist yet.

---

## What this is about

Whether Utterbar should ship an opt-in remote product-analytics channel from the desktop app. Today the desktop app has only a local SQLite rollup (`analytics_daily`); there is no remote analytics endpoint and no third-party analytics SDK in the Rust binary.

Adding remote analytics, even strictly opt-in, **changes the brand and trust posture**. It is not a routine feature.

---

## Current public promise — what we'd be changing

Utterbar's public copy currently makes hard, absolute "no telemetry" claims across at least 15 surfaces. Adding opt-in analytics — even one daily counter, off by default — invalidates the absolute framing on every one of these:

| Surface | Current claim |
| --- | --- |
| `Utterbar Website/resources/views/pages/privacy.blade.php:10` | `'No telemetry, not even opt-in'` (comparison row) |
| `Utterbar Website/resources/views/pages/privacy.blade.php:114` | Section heading **"Why there's no telemetry."** |
| `Utterbar Website/resources/views/pages/home.blade.php:8` | `'No account, no telemetry, no phone-home.'` |
| `Utterbar Website/resources/views/pages/home.blade.php:550` | `'Watch it not phone home.'` |
| `Utterbar Website/resources/views/pages/how-it-works.blade.php:20` | `'No account. No telemetry. No phone-home.'` |
| `Utterbar Website/resources/views/pages/features.blade.php:21` | `'No cloud', 'No account', 'No telemetry'` |
| `Utterbar Website/resources/views/pages/docs/it-deployment.blade.php:136` | "There is no telemetry, no analytics ping, no error-reporting beacon, no opt-out." |
| `Utterbar Website/resources/views/pages/compare/{wispr-flow,voice-access,superwhisper,dragon}.blade.php` | All four say `'None. The app does not phone home.'` |
| `Utterbar Website/resources/views/pages/compare/wispr-flow.blade.php:160` | `'No account, no telemetry, no rate limits'` |
| `Utterbar Website/resources/views/pages/pricing.blade.php:32` | `"The app doesn't phone home."` |
| `Utterbar Website/resources/views/pages/business.blade.php:149` | `"The app does not phone home for activation status."` |
| `Utterbar Website/resources/views/components/footer.blade.php:147` | `'LOCAL-ONLY · NO ACCOUNT · NO TELEMETRY'` (every page) |
| `Utterbar Website/public/og/og-default.svg:58` | Same line in social-share image |
| `Utterbar Website/resources/blog/2026-04-26-why-local-dictation.md:34` | `"fast accurate dictation that doesn't phone home"` |
| `Utterbar Website/resources/views/pages/legal/privacy.blade.php:17-23` | §3 enumerates the **exact list** of outbound desktop HTTP calls; would need a fifth entry. |

There is also one **forward-looking commitment** that already grants permission for opt-in analytics, provided two conditions:

> *"If I ever add diagnostics it will be opt-in, off by default, and you'll be able to read every payload in plain text before it's sent. Until then: zero."*
> — `Utterbar Website/resources/views/pages/privacy.blade.php:117`

Any implementation must honour those two conditions exactly.

The desktop onboarding screen already says **"No telemetry by default"** (with the "by default" qualifier — `Utterbar Windows App/src/routes/Onboarding.tsx:214`). The desktop and website are inconsistent on this point today; the website is the stricter surface.

---

## Recommended approach (only if Jay later approves)

Three branches in strict order. Branch N must merge before Branch N+1 starts.

1. **Copy / legal first** (`feature/usage-analytics-copy-legal`, website-only).
   - Rewrite the 15+ surfaces above to remove the absolute "no telemetry" claim and replace with an honest, precise framing (e.g. "No telemetry by default. Optional anonymous counters, off by default, opt-in only.").
   - Add a new section to `pages/privacy.blade.php` describing the opt-in counter with the disclosure text and the schema.
   - Add a new sub-section to `legal/privacy.blade.php` §3 enumerating the optional fifth outbound HTTP call.
   - Update `BRIEF.md` voice rules so future copy doesn't drift back to absolute "no telemetry".
   - **Reversible** — if Jay reads the rewritten copy and decides it doesn't sound like Utterbar, the project stops here with no code shipped.

2. **Backend second** (`feature/usage-analytics-backend`, website-only).
   - New migration + endpoint + admin viewer + retention cron.
   - Cannot land before Branch 1 merges and is deployed; the moment the endpoint exists, the website is technically capable of receiving data.

3. **Desktop sender third** (`feature/usage-analytics-desktop`).
   - Settings toggle + onboarding step + sender service + preview/disclosure modal.
   - Must include a denylist snapshot test that asserts none of the §"Banned fields" strings ever appear in a serialised payload.

4. **Beta only after v0.17.0 stable.** v0.17.0 is the bundle promotion of the soaked v0.16.1–v0.16.5 line; mixing the privacy-rewrite into that cut blurs an otherwise-coherent stable. Ship the desktop sender in v0.17.1 beta, soak on the beta channel, then bundle into v0.18.0 stable when readiness is met (gates green, no regression reports, Jay's explicit approval). No fixed soak duration — beta-time-on-clock is a signal, not a blocker.

---

## Recommended analytics model

- **Aggregate daily counters only.** Not events, not hybrid.
- **No third-party analytics SDK.** Hand-rolled `reqwest` POST reusing the existing `Utterbar/<version> (windows; x86_64)` User-Agent template. No PostHog, no Mixpanel, no Sentry-style auto-capture.
- **No persistent device ID.** Ever.
- **Ephemeral per-launch session ID** (26-char ULID held in memory only, forgotten on app close) **only if the deletion semantics require it**. If the chosen retention design is "no per-session row at all, just bumped aggregates", drop the session ID entirely.

Reasoning:
- The Jay-future commitment is "you'll be able to read every payload in plain text". A counter snapshot is one ~500-byte JSON object the user can audit; an event stream is a moving target.
- The local model is already aggregate-by-day. Sending a ~1/day rollup of the same shape is the minimal delta and the easiest to defend in copy.
- Aggregate counters can't accidentally smuggle in a transcript fragment or an active-app title. Event-level telemetry is one rushed PR away from someone adding `properties: { app_title: focused_window }`.
- A third-party SDK ships its own collection logic that we cannot fully audit in the desktop binary; that contradicts the "plain-text payload" promise.

The proposed payload shape (subject to approval): `{schema, client_version, os, os_version, session_id, day_utc, counters: {dictations_total, dictations_voice, dictations_message, dictations_meeting, words_total, seconds_spoken, seconds_saved}, config: {asr_model_id, asr_provider, trigger_style, smart_rewrite_enabled, mute_conferencing_while_recording, is_pro}}`. Closed enums on every string field so a typo in `settings.json` never reaches the wire.

---

## Banned fields — must NEVER appear in any payload

Hard-coded denylist. Enforced by a Rust unit test that snapshots an example payload and asserts none of these strings or shapes appear:

- transcript text (any field, any encoding)
- audio bytes (any field, any encoding)
- selected text
- clipboard content
- active window title / app exe / focused-window metadata
- file paths (anything starting `C:\` or matching `[A-Z]:\\`)
- vocabulary entries
- licence key (raw or hashed)
- machine fingerprint (mac address, hostname, BIOS UUID, etc.)
- persistent device ID
- email address
- raw IP address embedded by the app (the server can derive a country code from the request IP at the edge; the app must not embed it)
- crash logs / panic messages

Adding a new field to the payload requires a CLAUDE.md rule update and a release-notes line — deliberate friction.

---

## Approval decisions required before code

These must be answered on paper before Branch 1 starts. Default of "no decision" = "do not implement".

1. **Are we doing this at all?** "No telemetry, not even opt-in" is genuine brand equity. Trading it for product insight is a real loss. If "I just want to know which features get used" is the actual goal, a quarterly user-survey email through the existing update-subscriber consent gives ~80% of the value at ~10% of the trust cost.
2. **Replacement homepage / footer / privacy / OG-image wording.** What replaces "NO TELEMETRY"? Candidates: **"NO TELEMETRY BY DEFAULT"** (one extra word, preserves spirit) or **"YOUR VOICE STAYS LOCAL"** (refocuses on the actually-untouched bit).
3. **Endpoint host.** `utterbar.com/api/usage/v1` (reuses existing TLS / CSP / throttle posture) vs `stats.utterbar.com` (visual separation in network logs).
4. **Retention window.** Recommended 30 days for session-ID stripping; 7 or 14 day options exist (shorter = stronger claim, weaker self-serve delete).
5. **Country-code capture.** Yes (via Cloudflare `CF-IPCountry`, no IP storage) vs no.
6. **All tiers vs Pro-only.** Recommended all tiers (more representative; Free is most of the install base).
7. **Onboarding placement.** Insert between the existing `privacy` and `updates` steps (recommended — privacy decisions in one block) vs after `ready`.
8. **`BlockTelemetry` policy behaviour.** Recommended: keep the policy name; expand its meaning so the same DWORD now hard-overrides the remote sender in addition to the local rollup. Update the doc-comment in `services/policy.rs` accordingly. Defence in depth: the sender hard-refuses regardless of the user toggle when `BlockTelemetry` is set.

Until ALL of these are answered (and approved by Jay in writing), do not start Branch 1. Until Branch 1 is merged, deployed, and Jay has slept ≥48 hours on the rewritten copy, do not start Branch 2. Until Branch 2 has landed and verified, do not start Branch 3.

---

## What this document is NOT

- Not implementation code. No JSON schema validators, no migrations, no Rust services, no React components.
- Not a privacy policy edit. The legal page is untouched in this task.
- Not a green light. The default for every approval question above is "no".

If a future session decides to implement this, the corresponding implementation prompt should reference this document by path (`docs/decisions/opt-in-product-analytics.md`) and explicitly cite which approval decisions Jay has made. A prompt that does not cite these is incomplete.
