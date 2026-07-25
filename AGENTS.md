# AGENTS.md — operating guide for Codex (and any coding agent)

Entry point for an AI coding agent working in this repo. It points you at the
shared rulebook and adds the **operational runbook** (build the desktop app,
publish a release, deploy the website) with the hard-won details that aren't
obvious from the code.

> This file replaced an earlier auto-generated copy of `CLAUDE.md` that had
> find-replace damage (a bogus `Codex Opus 4.8` commit trailer, a `.Codex/`
> directory that doesn't exist). Don't reintroduce that pattern — point at the
> rulebook, don't duplicate it.

## 0. Read these first — they are authoritative

The rulebook is not agent-specific; it governs everyone. Read, in order:

1. **[CLAUDE.md](CLAUDE.md)** — the working rules: voice, engineering
   discipline, golden rules, secrets policy, what to touch where. It says
   "Claude", but it means **any** agent, including you. **Obey it.**
2. **[MONOREPO.md](MONOREPO.md)** — architecture and the cross-repo contracts.
3. **[TODO.md](TODO.md)** — live state. Drifts; trust it over §9 here.
4. **[RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md)** + **[Utterbar Windows App/RELEASE.md](Utterbar%20Windows%20App/RELEASE.md)** — the release ritual in full.

If this file disagrees with CLAUDE.md / MONOREPO.md / TODO.md, they win.

> **Commit trailer:** CLAUDE.md ends commits with a `Co-Authored-By: Claude …`
> line. That is Claude's, not yours — **do not copy it** and never claim to be
> Claude or Anthropic. Use your own attribution (or none). Everything else in
> CLAUDE.md §5.2 (why-not-what messages, British English, repo-relative paths,
> new commits over amends, don't commit unless asked) applies to you unchanged.

## 1. The two sub-projects

| Directory | What | Stack |
| --- | --- | --- |
| `Utterbar Windows App/` | The desktop dictation app | Tauri v2 + React + Rust |
| `Utterbar Website/` | Marketing site + Ed25519-signed updater/admin release APIs | Laravel 12 + Blade + Alpine + Tailwind v4 |

Single git repo, root here. Remote `badoomtch/Utterbar-App` (private). Website
deploy is out-of-band over SSH (Cloudways has no git deploy).

## 2. Toolchain / prerequisites

One-time setup on the machine Codex runs on.

**Desktop app build (Windows):**
- Rust (stable) + `cargo`; Node + `pnpm` (`npm i -g pnpm`).
- **LLVM/libclang** — `winget install LLVM.LLVM`, then a persistent user env var
  `LIBCLANG_PATH=C:\Program Files\LLVM\bin`. Without it the sherpa bindgen step
  won't compile.
- sherpa-onnx is **vendored** at `Utterbar Windows App/src-tauri/vendor/sherpa-onnx/`
  and linked via `SHERPA_LIB_PATH` in the in-repo `.cargo/config.toml` — works
  from a clean checkout, no action. `runtime/*.dll` are build-staged by
  `scripts/stage-sherpa-dlls.ps1` (gitignored).
- The Silero VAD model (Meeting Mode) is committed at
  `src-tauri/vendor/silero_vad.onnx` (a `.gitignore` exception) and bundled next
  to the exe via `tauri.conf.json → bundle.resources`. Don't delete it.

**Publishing a release additionally needs:**
- **AWS CLI** — `winget install Amazon.AWSCLI`. In Git Bash it's usually **not on
  PATH**; it lives at `C:/Program Files/Amazon/AWSCLIV2/aws.exe`, so prepend
  `export PATH="/c/Program Files/Amazon/AWSCLIV2:$PATH"`. An AWS profile
  **`utterbar-r2`** (in `~/.aws/`) must hold the Cloudflare R2 key/secret/endpoint;
  the script uploads with `--profile utterbar-r2`.
- **jq** — `winget install jqlang.jq`.
- **Python + `cryptography`** — for updater signing. **Gotcha:** on Windows
  `python3` is the Microsoft Store stub; the real interpreter is `python`. Shim
  it before publishing:
  ```sh
  mkdir -p /tmp/pybin && printf '#!/bin/sh\nexec python "$@"\n' > /tmp/pybin/python3 \
    && chmod +x /tmp/pybin/python3 && export PATH="/tmp/pybin:$PATH"
  python3 -c "import cryptography"   # must succeed
  ```

**Website deploy additionally needs:** PuTTY (`winget install PuTTY.PuTTY` →
`pscp` + `plink`), plus `composer`/`npm` locally.

## 3. Build & test — full CI parity (run ALL before pushing/shipping)

```sh
# Rust — from "Utterbar Windows App/src-tauri"
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test

# Frontend — from "Utterbar Windows App"
pnpm install
pnpm typecheck
pnpm lint
pnpm vitest run
pnpm build
```

**Generated-type churn:** `cargo build`/`test` regenerate `src/generated/*.ts`
via ts-rs, and Windows LF↔CRLF makes ~70 of them show "modified" with no real
change. After building, keep only real changes:
```sh
git diff --ignore-all-space --stat -- src/generated/   # the REAL changes
git checkout -- src/generated/                         # discard the CRLF noise, re-stage only real ones
```
Never stage the whitespace-only churn.

## 4. Ship a desktop release

**Versioning (CLAUDE.md §5.7):** minor `0.x.0` = **stable** channel; patch
`0.x.y` (y>0) = **beta** only. Features land on beta patches, soak, then promote
by cutting the next minor.

1. **Bump** three files, then sync the lock — **never `sed` Cargo.lock**:
   ```sh
   node scripts/bump-version.mjs <version>                 # from "Utterbar Windows App"
   (cd src-tauri && cargo update -p utterbar --precise <version>)
   ```
2. **Write** `releases/<version>.md` — user-facing changelog. British English,
   no em dashes, second person, **no internals** (no file paths, struct names,
   VAD/ONNX/sherpa jargon, exploit-level security detail). CLAUDE.md §5.8.
3. **Build:** `pnpm tauri:build:release` (includes `--features experimental_mic_mute`).
   → `src-tauri/target/release/bundle/nsis/Utterbar_<version>_x64-setup.exe`.
4. **§4 hard checks** (don't skip):
   - `sha256sum` the installer (record it).
   - `grep -ac send_hotkey_globally src-tauri/target/release/utterbar.exe` → must be ≥ 1.
   - `src-tauri/target/release/silero_vad.onnx` must exist (VAD model bundled).
5. **Smoke-test** — Jay installs and confirms. Capture/meeting behaviour can't be
   validated from code; a human runs it.
6. **Publish** (needs §2 tools + `~/.utterbar-deploy/.env`):
   ```sh
   export PATH="/tmp/pybin:/c/Program Files/Amazon/AWSCLIV2:$PATH"
   ./scripts/publish-release.sh <version> \
       "src-tauri/target/release/bundle/nsis/Utterbar_<version>_x64-setup.exe" \
       releases/<version>.md \
       [--beta]            # beta channel; omit for stable
       [--tag "latest"]    # stable "latest" pill; stable only
       # (--announce is a no-op now — the subscriber list was removed)
   ```
   Uploads to R2 (`releases/<v>/Utterbar-Setup-<v>.exe`), verifies CDN 200, signs
   the metadata with `UPDATER_PRIVATE_KEY_B64`, POSTs to `/api/admin/releases`.
   **Ships to real users — confirm with Jay first; every step deliberate** (CLAUDE.md §5.6).
7. **Verify** the feed:
   ```sh
   curl -s "https://utterbar.com/api/updater/check?current=0.0.0&channel=beta"    | jq .
   curl -s "https://utterbar.com/api/updater/check?current=0.20.0&channel=stable" | jq .
   ```
   Expect `available:true`, the right `version`, and a signature present.
8. **Tag** `v<version>` after committing the bump + notes. Push only when Jay asks.

**Microsoft Store SKU:** ship direct first, then point the Partner Center
submission at the exact R2 key
`https://dl.utterbar.com/releases/<version>/Utterbar-Setup-<version>.exe`. Don't
conflate the two SKUs in commits/notes (CLAUDE.md §5.7).

## 5. Deploy the website (Cloudways, over SSH)

No git deploy. From `Utterbar Website/`:
```sh
pwsh ./scripts/deploy-website.ps1     # reads Cloudways creds from ~/.utterbar-deploy/.env
```
It stages a clean payload (excludes `vendor/`, `node_modules/`, `.env`, logs,
tests), `pscp`-copies to `~/public_html`, clears caches, rebuilds assets,
health-checks. Manual equivalent:
```sh
pscp -pw "$PASS" -batch <local> utter@178.62.6.215:public_html/<remote>
plink utter@178.62.6.215 "cd ~/public_html && php artisan view:clear && php artisan config:clear && php artisan route:clear"
# CSS/JS changes need a server rebuild on Node 22 (default Node is too old for Tailwind v4):
plink utter@178.62.6.215 "cd ~/public_html && PATH=\$HOME/tmp/node22/bin:\$PATH HOME=\$HOME/tmp/npmhome npm run build"
```
Cloudflare caches PNG/MP4 hard; the `<x-video-embed>`/`<x-still>` helpers append
`?v={mtime}` to bust it. **Deploying is live and outward-facing — confirm first.**

## 6. Secrets & credentials — what's needed, and how Jay gives them

**No secret value ever goes in the repo, a commit, or chat.** The `.env` files
are gitignored and off-repo. **Jay places these on Codex's machine himself** —
the agent must never read, echo, or generate them. Values omitted here on
purpose.

`~/.utterbar-deploy/.env` (per-user, off-repo) — read by `publish-release.sh`
and `deploy-website.ps1`:

| Name | Purpose | Kind |
| --- | --- | --- |
| `UPDATER_PRIVATE_KEY_B64` | Ed25519 key that signs updater metadata. **Critical** — without it, releases publish unsigned and clients ≥0.9.0 refuse them. | secret |
| `UTTERBAR_ADMIN_API_TOKEN` | Bearer for `POST /api/admin/releases`; must match the server's `ADMIN_API_TOKEN`. | secret |
| `UTTERBAR_API_BASE` | e.g. `https://utterbar.com`. | config |
| `R2_BUCKET` | Cloudflare R2 bucket name. | config |
| `CLOUDWAYS_HOST` + `CLOUDWAYS_PASSWORD` | App-user SSH for website deploy (or the master set `CLOUDWAYS_MASTER_USER`/`_MASTER_PASS`/`_IP`/`_APP_FOLDER`). | secret |

Also required outside the `.env`:
- **AWS profile `utterbar-r2`** in `~/.aws/credentials` + `~/.aws/config` — the R2
  access key, secret, endpoint.
- **`LIBCLANG_PATH`** env var (§2).

The **live website `.env`** (on the server, not local) holds
`UPDATER_PUBLIC_KEY_B64`, `ADMIN_API_TOKEN`, `ADMIN_NOTIFY_EMAIL`, mail config.
(The Stripe keys and licence keypair are gone — that stack was removed.) Codex
only touches those by editing on the server during a deploy, never by
committing. `.env.example` (tracked) shows the shape with blank values. **Verify
production `APP_DEBUG=false`.**

The **desktop build needs no signing secret** — Utterbar is deliberately not
code-signed (SmartScreen warnings on first install are expected).

## 7. Golden rules you must not break (CLAUDE.md §5 — recap)

- **Confirm before anything outward-facing / hard to reverse:** publish, deploy,
  admin-API POST, tag, push, key rotation, live DB migration. Local reversible
  edits + tests: just go.
- **British English** (*licence*, *customise*, *colour*); **no em dashes** in copy.
- **Free and open source (MIT). No price, no paid tier, no licence, no account.**
  Never re-add a price (`£39.99`/`£40`/`$49`/`$29`), a "Pro" tier, Stripe, or a
  checkout (CLAUDE.md §5.1).
- **No invented features; no telemetry / phone-home; no LLM rewrite** — all
  durable REJECTED decisions (CLAUDE.md §5.1).
- **One word: `Utterbar`.**
- **Don't commit unless asked.** New commits over amends; no `--amend`/`--force`/
  `--no-verify`/`reset --hard` without explicit permission. On `main`, branch first.
- **Cross-side changes** (updater/admin release API, or a Remotion-imported
  component) touch BOTH sides in one pass (CLAUDE.md §5.5).

## 8. Where to look when something breaks

See CLAUDE.md §7 for the full table. Highlights: updater not appearing → hit
`/api/updater/check`; publish 422 `file_not_in_bucket` → the script's upload key
and the admin controller's expectation drifted.

## 9. Where things stand (2026-07-18 — drifts; verify against TODO.md)

- **Stable: 0.21.0** (admin row 96, live). **Beta: 0.20.11** (row 95). In-tree: `0.21.0`.
- Recent work is on branch **`release/0.20.10`** with tags `v0.20.9/10/11` and
  `v0.21.0` — **local, not pushed.** Push only on Jay's say-so.
- Latest feature: **live Meeting Mode transcription** (two separate channels,
  Silero-VAD-gated, live transcript panel) — replaces the old batch-mixed path.
- **Open thread:** a user reported dictation dropping its final sentence.
  Diagnosis from logs: not truncation (Parakeet doesn't truncate) — the recording
  ends ~500 ms after the manual stop (`TAIL_PAD_MS` in
  `src-tauri/src/services/audio/capture.rs`), so speech past that window isn't
  captured. Candidate fix: lengthen the grace tail. Awaiting Jay's confirm on
  hold-vs-toggle first.
- **Security:** a full desktop + website audit found no exploitable issues; only
  low-severity hardening ideas (deep-link confirm prompt, `shell:allow-open`
  scope, CSP report-only → enforcing, HSTS header, confirm prod `APP_DEBUG=false`).
