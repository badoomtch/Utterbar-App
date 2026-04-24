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
Utterbar/                        ← monorepo root (no git yet)
├── CLAUDE.md                    ← rules (read first)
├── MONOREPO.md                  ← this file (deep reference)
├── Utterbar Windows App/        ← desktop app — Tauri v2 + React + Rust (git)
└── Utterbar Website/            ← Laravel 12 marketing + billing + admin (no git)
```

Planned: single `git init` at the root, both sub-projects as top-level directories, one CI workflow. Until that happens, only the Windows App is a git repo; the website is deploy-from-laptop over `pscp`/`plink`.

---

## 2. Utterbar Windows App — architecture

### 2.1 Stack

- **Shell:** Tauri v2 (`tauri = "2"`, features `tray-icon` + `protocol-asset`), `tauri-plugin-single-instance`. Tauri v2's `plugin-updater` is declared but NOT wired — hand-rolled updater under `src-tauri/src/commands/updater.rs`.
- **Frontend:** React 18 + Vite + TypeScript, Tailwind v3.4 with a token-driven theme system, Radix UI primitives, zustand for state, framer-motion for pill animations, lucide-react icons.
- **Backend:** Rust (`src-tauri/`), `tokio = "full"`, `rusqlite` (bundled SQLite + `refinery` migrations), `reqwest` with rustls+stream, `cpal` capture, `hound` for WAV, `rubato` resampling, `ringbuf` for live audio, `sha2` for installer verification, `ts-rs` to emit shared types into `src/generated/`.
- **ASR:** sherpa-onnx → Parakeet-tdt-0.6b (English). Optional Python sidecar for experimental backends.
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
| Mail | Resend (`MAIL_MAILER=resend`, `hello@utterbar.com`) |
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
| `contact_messages` | Contact form submissions. Emailed to `hello@utterbar.com` on arrival. |

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
1. Browser admin (`/admin`, `/admin/releases`): `auth` + `admin` middleware. `admin` checks `$user->email` against `ADMIN_EMAILS` env var. Currently `jayparry1@gmail.com, hello@utterbar.com`.
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
| `POST /api/admin/releases` | Create a release record. Verifies file exists in R2. Returns 201 with `id`, `download_url`, `size_bytes`, `announced_to`. |
| `DELETE /api/admin/releases/{id}?delete_file=1` | Remove a release. Optional R2 file deletion. |

Body + responses documented in [Utterbar Windows App/RELEASE.md](Utterbar%20Windows%20App/RELEASE.md).

### 4.3 Pricing API — app → website (BROKEN)

App GETs `https://utterbar.com/api/pricing` from [pricing.rs](Utterbar%20Windows%20App/src-tauri/src/commands/pricing.rs). Expected shape:
```json
{ "amount_usd": 39.99, "currency": "GBP", "period": "one_time", "display": "£39.99" }
```
**Route does not exist on the website.** App always falls back. Two fixes needed:
1. Add the route to [routes/web.php](Utterbar%20Website/routes/web.php) + a controller action that reads from config or Stripe.
2. Update the app's fallback amount/currency/display from `$29 USD` to `£39.99 GBP`.

### 4.4 Updater API — app → website (BROKEN)

App GETs `$UTTERBAR_LICENCE_BASE_URL/api/updater/check?current=<v>&channel=stable` from [updater.rs](Utterbar%20Windows%20App/src-tauri/src/commands/updater.rs). Expected shape:
```json
{ "available": "0.6.1", "download_url": "https://dl.utterbar.com/releases/0.6.1/Utterbar-Setup-0.6.1.exe", "sha256": "...", "notes_markdown": "..." }
```
**Route does not exist.** All of `/api/updater/check`, `/api/updates/check`, `/api/update/check`, `/api/version/check` 404. Users on v0.6.0 cannot auto-upgrade to v0.6.1 — they see no toast; the check call returns 404 HTML, the client parses "no update," they stay on 0.6.0.

**Fix:** add a controller that queries the `releases` table for the latest stable > `current`, returns the metadata; 204 if current is newest.

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
      8. AdminNotify::newLicence → hello@utterbar.com
   → User lands on /thanks → licence email arrives minutes later
   → User pastes key in Tauri app
      → POST /api/licence/fetch {key} → signed blob
      → App verifies sig locally, unlocks Pro
      → POST /api/licence/activate {key, fingerprint, device_name, platform, app_version}
      → Activation row created, activation_token returned, 3-cap enforced via lockForUpdate
