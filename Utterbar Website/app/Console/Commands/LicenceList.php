<?php

namespace App\Console\Commands;

use App\Models\Licence;
use Illuminate\Console\Command;

/**
 * List licences, optionally filtered.
 *
 *   php artisan licence:list
 *   php artisan licence:list --email=jay@example.com
 *   php artisan licence:list --status=refunded
 *   php artisan licence:list --limit=50
 */
class LicenceList extends Command
{
    protected $signature = 'licence:list
                            {--email= : Filter by buyer email (exact match)}
                            {--status= : Filter by status (active|refunded|revoked)}
                            {--limit=30 : Max rows to show}';

    protected $description = 'List licences in the system.';

    public function handle(): int
    {
        $q = Licence::query()->withCount('activations');

        if ($email  = $this->option('email'))  $q->where('email', $email);
        if ($status = $this->option('status')) $q->where('status', $status);

        $rows = $q->orderByDesc('id')
                  ->limit((int) $this->option('limit'))
                  ->get()
                  ->map(fn ($l) => [
                      'id'      => $l->id,
                      'key'     => $l->displayKey(),
                      'email'   => $l->email,
                      'status'  => $l->status,
                      'devices' => $l->activations_count . '/' . $l->max_devices,
                      'issued'  => $l->issued_at?->format('Y-m-d') ?? '—',
                  ])
                  ->toArray();

        if (empty($rows)) {
            $this->line('No licences match.');
            return self::SUCCESS;
        }

        $this->table(
            ['ID', 'Key', 'Email', 'Status', 'Devices', 'Issued'],
            $rows
        );
        $this->line(sprintf('<comment>%d row%s.</comment>', count($rows), count($rows) === 1 ? '' : 's'));
        return self::SUCCESS;
    }
}
