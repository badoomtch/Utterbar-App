<?php

namespace App\Services\Licence;

use App\Models\Licence;

/**
 * Generates user-facing licence key lookup tokens.
 *
 * Format: 20 characters of Crockford base32 (no I, L, O, U to avoid ambiguity).
 * Roughly 100 bits of entropy. The token is what the buyer pastes into the app;
 * a separate signed blob is fetched on first paste via POST /api/licence/fetch.
 */
class LicenceKeys
{
    // Crockford base32 alphabet: 0-9 + A-Z minus I, L, O, U.
    private const ALPHABET = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
    private const LOOKUP_LENGTH = 20;

    /**
     * Generate a unique random key_lookup token.
     * Retries on the astronomically unlikely collision.
     */
    public static function generateLookup(): string
    {
        $maxAttempts = 5;
        for ($i = 0; $i < $maxAttempts; $i++) {
            $token = self::randomToken(self::LOOKUP_LENGTH);
            if (!Licence::where('key_lookup', $token)->exists()) {
                return $token;
            }
        }
        throw new \RuntimeException('Could not generate a unique licence key after 5 attempts.');
    }

    private static function randomToken(int $length): string
    {
        $alphabet = self::ALPHABET;
        $alphabetLen = strlen($alphabet);
        $out = '';
        // Pull random bytes, map each to an alphabet index via modulo.
        // Modulo bias on a 32-char alphabet with 256-value bytes is negligible for our purposes.
        $bytes = random_bytes($length);
        for ($i = 0; $i < $length; $i++) {
            $out .= $alphabet[ord($bytes[$i]) % $alphabetLen];
        }
        return $out;
    }

    /**
     * Normalise user input: strip dashes, uppercase, strip whitespace.
     * Returns null if the result doesn't look like a valid lookup token.
     */
    public static function normalise(string $input): ?string
    {
        $s = strtoupper(preg_replace('/[\s\-]+/', '', $input) ?? '');
        // Optional "UB" prefix — strip it if present.
        if (str_starts_with($s, 'UB')) {
            $s = substr($s, 2);
        }
        if (strlen($s) !== self::LOOKUP_LENGTH) {
            return null;
        }
        // Every char must be in the Crockford alphabet.
        if (preg_match('/[^0-9A-HJKMNP-TV-Z]/', $s)) {
            return null;
        }
        return $s;
    }
}
