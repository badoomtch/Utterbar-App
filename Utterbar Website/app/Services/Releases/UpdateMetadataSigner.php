<?php

namespace App\Services\Releases;

/**
 * Canonical metadata-blob format for updater signatures.
 *
 * The desktop updater verifies an Ed25519 signature over a deterministic,
 * line-oriented byte sequence built from the fields a client cares about:
 * which version it would download, where from, and what hash it should match.
 * Both the operator's signing tool (in the publish-release flow) and the
 * client (in commands/updater.rs) reconstruct exactly the same bytes from
 * the same inputs — any drift between the two sides voids the signature
 * and the client refuses the update.
 *
 * Format (UTF-8, LF-separated, NO trailing newline):
 *
 *     utterbar-update-v1
 *     version=<X.Y.Z>
 *     channel=<stable|beta>
 *     size_bytes=<integer>
 *     sha256=<lowercase 64-hex>
 *     download_url=<https://dl.utterbar.com/...>
 *
 * The leading magic line ("utterbar-update-v1") gives us a hard versioning
 * lever if we ever need to change the format: bump to v2 and the v1 client
 * stops accepting v2 signatures (its blob construction won't match), forcing
 * a manual upgrade path rather than silent acceptance of a different shape.
 *
 * We deliberately put `download_url` last and include the full URL — not just
 * the r2_key — so a server flip that started serving updates from a
 * different host (an attacker mounting `dl.attacker.com` and racing the
 * URL-pin check) would fail signature verification. The client's URL-pin
 * (must start with https://dl.utterbar.com/) is a belt; this is the braces.
 */
class UpdateMetadataSigner
{
    public const MAGIC = 'utterbar-update-v1';

    /**
     * Build the exact bytes the signature is computed over.
     *
     * @param array{version:string,channel:string,size_bytes:int,sha256:string,download_url:string} $meta
     */
    public static function canonicalBlob(array $meta): string
    {
        $version     = (string) $meta['version'];
        $channel     = (string) $meta['channel'];
        $sizeBytes   = (int)    $meta['size_bytes'];
        $sha256      = strtolower((string) $meta['sha256']);
        $downloadUrl = (string) $meta['download_url'];

        return implode("\n", [
            self::MAGIC,
            "version={$version}",
            "channel={$channel}",
            "size_bytes={$sizeBytes}",
            "sha256={$sha256}",
            "download_url={$downloadUrl}",
        ]);
    }

    /**
     * Verify a base64-encoded Ed25519 signature against the canonical blob
     * built from `$meta`. Returns true only when the configured public key
     * accepts the signature. Used at publish time to ensure the operator
     * actually signed the metadata they uploaded — server never holds the
     * private key, but it CAN sanity-check that the signature parses and
     * matches the public half before persisting it.
     *
     * @param array{version:string,channel:string,size_bytes:int,sha256:string,download_url:string} $meta
     */
    public static function verify(string $signatureB64, array $meta): bool
    {
        $sig = base64_decode($signatureB64, true);
        if ($sig === false || strlen($sig) !== SODIUM_CRYPTO_SIGN_BYTES) {
            return false;
        }
        $pkB64 = (string) config('services.updater.public_key_b64');
        if ($pkB64 === '') {
            return false;
        }
        $pk = base64_decode($pkB64, true);
        if ($pk === false || strlen($pk) !== SODIUM_CRYPTO_SIGN_PUBLICKEYBYTES) {
            return false;
        }
        $blob = self::canonicalBlob($meta);
        return sodium_crypto_sign_verify_detached($sig, $blob, $pk);
    }
}
