# MONOREPO.md — Utterbar unified reference

Merged brain dump of the Utterbar system, spanning the desktop app
and the website/backend. Written for any Claude (or human) landing
cold. If you only read one long doc, read this.

Companion rules file: [CLAUDE.md](CLAUDE.md).
Counterpart brain dumps: [Utterbar Windows App/CLAUDE.md](Utterbar%20Windows%20App/CLAUDE.md) and [Utterbar Website/CLAUDE.md](Utterbar%20Website/CLAUDE.md) — superseded by this doc where they disagree, but kept for historical context.

---

## 0. The product in one paragraph

Utterbar is a Windows-first, local-only dictation app. Press `Ctrl+Space`, speak, release, your text lands in the focused app — Chrome, Slack, VS Code, Notion, anywhere. ASR runs on-device via NVIDIA Parakeet-tdt (0.6B v2) through sherpa-onnx. No audio leaves the machine. Free tier covers the core flow. Pro is a one-time **£39.99 GBP** licence (3-device cap) that unlocks custom accent colours, unlimited modes, full vocabulary, file transcription, and more dialects. Windows 10/11 x64 only. No macOS, no Linux.

---

## 1. Repo shape

Today:

```
Utterbar/                        ← monorepo root, single git repo
├── .git/                        ← remote: badoomtch/Utterbar-App (private)
├── CLAUDE.md                    ← rules (read first)
├── MONOREPO.md                  ← this file (deep reference)
├── RELEASE_RUNBOOK.md           ← enforced release ritual
├── TODO.md                      ← live work tracker
├── Utterbar Windows App/        ← desktop app — Tauri v2 + React + Rust
└── Utterbar Website/            ← Laravel 12 marketing + billing + admin
```

The monorepo was consolidated in commit `3d7d4f7` (moving the old
`Utterbar Windows App/.git` up a level). v0.6.0 and v0.6.1 tags
preserved. Website deploy is still out-of-band via `pscp` to Cloudways
— Cloudways has no git-based deploy, so `Utterbar Website/scripts/deploy-website.ps1`
is the ritual.

---

## 2. Utterbar Windows App — architecture

### 2.1 Stack

- **Shell:** Tauri v2 (`tauri = "2"`, features `tray-icon` + `protocol-asset`), `tauri-plugin-single-instance`. Tauri v2's `plugin-updater` is declared but NOT wired — hand-rolled updater under `src-tauri/src/commands/updater.rs`.
- **Frontend:** React 18 + Vite + TypeScript, Tailwind v3.4 with a token-driven theme system, Radix UI primitives, zustand for state, framer-motion for pill animations, lucide-react icons.
- **Backend:** Rust (`src-tauri/`), `tokio = "full"`, `rusqlite` (bundled SQLite + `refinery` migrations), `reqwest` with rustls+stream, `cpal` capture, `hound` for WAV, `rubato` resampling, `ringbuf` for live audio, `sha2` for installer verification, `ts-rs` to emit shared types into `src/generated/`.
- **ASR:** sherpa-onnx → Parakeet-tdt-0.6b (English) for dictation; Whisper Large v3 for Meeting Mode. Optional Python sidecar for experimental backends.
- **Speaker diarisation (v0.15.0+, Meeting Mode only):** sherpa-onnx → pyannote segmentation 3.0 (MIT, CNRS) + NVIDIA NeMo TitaNet small (CC-BY-4.0). Threshold-based clustering. Lazy-downloaded one-time on first meeting use, ~47 MB compressed, SHA-pinned via `services::inference::diarize_assets`.
- **Packaging:** NSIS + MSI via `tauri build`. NSIS is the shipped one; MSI is a byproduct.

Versions in [Utterbar Windows App/package.json](Utterbar%20Windows%20App/package.json) + [Utterbar Windows App/src-tauri/Cargo.toml](Utterbar%20Windows%20App/src-tauri/Cargo.toml) — grep, don't guess.

### 2.2 Two-window model

One Tauri app, two windows:

- **MainShell** — chrome-free dashboard. Settings, history, modes, vocabulary, licence, about.
- **FloatingBar** — transparent, always-on-top, frameless 300–900 px pill centred at the top of the primary monitor. Hides independently of MainShell.

[AppBootstrap.tsx](Utterbar%20Windows%20App/src/routes/AppBootstrap.tsx) inspects the URL hash to decide which tree to mount. The Rust side spawns both windows at boot and syncs via Tauri events.

### 2.3 Directory map

```
src/
  routes/
    MainShell.tsx         Dashboard window root
    FloatingBar.tsx       The pill — transparent Tauri window
    AppBootstrap.tsx      URL-hash-based window selector
    Dashboard.tsx, HistoryRoute.tsx, ModesRoute.tsx,
    VocabularyRoute.tsx, FilesRoute.tsx, Onboarding.tsx, settings/
  components/
    bar/                  FloatingBar sub-components. Shared with Remotion
                          bundle — NEVER fork; update both.
    common/               Modal, Toggle, UpdateToast, shared primitives
    modes/, upgrade/      Modes UI, UpgradeModal
  state/                  zustand stores: dictation, licence, modes, settings
  hooks/                  useProPrice, etc.
  generated/              ts-rs output from Rust types — DO NOT EDIT
  styles/tokens.css       Single source of truth for colours/radii/typography
  lib/ipc.ts              Typed invoke() wrapper

src-tauri/
  src/
    main.rs               Tauri builder, IPC surface, single-instance
    app_state.rs          Shared AppState injected into commands
    commands/             One module per domain: analytics, audio, backup,
                          diagnostics, dictation, files, history, licence,
                          models, modes, pricing, settings, shortcuts,
                          updater, updates, vocabulary, window
    services/             Business logic — commands stay thin
      audio/              cpal capture, loopback, WAV decode, VAD preprocess
      inference/          sherpa_parakeet / py_sidecar / stub providers
      licence/            DPAPI-stored licence cache + periodic revalidation
      llm/                deterministic + llama.cpp post-processing
      hotkeys.rs          Global hotkey registration
      text_inject.rs      SendInput-based insertion into focused window
      tray.rs             System tray menu + click handlers
      window_watchdog.rs  Re-pins FloatingBar if OS demotes it
    domain/               Pure types + settings schema + ts-rs #[derive(TS)]
    db/                   SQLite migrations (refinery) + connection pool
    platform/             Windows-specific bits (DPAPI, SendInput, …)
  tauri.conf.json         Windows list, bundle config, NSIS tweaks

marketing/remotion/       Remotion bundle that renders brand MP4s + stills.
                          Imports real app components via alias `@` →
                          `../../../src/`. Output under `out/`.

scripts/
  publish-release.sh      Build → upload R2 → POST admin API
  bump-version.mjs        Syncs version across Cargo.toml + package.json +
                          tauri.conf.json

releases/<v>.md           User-voice release notes. POSTed verbatim as
                          notes_markdown to the admin API.
```

### 2.4 IPC conventions

- `#[tauri::command]` handlers live under `src-tauri/src/commands/`. One module per domain.
- Commands are thin: arg validation, service call, map errors to `OrliqError` (our `thiserror` enum) via `util::result::CmdResult`.
- Events via `AppHandle::emit` with structured payloads mirrored in `src/generated/`. Frontend subscribes through zustand middleware, NOT directly in components.
- Never put business logic in a command. Services own state; commands are the IPC shim.
- Components never call `invoke()` directly — go through the relevant zustand store. If you find one that does, hoist it into the store.

### 2.5 State (frontend)

zustand stores under `src/state/`:

- `dictation.ts` — state (idle/warming/listening/processing/success/error), live transcript, last-inserted text, timer.
- `settings.ts` — persisted app settings, hydrated from Rust on boot, writes round-trip through `settings_*` commands.
- `modes.ts` — user-defined transcription modes.
- `licence.ts` — tier, activation key, last revalidation timestamp.

### 2.6 Persistence

