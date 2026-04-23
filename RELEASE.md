# Utterbar — Release & Deploy Guide

> How to ship a new version of the Utterbar app: build → upload to Cloudflare R2 → publish on utterbar.com → optionally email subscribers.
>
> **Audience:** Jay, or Claude working on this repo.
> **Do not commit R2 credentials to this repo.** Ever. They live in `.env.local` (gitignored) or your shell environment.

---

## Architecture in one paragraph

The Utterbar installer lives in a Cloudflare R2 bucket called `utterbar`, exposed publicly via the custom domain `dl.utterbar.com`. Anyone can download from `https://dl.utterbar.com/releases/{version}/Utterbar-Setup.exe` — gating happens inside the app (licence key unlocks Pro features), not at the download URL. The website's `/download` page and `/admin/releases` admin form pull the current version metadata from the Laravel DB. A release row in the DB connects: the version string → the R2 object key → the user-facing release notes.

---

## One-time setup (per dev machine)

### 1. Install AWS CLI

Windows (PowerShell as admin):
```powershell
winget install Amazon.AWSCLI
```

macOS:
```bash
brew install awscli
```

Verify:
```bash
aws --version
```

### 2. Get R2 credentials

Ask Jay for a **dedicated R2 dev token**. The server has its own token — don't share it with dev machines. Jay creates one in:

> Cloudflare dashboard → R2 → Overview → Manage R2 API Tokens → **User API Tokens** → Create User API Token
> Permissions: `Object Read & Write`
> Specify buckets: `utterbar` only
> TTL: 1 year (rotate annually)

Jay gives you the **Access Key ID** and **Secret Access Key**. The endpoint URL is fixed:

```
https://bf4d2d2ba6ffeb5eb8472648bacc2f78.r2.cloudflarestorage.com
```

### 3. Configure an AWS CLI profile named `utterbar-r2`

```bash
aws configure --profile utterbar-r2
```

When prompted:
```
AWS Access Key ID:     <paste from Jay>
AWS Secret Access Key: <paste from Jay>
Default region name:   auto
Default output format: json
```

Then edit `~/.aws/config` (or `%USERPROFILE%\.aws\config` on Windows) and add the R2 endpoint to that profile:

```ini
[profile utterbar-r2]
region = auto
output = json
endpoint_url = https://bf4d2d2ba6ffeb5eb8472648bacc2f78.r2.cloudflarestorage.com
s3 =
    addressing_style = virtual
```

Test it:

```bash
aws s3 ls s3://utterbar/ --profile utterbar-r2
```

You should get an empty listing (if nothing uploaded yet) or the existing files, not an error.

### 4. Get an admin login on the website

Ask Jay to add your email to `ADMIN_EMAILS` on the server if it isn't already. Then sign in at <https://utterbar.com/sign-in>. After first sign-in, `https://utterbar.com/admin` and `/admin/releases` are reachable.

---

## Per-release workflow

### 0. Cut the version

Bump `Cargo.toml`, `package.json`, and `tauri.conf.json` to the new version. Commit. Tag.

```bash
git commit -am "Release v0.5.1"
git tag v0.5.1
```

### 1. Build the installer

```bash
pnpm tauri build
```

Output lives in:
```
src-tauri/target/release/bundle/nsis/Utterbar_<version>_x64-setup.exe
```

### 2. Compute the SHA-256

Windows (PowerShell):
```powershell
Get-FileHash src-tauri/target/release/bundle/nsis/Utterbar_0.5.1_x64-setup.exe -Algorithm SHA256
```

macOS/Linux:
```bash
sha256sum src-tauri/target/release/bundle/nsis/Utterbar_0.5.1_x64-setup.exe
```

Copy the hex string — you'll paste it into the admin form.

### 3. Upload to R2

Follow the **canonical path convention**: `releases/{version}/Utterbar-Setup.exe`.

```bash
aws s3 cp \
    "src-tauri/target/release/bundle/nsis/Utterbar_0.5.1_x64-setup.exe" \
    "s3://utterbar/releases/0.5.1/Utterbar-Setup.exe" \
    --profile utterbar-r2
```

Verify it's reachable:

```bash
curl -I https://dl.utterbar.com/releases/0.5.1/Utterbar-Setup.exe
```

