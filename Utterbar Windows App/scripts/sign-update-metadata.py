#!/usr/bin/env python3
"""
sign-update-metadata.py — operator-local signer for the desktop updater.

Reads the canonical metadata blob from stdin (or from --version/--channel/...
arg flags) and emits a base64-encoded Ed25519 signature on stdout. The blob
format is fixed and MUST match the server-side and client-side reconstruction:

    utterbar-update-v1
    version=<X.Y.Z>
    channel=<stable|beta>
    size_bytes=<integer>
    sha256=<lowercase 64-hex>
    download_url=<https://dl.utterbar.com/...>

(LF separators, no trailing newline.)

The private key is loaded from the UPDATER_PRIVATE_KEY_B64 environment
variable — sourced from ~/.utterbar-deploy/.env by publish-release.sh and
NEVER committed.

Why python and not bash + openssl: openssl's CLI cannot sign with a raw
Ed25519 key without a PEM wrapper, and bash on Windows git-bash + openssl
shipping varies per machine. Python's cryptography library is one
`pip install cryptography` away on every platform we target.

Usage from publish-release.sh:

    SIG=$(UPDATER_PRIVATE_KEY_B64="$UPDATER_PRIVATE_KEY_B64" \
        python3 scripts/sign-update-metadata.py \
        --version "$VERSION" --channel "$CHANNEL" \
        --size-bytes "$SIZE" --sha256 "$SHA" \
        --download-url "https://dl.utterbar.com/$KEY")
"""
import argparse
import base64
import os
import sys

try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
except ImportError:
    sys.stderr.write(
        "missing dependency: cryptography\n"
        "install with: pip install cryptography\n"
    )
    sys.exit(2)


CANONICAL_MAGIC = "utterbar-update-v1"


def canonical_blob(version: str, channel: str, size_bytes: int, sha256: str, download_url: str) -> bytes:
    return "\n".join([
        CANONICAL_MAGIC,
        f"version={version}",
        f"channel={channel}",
        f"size_bytes={size_bytes}",
        f"sha256={sha256.lower()}",
        f"download_url={download_url}",
    ]).encode("utf-8")


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--version", required=True)
    p.add_argument("--channel", required=True, choices=["stable", "beta"])
    p.add_argument("--size-bytes", required=True, type=int)
    p.add_argument("--sha256", required=True)
    p.add_argument("--download-url", required=True)
    args = p.parse_args()

    sk_b64 = os.environ.get("UPDATER_PRIVATE_KEY_B64", "").strip()
    if not sk_b64:
        sys.stderr.write("UPDATER_PRIVATE_KEY_B64 not set in environment\n")
        return 2

    try:
        sk_bytes = base64.b64decode(sk_b64, validate=True)
    except Exception as e:
        sys.stderr.write(f"UPDATER_PRIVATE_KEY_B64 is not valid base64: {e}\n")
        return 2

    # libsodium's secret key is 64 bytes (seed||public). Ed25519PrivateKey
    # wants the 32-byte seed, which is the first 32 bytes.
    if len(sk_bytes) == 64:
        seed = sk_bytes[:32]
    elif len(sk_bytes) == 32:
        seed = sk_bytes
    else:
        sys.stderr.write(f"UPDATER_PRIVATE_KEY_B64 must decode to 32 or 64 bytes (got {len(sk_bytes)})\n")
        return 2

    blob = canonical_blob(
        version=args.version,
        channel=args.channel,
        size_bytes=args.size_bytes,
        sha256=args.sha256,
        download_url=args.download_url,
    )

    key = Ed25519PrivateKey.from_private_bytes(seed)
    sig = key.sign(blob)
    sys.stdout.write(base64.b64encode(sig).decode("ascii"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