| What | Where | Format |
| --- | --- | --- |
| Settings (accent, theme, hotkeys, toggles) | `%APPDATA%\Utterbar\settings.json` | JSON, schema in `domain/settings.rs` |
| Modes, vocabulary, history | `%APPDATA%\Utterbar\utterbar.db` | SQLite (refinery migrations in `db/migrations/`) |
| Licence key + tier | Windows Credential Manager via DPAPI | Encrypted blob, `services/licence/cache.rs` |
| Downloaded models | `%APPDATA%\Utterbar\models\` | sherpa-onnx bundles |
| Recordings cache | `%APPDATA%\Utterbar\recordings\` | WAV, retention via settings |

**Upgrade survival:** NSIS's "uninstall previous version" step wipes the install dir, so nothing user-facing lives there. Licence → Credential Manager. Everything else → `%APPDATA%\Utterbar\`. Do NOT regress this: a future refactor that writes into the install dir will disappear licences on every update, and users can't tell that apart from "my licence got deleted."

### 2.7 Theming

- [tokens.css](Utterbar%20Windows%20App/src/styles/tokens.css) is the single source of truth for colours, radii, type scale. Tailwind's config extends from these CSS vars. **Never hardcode hex in components.**
- Two themes ship: Midnight (default, dark) and Paper (cream light). Switching toggles `[data-theme]` on `<html>`.
- Accent colour is user-settable. Free users: 6 presets. Pro users: full hex picker. Tint/shade gradient across accent-using surfaces (wordmark, pill glow, CTAs) is derived from the picked hex — do NOT flat-paint a single value.
- FloatingBar wrapper uses `h-screen w-screen items-start justify-center bg-transparent p-0` to glue to the top of the Tauri floating window. Remotion bundle overrides this to `h-auto w-auto items-center` in `marketing/remotion/src/app.css`. Touching the wrapper breaks the marketing render — update both.

---

## 3. Utterbar Website — architecture

### 3.1 Stack

| Layer | Choice |
| --- | --- |
| Framework | Laravel 12 (PHP 8.2+) |
| Templating | Blade (no JSX/React on this side) |
| Styling | Tailwind CSS v4 (`@theme` block, CSS custom properties) |
| JS | Alpine.js 3.x + `@alpinejs/collapse`. No framework beyond Vite. |
| Bundler | Vite 7 |
| Fonts | Self-hosted Instrument Sans Variable (body) + Geist Mono Variable (tech data) |
| DB | MySQL 8 (Cloudways managed) |
| Sessions | `database` driver, 120-min lifetime, httpOnly + SameSite=Lax + Secure |
| Mail | Resend (`MAIL_MAILER=resend`, `hello@ddraig.dev` — outbound moved off utterbar.com to the studio domain on 2026-04-27, separate API key, DKIM passing) |
| Payments | Stripe Checkout, **live mode**. `stripe/stripe-php ^17.0` |
| File storage | Cloudways local (`public/`) for most. R2 for installers only. |
| Signing | Ed25519 via `ext-sodium`. |
| Hosting | Cloudways DigitalOcean droplet + Cloudflare DNS/CDN |
| Domain | `utterbar.com` (apex) |

Server Node lives at `~/tmp/node22` because Cloudways' default is too old for Tailwind v4:
```
PATH=$HOME/tmp/node22/bin:$PATH HOME=$HOME/tmp/npmhome npm run build
```

### 3.2 Directory map

```
app/
  Console/Commands/      licence:keygen, licence:list, licence:revoke,
                         licence:unrevoke, licence:resend, updates:announce
  Http/
    Controllers/
      AccountController           /account dashboard, device deactivation
      AdminController              /admin overview
      Api/
        AdminReleaseApiController  POST /api/admin/releases
        LicenceApiController       fetch / activate / deactivate / heartbeat
      Admin/ReleaseController      /admin/releases browser UI
      CheckoutController           Stripe session creation
      ContactController            /api/contact
      PageController               All marketing pages
      SignInController             Magic-link auth
      SitemapController            /sitemap.xml
      StripeWebhookController      POST /api/stripe/webhook
      UpdatesController            /api/updates/subscribe + unsub links
      WaitlistController           (legacy)
    Middleware/
      AdminApiToken                bearer auth for /api/admin/*
      AdminOnly                    email allow-list for browser admin
  Mail/                   LicenceDelivery, SignInLink, UpdateAnnouncement, …
  Models/                 Activation, ContactMessage, Licence, Release,
                          SignInToken, UpdateSubscriber, User, WaitlistEntry
  Services/
    Licence/              LicenceKeys (Crockford base32), LicenceSigner (Ed25519)
    Notify/               AdminNotify

bootstrap/app.php         CSRF exemptions, middleware aliases, guest redirect
config/marketing.php      Mirror of app-repo asset-manifest.json
database/migrations/      Standard Laravel + licences, activations,
                          stripe_webhook_events, sign_in_tokens, releases,
                          update_subscribers
public/brand/, videos/, posters/, stills/, og/, favicon.svg
resources/
  css/app.css             Single stylesheet, ~46 KB built
  js/app.js               Alpine init + custom directives
  views/
    components/           x-layouts.app, x-nav, x-footer, x-logo, …
    emails/               All Mail view files
    pages/                Every top-level route's Blade
routes/web.php            ALL routes — no separate api.php
scripts/render-brand-png.mjs, sync-marketing-assets.mjs
```

### 3.3 Data model

| Table | What |
| --- | --- |
| `users` | Laravel default. Password unused (magic-link only). Email = identity. |
| `sessions` | Laravel default. Durable, revocable. |
| `sign_in_tokens` | 64-char tokens, 20-min TTL, single-use, enumeration-safe |
| `licences` | One per Stripe purchase. `key_lookup` (20-char Crockford), `signed_blob` (96 bytes), email, Stripe IDs, `max_devices=3`, status (active/refunded/revoked) |
| `activations` | One per (licence × device fingerprint). Unique on `(licence_id, fingerprint_hash)`. Tracks `last_heartbeat`, `last_ip`, `app_version`. |
| `stripe_webhook_events` | Idempotency log. Prevents double-processing replays. |
| `releases` | Published app versions. `r2_key`, `sha256`, `notes_markdown`, `published_at`, optional `announced_at`, `tag`, `channel` (stable/beta). |
| `update_subscribers` | Release mailing list. `confirmed_at` + `unsubscribed_at` + per-subscriber `token`. |
| `waitlist_entries` | Legacy. Retained but no longer primary CTA. |
| `contact_messages` | Contact form submissions. Emailed to the admin notify inbox on arrival (`ADMIN_NOTIFY_EMAIL`, currently `hello@ddraig.dev`). |

### 3.4 Licence key format

Two forms:
```
Display:   UB-7QX4-K2YM-R8FP-3JZN-WA6T      (23 chars with dashes)
Canonical: 7QX4K2YMR8FP3JZNWA6T             (20 chars, no prefix, no dashes)
```
Alphabet: Crockford base32 (`0-9A-Z` minus `I, L, O, U`). 100 bits entropy. Stored canonical; display via `$licence->displayKey()`.

### 3.5 Signed blob (96 bytes)

What the Tauri app verifies offline. Full layout in [LICENCE_APP_HANDOFF.md §3](Utterbar%20Website/LICENCE_APP_HANDOFF.md):

```
[0]   1   version u8
[1]   1   sku u8
[2]   6   issued_at u48 BE (unix ms)
[8]   2   nonce u16 BE
[10]  8   key_id u64 BE = licences.id
[18]  14  reserved (zero)
[32]  64  Ed25519 signature over sha256(payload)
= 96 bytes
```

Signed at licence mint time (in the Stripe webhook handler). Delivered via `POST /api/licence/fetch` on app's first paste.

### 3.6 Auth

**Customer auth — magic-link only.** No passwords. `users.password` stores a random unusable hash.
1. `GET /sign-in` — email input
2. `POST /sign-in` — creates user if they have licences but no account, creates token, emails 20-min link. Always returns the same confirmation page regardless of account existence (blocks enumeration).
3. `GET /sign-in/{token}` — validates, `Auth::login()`, regenerates session, burns sibling tokens, redirects `/account`.
4. `POST /sign-out` — standard logout.

Guest redirect configured in `bootstrap/app.php` to `/sign-in`, not Laravel's `/login`.

**Admin auth — two gates.**
1. Browser admin (`/admin`, `/admin/releases`): `auth` + `admin` middleware. `admin` checks `$user->email` against `ADMIN_EMAILS` env var. Currently `jayparry1@gmail.com, hello@ddraig.dev` (post-2026-04-27 email-domain switch).
2. API admin (`POST /api/admin/releases`, `DELETE /api/admin/releases/{id}`): `admin.api` middleware. Bearer token `hash_equals` vs `ADMIN_API_TOKEN`. Timing-safe. 503 if env var unset.

---

## 4. The cross-repo contracts

These endpoints are the contract between app and website. **Breaking any of them breaks real customers.**

### 4.1 Licence API — app → website

All CSRF-exempt (via `bootstrap/app.php`). All return JSON. Errors: `{ error: slug, message: string, meta: mixed|null }`. Full shapes in [LICENCE_APP_HANDOFF.md §4](Utterbar%20Website/LICENCE_APP_HANDOFF.md).

| Endpoint | Purpose | Rate limit |
| --- | --- | --- |
| `POST /api/licence/fetch` | Short key → signed blob | 20 per 10 min / IP |
| `POST /api/licence/activate` | Record device, enforce 3-cap | 30 per 10 min / IP |
| `POST /api/licence/deactivate` | Free a device slot | 30 per 10 min / IP |
| `POST /api/licence/heartbeat` | Last-seen ping, picks up revocations | 60 per hour / IP |

### 4.2 Admin release API — app → website

Called by `Utterbar Windows App/scripts/publish-release.sh` to publish a release without the admin form.

| Endpoint | Purpose |
| --- | --- |
| `POST /api/admin/releases` | Create a release record. Verifies file exists in R2 AND recomputes SHA-256 from the R2 object — pasted SHA must match the recomputed value or publish is refused. Persists the recomputed SHA as canonical. When `UPDATER_PUBLIC_KEY_B64` is set, requires a valid `updater_signature` over the canonical metadata blob (built with the recomputed SHA). Returns 201 with `id`, `download_url`, `size_bytes`, `announced_to`. |
| `DELETE /api/admin/releases/{id}?delete_file=1` | Remove a release. Optional R2 file deletion. |

Body + responses documented in [Utterbar Windows App/RELEASE.md](Utterbar%20Windows%20App/RELEASE.md). Validation rules shared between the API path and the browser admin form (`POST /admin/releases`) live in `App\Services\Releases\ReleaseValidator` so the two entry points can't drift on the SHA / signature / channel-confirmation rules.

### 4.3 Pricing API — app → website (LIVE)

App GETs `https://utterbar.com/api/pricing` from [pricing.rs](Utterbar%20Windows%20App/src-tauri/src/commands/pricing.rs). Shape:
```json
{ "amount_usd": 39.99, "currency": "GBP", "period": "one_time", "display": "£39.99" }
```
Implementation: [PricingApiController.php](Utterbar%20Website/app/Http/Controllers/Api/PricingApiController.php), wired in [routes/web.php](Utterbar%20Website/routes/web.php) at `Route::get('/api/pricing', …)` with a 60/min throttle. App-side fallback matches (`£39.99 GBP` in both pricing.rs and useProPrice.ts).

### 4.4 Updater API — app → website (LIVE)

App GETs `$UTTERBAR_LICENCE_BASE_URL/api/updater/check?current=<v>&channel=stable` from [updater.rs](Utterbar%20Windows%20App/src-tauri/src/commands/updater.rs). Shape when an upgrade is available (currently the v0.7.2 row):
```json
{ "available": true, "version": "0.7.2", "download_url": "https://dl.utterbar.com/releases/0.7.2/Utterbar-Setup-0.7.2.exe", "sha256": "...", "notes_markdown": "...", "size_bytes": 11014909 }
```
Implementation: [UpdaterApiController.php](Utterbar%20Website/app/Http/Controllers/Api/UpdaterApiController.php), wired in [routes/web.php](Utterbar%20Website/routes/web.php) at `Route::get('/api/updater/check', …)`. Queries the `releases` table for the newest eligible row (valid X.Y.Z version, valid SHA-256, non-empty r2_key), returns `{ "available": false }` otherwise.

### 4.5 Stripe webhook — Stripe → website

Full flow:
```
User clicks "Get Utterbar Pro" → POST /api/checkout/create-session
   → CheckoutController → Stripe Checkout (live, £39.99 GBP)
   → User pays → checkout.session.completed fires
   → StripeWebhookController:
      1. Verify Stripe-Signature HMAC
      2. Insert stripe_webhook_events row (idempotency)
      3. Generate key_lookup (Crockford base32)
      4. Insert Licence row
      5. LicenceSigner signs blob (Ed25519)
      6. Save signed_blob
      7. Mail::send(LicenceDelivery)
      8. AdminNotify::newLicence → admin notify inbox (env: ADMIN_NOTIFY_EMAIL)
   → User lands on /thanks → licence email arrives minutes later
   → User pastes key in Tauri app
      → POST /api/licence/fetch {key} → signed blob
      → App verifies sig locally, unlocks Pro
      → POST /api/licence/activate {key, fingerprint, device_name, platform, app_version}
      → Activation row created, activation_token returned, 3-cap enforced via lockForUpdate
```

Webhook events handled: `checkout.session.completed`, `charge.refunded`, `charge.dispute.created`. Idempotency via `stripe_webhook_events.stripe_event_id` unique constraint; replays return 200 without re-processing.

### 4.6 R2 release storage

`utterbar` bucket. Canonical key per release: `releases/<v>/Utterbar-Setup-<v>.exe`. Served via `dl.utterbar.com`. Anyone can download; gating happens in-app via licence check.

Historical note: until v0.6.5, the admin API's `file_not_in_bucket` fallback defaulted to the unversioned `Utterbar-Setup.exe`, so earlier releases may still have a second R2 object at that key as an orphan. Safe to delete, but no rush.

---

## 5. Release pipeline (cross-repo)

Canonical doc: [Utterbar Windows App/RELEASE.md](Utterbar%20Windows%20App/RELEASE.md).

### 5.1 Steps

1. **Bump versions.** [bump-version.mjs](Utterbar%20Windows%20App/scripts/bump-version.mjs) syncs `package.json` + `Cargo.toml` + `tauri.conf.json`.
2. **Build.** `pnpm tauri build` → `src-tauri/target/release/bundle/nsis/Utterbar_<v>_x64-setup.exe`.
3. **Write release notes.** `releases/<v>.md` in the user's voice. POSTed verbatim as `notes_markdown` and rendered on `/download`.
4. **Publish.**
   ```
   ./scripts/publish-release.sh <version> \
       "src-tauri/target/release/bundle/nsis/Utterbar_<v>_x64-setup.exe" \
       releases/<v>.md \
       --announce --tag "latest"
   ```
   The script: computes SHA-256, uploads to R2 via `utterbar-r2` AWS profile, HEAD-verifies CDN, POSTs metadata to `$UTTERBAR_API_BASE/api/admin/releases` with `Authorization: Bearer $UTTERBAR_ADMIN_API_TOKEN`.
5. **Tag and push.** `git tag v<v> && git push origin main v<v>`.

### 5.2 Secrets

Live off-repo in `~/.utterbar-deploy/.env`:
```
R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_ENDPOINT, R2_BUCKET
UTTERBAR_ADMIN_API_TOKEN, UTTERBAR_API_BASE
```
Plus `~/.aws/config` + `~/.aws/credentials` with the `utterbar-r2` profile pointing at R2's S3-compatible endpoint.

### 5.3 409 recovery

If the admin API returns `409 version_exists`:
```
curl -X DELETE "$UTTERBAR_API_BASE/api/admin/releases/<id>?delete_file=1" \
  -H "Authorization: Bearer $UTTERBAR_ADMIN_API_TOKEN"
```

### 5.4 Deploy tooling requirements

On fresh dev machines:
```
winget install Amazon.AWSCLI jqlang.jq
```

---

## 6. Licence system

Full arch spec: [LICENCE_SYSTEM_SPEC.md](Utterbar%20Website/LICENCE_SYSTEM_SPEC.md). Contract handoff: [LICENCE_APP_HANDOFF.md](Utterbar%20Website/LICENCE_APP_HANDOFF.md).

### 6.1 Flow (app side)

1. User pastes key into Settings → Licence, or clicks `utterbar://activate?key=…`.
2. `commands/licence::activate_licence` POSTs to `/api/licence/activate` with key + hardware fingerprint. Server returns tier + expiry + refresh token.
3. Key + tier encrypted via DPAPI (`services/licence/cache.rs`). Revalidated daily on app start and wake-from-sleep. 30-day offline grace.
4. `licence.ts` store exposes `tier` to components. Pro gates check `tier === 'pro'` before firing; otherwise open UpgradeModal.
5. UpgradeModal fetches current price via [useProPrice](Utterbar%20Windows%20App/src/hooks/useProPrice.ts) (backed by `commands/pricing`) and deep-links to website checkout.

### 6.2 Pro-gated features (v0.6.1)

Custom accent, >N modes, >N vocab entries, file transcription, multi-language dialects.

**Never trust the frontend `tier` flag for anything security-critical.** Transcription quality, vocabulary expansion, file decoding all re-check with the Rust-side licence cache before running.

### 6.3 Ed25519 keypair

**Currently sandbox keypair** — generated as a test key and reused when cutting over to live Stripe. Documented risk: private key was exposed in chat during setup. Real-world exploitability is low because forging a licence requires BOTH the private key AND write access to the DB (the `/api/licence/fetch` returns the blob from DB, not from what the app claims), OR patching the Tauri binary to skip verification — at which point patching out the licence check is easier than forging.

**Rotation plan** in [LAUNCH_CHECKLIST.md §A2](Utterbar%20Website/LAUNCH_CHECKLIST.md): at v0.7, run `php artisan licence:keygen` on a dev machine (not the server), put new private key in server `.env` as `LICENCE_PRIVATE_KEY_B64`, bump `LICENCE_PAYLOAD_VERSION=2`, give app Claude the new public key so the app bakes BOTH v1 and v2 keys. Existing sandbox licences still verify (v1); new licences sign v2.

### 6.4 Artisan commands

```
php artisan licence:list [--email=foo@bar.com]
php artisan licence:revoke UB-XXXX-... --reason="abuse pattern"
php artisan licence:unrevoke UB-XXXX-...
php artisan licence:resend foo@bar.com
php artisan licence:keygen       # NEW keypair — for rotation
```

---

## 7. Auto-updater

Client side ([updater.rs](Utterbar%20Windows%20App/src-tauri/src/commands/updater.rs)):

1. `check_update()` — GETs `$UTTERBAR_LICENCE_BASE_URL/api/updater/check?current=<v>&channel=stable`. Expects `{ available, download_url, sha256, notes_markdown }`.
2. `UpdateToast` renders bottom-right of MainShell when newer available.
3. `install_update()` — downloads `download_url` to `%TEMP%\`, verifies SHA-256 against server value, runs installer silently (`/S` flag on NSIS), relaunches.

**Server side:** implemented in `UpdaterApiController.php`; see §4.4. Not yet deployed to Cloudways at time of writing (run `deploy-website.ps1`).

---

## 8. Marketing pipeline (Remotion → website)

### 8.1 Remotion bundle

Lives at `Utterbar Windows App/marketing/remotion/`. Renders brand MP4s + stills by bundling the real app components.

Key tricks:
- `webpack-override.mjs` aliases `@` → the shipping app's `src/` so `@/components/bar/UtterbarLogoMark` lands in the real tree, not a fork.
- `@tauri-apps/api/{core,window,event}` aliased to inert stubs under `src/stubs/` so render-time invokes and listeners no-op.
- `BarDriver.tsx` seeds real zustand stores with mock state (idle/listening/…) and renders real `FloatingBar.tsx`.
- Tailwind pipeline hand-rolled (`webpack-override.mjs` builds the CSS rule itself) because `@remotion/bundler/css-loader` hardcodes `-!` pre-requests that bypass postcss on CSS `@import`s. `globals.css` imported as a top-level JS sibling from `Root.tsx` to dodge that code path.

### 8.2 Sign-off gate

When you change a component the Remotion bundle imports (anything under `src/components/bar/`, `src/components/common/`, or `FloatingBar.tsx`), re-render `out/parity/*.png` to confirm nothing drifted visually. Six parity stills (idle/warming/listening/processing/success/error) are the sign-off gate.

`render.mjs` iterates the composition registry. `out/` IS committed so the website can consume it by path.

### 8.3 Website consumption

Historical: R2 bucket `/marketing/{videos,posters,stills}/*` served via `dl.utterbar.com`.
Current: Laravel `public/{videos,posters,stills}/*` served via `utterbar.com` directly. 9 MB total — Cloudways bandwidth is fine; R2 was overkill. R2 copies may still exist as orphans; delete later.

Blade components:
- `<x-video-embed id="{manifest-id}" />` — `<video>` with poster, muted, loop, `preload="none"`, `aria-label` from alt. Alpine's `ubVideoEmbed` attaches `IntersectionObserver` to auto-play loops in view and pause when out. Respects `prefers-reduced-motion` (renders `<img>` poster only). Appends `?v={mtime}` to URLs so re-renders cache-bust.
- `<x-still id="{manifest-id}" />` — `<img>` with lazy loading + alt from manifest + `?v={mtime}`.

---

## 9. Environment variables

Website (`Utterbar Website/.env`), sensitive values redacted:

```
APP_ENV=production
APP_DEBUG=false
APP_URL=https://utterbar.com
APP_KEY=<laravel key>

# DB (Cloudways managed)
DB_CONNECTION=mysql
DB_HOST=localhost
DB_DATABASE=<...>
DB_USERNAME=<...>
DB_PASSWORD=<...>

# Session
SESSION_DRIVER=database
SESSION_LIFETIME=120
SESSION_ENCRYPT=false

# Logging
LOG_LEVEL=warning

# Mail (Resend)
MAIL_MAILER=resend
RESEND_API_KEY=<key>
MAIL_FROM_ADDRESS="hello@ddraig.dev"
MAIL_FROM_NAME="Ddraig Dev"

# Stripe (live)
STRIPE_ENABLED=true
STRIPE_PUBLISHABLE_KEY=pk_live_...
STRIPE_SECRET_KEY=sk_live_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_PRO_PRICE_ID=price_1TPGCIBGkUvjTVVkBQ4qMzlu
STRIPE_PRO_AMOUNT=3999           # pence fallback when price ID missing
STRIPE_PRO_CURRENCY=gbp
STRIPE_AUTOMATIC_TAX=true

# Licence signing (sandbox — rotate at 0.7)
LICENCE_PRIVATE_KEY_B64=<64-byte base64, sandbox>
LICENCE_PUBLIC_KEY_B64=/1YHkclcfLVi4HWtZAa3b6Xd8Mx3tDtHjlcf2gFOJEI=
LICENCE_SKU_ID=1
LICENCE_SKU_STRING=utterbar-pro-lifetime
LICENCE_MAX_DEVICES=3
LICENCE_PAYLOAD_VERSION=1

# Admin gates
ADMIN_EMAILS=jayparry1@gmail.com,hello@ddraig.dev
ADMIN_NOTIFY_EMAIL=hello@ddraig.dev
ADMIN_API_TOKEN=<64-hex>

# Cloudflare R2
CLOUDFLARE_R2_ACCOUNT_ID=bf4d2d2ba6ffeb5eb8472648bacc2f78
CLOUDFLARE_R2_BUCKET=utterbar
CLOUDFLARE_R2_PUBLIC_URL=https://dl.utterbar.com
CLOUDFLARE_R2_ACCESS_KEY_ID=<...>
CLOUDFLARE_R2_SECRET_ACCESS_KEY=<secret>

MARKETING_CDN_BASE=              # empty → served from public/
```

App (`Utterbar Windows App/.env.local`):
```
UTTERBAR_LICENCE_BASE_URL=https://utterbar.com   # overridable for dev
```

Deploy (`~/.utterbar-deploy/.env` — off-repo):
```
R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_ENDPOINT, R2_BUCKET
UTTERBAR_ADMIN_API_TOKEN, UTTERBAR_API_BASE
```

---

## 10. Known contract drift + open threads

### 10.1 Pro price currency + amount mismatch — FIXED (not yet shipped)

Both app-side fallbacks now return `£39.99 GBP`:
- [useProPrice.ts](Utterbar%20Windows%20App/src/hooks/useProPrice.ts)
- [pricing.rs](Utterbar%20Windows%20App/src-tauri/src/commands/pricing.rs)

Shipped in v0.7.0. The old `$29` string in [releases/0.6.1.md](Utterbar%20Windows%20App/releases/0.6.1.md) is historical and intentionally untouched.

### 10.2 Pricing API route missing — SHIPPED

`GET /api/pricing` is live and returns `{ amount_usd: 39.99, currency: "GBP", period: "one_time", display: "£39.99" }` from the Stripe-backed price config.

### 10.3 Updater API route missing — SHIPPED

`GET /api/updater/check?current=<v>&channel=stable` is live. Returns the latest stable release's version, download URL, SHA-256, and notes when a newer release exists, or `{ available: false }` otherwise.

### 10.4 Installer filename legacy key — SHIPPED

`publish-release.sh` uploads to the versioned key `releases/<v>/Utterbar-Setup-<v>.exe` and sends `r2_key` explicitly in the POST body. [AdminReleaseApiController.php](Utterbar%20Website/app/Http/Controllers/Api/AdminReleaseApiController.php) defaults its fallback to the versioned path. Legacy `Utterbar-Setup.exe` copies still exist in R2 from earlier releases and can be cleaned up.

### 10.5 `publish-release.sh` curl quoting — FIXED

Script now uses `--data-binary "@$TMP_PAYLOAD"` with a temp file. Also explicitly includes `r2_key` in the payload so the server doesn't fall back to the legacy key. See [publish-release.sh](Utterbar%20Windows%20App/scripts/publish-release.sh).

### 10.6 Idle RAM 4.9 GB spike (user flagged 2026-04-23, demoted 2026-04-27)

**Status:** demoted to "watching, not active." Two of the top hypotheses are ruled out by code review and the report has not been reproduced on v0.8.x or v0.9.0:

- llama.cpp is feature-gated off (`Cargo.toml: llama = ["dep:llama-cpp-2"]`, optional, not in default features). Shipped binaries don't link it.
- sherpa-onnx Parakeet session is built on-demand inside `transcribe()` flow — `inference/mod.rs` uses a write lock + lazy `provider.warm()`, not eager init.

If the spike resurfaces on v0.9.0+: attach VMMap to the running wry process, screenshot the heap commit by region, and reopen this section. Hypotheses #3 (recording-buffer leak) and #4 (WebView2 over-commit) remain plausible but unreproduced.

Original report kept below for context.

**Original 2026-04-23 report.** Running Utterbar (main + webview) showed ~4.9 GB committed at idle. Previous baseline was ~400–700 MB. Not a slow leak; appeared on launch. Top hypotheses ranked at the time:

1. **llama.cpp context loaded globally at boot.** 7B quant ≈ 4–5 GB — exact size match. Check [services/llm/llama_cpp.rs](Utterbar%20Windows%20App/src-tauri/src/services/llm/) — when is the context instantiated? Should be on-demand. **RULED OUT** by feature-gating.
2. **sherpa-onnx / Parakeet session allocated non-lazily.** If ONNX runtime flipped from mmap to heap, 3–4× on-disk size. Check `services/inference/sherpa_parakeet.rs` for `use_arena = true` / `graph_optimization_level` changes. **RULED OUT** by code review.
3. **Recording buffer not freed after transcription.** Check `services/audio/capture.rs` + `preprocess.rs` — ringbuf should be bounded.
4. **WebView2 over-commit.** Usually caps at ~500 MB so probably not the whole story.

First diagnostic: VMMap (SysInternals) or `tasklist /FI "IMAGENAME eq Utterbar.exe" /V /FO LIST` to split private bytes vs working set vs shared, correlate with subsystem.

### 10.7 History page two-column layout

[HistoryRoute.tsx](Utterbar%20Windows%20App/src/routes/HistoryRoute.tsx) is a fixed 2-col split. Below ~1100 px the list column squeezes to 6-char-wide vertical words. v0.6.1's release notes claimed a fix — that was lipstick only, structural problem untouched.

Correct design: ONE column, full width. Each entry = card (app icon, timestamp, mode badge, transcript preview). Click opens raw/processed/context detail in a modal or slide-over. Free-tier "Full history is a Pro feature" message is the list's first card, not a half-width empty state.

Not a minor-release patch — real redesign, queue for next polish cycle.

### 10.8 Remotion 400 px mobile pass

12 MP4s render correctly at 1920×1080 but not verified at the website's mobile breakpoint. Queue for when the website is ready to consume them.

### 10.9 Tauri v2.10 IPC heap corruption — SHIPPED IN v0.7.0 VIA VENDORED RUNTIME

Three symbolicated minidumps from 2026-04-24 testing, all exception code `0xC0000374` (heap corruption), all final frame `ntdll!RtlReportFatalFailure+0x9` at the same address:

| Time (UTC) | Version | Thread | Stack top (app frames) |
|---|---|---|---|
| 16:24:22 | v0.6.4 | tokio worker | `respond_async_serialized_inner` → `InvokeResolver::return_result` → `Webview::on_message` → `handle_ipc_message::closure$1` → drop `Webview` → drop `Context` → `Rc::drop_slow<EventLoopRunner>` → `RtlFreeHeap` (detect) |
| 16:57:26 | v0.6.5 | tokio worker | *Identical* pattern. Additional detail in parent frame: `window::plugin::init::closure$0::closure$72::async_block_env$0` — the async IPC that was resolving when the drop corrupted the heap was inside Tauri's built-in `window` plugin. |
| 17:34:36 | v0.6.5 | **main thread** | `tao::event_loop::run_return` → `user32::DispatchMessageWorker` → `wry::webview2::attach_ipc_handler::closure$0` → `tauri::ipc::protocol::message_handler::closure$0` → `RtlFreeHeap` (detect) |

**Two distinct stack shapes.** The first two are the `Rc<EventLoopRunner>` drop on a tokio worker — a `DispatcherMainThreadContext<T>` pattern that Tauri's own code has `unsafe impl Send` on, despite containing `Rc`s internally. The third is on the main thread during a WebView2 IPC arrival — no tokio involvement.

**Shared property.** All three: the detection fires at `RtlFreeHeap` inside Tauri/wry IPC machinery, downstream of the actual bad write. The seed allocation and the detection can be on different threads — Low-Fragmentation Heap arenas see the corruption later at an unrelated free within the same arena.

**Attempted diagnostic: Windows PageHeap** (`gflags /p /enable utterbar.exe /full` then `/p /enable utterbar.exe` for header-only). Both forms caused the crash to stop reproducing in testing. This is circumstantial evidence that the root cause is concurrency-shaped (a race that resolves badly only under specific timing): the extra per-allocation overhead changes the timing enough that the bad-drop window closes. PageHeap was reverted after 30 min of testing. Revert command: `gflags /p /disable utterbar.exe` from an Administrator PowerShell.

**Also relevant for differential diagnosis.** Our code amplifies the bug: `clamp_floating_position` was previously an `#[tauri::command] pub async fn` called via `invoke()` on every `onMoved` event during a drag — dozens of async-IPC calls per second, each resolving through `respond_async_serialized_inner`. v0.6.6 changes it to `pub fn` (non-async), which routes through Tauri's sync IPC path instead. **Narrow experiment, not a fix.** If crashes continue in v0.6.6 testing, the Tauri fork is confirmed necessary. If they stop, we were the amplifier.

**v0.6.6 status:** shipped with diagnostics + frontend/Rust stuck-hotkey self-heal + `clamp_floating_position` sync experiment, but the crash reproduced again on v0.6.6 with matching symbols — new minidump at 2026-04-24T18:28:15Z symbolicated to `tauri_runtime_wry::impl$50::clone<EventLoopMessage>+0x149` at [tauri-runtime-wry-2.10.1/src/lib.rs:256](Utterbar%20Windows%20App/src-tauri/vendor/tauri-runtime-wry-2.10.1/src/lib.rs). Failure bucket: `ILLEGAL_INSTRUCTION_c000001d_utterbar.exe!tauri_runtime_wry::impl$50::clone<enum2$<tauri::EventLoopMessage>>`.

**v0.6.7 fix — vendored runtime patch.** Rather than wait on upstream (issue [#10001](https://github.com/tauri-apps/tauri/issues/10001) is open, labelled "needs triage" with no movement; the duplicate [#10987](https://github.com/tauri-apps/tauri/issues/10987) was closed "not planned"), we vendor `tauri-runtime-wry-2.10.1` at [src-tauri/vendor/tauri-runtime-wry-2.10.1/](Utterbar%20Windows%20App/src-tauri/vendor/tauri-runtime-wry-2.10.1/) and wire it via `[patch.crates-io]` in [src-tauri/Cargo.toml](Utterbar%20Windows%20App/src-tauri/Cargo.toml).

The patch is **two lines**: change `Context<T>::main_thread` from `DispatcherMainThreadContext<T>` to `Arc<DispatcherMainThreadContext<T>>`, and wrap the one construction site in `Arc::new(...)`. Rationale (full version in the file's module-level comment):

- `#[derive(Clone)]` on `Context<T>` walks `main_thread`'s derived `Clone` → `EventLoopWindowTarget::clone` → `Rc::clone` on `runner_shared: tao::EventLoopRunnerShared<T> = Rc<EventLoopRunner<T>>` ([tao-0.34.8/src/platform_impl/windows/event_loop/runner.rs:27](https://docs.rs/tao/0.34.8/src/tao/platform_impl/windows/event_loop/runner.rs.html)). `Context::clone` runs on every IPC message, including on tokio workers via `tauri::ipc::respond_async_serialized_inner`. The non-atomic `Rc::clone` off the main thread either fires `assert_unchecked` (ud2 → ILLEGAL_INSTRUCTION) or corrupts heap header state that `RtlFreeHeap` detects at the next unrelated free.
- Wrapping `main_thread` in `Arc` makes the per-IPC-message clone an atomic refcount bump on the Arc — never touches the inner `Rc`. The `Rc<EventLoopRunner>` is only consulted by main-thread-gated code paths (Tauri's own `current_thread().id() == main_thread_id` check at lib.rs:238 + display-handle accessor at 2659) which continue to work via Arc's `Deref`.
- **Not a `Rc → Arc` cargo-cult fix.** The inner `EventLoopRunner` contains `Cell`, `RefCell`, raw `HWND`, `Box<dyn FnMut>` — none of those become thread-safe. The patch does not widen access to that state. It only stops the wrong-thread `Rc::clone`/`Rc::drop` that the derived `Clone` was invoking via the ownership chain.
- Upstream's own source comment at `DispatcherMainThreadContext::windows` — `// changing this to an Rc will cause frequent app crashes.` — shows the maintainers already applied this same treatment to one field. We extend it to the outer wrapper.

**Verification.** `cargo check` + `cargo clippy --all-targets -- -D warnings` both green. `cargo tree --package tauri-runtime-wry --edges=no-build` reports `tauri-runtime-wry v2.10.1 (C:\Users\jaypa\...\src-tauri\vendor\tauri-runtime-wry-2.10.1)` — confirming the resolver picked the vendored override, not the crates.io copy. No other code changes were necessary.

**Maintenance cost.** Any `cargo update` that moves Tauri past 2.10.x must be followed by a rebase of the vendored directory against the new upstream source. The module-level comment in the vendored `lib.rs` documents the full rationale so a future maintainer can re-apply the patch. If the Tauri maintainers accept an upstream PR for this, we drop the vendor dir and the `[patch.crates-io]` block.

**Follow-up (non-blocking).** File a new comment on upstream [#10001](https://github.com/tauri-apps/tauri/issues/10001) with the four minidumps' symbolicated stacks + the failure bucket, so other Tauri users benefit. Draft is in the v0.6.7 session log.

**Crash-analyser tooling.** [scripts/analyse-crash.ps1](Utterbar%20Windows%20App/scripts/analyse-crash.ps1) was rewritten in the v0.6.6 session to parse cleanly and symbolicate matching-version dumps. Previous versions had a string-quoting bug that broke on paths with spaces. Usage: `.\scripts\analyse-crash.ps1` (newest dump) or `.\scripts\analyse-crash.ps1 -Dump <path>` or `.\scripts\analyse-crash.ps1 -All`. Output includes `!analyze -v` with the failure bucket.

### 10.10 Licence persistence across app restarts — SHIPPED IN v0.7.0

Dual-cache: Windows Credential Manager + `%APPDATA%\Utterbar\licence.json`. Cache is loaded synchronously in `LicenceService::new()` rather than deferred to a tokio task, so the tier is correct from the first paint and no longer races the UI mount. See [services/licence/mod.rs](Utterbar%20Windows%20App/src-tauri/src/services/licence/mod.rs).

### 10.11 Docs vs code drift — RESOLVED 2026-04-24

Website docs were brought into line with v0.7.0 in the post-ship audit:

- `docs/shortcuts.blade.php` now shows real defaults for all seven slots.
- `docs/modes.blade.php` documents the three built-in modes (was six).
- `docs/vocabulary.blade.php` no longer mentions a Team tier or per-mode replacements.
- `docs/troubleshooting.blade.php` — "red LED" entry replaced with the v0.7.0 "no audio detected" path; tray label corrected to "Reset Utterbar position".
- `docs/privacy-controls.blade.php` reframed as global switches (v0.7 removed per-mode context toggles).
- Pricing, features, home, and the download/waitlist copy were also softened — no more "coming 1.0" language now that the public build has shipped.

`Utterbar Website/DOCS_AUDIT_v0.6.1.md` is now historical — items there are either shipped or tracked in `TODO.md`.

---

## 11. Gotchas (absorbed from both sides)

### 11.1 Build / CI

- Clippy `-D warnings` fails CI on the Rust side. Run `cargo clippy --all-targets -- -D warnings` locally before pushing.
- Unused `tauri::Manager` imports have bitten us twice.
- Frontend CI job runs pnpm lint + pnpm build; both must be green.

### 11.2 Release / deploy

- `publish-release.sh` needs `aws` + `jq`. Fresh Windows: `winget install Amazon.AWSCLI jqlang.jq`.
- On fresh machines, seed `~/.aws/config` + `~/.aws/credentials` from `~/.utterbar-deploy/.env`.
- `publish-release.sh` already uses `--data-binary "@$TMP_PAYLOAD"` and explicitly sends `r2_key`. A manual `curl -d "$PAYLOAD"` from git-bash on Windows would mangle multiline JSON — if you're writing a one-off POST, use the temp-file form.
- The admin API's legacy fallback now defaults to the versioned key (see §10.4). Once the website is deployed, manual POSTs that omit `r2_key` will still work correctly.

### 11.3 Laravel-specific

- **Resend PHP driver requires `resend/resend-php`.** Laravel 12's built-in Resend transport is a peer dependency. `composer update --no-dev` once accidentally dropped it — webhook minted licences but emails failed with `Class "Resend" not found`.
- **Blade `@json()` chokes on complex nested arrays.** Use `@php $data = [...]; @endphp` + `{!! json_encode($data, ...) !!}`.
- **Cloudflare caches PNG/MP4 aggressively** (1-year max-age). `<x-video-embed>` and `<x-still>` append `?v={filemtime}` to force cache-miss on updates.
- **Cloudflare "Manage robots.txt" feature** overrides origin `robots.txt`. Noindex meta on private pages is belt-and-braces.
- **`php artisan tinker` errors on Cloudways** because psysh can't write to `~/.config/psysh`. Use `php -r` one-liners.
- **`favicon.ico` is cached empty on Cloudflare** even after deleting from origin. Let cache expire.
- **Stripe webhook setup:** when adding the endpoint, Stripe asks "Your account" vs "Connected and v2 accounts". Always pick "Your account" — the other filter excludes Utterbar's events.
- **`.env` perms on Cloudways default to 664.** `chmod 600 .env` before anything else.

### 11.4 Tauri-specific

- **NSIS upgrade wipes the install dir.** All user-facing state must live outside it. See §2.6.
- **Single-instance plugin fires a "another instance started" event** on hot-reloads in dev; benign, filter it.
- **Tauri v2's `plugin-updater` is declared but unused** — we hand-roll. Don't accidentally wire it when touching the updater.
- **ts-rs regenerates `src/generated/`** on every Rust build if you import it. Don't edit those files.

### 11.5 Remotion-specific

- Webpack alias `@` → `../../../src/` is load-bearing. Breaking it = "marketing videos are a fork of the real UI" all over again.
- Tauri API stubs are the trick that lets real components render in a Node bundler.
- Rendered outputs (`out/`) are committed so the website can consume without running Remotion.

---

## 12. Current state (2026-07-22, post-v0.21.0 stable)

### Superseding release snapshot

- **Last publicly-shipped stable:** `v0.21.0` (2026-07-21). It ships live local Meeting Mode transcription with separate microphone and system-audio tracks, speaker-side labelling, silence-aware chunks, Parakeet v2, and a simplified graphics-provider picker. The stable updater feed serves installer `https://dl.utterbar.com/releases/0.21.0/Utterbar-Setup-0.21.0.exe`, SHA-256 `0032462b3642c31a2968747a00d77b321d041f84ebad60ccf9f95dad36dad568`, 20,573,793 bytes.
- **Beta channel:** `v0.21.3` (2026-07-22) adds a 250-1,500 ms clipboard-restoration delay control, retaining 1,500 ms as the conservative default for slower apps. Its installer is `https://dl.utterbar.com/releases/0.21.3/Utterbar-Setup-0.21.3.exe`, SHA-256 `9abc98ea2126c1360f208d094f0cd74e17175e3ce3e525b5d0d81a1932bc7eb7`, 20,577,828 bytes. It is beta-only, with no announcement and no `latest` tag. `v0.21.2` restores copied images after dictation; `v0.21.1` remains the text-delivery and final-audio reliability beta; `v0.20.11` remains the historical Meeting Mode beta row.
- **In-tree version:** `v0.21.3` across the desktop package, Rust crate, lockfile, and Tauri configuration. Tag `v0.21.3` points at the release commit.
- **Distribution:** `/download` presents `v0.21.0`; beta-channel clients remain on the beta feed until they select the stable channel or a newer beta is published.

The historical desktop-app notes below predate this snapshot. Use the four bullets above for current release state.

### Desktop app
- **Last publicly-shipped stable:** `v0.19.0` (admin release row id=78, 2026-05-06). Shown on `/download` with the `latest` pill. Stable chain: v0.7.0 → v0.8.7 → v0.10.0 → v0.12.0 → v0.15.6 → v0.16.0 → v0.17.0 → v0.18.0 → v0.19.0. v0.19.0 bundles the soaked v0.18.1 → v0.18.4 beta line on top of v0.18.0: Settings search v2 (sidebar + per-pane drill-down with help-text matching) plus the experimental Teams + Discord microphone-mute toggles (off by default, hidden under Settings → Advanced → Experimental). Stable build produced with `npx tauri build --features experimental_mic_mute` so users on stable can flip the toggle after upgrade; the Cargo feature stays off in `default = []`.
- **Beta channel:** `v0.19.5` (admin row id=83, 2026-05-07). Performance pass on top of v0.19.4. Bundles five Tier-1 perf changes from the in-session audit: runtime capabilities cached once per process via `OnceLock<RuntimeCapabilities>`; `CaptureHandle::stop` moves the final PCM buffer with `std::mem::take` instead of cloning twice (eliminates up to 2 × ~920 MB allocator churn at the 8-hour safety cap); the launch prewarm task now fires for any non-zero `warm_duration_secs` (so the shipped 120 s default warm window pre-loads the model at launch, not only the keep-forever sentinel); dead `vocabulary_hints` plumbing removed from the dictation hot path (no active provider read it); Vite bundle code-split via `React.lazy` so the floating bar boots against a 449 kB / 147 kB gzip chunk instead of 711 kB / 213 kB gzip. Beta channel only; no `--announce`, no `latest` pill. Tag `v0.19.5` at `c4d48b8`. SHA-256 `952b9476c65c8109cbf836c11b12c8ebb4c61f3d709b23b320fa1d64181363db`. Installer at `https://dl.utterbar.com/releases/0.19.5/Utterbar-Setup-0.19.5.exe`, 20,102,874 bytes.
- **Beta soak chain on top of v0.19.0:** five-deep, all bundle into the next v0.20.0 stable when soak completes per [CLAUDE.md §5.7](CLAUDE.md):
  - v0.19.1 (row 79): English i18n foundation across the Settings panes, Floating Bar chrome, and the Update Toast; language-aware onboarding step that picks the dictation language and recommends a multilingual Whisper model when non-English. App UI itself ships English only; the picker controls only `models.whisper_language`, what Utterbar listens for.
  - v0.19.2 (row 80): Settings, Models warning when the dictation language is non-English and the default model is English-only. Surfaces a one-click "Switch to multilingual model" button if such a model is installed; an amber install-hint card otherwise.
  - v0.19.3 (row 81): placement hotfix. The v0.19.2 warning now sits directly beside the Whisper-language picker that triggered it, not further down the page. Logic byte-identical to v0.19.2.
  - v0.19.4 (row 82): auto-update Install & restart first-click fix. Closes the v0.19.x bug where Install & restart could be intercepted by close-to-tray and hide the app instead of installing the update. Adds an `updater_install_in_progress` AtomicBool on AppState that the close handler reads before the close-to-tray branch.
  - v0.19.5 (row 83): performance pass, as above.
- **What v0.16 → v0.18 shipped (now stable):** v0.16.0 bundled the v0.15.7 → v0.15.12 beta line: ASR output validation across every transcribe surface, an auto-stop on regular dictation, and a B2B layer covering managed fleet licences, air-gapped activation, and volume self-serve checkout. v0.17.0 bundled v0.16.1 → v0.16.5: launch-at-startup repair, ASR prewarm and timing improvements, desktop UX polish, an opt-in deterministic Smart rewrite, and the conferencing output-mute settings clarity work. v0.18.0 bundled v0.17.1 and v0.17.2: narrow tunes to the floating-bar waveform.
- **In-tree version:** `v0.19.5` across `package.json`, `Cargo.toml`, `tauri.conf.json`. Release commit: `c4d48b8` ("Release v0.19.5"). The five perf merges that v0.19.5 bundles sit at `55a05b8` (gpu cache), `3e91ce3` (capture-stop mem::take), `5447076` (prewarm), `69a0bdf` (drop vocab hints), `b94aca7` (frontend code-split), all merged through their `--no-ff` integration commits. Tag `v0.19.5` points at the build commit `c4d48b8`.
- **Versioning convention** (effective from v0.12.0; see [CLAUDE.md §5.7](CLAUDE.md)): `0.x.0` minor = mainline / stable, `0.x.y` patch = beta soak. The cadence is feature work lands on a series of `0.x.1`, `0.x.2`, ... betas; once the line has soaked, the next mainline `0.(x+1).0` promotes the lot to stable. v0.16, v0.17, v0.18, and v0.19 all followed the rule.
- **Long-form transcription cap, addressed in v0.12.1, unchanged through v0.19.5.** Recording cap is 8 hours. Transcribe-input ceiling on regular (non-meeting) dictation is 5 minutes (302 s); meeting mode chunks past that. The 5-min cap manifests on regular dictation as a 302 s refusal-with-audio-loss UX issue tracked in [TODO.md](TODO.md); the proper fix (streaming-recogniser rebuild) is queued as a future major work item.
- **DirectML execution provider, shipped in v0.15.6, unchanged through v0.19.5.** Vendored sherpa-onnx + ORT 1.14.1 + DirectML 1.15.0 bundle at [src-tauri/vendor/sherpa-onnx/](Utterbar%20Windows%20App/src-tauri/vendor/sherpa-onnx/). Model-aware Auto policy: Medium / Large / Parakeet route to DML on DX12 hardware, Small stays on CPU.
- **Speaker diarisation:** disabled in shipping builds since v0.15.1. The v0.15.0 attempt produced 91 speaker clusters on a 5-min meeting with cross-language Whisper hallucinations; pulled by v0.15.1 hotfix. Diariser code stays in tree behind `#[allow(dead_code)]` for the eventual rework. See [TODO.md](TODO.md) for the architecture sketch + open verification questions. v0.19.5 explicitly preserved the dead-code path when removing the unrelated `vocabulary_hints` field from `transcribe_meeting_diarised`'s signature.
- v0.6.1 was pulled (crashes); v0.6.2 → v0.6.9 were internal-only iterations. All their user-facing fixes rolled forward into v0.7.0.
- **Tauri IPC crash** — patched via vendored `tauri-runtime-wry` at [src-tauri/vendor/tauri-runtime-wry-2.10.1/](Utterbar%20Windows%20App/src-tauri/vendor/tauri-runtime-wry-2.10.1/). Still load-bearing as of v0.19.5. See §10.9.
- Windows SDK Debugging Tools (cdb.exe + `analyse-crash.ps1`) installed.
- Multi-engine ASR live: Parakeet-TDT (default English, INT8, greedy_search), Whisper (Small / Medium / Large v3, multilingual; honours the user-selected provider since v0.15.6 lifted the v0.8.7 CPU-pin), Zipformer.

### B2B / business surface (2026-04-27 first-pass, extended through v0.16.0 stable)
- **Marketing:** `/business` landing page (privacy + deployment +
  predictable licensing angle, pitched at call centres, legal,
  healthcare, internal IT). Volume-pricing block on `/pricing`
  (Team £29/seat, Business £19/seat, Enterprise contact). All one-time,
  no subscription.
- **Lead capture:** `/business/contact` typed enquiry form
  (name, company, email, seat count, use case, typed compliance/
  deployment requirement checkboxes). Persists to a separate
  `business_enquiries` table with admin index/show + status update
  at `/admin/business`. Email goes to `AdminNotify`.
- **Volume self-serve checkout (shipped via v0.15.x → v0.16.0).**
  `POST /api/checkout/create-volume-session` accepts quantity 5–100,
  mints a Pro licence with `max_devices = quantity`, SKU
  `utterbar-pro-volume`. Two-tier rate (£29/seat to 24, £19/seat
  from 25) lives in [VolumePricing.php](Utterbar%20Website/app/Services/Pricing/VolumePricing.php).
  Website-only; no desktop app change required. Larger orders still
  flow through the enquiry form so Jay can hand-mint with bespoke
  `max_devices`.
- **Managed fleet licences and air-gapped activation (shipped via the v0.15.11 / v0.15.12 betas, bundled into v0.16.0 stable).**
  Air-gapped flow: admin exports a fingerprint request from the
  desktop app, uploads it to the admin console, downloads the
  resulting `licence.bin`, places it at `<data_dir>/licence.bin` on
  the locked-down machine. The desktop app picks it up at next
  launch via [commands/licence.rs](Utterbar%20Windows%20App/src-tauri/src/commands/licence.rs).
  No outbound network call from the locked-down machine.
- **IT documentation:** `/docs/it-deployment` covers system reqs,
  silent NSIS install, exact network egress (5 calls enumerated),
  air-gapped deployment, group-policy registry pinning, code-signing
  status (honest "not yet"), licence/seat model.
- **Group-policy registry overlay (desktop app):**
  [src-tauri/src/services/policy.rs](Utterbar%20Windows%20App/src-tauri/src/services/policy.rs).
  Reads `HKLM\SOFTWARE\Policies\Utterbar` at boot (HKCU as per-user
  fallback, HKLM wins on conflict). Six pinnable values:
  `UpdateChannel` (REG_SZ: stable|beta), `CheckUpdatesAutomatically`
  (REG_DWORD: 0|1), `BlockModelDownloads` (REG_DWORD: 0|1),
  `BlockHistory` (REG_DWORD: 0|1), `BlockHistoryExport` (REG_DWORD:
  0|1; superset semantics with `BlockHistory`), `BlockTelemetry`
  (REG_DWORD: 0|1; redundant in practice because no telemetry ships
  by [CLAUDE.md §5.1](CLAUDE.md), but kept so IT can pin the
  guarantee). Wired into `SettingsStore::snapshot()` so policy
  overlays user settings on every read, and into
  `ModelRegistry::install()` so a pinned block surfaces a clear
  refusal. Unit-tested.
- **What we deliberately did NOT build:** SAML/SSO, an enterprise
  admin dashboard, cloud dictation portals, a Salesforce/Jira
  integration, SOC2 badges, anything that would replace the local-only
  story. Code-signing certificate explicitly skipped per
  [CLAUDE.md §4](CLAUDE.md); SmartScreen warnings on first install
  are expected. Machine-bound `licence.bin` (v3 payload) deferred
  until a buyer asks; the v2 signed payload reserves 14 bytes for
  a fingerprint hash but the v2 verifier silently ignores them.

### Website
- Live on Cloudways. All contract endpoints deployed:
  - `GET /api/pricing` → `£39.99 GBP`.
  - `GET /api/updater/check?current=<v>&channel=<stable|beta>` — returns the latest **eligibility-filtered** row for the channel. Eligibility includes `updater_signature IS NOT NULL` when `UPDATER_PUBLIC_KEY_B64` is configured. Net effect today (verified 2026-05-07, post-v0.19.5-beta-publish): stable returns v0.19.0 (admin row 78); beta returns v0.19.5 (admin row 83, SHA `952b9476c65c8109cbf836c11b12c8ebb4c61f3d709b23b320fa1d64181363db`). Both rows are Ed25519-signed; channel separation is enforced server-side, so a stable client never sees the v0.19.5 beta row.
  - `POST /api/licence/{fetch,activate,deactivate,heartbeat}` → working under v2 keypair.
  - `POST /api/stripe/webhook` → live; mints licences on `checkout.session.completed` under v2 keypair.
  - `POST /api/admin/releases` → admin token + audit log + signed-metadata required when key is configured.
- Sign-in + update-subscriber tokens hashed; staged rotation in place.
- Release table populated; `/changelog` + `/download` render from it.
- `config/site.php` `version` is informational fallback only — real release data served from the `releases` table.
- 58 PHP tests still only one-machine-green, not CI-verified. Re-run before each deploy.
- Deploy: `Utterbar Website/scripts/deploy-website.ps1`.

### Environment
- Stripe: **live mode**, £39.99 GBP, Stripe Tax on, adaptive pricing on.
- **Licence keypair:** v2 only. `LICENCE_PAYLOAD_VERSION=2` server-side; `PUBLIC_KEY_BYTES_V2` baked into the desktop app at [verify.rs](Utterbar%20Windows%20App/src-tauri/src/services/licence/verify.rs). The transitional v1 acceptance has been removed from the desktop verifier; pre-v0.9.0 caches no longer verify and need a re-fetch from the server on next launch (rare in practice now that v0.9.0 is many releases behind).
- **Updater keypair:** separate Ed25519 key, operator-local. Private key in `~/.utterbar-deploy/.env` only. Public key in server `.env` and baked into [updater.rs](Utterbar%20Windows%20App/src-tauri/src/commands/updater.rs).
- Licence cache: DPAPI-encrypted at rest (`%APPDATA%\Utterbar\licence.json` wrapped via `CryptProtectData`); legacy plaintext caches auto-upgrade on next save.
- R2 CDN: `dl.utterbar.com` — installers at `releases/<v>/Utterbar-Setup-<v>.exe` (+ legacy `Utterbar-Setup.exe` alias for the versioned-key validator fix).
- Admin notify: `hello@ddraig.dev` (post-2026-04-27 email-domain switch off utterbar.com to the studio domain).

---

## 13. Queue for next Claude in line

Ranked by user-impact, current priorities:

1. ~~**Ed25519 key rotation.**~~ — done. v2 keypair shipped 2026-04-26; the desktop verifier in [verify.rs](Utterbar%20Windows%20App/src-tauri/src/services/licence/verify.rs) now exposes `PUBLIC_KEY_BYTES_V2` only, the transitional v1 acceptance window is closed. Pre-v0.9.0 caches re-fetch from the server on next launch.
2. ~~**Legal pages refresh**~~ — done. `legal/privacy.blade.php` now lists purchase records, device activations, and the fingerprint-hash policy.
3. **File a comment on upstream [tauri#10001](https://github.com/tauri-apps/tauri/issues/10001)** with the minidumps' symbolicated stacks + failure bucket. Helps upstream.
4. **Admin dashboard cohesion** — no releases link from `/admin`, no licence-revoke UI. Website-only, ships whenever.
5. **Remotion video quality pass** — current renders are slow, zoomed-out, bordered, waveform speed wrong. Separate dedicated session. Consider R2-backed CDN delivery.
6. **Clean up orphan R2 objects** — legacy `releases/<v>/Utterbar-Setup.exe` copies from the double-upload era. Safe to delete from the bucket now the versioned-key validator ships.
7. **4.9 GB idle RAM** (§10.6) — demoted to "watching." Top hypotheses ruled out by code review and not reproduced on v0.9.0+. If it surfaces, VMMap-attach + reopen.
8. ~~**Waitlist tidy-up**~~ — done. Waitlist controller, model, mailable, view, and the drop-table migration shipped earlier this session; the table is gone on production.

---

End of unified reference. Rules live in [CLAUDE.md](CLAUDE.md); canonical release flow in [Utterbar Windows App/RELEASE.md](Utterbar%20Windows%20App/RELEASE.md); product/brand in [Utterbar Website/BRIEF.md](Utterbar%20Website/BRIEF.md); design in [Utterbar Website/DESIGN_SYSTEM.md](Utterbar%20Website/DESIGN_SYSTEM.md); licence contract in [Utterbar Website/LICENCE_APP_HANDOFF.md](Utterbar%20Website/LICENCE_APP_HANDOFF.md).
