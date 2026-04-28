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

# File name carries the version too (not just the path segment) so a user who
# drags the downloaded installer anywhere still knows what version it is.
# Rule: every release asset includes the version in the filename.
KEY="releases/$VERSION/Utterbar-Setup-$VERSION.exe"
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

# Updater metadata signing. Signs the canonical blob with the operator-local
# Ed25519 key from UPDATER_PRIVATE_KEY_B64. The server stores the signature
# but never sees the private key; the desktop client embeds the public key
# at compile time and verifies before downloading the installer. See
# docs canon: app/Services/Releases/UpdateMetadataSigner.php.
SIG=""
if [[ -n "${UPDATER_PRIVATE_KEY_B64:-}" ]]; then
    DOWNLOAD_URL="https://dl.utterbar.com/$KEY"
    SIZE=$(stat -c %s "$EXE_PATH" 2>/dev/null || stat -f %z "$EXE_PATH")
    echo "→ signing updater metadata…"
    SIG=$(UPDATER_PRIVATE_KEY_B64="$UPDATER_PRIVATE_KEY_B64" \
        python3 "$(dirname "$0")/sign-update-metadata.py" \
            --version "$VERSION" \
            --channel "$CHANNEL" \
            --size-bytes "$SIZE" \
            --sha256 "$SHA" \
            --download-url "$DOWNLOAD_URL") || {
                echo "✗ signing failed; aborting publish" >&2; exit 1
            }
    echo "  signed (${#SIG} chars)"
else
    echo "⚠  UPDATER_PRIVATE_KEY_B64 not set — releasing WITHOUT updater signature."
    echo "    Clients on >=0.9.0 will refuse this update once UPDATER_PUBLIC_KEY_B64"
    echo "    is configured server-side. OK only for first-time bootstrap."
fi

echo "→ publishing metadata to $UTTERBAR_API_BASE…"
# Include `r2_key` explicitly so the server doesn't fall back to its
# legacy default of `releases/{version}/Utterbar-Setup.exe` (unversioned).
# The admin API then verifies that KEY exists in R2 (the key we just
# uploaded to above, $KEY). Without this, the server defaulted to the
# legacy key, saw no file there, and returned `file_not_in_bucket`
# every time.
#
# MSYS_NO_PATHCONV=1 disables Git-Bash on Windows from auto-converting
# argument values that look like Unix paths. Ed25519 signatures are
# random base64; ~1 in 4 of them happens to start with `/` and triggers
# MSYS to silently rewrite the value to `C:/Program Files/Git/<rest>`,
# producing an invalid signature the server can't verify. Found this
# during the v0.11.1 publish; before that, lucky signatures didn't
# start with `/` and the bug didn't surface.
# Stable + announce destructive-flag confirmation. The server-side
# release validator (post-v0.14.13 hardening) refuses publish when
# channel=stable AND announce=true unless confirm_stable_announce is
# also true. Set it automatically here so the existing CLI invocation
# (`publish-release.sh <ver> <exe> <notes> --announce --tag "latest"`)
# keeps working without operators having to remember a new flag — the
# guard exists for the *browser* form where a typo on the announce
# checkbox could blast subscribers, not for the deliberate CLI call
# the operator runs after re-reading the runbook.
CONFIRM_STABLE_ANNOUNCE="false"
if [[ "$ANNOUNCE" == "true" && "$CHANNEL" == "stable" ]]; then
    CONFIRM_STABLE_ANNOUNCE="true"
fi

PAYLOAD=$(MSYS_NO_PATHCONV=1 jq -n \
    --arg v "$VERSION" \
    --arg ch "$CHANNEL" \
    --arg r2 "$KEY" \
    --arg sha "$SHA" \
    --arg notes "$NOTES" \
    --arg tag "$TAG" \
    --arg sig "$SIG" \
    --argjson ann "$ANNOUNCE" \
    --argjson confirm "$CONFIRM_STABLE_ANNOUNCE" \
    '{version:$v, channel:$ch, r2_key:$r2, sha256:$sha, notes_markdown:$notes, announce:$ann, confirm_stable_announce:$confirm}
        + (if $tag == "" then {} else {tag:$tag} end)
        + (if $sig == "" then {} else {updater_signature:$sig} end)')

# Write the payload to a temp file and curl via --data-binary @<file>
# instead of -d "$PAYLOAD". The inline -d form mangles multiline JSON
# on Windows git-bash (bash on MSYS reinterprets embedded newlines in
# a way curl's arg parsing can't recover from), producing a request
# body that the server rejects as "missing version" or similar even
# though jq generated a valid JSON blob. The temp-file form bypasses
# bash's word splitting entirely.
TMP_PAYLOAD=$(mktemp)
trap 'rm -f "$TMP_PAYLOAD"' EXIT
printf '%s' "$PAYLOAD" > "$TMP_PAYLOAD"

RESP=$(curl -s -w "\n%{http_code}" -X POST "$UTTERBAR_API_BASE/api/admin/releases" \
    -H "Authorization: Bearer $UTTERBAR_ADMIN_API_TOKEN" \
    -H "Accept: application/json" \
    -H "Content-Type: application/json" \
    --data-binary "@$TMP_PAYLOAD")

BODY=$(echo "$RESP" | head -n -1)
CODE=$(echo "$RESP" | tail -n 1)

if [[ "$CODE" != "201" ]]; then
    echo "✗ release API returned $CODE:" >&2
    echo "$BODY" | jq . >&2 || echo "$BODY" >&2
    exit 1
fi

echo "✓ Released v$VERSION"
echo "$BODY" | jq -r '"  id:           \(.id)\n  download_url: \(.download_url)\n  size:         \(.size_bytes) bytes\n  announced_to: \(.announced_to) subscribers\n  admin:        \(.admin_url)"'
