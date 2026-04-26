# TODO.md — Utterbar work tracker

One place to see what's done, what's staged, what's live, and what's
still open. Kept at the monorepo root so both sub-projects are visible
from one glance. Update at the end of each session.

**Last updated: 2026-04-26 (post-0.8.0 catalogue refactor).**

---

## Where we are right now

**Last publicly shipped app version: v0.7.8** (file transcription
review — clickable rows, transcript modal, copy / show in folder /
delete). Tip jar is live on home + pricing + download. GA4 is
consent-gated. Per-page OG images, admin tip dashboard, /blog with
file-backed Markdown posts.

**In-tree app version: v0.8.0** — multi-engine inference scaffolding.
- New `ModelEngine` enum on `ModelInfo` (Parakeet-TDT / Whisper /
  Zipformer). Inference layer dispatches on it; only Parakeet is
  wired today, Whisper + Zipformer return errors so a user can't
  install one and crash.
- Catalogue trimmed to two entries: Parakeet v3 int8 (Free,
  Recommended) and Parakeet v3 fp16 (Pro, GPU-only).
- Parakeet v2 (int8 + fp16) **dropped entirely**. Settings + DB
  migrations rewrite any v2 reference to v3 on first run; the v2
  files on disk are left alone (the user paid the download).
- Models settings page shows engine label per row.
- Onboarding: filter is now `!requires_pro` only (no more `-v2-` exclusion).
- Pricing page reflects Pro = fp16 access, batch transcription,
  unlimited custom modes etc.
- Honest copy on /home FAQ: English-only today, Whisper multilingual
  in v0.8.1 per the three-release roadmap.

**Roadmap for the rest of 0.8.x:**
- **v0.8.1**: Whisper engine wired, multilingual support, language
  picker on Whisper models, onboarding picker gets a multilingual
  third option.
- **v0.8.2**: Zipformer engine wired (fast English, smaller).

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

**Versions in tree (verified):**
- App `package.json` / `Cargo.toml` / `tauri.conf.json`: `0.8.0`
  (in-flight at time of writing — about to bump + build + publish).
- Website `config/site.php`: `0.7.8` (informational; real release
  data served from the `releases` table).

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
