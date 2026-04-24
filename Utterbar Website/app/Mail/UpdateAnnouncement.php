<?php

namespace App\Mail;

use App\Models\UpdateSubscriber;
use Illuminate\Bus\Queueable;
use Illuminate\Mail\Mailable;
use Illuminate\Mail\Mailables\Content;
use Illuminate\Mail\Mailables\Envelope;
use Illuminate\Queue\SerializesModels;

class UpdateAnnouncement extends Mailable
{
    use Queueable, SerializesModels;

    // `$subject` is declared on the parent Mailable without a type; PHP
    // 8.2 rejects re-declaring it here with a type. We carry our own
    // `$announceSubject` and feed it into the envelope instead.
    public function __construct(
        public UpdateSubscriber $subscriber,
        public string $release,
        public string $announceSubject,
        public string $notesMd,
    ) {}

    public function envelope(): Envelope
    {
        return new Envelope(subject: $this->announceSubject);
    }

    public function content(): Content
    {
        // See UpdateSubscribeConfirm for rawToken contract. Callers must
        // stage a per-recipient token via $subscriber->prepareRotation()
        // immediately before `Mail::send()`, and then call
        // $subscriber->commitRotation() only on successful delivery (or
        // discardRotation() on failure). Using `rotateTokenAndSave()` here
        // would persist the new hash before the send succeeds — a
        // mid-blast SMTP failure would then leave the subscriber without
        // a working unsubscribe link from their last-delivered email.
        $raw = $this->subscriber->rawToken;
        if (!$raw) {
            throw new \RuntimeException(
                'UpdateAnnouncement requires $subscriber->rawToken to be set (call prepareRotation() before Mail::send(), then commitRotation() on success).'
            );
        }
        return new Content(
            view: 'emails.update-announcement',
            text: 'emails.update-announcement-text',
            with: [
                'release'        => $this->release,
                'notesMd'        => $this->notesMd,
                'email'          => $this->subscriber->email,
                'unsubscribeUrl' => route('updates.unsubscribe', ['t' => $raw]),
                'downloadUrl'    => url('/download'),
            ],
        );
    }
}
