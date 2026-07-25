# OPEN_SOURCE_PLAN.md — take Utterbar free + open source

Goal: make Utterbar **fully free and open source**. Remove all paid/licensing
machinery, keep the auto-updater, publish the repo publicly and safely.

**Decisions (locked in 2026-07-18):**
- **Keep auto-updates** — self-hosted signed updater stays (updater API + admin
  release API + Ed25519 signing). Publishing keeps working via `publish-release.sh`.
- **Kill accounts** — remove customer auth (magic link), the account portal, the
  subscriber list, and the admin **web** panel. Release publishing stays on the
  token-authed admin API (no login).
- **No Pro customers** — clean removal, no refunds, no migration.
- **Don't rewrite git history** — rotate the still-live secrets so the old copies
  are worthless; let the rest die with the systems being removed.

Work on a branch; run full CI parity (AGENTS.md §3) at each phase; British
English throughout; confirm before anything outward-facing.

---

## Phase 0 — Rotate live secrets (JAY does this; the agent cannot touch secrets)

Do this **before** the repo goes public, independent of the code work. These
values are recoverable from old commits once the repo is public:

- [ ] **Rotate `ADMIN_API_TOKEN`** — regenerate on the server (`.env`), update the
      matching `UTTERBAR_ADMIN_API_TOKEN` in `~/.utterbar-deploy/.env`. (Old token
      is in old commits.)
- [ ] **Roll the Stripe test key** + **disable the Stripe webhook endpoint**
      (both are being removed anyway; rolling makes the leaked copies dead). If
      the Stripe account won't be used again, deactivate it.
- [ ] The **licence signing private key** is in history but becomes meaningless
      once licensing is removed (nothing verifies licences anymore). No rotation
      needed — just don't reuse it for anything.
- [ ] **Confirmed safe, no action:** the updater signing key
      (`UPDATER_PRIVATE_KEY_B64`) was never committed. Keep it that way — it must
      stay operator-only in `~/.utterbar-deploy/.env`.

## Phase 1 — Desktop app: remove licensing, make everything free

**Remove (Rust):**
- `src-tauri/src/services/licence/` (whole module: api, cache, keyfmt, managed,
  request, verify, mod)
- `src-tauri/src/commands/licence.rs`, `src-tauri/src/commands/pricing.rs`
- `src-tauri/src/domain/licence.rs`
- `src-tauri/src/util/gating.rs` (or gut it to always-allow)
- `src-tauri/src/platform/windows/fingerprint.rs` (device id for the 3-device cap)
- The `utterbar://licence/…` deep-link handler in `src-tauri/src/main.rs`
  (and the `deep-link:default` capability if nothing else uses it)
- Licence fields on `app_state.rs` / `domain/settings.rs`; unregister the removed
  commands in `commands/mod.rs`.

**Flip to free (Rust):**
- Set every model's `requires_pro` to `false` in `services/models.rs` (or drop the
  field). Remove `requires_pro` gating checks in dictation/files/history/modes/
  subtitles/etc. — every feature is available to everyone.
- Remove the outbound licence-check network call. **Bonus: one fewer outbound
  call — update the "five outbound calls" copy (privacy page, footer, AGENTS/
  CLAUDE §5.1) to four.**

**Remove (frontend):**
- `src/components/common/ProNudge.tsx`, the upgrade modal, `useProPrice.ts`,
  any Pro badges / "Upgrade" CTAs, the licence/activation settings UI.
- Anything importing the removed pieces; keep the app compiling.

**Result:** the app never checks a licence, never phones home for one, and every
feature is on for everyone.

## Phase 2 — Website: remove billing + accounts, keep the updater

**Remove:**
- Stripe: `CheckoutController.php`, `StripeWebhookController.php`, the
  `stripe_webhook_events` table/migration, Stripe config.
- Licence: `Api/LicenceApiController.php`, `Admin/LicenceController.php`,
  `Models/Licence.php`, `Models/Activation.php`, `Services/Licence/*`
  (`LicenceKeys`, `LicenceSigner`), `Console/Commands/Licence*` (Keygen, List,
  Remint, Resend, Revoke, Unrevoke), `Mail/LicenceDelivery.php`, licence tables.
