# Utterbar — Release & Deploy Guide

> How to ship a new version of the Utterbar app — **fully automatable**:
> build → upload to R2 → `curl POST /api/admin/releases` → done.
> No browser, no admin form, no manual clicks.

**Audience:** Jay, or Claude working on this repo.
**Security:** R2 credentials and the Utterbar admin API token live in `~/.utterbar-deploy/.env` on the builder's machine — never committed to this repo or any other.

---

## Architecture in one paragraph

The Utterbar installer lives in the Cloudflare R2 bucket `utterbar`, exposed publicly via `dl.utterbar.com`. Anyone can download from `https://dl.utterbar.com/releases/{version}/Utterbar-Setup-{version}.exe` — the version appears in both the path *and* the filename so a user who drags the `.exe` to their desktop still knows which build it is. Gating happens inside the app (licence check unlocks Pro), not at the download URL. The website tracks release metadata (version, SHA-256, notes, file size) in Laravel. Two ways to publish a release on the website: (a) the admin form at `/admin/releases/create` (manual), or (b) `POST /api/admin/releases` with a bearer token (automated). This guide uses (b).

---

## One-time setup (per dev machine)

### 1. Install AWS CLI

Windows (PowerShell as admin):
```powershell
winget install Amazon.AWSCLI
```

macOS / Linux:
```bash
brew install awscli
```

### 2. Get R2 credentials from Jay