```

Webhook events handled: `checkout.session.completed`, `charge.refunded`, `charge.dispute.created`. Idempotency via `stripe_webhook_events.stripe_event_id` unique constraint; replays return 200 without re-processing.

### 4.6 R2 release storage

`utterbar` bucket. Two keys per release currently:
- `releases/<v>/Utterbar-Setup-<v>.exe` — versioned (new, preferred)
- `releases/<v>/Utterbar-Setup.exe` — legacy, still required by the admin API's `file_not_in_bucket` check

Served via `dl.utterbar.com`. Anyone can download; gating happens in-app via licence check.

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

**Server side: not built.** See §4.4.

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
MAIL_FROM_ADDRESS="hello@utterbar.com"

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
ADMIN_EMAILS=jayparry1@gmail.com,hello@utterbar.com
ADMIN_NOTIFY_EMAIL=hello@utterbar.com
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

### 10.1 Pro price currency + amount mismatch

- Desktop fallback: `$29 USD` at [useProPrice.ts:31](Utterbar%20Windows%20App/src/hooks/useProPrice.ts#L31) and [pricing.rs:59](Utterbar%20Windows%20App/src-tauri/src/commands/pricing.rs#L59).
- Release notes: "offline fallback of $29 one-time" at [releases/0.6.1.md:9](Utterbar%20Windows%20App/releases/0.6.1.md#L9).
- Website Stripe price: `£39.99 GBP` (`price_1TPGCIBGkUvjTVVkBQ4qMzlu`).
- Website JSON-LD, meta descriptions, pricing page: all say `£39.99`.

**Fix both fallbacks** (Rust + TS + comments in useProPrice + pricing.rs doc comment that says `launch price at v0.6`). The real purchase enforces Stripe's price, so this is cosmetic drift, but it's a trust hit on first launch.

### 10.2 Pricing API route missing

App hits `/api/pricing`; website doesn't expose it. [routes/web.php](Utterbar%20Website/routes/web.php) only has `/pricing` (page). Add a tiny controller that returns current price from config or Stripe.

### 10.3 Updater API route missing

App hits `/api/updater/check`; website returns 404 on every candidate path. v0.6.0 users can't auto-upgrade. **Highest-priority cross-repo fix.** Schema and behaviour documented in §4.4.

### 10.4 Installer filename legacy key

`publish-release.sh` uploads `Utterbar-Setup-<v>.exe`; admin API's `file_not_in_bucket` check hardcodes `Utterbar-Setup.exe`. Currently double-uploading. Website repo owns the fix; once done, drop the double-upload in the script.

### 10.5 `publish-release.sh` curl quoting

Uses `-d "$PAYLOAD"`; multiline JSON mangles on Windows git-bash, server 422's on missing fields. Use `--data-binary "@$TMPFILE"`. Still in the committed script.

### 10.6 Idle RAM 4.9 GB spike (user flagged 2026-04-23)

Running Utterbar (main + webview) shows ~4.9 GB committed at idle. Previous baseline was ~400–700 MB. Not a slow leak; appears on launch. Top hypotheses ranked:

1. **llama.cpp context loaded globally at boot.** 7B quant ≈ 4–5 GB — exact size match. Check [services/llm/llama_cpp.rs](Utterbar%20Windows%20App/src-tauri/src/services/llm/) — when is the context instantiated? Should be on-demand.
2. **sherpa-onnx / Parakeet session allocated non-lazily.** If ONNX runtime flipped from mmap to heap, 3–4× on-disk size. Check `services/inference/sherpa_parakeet.rs` for `use_arena = true` / `graph_optimization_level` changes.
3. **Recording buffer not freed after transcription.** Check `services/audio/capture.rs` + `preprocess.rs` — ringbuf should be bounded.
4. **WebView2 over-commit.** Usually caps at ~500 MB so probably not the whole story.

First diagnostic: VMMap (SysInternals) or `tasklist /FI "IMAGENAME eq Utterbar.exe" /V /FO LIST` to split private bytes vs working set vs shared, correlate with subsystem.

### 10.7 History page two-column layout

[HistoryRoute.tsx](Utterbar%20Windows%20App/src/routes/HistoryRoute.tsx) is a fixed 2-col split. Below ~1100 px the list column squeezes to 6-char-wide vertical words. v0.6.1's release notes claimed a fix — that was lipstick only, structural problem untouched.

Correct design: ONE column, full width. Each entry = card (app icon, timestamp, mode badge, transcript preview). Click opens raw/processed/context detail in a modal or slide-over. Free-tier "Full history is a Pro feature" message is the list's first card, not a half-width empty state.

Not a minor-release patch — real redesign, queue for next polish cycle.

### 10.8 Remotion 400 px mobile pass

12 MP4s render correctly at 1920×1080 but not verified at the website's mobile breakpoint. Queue for when the website is ready to consume them.

### 10.9 Docs vs code drift (from DOCS_AUDIT_v0.6.1.md)

- `docs/shortcuts.blade.php` claims all bindings are "unbound"; the app ships real defaults (see [DOCS_AUDIT_v0.6.1.md](Utterbar%20Website/DOCS_AUDIT_v0.6.1.md)).
- `docs/modes.blade.php` claims 6 built-in modes; app ships 3 (`voice-to-text`, `message`, `meeting`).
- `docs/vocabulary.blade.php` mentions a "Team" tier that doesn't exist.
- `docs/troubleshooting.blade.php` references a pill-visibility hotkey that doesn't exist; missing a v0.6.1 updater section.

Not blocking; land in priority order from the audit doc.

---

## 11. Gotchas (absorbed from both sides)

### 11.1 Build / CI

- Clippy `-D warnings` fails CI on the Rust side. Run `cargo clippy --all-targets -- -D warnings` locally before pushing.
- Unused `tauri::Manager` imports have bitten us twice.
- Frontend CI job runs pnpm lint + pnpm build; both must be green.

### 11.2 Release / deploy

- `publish-release.sh` needs `aws` + `jq`. Fresh Windows: `winget install Amazon.AWSCLI jqlang.jq`.
- On fresh machines, seed `~/.aws/config` + `~/.aws/credentials` from `~/.utterbar-deploy/.env`.
- `publish-release.sh` with `-d "$PAYLOAD"` mangles multiline JSON on git-bash Windows. Use `--data-binary "@$TMPFILE"`.
- R2 legacy-key uploads are currently double-done. Drop after website fix.

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

## 12. Current state (2026-04-23)

- Latest shipped version: **v0.6.1** (stable, tag=latest).
- Stripe: **live mode**, £39.99 GBP, Stripe Tax on, adaptive pricing on.
- Ed25519 keypair: **sandbox** still in use. Rotate at v0.7.
- Auto-updater endpoint: **not built server-side** (known drift).
- Pricing API endpoint: **not built server-side** (known drift). App always falls back to `$29` (also wrong — should be `£39.99`).
- Remotion: all 12 MP4s + 12 posters + 5 stills rendered and consumed from `public/`. Pixel-parity stills pending Jay sign-off.
- Test licences in DB: id=1 (`UB-B31Q-PMXB-174D-SPQK-7B4F`, `jayparry1@gmail.com`, sandbox) + id=2+ smoke tests.
- Admin notify: `hello@utterbar.com`.
- Latest app-repo commits: `c764b05` (v0.6.1) → `c394161` (marketing) → `3914e9f` (fmt) → `76d200c` (clippy).

---

## 13. Queue for next Claude in line

Ranked by user-impact:

1. **Build the `/api/updater/check` endpoint** on the website. Users on v0.6.0 can't auto-upgrade until this ships. §4.4.
2. **Fix the in-app pricing fallback** to `£39.99 GBP` in useProPrice.ts, pricing.rs, and the release-notes boilerplate. §10.1.
3. **Build the `/api/pricing` endpoint** on the website so the app can fetch the live price instead of falling back. §4.3.
4. **Investigate 4.9 GB idle RAM.** §10.6.
5. **Drop the legacy R2 key** from the admin API `file_not_in_bucket` check, then drop the double-upload from `publish-release.sh`. §10.4.
6. **Patch `publish-release.sh` curl** to `--data-binary @file`. §10.5.
7. **Redesign History page** as single-column + modal detail. §10.7.
8. **Sign off Remotion parity stills** with Jay; after that, the Remotion refactor is done.
9. **Land docs audit fixes** from [DOCS_AUDIT_v0.6.1.md](Utterbar%20Website/DOCS_AUDIT_v0.6.1.md) in priority order: shortcuts → modes → vocabulary → troubleshooting → FAQ.

---

End of unified reference. Rules live in [CLAUDE.md](CLAUDE.md); canonical release flow in [Utterbar Windows App/RELEASE.md](Utterbar%20Windows%20App/RELEASE.md); product/brand in [Utterbar Website/BRIEF.md](Utterbar%20Website/BRIEF.md); design in [Utterbar Website/DESIGN_SYSTEM.md](Utterbar%20Website/DESIGN_SYSTEM.md); licence contract in [Utterbar Website/LICENCE_APP_HANDOFF.md](Utterbar%20Website/LICENCE_APP_HANDOFF.md).
