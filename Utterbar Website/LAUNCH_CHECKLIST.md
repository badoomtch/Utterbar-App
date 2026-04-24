<!-- Internal — do not deploy. -->

# Utterbar — Go-Live Checklist

> The ordered list of things to do when cutting from **sandbox** to **live** (real money, real customers). Follow top to bottom; each item unlocks the next.
> Expect ~1 focused day end-to-end, plus lead time on the code-signing cert.

**Current state:** everything below runs in Stripe sandbox with a rotatable Ed25519 keypair. Nothing on this list is urgent until you have a signed Utterbar app to ship.

---

## Phase A — Weeks ahead of launch (lead-time items)

### A1. Order a Windows code-signing certificate

- [ ] Choose a CA. For indie budgets:
  - **SSL.com OV** — ~£280/year, 1-day verification, HSM or USB token delivery.
  - **DigiCert OV** — ~£400/year, gold-standard trust store, slower verification.
  - **Sectigo/Comodo OV** — cheaper (£200-ish) but reputation-seeding is slower with Windows SmartScreen.
- [ ] Submit business verification: registered company name + address (use your existing UK registration; sole trader works). Budget 2–5 working days for turnaround.
- [ ] Receive either a physical YubiKey-style token or an HSM cloud slot. **Do not lose it** — you cannot replace it, only buy a new cert.
- [ ] Test sign a scratch `.exe` with `signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /f cert.pfx scratch.exe` and confirm Windows SmartScreen shows "Verified publisher: [Your Ltd]" rather than a red block.

### A2. Rotate the Ed25519 signing keypair to a production value

The sandbox keypair is committed to `LICENCE_APP_HANDOFF.md`. That's fine for testing; not fine for production.

- [ ] On your **own laptop** (never the production server), run:
      ```
      php artisan licence:keygen
      ```
- [ ] Copy the **private** key output into the production server's `.env`:
      ```
      LICENCE_PRIVATE_KEY_B64=<new secret>
      LICENCE_PUBLIC_KEY_B64=<new public>
      LICENCE_PAYLOAD_VERSION=2   # bump from 1, so app can distinguish signers
      ```
- [ ] **Do not** remove the old sandbox public key from app Claude's Rust code. The app should ship with BOTH public keys — pick which to verify against using the `version` byte in the payload. This means licences issued in sandbox keep working even after rotation.
- [ ] Give app Claude the new hex + base64 public key. They rebuild with `PUBLIC_KEY_V2` added alongside `PUBLIC_KEY_V1`.
- [ ] `php artisan config:clear` on the server.

### A3. Decide the final price

Currently `STRIPE_PRO_AMOUNT=4900` (£49). Change in `.env` once — no redeploy needed.