- Pricing: `Api/PricingApiController.php`, `Services/Pricing/VolumePricing.php`,
  the pricing/buy pages + routes.
- Accounts: `SignInController.php`, `Models/SignInToken.php`, `Mail/SignInLink.php`,
  `AccountController.php`, `Models/UpdateSubscriber.php`, and the admin **web**
  panel (`AdminController.php`, `Admin/*`) + the `auth`/`admin` middleware routes.

**Keep (this is what makes auto-updates work):**
- `Api/UpdaterApiController.php`, `Api/AdminReleaseApiController.php`,
  `Services/Releases/ReleaseValidator.php`, `Services/Releases/UpdateMetadataSigner.php`,
  `Console/Commands/UpdaterKeygen.php`, the releases table, the
  `ADMIN_API_TOKEN` (rotated) bearer auth on the admin release route.
- `DownloadController.php`, `PageController.php`, `SitemapController.php`,
  marketing pages, email templates that remain relevant.

**Rewrite marketing copy:** `/pricing` and any "£39.99 / buy / Pro" language
becomes "free and open source". Update `BRIEF.md` positioning, the OG image, the
footer. Remove the price everywhere (this reverses CLAUDE.md §5.1's price rule —
that rule is now obsolete; note it).

## Phase 3 — Open-source packaging

- [ ] **Pick a licence.** Recommend **MIT** or **Apache-2.0** (permissive; no dep
      forces GPL). Add `LICENSE` at repo root.
- [ ] **`THIRD_PARTY_NOTICES.md`** — attributions for bundled/downloaded
      components: sherpa-onnx (Apache-2.0), ONNX Runtime (MIT), Silero VAD (MIT),
      Whisper (MIT), **NVIDIA Parakeet / NeMo model weights (CC-BY-4.0 — requires
      attribution)**, DirectML (MS redistributable terms).
- [ ] **Public `README.md`** — what it is, build-from-source steps (pull the
      toolchain notes from AGENTS.md §2), download link, contributing note.
- [ ] **Delete secret-bearing setup docs** from the tree:
      `Utterbar Website/STRIPE_SETUP_TODO.md`, `LAUNCH_CHECKLIST.md`, and scrub
      the Stripe/admin-token/R2-id/licence-key references out of `MONOREPO.md`,
      `RELEASE.md`, `Utterbar Website/CLAUDE.md`. (Removing from the current tree,
      not history — the values are dead after Phase 0.)
- [ ] **Rewrite the internal docs for a public audience** or move them out of the
      published tree: `CLAUDE.md`/`AGENTS.md`/`MONOREPO.md` contain operational
      detail (server IP, deploy ritual, infra). Decide what's fine to be public
      (build instructions) vs what should be trimmed (Cloudways specifics).
- [ ] **Final secret sweep** of the *current* tree (`git grep` for
      `sk_`, `whsec_`, `_TOKEN=`, `PRIVATE_KEY`, the R2 id) → must be clean.

## Phase 4 — Go public

- [ ] All CI green; app builds + runs free with every feature on; website builds;
      updater feed still serves signed releases.
- [ ] Flip the GitHub repo to public.
- [ ] Ship a release note / blog: "Utterbar is now free and open source."

---

## Notes & non-obvious bits

- **The updater stays signed.** Open-sourcing the code does not weaken update
  security: the signing *private key* is not in the repo, only the public key
  (which is meant to be public). Users still only accept updates you signed.
- **Don't rewrite history.** Once Phase 0 rotates the live secrets, the old
  committed copies are useless. Rewriting 430 commits would break every hash and
  any fork/clone for no security gain.
- **Obsolete rules:** several CLAUDE.md §5 rules assume a paid product (the
  £39.99 price, "no invented features on /pricing"). Update or strike them as
  part of Phase 2 so the rulebook doesn't contradict the new reality.
- **Privacy win:** removing the licence check drops the desktop app to four
  outbound calls (model download, update check, pricing lookup → also gone,
  conferencing-mute list). Recount and update the durable "outbound calls" copy.
- **CI parity every phase** (AGENTS.md §3); commit in coherent chunks; don't push
  or flip-public without Jay's explicit go.
