<?php

namespace App\Console\Commands;

use App\Mail\UpdateAnnouncement;
use App\Models\UpdateSubscriber;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\Mail;

/**
 * Blast a release announcement email to every active update subscriber.
 *
 * Usage:
 *   php artisan updates:announce --release=0.6.0 --subject="Utterbar 0.6 is out" --notes=./notes/0.6.0.md
 *   php artisan updates:announce --release=0.6.0 --subject="..." --notes-text="short announcement text"
 *   php artisan updates:announce --release=0.6.0 --subject="..." --notes=... --dry-run
 *
 * Notes can come from a markdown file (--notes=path) or inline (--notes-text=...).
 * Use --dry-run to preview recipient count + one rendered email without sending.
 */
class UpdatesAnnounce extends Command
{
    protected $signature = 'updates:announce
                            {--release= : Version string (e.g. 0.6.0)}
                            {--subject= : Email subject line}
                            {--notes= : Path to markdown file containing release notes}
                            {--notes-text= : Inline release notes text (alternative to --notes)}
                            {--dry-run : Count and preview without sending}';

    protected $description = 'Email the active update subscribers about a new release.';

    public function handle(): int
    {
        $release = trim((string) $this->option('release'));
        $subject = trim((string) $this->option('subject'));

        if ($release === '' || $subject === '') {
            $this->error('Both --release and --subject are required.');
            return self::FAILURE;
        }

        $notes = '';
        if ($path = $this->option('notes')) {
            if (!is_file($path)) {
                $this->error("Notes file not found: {$path}");
                return self::FAILURE;
            }
            $notes = trim((string) file_get_contents($path));
        } elseif ($txt = $this->option('notes-text')) {
            $notes = trim((string) $txt);
        }

        if ($notes === '') {
            $this->error('Provide release notes via --notes=path/to/notes.md or --notes-text=...');
            return self::FAILURE;
        }

        $subs = UpdateSubscriber::active()->get();
        $this->line("Release:     <info>{$release}</info>");
        $this->line("Subject:     <info>{$subject}</info>");
        $this->line("Recipients:  <info>{$subs->count()}</info> active subscriber(s)");
        $this->line("Notes:       " . strlen($notes) . " chars");

        if ($this->option('dry-run')) {
            $this->line('');
            $this->line('<comment>--- DRY RUN: no emails sent ---</comment>');
            if ($subs->isNotEmpty()) {
                $first = $subs->first();
                $this->line("Example recipient: {$first->email}");
                // Dry-run shows a synthetic example URL only — we do NOT
                // rotate the subscriber's token just to print a sample
                // (that would invalidate their real last-email unsubscribe
                // link for nothing). Use a placeholder so the operator
                // sees the URL shape without burning a real token.
                $this->line("Unsubscribe URL:   " . route('updates.unsubscribe', ['t' => '<fresh-token-will-be-minted-per-recipient>']));
            }
            return self::SUCCESS;
        }

        if (!$this->confirm("Send to {$subs->count()} subscribers?", false)) {
            return self::SUCCESS;
        }

        $sent = 0;
        $failed = 0;
        foreach ($subs as $sub) {
            // Stage a per-subscriber token rotation WITHOUT persisting.
            // $sub->rawToken carries the fresh plaintext for the Mailable
            // to build the URL. The DB hash only advances if the send
            // actually succeeds — if it throws, we discard the staged
            // rotation so the previous (still-valid) unsubscribe link in
            // the user's last successful email keeps working.
            $sub->prepareRotation();
            try {
                Mail::to($sub->email)->send(new UpdateAnnouncement(
                    subscriber: $sub,
                    release:    $release,
                    subject:    $subject,
                    notesMd:    $notes,
                ));
                $sub->commitRotation();
                $sent++;
            } catch (\Throwable $e) {
                $sub->discardRotation();
                $failed++;
                $this->warn("  failed: {$sub->email} — {$e->getMessage()}");
            }
        }

        $this->info("Sent {$sent}, failed {$failed}.");
        return self::SUCCESS;
    }
}