Ask Jay for a **dedicated R2 dev token** (separate from the server's). He creates one in:

> Cloudflare dashboard → R2 → Manage R2 API Tokens → **Account API Tokens** → Create
> Permissions: `Object Read & Write`
> Specify buckets: `utterbar` only
> TTL: 1 year

He gives you the **Access Key ID** and **Secret Access Key**.

### 3. Get the admin API token from Jay

Ask Jay for `ADMIN_API_TOKEN` — a 64-char hex string that lives in the Utterbar server's `.env`. This is the bearer token for `POST /api/admin/releases`. Never commit it; store it only in `~/.utterbar-deploy/.env`.

### 4. Create the local credentials file

```bash
mkdir -p ~/.utterbar-deploy
chmod 700 ~/.utterbar-deploy
cat > ~/.utterbar-deploy/.env <<'EOF'
# Cloudflare R2 (from step 2)
R2_ACCESS_KEY_ID=<paste from Jay>
R2_SECRET_ACCESS_KEY=<paste from Jay>
R2_ENDPOINT=https://bf4d2d2ba6ffeb5eb8472648bacc2f78.r2.cloudflarestorage.com
R2_BUCKET=utterbar

# Utterbar admin API (from step 3)
UTTERBAR_ADMIN_API_TOKEN=<paste from Jay>
UTTERBAR_API_BASE=https://utterbar.com
EOF
chmod 600 ~/.utterbar-deploy/.env
```

Configure AWS CLI profile `utterbar-r2`:

```bash
aws configure --profile utterbar-r2
# paste R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY
# region:        auto
# output format: json
```

Then edit `~/.aws/config` (or `%USERPROFILE%\.aws\config`) and add the endpoint:

```ini
[profile utterbar-r2]
region = auto
output = json
endpoint_url = https://bf4d2d2ba6ffeb5eb8472648bacc2f78.r2.cloudflarestorage.com
s3 =
    addressing_style = virtual
```

Test:
```bash
aws s3 ls s3://utterbar/releases/ --profile utterbar-r2
```

---

## Per-release workflow

### 0. Cut the version

Bump `Cargo.toml`, `package.json`, and `tauri.conf.json` to the new version. Commit. Tag.

```bash
git commit -am "Release v0.7.0"
git tag v0.7.0
```

### 1. Build the installer

```bash
pnpm tauri build
```

Output: `src-tauri/target/release/bundle/nsis/Utterbar_<version>_x64-setup.exe`

### 2. Write release notes

Create `releases/<version>.md` in this repo, markdown format:

```markdown
- First user-facing change
- Second user-facing change
- Third user-facing change

Fixed:
- Bug fix
- Bug fix
```

Commit the `.md` — it's the canonical changelog.

### 3. Use `scripts/publish-release.sh`

The full workflow in one command. See the script source at the bottom of this doc; you can copy it verbatim into `scripts/publish-release.sh` in this repo.

```bash
./scripts/publish-release.sh 0.7.0 "src-tauri/target/release/bundle/nsis/Utterbar_0.7.0_x64-setup.exe" releases/0.7.0.md
```

Flags:
- `--announce` — send the release email to all confirmed update-subscribers (only for stable releases you're confident about)
- `--beta` — channel = beta (defaults to stable)
- `--tag "latest"` — small cyan pill shown next to the version on the download page
- `--dry-run` — print what would happen, don't upload or POST

On success you'll see:

```
→ computing SHA-256… 47bd8249f82d1210f500d738bd0529a06f5970fa1e5282138bb6d9acf17920da
→ uploading to s3://utterbar/releases/0.7.0/Utterbar-Setup-0.7.0.exe…
  complete (10.4 MB uploaded in 2.1s)
→ verifying at https://dl.utterbar.com/releases/0.7.0/Utterbar-Setup-0.7.0.exe… 200 OK, size matches
→ publishing metadata to utterbar.com…
✓ Released v0.7.0
  id:           4
  download_url: https://dl.utterbar.com/releases/0.7.0/Utterbar-Setup-0.7.0.exe
  size:         10,962,662 bytes
  announced_to: 142 subscribers
  admin:        https://utterbar.com/admin/releases
```

If anything fails, the script prints the error and exits non-zero. Safe to re-run — the admin API rejects duplicate (version, channel) pairs with `409 version_exists`, so a half-done release doesn't re-mint.

---

## Manual equivalent (if the script is unavailable or you want to understand what it does)

### 3a. Compute the SHA-256

Windows (PowerShell):
```powershell
(Get-FileHash src-tauri/target/release/bundle/nsis/Utterbar_0.7.0_x64-setup.exe -Algorithm SHA256).Hash.ToLower()
```

macOS / Linux:
```bash
sha256sum src-tauri/target/release/bundle/nsis/Utterbar_0.7.0_x64-setup.exe | awk '{print $1}'
```

### 3b. Upload to R2

```bash
aws s3 cp \
  "src-tauri/target/release/bundle/nsis/Utterbar_0.7.0_x64-setup.exe" \
  "s3://utterbar/releases/0.7.0/Utterbar-Setup.exe" \
  --profile utterbar-r2
```

### 3c. Verify it's live on the CDN

```bash
curl -I "https://dl.utterbar.com/releases/0.7.0/Utterbar-Setup.exe"
# expect: HTTP/2 200, content-length matching the .exe
```

### 3d. POST the metadata

```bash
source ~/.utterbar-deploy/.env
NOTES=$(cat releases/0.7.0.md)

curl -s -X POST "$UTTERBAR_API_BASE/api/admin/releases" \
  -H "Authorization: Bearer $UTTERBAR_ADMIN_API_TOKEN" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d "$(jq -n \
      --arg v "0.7.0" \
      --arg sha "47bd8249f82d1210f500d738bd0529a06f5970fa1e5282138bb6d9acf17920da" \
      --arg notes "$NOTES" \
      '{version: $v, channel: "stable", sha256: $sha, notes_markdown: $notes, announce: true, tag: "latest"}'
  )"
```

Expected response (HTTP 201):

```json
{
  "ok": true,
  "id": 4,
  "version": "0.7.0",
  "channel": "stable",
  "download_url": "https://dl.utterbar.com/releases/0.7.0/Utterbar-Setup.exe",
  "size_bytes": 10962662,
  "announced_to": 142,
  "admin_url": "https://utterbar.com/admin/releases"
}
```

---

## API reference

### `POST /api/admin/releases`

**Auth:** `Authorization: Bearer <ADMIN_API_TOKEN>`

**Body (JSON):**
```json
{
  "version": "0.7.0",                 // required, semver-ish
  "channel": "stable",                // optional, "stable" | "beta" (default: stable)
  "r2_key":  "releases/0.7.0/Utterbar-Setup.exe",  // optional, defaults to this pattern from version
  "tag":     "latest",                // optional, small pill shown on download page
  "sha256":  "47bd8249…",             // optional but recommended, 64 hex chars
  "notes_markdown": "- first change\n- second change",  // required
  "announce": true,                   // optional, fires the release email
  "announce_subject": "Utterbar 0.7.0 is out"  // optional, defaults to this format
}
```

**Responses:**

| Code | Meaning |
|---|---|
| 201 | Created. Response body includes id, download_url, size_bytes, announced_to. |
| 401 | Missing or wrong bearer token. |
| 409 | `version_exists` — that (version, channel) pair is already published. Nothing created. |
| 422 | Validation failure (missing/malformed fields, bad SHA-256, file not in R2 bucket). |
| 503 | Admin API is not configured on the server (`ADMIN_API_TOKEN` unset). |

File must already exist at `r2_key` (or the default derived from version). If it doesn't, you get 422 `file_not_in_bucket` with the key it tried in `meta.r2_key`.

### `DELETE /api/admin/releases/{id}`

**Auth:** same bearer token.

**Query param:** `?delete_file=1` to also remove the `.exe` from R2 (otherwise just deletes the DB row, R2 object remains).

**Response 200:**
```json
{ "ok": true, "deleted_file": true, "version": "0.5.0" }
```

Use this to remove a buggy release. Don't delete a working release unless you also want to revoke its download URL — existing users who already installed aren't affected (the binary is already on their machine).

---

## The publish-release.sh script

Put this in `scripts/publish-release.sh` in this repo. Commit it — it's not sensitive.

```bash
#!/usr/bin/env bash
# scripts/publish-release.sh — build → upload → publish → announce.
# Usage: ./scripts/publish-release.sh <version> <path/to/.exe> <path/to/notes.md> [flags]
# Flags: --announce | --beta | --tag "<tag>" | --dry-run
set -euo pipefail

VERSION="${1:-}"
EXE_PATH="${2:-}"
NOTES_PATH="${3:-}"
shift 3 2>/dev/null || true

CHANNEL="stable"
ANNOUNCE="false"
TAG=""
DRY="false"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --announce) ANNOUNCE="true"; shift ;;
        --beta)     CHANNEL="beta"; shift ;;
        --tag)      TAG="$2"; shift 2 ;;
        --dry-run)  DRY="true"; shift ;;
        *)          echo "unknown flag: $1" >&2; exit 2 ;;
    esac
done

if [[ -z "$VERSION" || -z "$EXE_PATH" || -z "$NOTES_PATH" ]]; then
    echo "usage: $0 <version> <exe_path> <notes_md_path> [--announce] [--beta] [--tag \"latest\"] [--dry-run]" >&2
    exit 2
fi
if [[ ! -f "$EXE_PATH"   ]]; then echo "exe not found: $EXE_PATH" >&2; exit 1; fi
if [[ ! -f "$NOTES_PATH" ]]; then echo "notes not found: $NOTES_PATH" >&2; exit 1; fi

# Load secrets from ~/.utterbar-deploy/.env
ENV_FILE="$HOME/.utterbar-deploy/.env"
if [[ ! -f "$ENV_FILE" ]]; then echo "missing $ENV_FILE — see RELEASE.md" >&2; exit 1; fi
set -a; source "$ENV_FILE"; set +a

KEY="releases/$VERSION/Utterbar-Setup.exe"
NOTES=$(cat "$NOTES_PATH")

echo "→ computing SHA-256…"
SHA=$(sha256sum "$EXE_PATH" | awk '{print $1}')
echo "  $SHA"

if [[ "$DRY" == "true" ]]; then
    echo "DRY RUN — would:"
    echo "  upload:   aws s3 cp $EXE_PATH s3://$R2_BUCKET/$KEY --profile utterbar-r2"
    echo "  announce: $ANNOUNCE"
    echo "  channel:  $CHANNEL"
    echo "  tag:      $TAG"
    exit 0
fi

echo "→ uploading to s3://$R2_BUCKET/$KEY…"
aws s3 cp "$EXE_PATH" "s3://$R2_BUCKET/$KEY" --profile utterbar-r2

echo "→ verifying at https://dl.utterbar.com/$KEY…"
HTTP=$(curl -s -o /dev/null -w '%{http_code}' -I "https://dl.utterbar.com/$KEY")
if [[ "$HTTP" != "200" ]]; then
    echo "  verify failed: HTTP $HTTP" >&2; exit 1
fi
echo "  200 OK"

echo "→ publishing metadata to $UTTERBAR_API_BASE…"
PAYLOAD=$(jq -n \
    --arg v "$VERSION" \
    --arg ch "$CHANNEL" \
    --arg sha "$SHA" \
    --arg notes "$NOTES" \
    --arg tag "$TAG" \
    --argjson ann "$ANNOUNCE" \
    '{version:$v, channel:$ch, sha256:$sha, notes_markdown:$notes, announce:$ann} + (if $tag == "" then {} else {tag:$tag} end)')

RESP=$(curl -s -w "\n%{http_code}" -X POST "$UTTERBAR_API_BASE/api/admin/releases" \
    -H "Authorization: Bearer $UTTERBAR_ADMIN_API_TOKEN" \
    -H "Accept: application/json" \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD")

BODY=$(echo "$RESP" | head -n -1)
CODE=$(echo "$RESP" | tail -n 1)

if [[ "$CODE" != "201" ]]; then
    echo "✗ release API returned $CODE:" >&2
    echo "$BODY" | jq . >&2 || echo "$BODY" >&2
    exit 1
fi

echo "✓ Released v$VERSION"
echo "$BODY" | jq -r '"  id:           \(.id)\n  download_url: \(.download_url)\n  size:         \(.size_bytes) bytes\n  announced_to: \(.announced_to) subscribers\n  admin:        \(.admin_url)"'
```

Requires `jq`, `curl`, `aws`, `sha256sum`. All standard on macOS / Linux; on Windows use Git Bash or WSL.

---

## Admin cross-reference

Back-office surfaces (auth required + email on `ADMIN_EMAILS` allow-list):

| URL | What |
|---|---|
| `/admin` | Overview dashboard |
| `/admin/releases` | List all releases, delete, create new |
| `/admin/releases/create` | Manual release form (browser equivalent of the API) |

The admin form and the API both talk to the same backend; neither is better than the other. Use the API for scripted releases and the form for one-off edits or quick-fix deletes.

---

## Upgrade-survival guarantee

An NSIS upgrade replaces the contents of `C:\Program Files\Utterbar\` (or the per-user install dir — `installMode: "currentUser"` puts us under `%LOCALAPPDATA%\Programs\Utterbar\`). Anything inside that folder is wiped and re-extracted; anything outside it survives. Utterbar deliberately keeps all user-facing state outside the install dir:

- **Licence blob** → Windows Credential Manager (DPAPI-encrypted, service `Utterbar/Licence`). Independent of the install dir — survives uninstall, reinstall, even "Add/Remove Programs → Repair". See `src-tauri/src/services/licence/cache.rs`.
- **Settings** → `%APPDATA%\Utterbar\config\settings.json`. Survives. See `src-tauri/src/services/settings.rs`.
- **History DB** → `%APPDATA%\Utterbar\data\*.sqlite`. Survives.
- **Downloaded models** → `%APPDATA%\Utterbar\models\`. Survives.
- **Recordings** (if `retain_audio` is on) → `%APPDATA%\Utterbar\recordings\`. Survives.

If a future refactor ever moves any of these into the install dir or a temp folder, the user's licence and preferences disappear on the next update — users can't distinguish that from "my licence got deleted". Please don't.

---

## Security notes

- **Never commit any of the three tokens** (R2 access key, R2 secret, admin API token). `.env.local` files and `~/.utterbar-deploy/.env` are untracked.
- **Rotate tokens annually** at minimum, or immediately on suspected compromise. R2 tokens rotate via Cloudflare dashboard; admin API token rotates via Jay editing the server `.env`.
- **One token per machine.** If you work across a laptop and desktop, each gets its own R2 dev token. Makes revocation scoped.
- **Delete compromised releases.** If you ever publish a build that ships something sensitive (debug logs, leaked credential, etc.), call `DELETE /api/admin/releases/{id}?delete_file=1` immediately. Users who already installed aren't recovered, but new downloads get blocked.

---

## Changelog of this document

- **2026-04-23** — Rewrote around the headless `POST /api/admin/releases` API. First version was admin-form-only.
- **2026-04-23** — Initial (admin-form-only flow).
