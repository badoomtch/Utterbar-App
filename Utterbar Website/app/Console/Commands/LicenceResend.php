<?php

namespace App\Console\Commands;

use App\Mail\LicenceDelivery;
use App\Models\Licence;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\Mail;

/**
 * Resend the licence delivery email for a given licence.
 *
 *   php artisan licence:resend user@example.com
 *   php artisan licence:resend UB-XXXX-XXXX-XXXX-XXXX-XXXX
 *   php artisan licence:resend 123                   # licence id
 *   php artisan licence:resend user@example.com --to=alt@example.com
 *
 * Use this when the webhook minted a licence but email delivery failed, or
 * when a buyer emails saying "I lost the email". Does not mint a new licence.
 */
class LicenceResend extends Command
{
    protected $signature = 'licence:resend
                            {identifier : Email, licence key (UB-…), or numeric licence ID}
                            {--to= : Send to a different email than the licence\'s own}';

    protected $description = 'Resend the licence delivery email for a given licence.';

    public function handle(): int
    {
        $id = trim($this->argument('identifier'));
        $licence = $this->findLicence($id);

        if (!$licence) {
            $this->error("No licence found matching: {$id}");
            return self::FAILURE;
        }

        $to = $this->option('to') ?: $licence->email;

        $this->line("Licence:  <info>{$licence->displayKey()}</info> (id {$licence->id})");
        $this->line("Status:   <info>{$licence->status}</info>");
        $this->line("Owner:    {$licence->email}");
        $this->line("Sending to: <info>{$to}</info>");

        if (!$this->confirm('Send now?', true)) {
            return self::SUCCESS;
        }

        try {
            Mail::to($to)->send(new LicenceDelivery($licence));
            $this->info("Sent.");
            return self::SUCCESS;
        } catch (\Throwable $e) {
            $this->error("Send failed: {$e->getMessage()}");
            return self::FAILURE;
        }
    }

    private function findLicence(string $id): ?Licence
    {
        // Numeric → licence id
        if (ctype_digit($id)) {
            return Licence::find((int) $id);
        }
        // Looks like an email
        if (filter_var($id, FILTER_VALIDATE_EMAIL)) {
            return Licence::where('email', $id)->latest('id')->first();
        }
        // Treat as a licence key: normalise and look up
        $normalised = strtoupper(preg_replace('/[\s\-]+/', '', $id) ?? '');
        if (str_starts_with($normalised, 'UB')) {
            $normalised = substr($normalised, 2);
        }
        return Licence::where('key_lookup', $normalised)->first();
    }
}
