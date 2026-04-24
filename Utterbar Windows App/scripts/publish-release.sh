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
