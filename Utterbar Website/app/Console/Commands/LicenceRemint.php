<?php

namespace App\Console\Commands;

use App\Models\Licence;
use App\Services\Licence\LicenceSigner;
use Illuminate\Console\Command;

/**
 * Re-sign a licence's `signed_blob` with the currently-configured private key.
 *
 *   php artisan licence:remint 123            # one licence by id
 *   php artisan licence:remint --all          # every licence row
 *   php artisan licence:remint --all --non-interactive
 *
 * Use this after an Ed25519 keypair rotation: once the new
 * LICENCE_PRIVATE_KEY_B64 is in .env and config has been cleared, re-mint
 * every licence so their stored blobs verify under the new public key.
 *
 * Read-only with respect to `key_lookup`, `email`, `status` and every other
 * licence field — only `signed_blob` changes.
 */
class LicenceRemint extends Command
{
    protected $signature = 'licence:remint
                            {id? : Numeric licence id (omit if using --all)}
                            {--all : Re-sign every licence row}
                            {--non-interactive : Skip the confirm prompt (for scripts)}';

    protected $description = 'Re-sign licence signed_blob(s) with the current private key.';

    public function handle(): int
    {
        $all = (bool) $this->option('all');
        $id  = $this->argument('id');

        if (!$all && !$id) {
            $this->error('Pass a licence id or --all.');
            return self::FAILURE;
        }
        if ($all && $id) {
            $this->error('Pass either a licence id OR --all, not both.');
            return self::FAILURE;
        }

        $licences = $all
            ? Licence::orderBy('id')->get()
            : collect(array_filter([Licence::find((int) $id)]));

        if ($licences->isEmpty()) {
            $this->error($all ? 'No licences in the database.' : "No licence with id {$id}.");
            return self::FAILURE;
        }

        $this->line('Re-signing <info>' . $licences->count() . '</info> licence(s).');
        foreach ($licences as $l) {
            $this->line("  id={$l->id}  key={$l->displayKey()}  email={$l->email}  status={$l->status}");
        }

        if (!$this->option('non-interactive') && !$this->confirm('Proceed?', true)) {
            return self::SUCCESS;
        }

        $done = 0;
        $failed = 0;
        foreach ($licences as $l) {
            try {
                $l->signed_blob = LicenceSigner::sign($l);
                $l->save();
                $done++;
            } catch (\Throwable $e) {
                $failed++;
                $this->error("  id={$l->id} failed: {$e->getMessage()}");
            }
        }

        $this->info("Re-signed {$done} licence(s).");
        if ($failed > 0) {
            $this->error("{$failed} failed. See above.");
            return self::FAILURE;
        }
        return self::SUCCESS;
    }
}