- [ ] Pick live price. Defensible ranges: £49 (entry), £59 (recommended), £69 (if you have testimonials), £79 (if you have two screenshots of it saving someone's hour). Don't overthink; easy to raise later, harder to lower.
- [ ] Consider a launch-window discount for waitlist (e.g., first 200 at £29). If yes, we'll need a one-off `discount_code` Stripe coupon — say the word.

---

## Phase B — Launch day (do these in order)

### B1. Stripe dashboard — flip to live mode

- [ ] Dashboard → top-right toggle: **Live mode** ON.
- [ ] Developers → API keys: copy the **live** publishable + secret keys (`pk_live_…` / `sk_live_…`).
- [ ] Developers → Webhooks → Add endpoint:
      - URL: `https://utterbar.com/api/stripe/webhook`
      - Events: `checkout.session.completed`, `charge.refunded`, `charge.dispute.created`
      - Description: `Utterbar licence minting (live)`
- [ ] Copy the live webhook signing secret (`whsec_…`). **Different from the sandbox one.**
- [ ] Settings → Tax → enable Stripe Tax.
      - Origin address: your UK registered address.
      - Registrations: **UK** (mandatory), **EU-OSS via Ireland** (recommended — one window covers all 27 EU countries).
      - Yearly cost: 0.5% on taxed transactions. Worth it vs DIY VAT registration in every EU member state.
- [ ] Settings → Branding → set the public logo + colour.

### B2. Server env — swap sandbox for live

On the Cloudways shell, edit `~/public_html/.env`:

- [ ] Replace `STRIPE_PUBLISHABLE_KEY` with `pk_live_…`
- [ ] Replace `STRIPE_SECRET_KEY` with `sk_live_…`
- [ ] Replace `STRIPE_WEBHOOK_SECRET` with the new live `whsec_…`
- [ ] Set `STRIPE_AUTOMATIC_TAX=true`
- [ ] Confirm `STRIPE_ENABLED=true`
- [ ] Set `APP_ENV=production` (should already be)
- [ ] Set `APP_DEBUG=false` (should already be, but double-check — no stack traces in customer browsers)
- [ ] `php artisan config:clear && php artisan route:clear && php artisan view:clear`

### B3. Smoke test with a real £0.50 product (pre-cutover)

**Don't** flip to the real £49 price straight away. De-risk it:

- [ ] In Stripe dashboard, **live mode**, create a "Utterbar smoke test" product at £0.50.
- [ ] Temporarily set `STRIPE_PRO_PRICE_ID=price_...` in `.env` pointing at the £0.50 product (so `price_data` fallback isn't used).
- [ ] Buy it with a real card from a real browser.
- [ ] Confirm:
      - Land on `/thanks`
      - Licence email arrives
      - Webhook event on Stripe dashboard shows green 200
      - `/admin` shows the licence and the event
      - `/account` (after magic-link sign-in) shows the licence
      - Refund it from Stripe dashboard
      - Confirm licence `status` flips to `refunded` on `/admin`
      - Confirm the refunded licence returns 410 from `/api/licence/heartbeat`
- [ ] Remove the smoke test product + price from Stripe. Remove `STRIPE_PRO_PRICE_ID` from `.env` (or point it at the real price).
- [ ] Set `STRIPE_PRO_AMOUNT` to the real price in pence.

### B4. Code-sign and publish the Windows installer

This is app Claude's turf, but it needs to happen before the download link works.

- [ ] App Claude builds the release `.exe` against the **v2 public key**.
- [ ] Sign it with the OV cert from A1.
- [ ] Upload the signed `.exe` to wherever downloads live (S3 bucket, GitHub release, Cloudways static path — your call).
- [ ] Update the `/download` page's installer URL to point at the signed file.
- [ ] Test: download it, install on a clean Windows VM, paste a real licence, confirm Pro unlocks.

### B5. Go live — flip the last switches

- [ ] Add `ADMIN_EMAILS=jay@utterbar.com,jayparry1@gmail.com` in `.env` (whichever emails should see `/admin`). Already set for sandbox — verify it's still there.
- [ ] Clear one more time: `php artisan optimize:clear`
- [ ] Hit `/pricing` in an incognito window — "Get Utterbar Pro" button should POST to `/api/checkout/create-session` and kick you to live Stripe.
- [ ] Make one more real purchase at the real price to prove the live flow is clean.
- [ ] Announce.

---

## Phase C — First week after launch (monitoring)

### C1. Error observability

Currently Laravel logs to `stderr` on Cloudways. Find them via `tail -f ~/public_html/storage/logs/laravel.log` on the Cloudways shell. That's fine for day 1, not fine for week 1.

- [ ] Wire a log channel for 500s → Slack webhook or email (Resend can do this — send to `hello@utterbar.com` with level filter `error`). ~20 minute change, say the word and I'll do it.
- [ ] Consider Sentry ([sentry.io](https://sentry.io)) — free tier handles 5k events/month, one-line Laravel integration. Worth it once you have 100 paying customers.

### C2. Watch the `/admin` dashboard

Each morning for the first week:

- [ ] New licences match new Stripe charges — if they don't, the webhook is dropping events.
- [ ] Webhook errors column is 0. If non-zero, investigate which event failed and why.
- [ ] No sign-in email failures (server logs).
- [ ] Device activation counts grow; no licence is stuck at 0 devices a week after purchase (indicates app-side integration issue).

### C3. Refund fire drill

- [ ] Refund one licence manually from Stripe to verify:
      - `charge.refunded` event fires
      - Licence status flips to `refunded`
      - App returns 410 on next heartbeat
      - Customer email (optional courtesy message) is clear and non-shaming
- [ ] If any step fails: fix before handling real refund requests.

---

## Phase D — Nice-to-haves, post-launch

Non-blocking; batch whenever the mood strikes.

- [ ] **Queued mail.** Currently `Mail::send()` is synchronous inside the webhook. At scale this could time out Stripe's 10-second budget. Switch to `Mail::queue()` + run a queue worker (`php artisan queue:work`). 10-minute change.
- [ ] **Rate-limit tuning.** Sandbox limits are defensive (20 fetch/10min, 30 activate/10min). Review after real traffic.
- [ ] **Stripe customer portal (disabled)** — customers can't update cards anyway (one-off purchase), but we could surface receipt history. Low priority.
- [ ] **Email address change flow.** Currently I'd update `licences.email` via tinker if a customer asks. Once it happens twice, automate it via the portal.
- [ ] **Team/site licences.** Different SKU with `seats` instead of `max_devices`. Separate product surface entirely — probably after the first 100 paying customers.
- [ ] **Subscription Pro.** Low-probability, but if users ask for "Pro features behind a £5/mo instead of £49 one-off", we add a new SKU with Stripe subscription mode. Webhook handler already has the scaffolding.

---

## Cutover-day quick reference

When you're ready to do it, one scroll from top of Phase B:

```
Stripe live mode → copy live keys → add webhook endpoint → copy whsec
↓
ssh → edit .env → swap test keys for live → STRIPE_AUTOMATIC_TAX=true → config:clear
↓
£0.50 smoke test → refund → verify admin + account + heartbeat
↓
Rotate Ed25519 key (if not already done) → new app build with v2 public key
↓
Announce.
```

That's the whole move.
