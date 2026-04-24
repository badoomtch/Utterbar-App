<?php

namespace App\Console\Commands;

use App\Models\Licence;
use Illuminate\Console\Command;

/**
 * Reverse a revocation. Use this if you revoked by mistake or the user
 * appealed and you're giving them the licence back.
 *
 *   php artisan licence:unrevoke UB-XXXX-XXXX-XXXX-XXXX-XXXX
 */
class LicenceUnrevoke extends Command
{
    protected $signature = 'licence:unrevoke {identifier}';
    protected $description = 'Re-activate a revoked licence.';

    public function handle(): int
    {
        $id = trim($this->argument('identifier'));

        $licence = $this->find($id);
        if (!$licence) {
            $this->error("No licence matches: {$id}");
            return self::FAILURE;
        }

        if ($licence->status === 'active') {
            $this->info('Already active. Nothing to do.');
            return self::SUCCESS;
        }

        $this->line("Licence:  <info>{$licence->displayKey()}</info> (id {$licence->id})");
        $this->line("Email:    {$licence->email}");
        $this->line("Was:      <comment>{$licence->status}</comment>" . ($licence->revoked_reason ? " ({$licence->revoked_reason})" : ''));

        if (!$this->confirm('Re-activate?', true)) {
            return self::SUCCESS;
        }

        $licence->status = 'active';
        $licence->revoked_reason = null;
        $licence->revoked_at = null;
        $licence->save();

        $this->info("Re-activated. The app will resume Pro on next heartbeat or paste.");
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
