<?php

namespace App\Services\Licence;

use App\Models\Licence;

/**
 * Signs a licence payload with the project's Ed25519 private key.
 *
 * Payload layout (32 bytes):
 *   version:u8          // 1
 *   sku:u8              // 1
 *   issued_at:u48_be    // 6  (unix milliseconds, big-endian)
 *   nonce:u16_be        // 2
 *   key_id:u64_be       // 8  (= licences.id)
 *   reserved:u8[14]     // 14 (zero-filled, room for future fields)
 *   TOTAL               // 32
 *
 * Output: 96 bytes = 32-byte payload || 64-byte Ed25519 signature.
 *
 * The Tauri app verifies the signature offline using the public key baked into
 * the binary at compile time. See LICENCE_APP_HANDOFF.md for the app-side spec.
 */
class LicenceSigner
{
    public const PAYLOAD_BYTES = 32;
    public const SIGNATURE_BYTES = 64;
    public const SIGNED_BYTES = 96;

    public static function sign(Licence $licence): string
    {
        $version = (int) config('services.licence.payload_version', 1);
        $sku     = (int) config('services.licence.sku_id', 1);
        $issuedAtMs = (int) ($licence->issued_at?->getTimestampMs() ?? round(microtime(true) * 1000));
        $nonce   = random_int(0, 65535);
        $keyId   = (int) $licence->id;

        $payload = self::buildPayload($version, $sku, $issuedAtMs, $nonce, $keyId);

        $secretKey = self::loadSecretKey();
        // Sign sha256(payload) — keeps the signed-message short and fixed-size.
        $digest = hash('sha256', $payload, true);
        $sig = sodium_crypto_sign_detached($digest, $secretKey);
        sodium_memzero($secretKey);

        return $payload . $sig;
    }

    /**
     * Build the 32-byte payload.
     */
    public static function buildPayload(int $version, int $sku, int $issuedAtMs, int $nonce, int $keyId): string
    {
        if ($version < 0 || $version > 0xFF)   throw new \InvalidArgumentException('version out of range');
        if ($sku < 0 || $sku > 0xFF)           throw new \InvalidArgumentException('sku out of range');
        if ($nonce < 0 || $nonce > 0xFFFF)     throw new \InvalidArgumentException('nonce out of range');

        // u48 BE for issued_at: top 2 bytes then low 4 bytes.
        $issuedHi = ($issuedAtMs >> 32) & 0xFFFF;
        $issuedLo = $issuedAtMs & 0xFFFFFFFF;
        $issuedBytes = pack('n', $issuedHi) . pack('N', $issuedLo);

        // u64 BE for key_id.
        $keyIdHi = ($keyId >> 32) & 0xFFFFFFFF;
        $keyIdLo = $keyId & 0xFFFFFFFF;
        $keyIdBytes = pack('N', $keyIdHi) . pack('N', $keyIdLo);

        $payload = chr($version)
                 . chr($sku)
                 . $issuedBytes              // 6 bytes
                 . pack('n', $nonce)         // 2 bytes
                 . $keyIdBytes               // 8 bytes
                 . str_repeat("\x00", 14);   // reserved

        if (strlen($payload) !== self::PAYLOAD_BYTES) {
            throw new \LogicException('Payload length mismatch: ' . strlen($payload));
        }
        return $payload;
    }

    /**
     * Verify a signed blob against the public key. Returns parsed payload or null.
     * Useful for admin tooling and tests.
     */
    public static function verify(string $signedBlob): ?array
    {
        if (strlen($signedBlob) !== self::SIGNED_BYTES) return null;

        $payload = substr($signedBlob, 0, self::PAYLOAD_BYTES);
        $sig = substr($signedBlob, self::PAYLOAD_BYTES, self::SIGNATURE_BYTES);
        $digest = hash('sha256', $payload, true);

        $publicKey = self::loadPublicKey();
        if (!sodium_crypto_sign_verify_detached($sig, $digest, $publicKey)) {
            return null;
        }

        return [
            'version'    => ord($payload[0]),
            'sku'        => ord($payload[1]),
            'issued_at'  => (unpack('n', substr($payload, 2, 2))[1] << 32)
                          | unpack('N', substr($payload, 4, 4))[1],
            'nonce'      => unpack('n', substr($payload, 8, 2))[1],
            'key_id'     => (unpack('N', substr($payload, 10, 4))[1] << 32)
                          | unpack('N', substr($payload, 14, 4))[1],
        ];
    }

    private static function loadSecretKey(): string
    {
        $b64 = config('services.licence.private_key_b64');
        if (!$b64) {
            throw new \RuntimeException('LICENCE_PRIVATE_KEY_B64 is not configured. Run: php artisan licence:keygen');
        }
        $sk = base64_decode($b64, true);
        if ($sk === false || strlen($sk) !== SODIUM_CRYPTO_SIGN_SECRETKEYBYTES) {
            throw new \RuntimeException('LICENCE_PRIVATE_KEY_B64 is malformed (wrong length after decode).');
        }
        return $sk;
    }

    private static function loadPublicKey(): string
    {
        $b64 = config('services.licence.public_key_b64');
        if (!$b64) throw new \RuntimeException('LICENCE_PUBLIC_KEY_B64 is not configured.');
        $pk = base64_decode($b64, true);
        if ($pk === false || strlen($pk) !== SODIUM_CRYPTO_SIGN_PUBLICKEYBYTES) {
            throw new \RuntimeException('LICENCE_PUBLIC_KEY_B64 is malformed.');
        }
        return $pk;
    }
}