Expect `HTTP/2 200` with `content-length` matching the `.exe`'s size.

### 4. Write release notes

Create a markdown file in the Orliq repo at `releases/0.5.1.md` (new folder, commit to git). Format:

```markdown
- First user-facing thing that changed, one line
- Second thing
- Third thing
- **Bold** for the headline feature if there is one
- Keep it honest, keep it short, keep it readable to someone who has no idea what `WASAPI loopback` means

Fixed:
- Bug fix, one line
- Another bug fix
```

Keep it plain markdown — no HTML. The Laravel side renders `ul` / `li` with the brand bullets, and `**bold**`, `code`, and `[links](...)` with the brand styles. Lists are the bread and butter.

Guiding voice: "what shipped, told to a curious user". Not "what changed, told to another engineer". Save internal technical notes for git commit messages.

### 5. Publish the release on the website

Sign in at <https://utterbar.com/admin/releases>, click **New release**. Fill in:

| Field | Value |
|---|---|
| Version | `0.5.1` (must match your Cargo.toml / tauri.conf.json) |
| Channel | `stable` |
| R2 object key | `releases/0.5.1/Utterbar-Setup.exe` (pre-filled from version) |
| Tag | `latest` if you want the small cyan pill next to the version on the download page. Blank otherwise. |
| SHA-256 | the 64-char hex from step 2 |
| Notes (markdown) | paste the content of `releases/0.5.1.md` |
| **Email subscribers** | tick the box if this is a stable release worth announcing |
| Announcement subject | pre-fills to `Utterbar 0.5.1 is out` — customise if you like |

Submit. You'll land back on `/admin/releases` with a flash confirming "Release v0.5.1 (stable) published. Announced to N subscriber(s)."

### 6. Verify

- <https://utterbar.com/download> — hero card should now say `v0.5.1`, Download button should point at `dl.utterbar.com/releases/0.5.1/Utterbar-Setup.exe`
- Changelog section at the bottom should show v0.5.1 with your release notes rendered
- If you ticked "email subscribers", inbox a random active subscriber and confirm the release email arrived
- Click the download button on a clean browser and confirm the `.exe` actually downloads

### 7. If something's wrong

- **Download button 404s:** the R2 key in the form doesn't match where you uploaded. Go to `/admin/releases`, delete the row (leave "also delete from R2" unchecked), re-create with the correct key.
- **Wrong SHA-256 on the download page:** same — delete and re-create with the correct hash.
- **Release notes look wrong:** delete and re-create. No in-place edit in v1 (keep it simple).
- **Needed to un-announce an email:** you can't. If you fired the email by mistake, send a follow-up announcement with a correction and get on with your life.

---

## Admin cross-reference

Back-office surfaces (auth required + email on `ADMIN_EMAILS` allow-list):

| URL | What |
|---|---|
| `/admin` | Overview dashboard (counts, recent licences, recent webhooks) |
| `/admin/releases` | List all releases, delete, create new |
| `/admin/releases/create` | New release form |

Artisan commands (ssh into the Cloudways server):

```bash
php artisan updates:announce --release=0.5.1 --subject="..." --notes=./releases/0.5.1.md --dry-run
php artisan licence:list
php artisan licence:revoke UB-XXXX-XXXX-XXXX-XXXX-XXXX --reason="..."
php artisan licence:unrevoke UB-XXXX-XXXX-XXXX-XXXX-XXXX
```

The `updates:announce` artisan command is equivalent to ticking the "email subscribers" box on the release form, but callable offline (for re-sends or back-fills).

---

## Security notes

- **R2 credentials never in this repo.** Not in `.env`, not in `package.json`, not in `build.log`, not in a commit message. The `.gitignore` includes `.env.local` — keep everything sensitive there.
- **Rotate the R2 dev token annually.** Cloudflare dashboard → R2 → Manage API Tokens → find yours → roll.
- **Don't share profiles across machines.** Each dev machine should have its own R2 user token, so a compromise is scoped.
- **If you leak a token:** Cloudflare dashboard → R2 → Manage API Tokens → revoke it. Generate a new one. Update `aws configure --profile utterbar-r2`. Existing uploaded files are untouched; only write access is rotated.

---

## Changelog of this document

- **2026-04-23** — initial. Covers Cloudflare R2 + Laravel admin flow.
