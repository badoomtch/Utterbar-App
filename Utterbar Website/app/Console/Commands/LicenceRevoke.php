<?php

namespace App\Console\Commands;

use App\Models\Licence;
use Illuminate\Console\Command;

/**
 * Revoke a licence. The licence stays in the DB; its status flips to
 * "revoked" and the next heartbeat the app makes will return 410 and the
 * app will revert to Free tier.
 *
 *   php artisan licence:revoke UB-XXXX-XXXX-XXXX-XXXX-XXXX --reason="shared on twitter"
 *   php artisan licence:revoke 42 --reason="account compromised"
 */
class LicenceRevoke extends Command
{
    protected $signature = 'licence:revoke
                            {identifier : Licence key (UB-…) or numeric licence ID}
                            {--reason= : Short reason stored with the revocation (required)}';

    protected $description = 'Revoke a licence. Devices will disable on next heartbeat.';

    public function handle(): int
    {
        $id     = trim($this->argument('identifier'));
        $reason = trim((string) $this->option('reason'));

        if ($reason === '') {
            $this->error('Provide a reason with --reason="..."');
            return self::FAILURE;
        }

        $licence = $this->find($id);
        if (!$licence) {
            $this->error("No licence matches: {$id}");
            return self::FAILURE;
        }

        $this->line("Licence:  <info>{$licence->displayKey()}</info> (id {$licence->id})");
        $this->line("Email:    {$licence->email}");
        $this->line("Status:   <info>{$licence->status}</info>");
        $this->line("Devices:  {$licence->activations()->count()} / {$licence->max_devices}");
        $this->line("Reason:   <comment>{$reason}</comment>");

        if ($licence->status === 'revoked') {
            $this->warn("Already revoked.");
            return self::SUCCESS;
        }

        if (!$this->confirm('Revoke now?', false)) {
            return self::SUCCESS;
        }

        $licence->status = 'revoked';
        $licence->revoked_reason = $reason;
        $licence->revoked_at = now();
        $licence->save();

        $this->info("Revoked. Next heartbeat from any device will return 410 and the app will degrade to Free.");
        return self::SUCCESS;
    }

    private function find(string $id): ?Licence
    {
        if (ctype_digit($id)) {
            return Licence::find((int) $id);
        }
        $normalised = strtoupper(preg_replace('/[\s\-]+/', '', $id) ?? '');
        if (str_starts_with($normalised, 'UB')) {
            $normalised = substr($normalised, 2);
        }
        return Licence::where('key_lookup', $normalised)->first();
    }
}
