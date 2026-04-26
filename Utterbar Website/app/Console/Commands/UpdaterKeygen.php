<?php

namespace App\Console\Commands;

use Illuminate\Console\Command;

/**
 * Generates a fresh Ed25519 keypair for signing Utterbar update metadata.
 *
 *   php artisan updater:keygen
 *
 * Outputs:
 *   - UPDATER_PRIVATE_KEY_B64 (operator-local secret — DO NOT put on the server)
 *   - UPDATER_PUBLIC_KEY_B64  (server .env + baked into the Tauri app)
 *   - public key as a Rust byte array (paste into commands/updater.rs)
 *
 * Distinct from the licence keypair on purpose. The licence private key
 * lives on the signing server because licences are minted server-side
 * after every Stripe purchase. The updater private key only needs to
 * sign once per release and only ever from the operator's machine, so we
 * keep it strictly off-server. This means a server compromise CANNOT push
 * malicious updates — the worst it can do is shadow real releases with
 * stale ones, which the client's downgrade-prevention check already
 * rejects.
 */
class UpdaterKeygen extends Command
{
    protected $signature = 'updater:keygen';
    protected $description = 'Generate an Ed25519 signing keypair for Utterbar update metadata.';

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

        // Format the 32-byte public key as a Rust slice literal so it can
        // be pasted straight into commands/updater.rs.
        $bytes = array_map(fn(int $b) => sprintf('0x%02x', $b), array_values(unpack('C*', $pk)));
        $chunks = array_map(fn($chunk) => implode(', ', $chunk), array_chunk($bytes, 8));
        $rustArray = "[\n    " . implode(",\n    ", $chunks) . ",\n]";

        $this->newLine();
        $this->line('<fg=cyan>━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━</>');
        $this->line('<fg=cyan>  Utterbar UPDATER signing keypair — Ed25519 via libsodium</>');
        $this->line('<fg=cyan>━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━</>');
        $this->newLine();

        $this->line('<comment># Operator-local — paste into ~/.utterbar-deploy/.env, NEVER on the server:</comment>');
        $this->line('UPDATER_PRIVATE_KEY_B64=' . $skB64);
        $this->newLine();

        $this->line('<comment># Server .env (publish-time signature verification):</comment>');
        $this->line('UPDATER_PUBLIC_KEY_B64=' . $pkB64);
        $this->newLine();

        $this->line('<comment># Rust side — paste as `UPDATER_PUBLIC_KEY` in src-tauri/src/commands/updater.rs:</comment>');
        $this->line('pub const UPDATER_PUBLIC_KEY: [u8; 32] = ' . $rustArray . ';');
        $this->newLine();

        $this->line('<fg=red>Do not lose the private key. Anyone who has it can push updates to every Utterbar install.</fg=red>');
        $this->line('<fg=red>Do not commit it. Do not put it on the server. Operator machine only.</fg=red>');
        $this->newLine();

        sodium_memzero($sk);

        return self::SUCCESS;
    }
}
