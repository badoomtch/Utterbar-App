<?php

namespace App\Console\Commands;

use Illuminate\Console\Command;

/**
 * Generates a fresh Ed25519 keypair for signing Utterbar licences.
 *
 *   php artisan licence:keygen
 *
 * Outputs:
 *   - LICENCE_PRIVATE_KEY_B64 (for .env on the server, never in git)
 *   - LICENCE_PUBLIC_KEY_B64  (same .env, ALSO hardcoded into the Tauri app)
 *   - public key in hex (convenience for the Rust side)
 *
 * Never run this on the production server. Generate once, copy the private key
 * into the server's .env, and keep the printout of the public key somewhere safe
 * (it's not secret but losing it means the Rust app can never be rebuilt to
 * trust this signer again).
 */
class LicenceKeygen extends Command
{
    protected $signature = 'licence:keygen';
    protected $description = 'Generate an Ed25519 signing keypair for Utterbar licences.';

    public function handle(): int
    {
        if (!function_exists('sodium_crypto_sign_keypair')) {
            $this->error('libsodium not available. Enable ext-sodium in php.ini.');
            return self::FAILURE;
        }

        $kp = sodium_crypto_sign_keypair();
        $sk = sodium_crypto_sign_secretkey($kp);   // 64 bytes
        $pk = sodium_crypto_sign_publickey($kp);   // 32 bytes

        $skB64 = base64_encode($sk);
        $pkB64 = base64_encode($pk);
        $pkHex = bin2hex($pk);

        $this->newLine();
        $this->line('<fg=cyan>━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━</>');
        $this->line('<fg=cyan>  Utterbar licence signing keypair — Ed25519 via libsodium</>');
        $this->line('<fg=cyan>━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━</>');
        $this->newLine();

        $this->line('<comment># Add to .env on the production server (never commit):</comment>');
        $this->line('LICENCE_PRIVATE_KEY_B64=' . $skB64);
        $this->line('LICENCE_PUBLIC_KEY_B64=' . $pkB64);
        $this->newLine();

        $this->line('<comment># Give this to the Tauri app — bake as PUBLIC_KEY_BYTES (32 bytes):</comment>');
        $this->line('Public key (hex):    ' . $pkHex);
        $this->line('Public key (base64): ' . $pkB64);
        $this->newLine();

        $this->line('<fg=red>Do not lose the private key. Anyone who has it can forge Utterbar licences.</>');
        $this->line('<fg=red>Do not store the private key in git. Only in .env on the signing server.</>');
        $this->newLine();

        sodium_memzero($sk);

        return self::SUCCESS;
    }
}
