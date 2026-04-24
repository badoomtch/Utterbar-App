<?php

namespace App\Mail;

use App\Models\UpdateSubscriber;
use Illuminate\Bus\Queueable;
use Illuminate\Mail\Mailable;
use Illuminate\Mail\Mailables\Content;
use Illuminate\Mail\Mailables\Envelope;
use Illuminate\Queue\SerializesModels;

class UpdateSubscribeConfirm extends Mailable
{
    use Queueable, SerializesModels;

    public function __construct(public UpdateSubscriber $sub) {}

    public function envelope(): Envelope
    {
        return new Envelope(
            subject: 'Confirm your Utterbar release emails',
        );
    }

    public function content(): Content
    {
        // $rawToken is the ephemeral plaintext set by the controller when
        // the subscriber was created or had their token rotated; the DB
        // stores only its SHA-256 hash. For blast-style announcement
        // sends prefer prepareRotation() + commitRotation() so a failed
        // send doesn't burn the subscriber's last-delivered unsubscribe
        // link. If this Mailable ever runs without a rawToken (e.g.
        // queued from a serialised model that lost the non-persistent
        // property) we fail loud rather than send a broken URL.
        $raw = $this->sub->rawToken;
        if (!$raw) {
            throw new \RuntimeException(
                'UpdateSubscribeConfirm requires $sub->rawToken to be set before send.'
            );
        }
        return new Content(
            view: 'emails.update-subscribe-confirm',
            text: 'emails.update-subscribe-confirm-text',
            with: [
                'confirmUrl'     => route('updates.confirm',     ['t' => $raw]),
                'unsubscribeUrl' => route('updates.unsubscribe', ['t' => $raw]),
                'email'          => $this->sub->email,
            ],
        );
    }
}
