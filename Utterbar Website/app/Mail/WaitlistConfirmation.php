<?php

namespace App\Mail;

use Illuminate\Bus\Queueable;
use Illuminate\Mail\Mailable;
use Illuminate\Mail\Mailables\Content;
use Illuminate\Mail\Mailables\Envelope;
use Illuminate\Queue\SerializesModels;

class WaitlistConfirmation extends Mailable
{
    use Queueable, SerializesModels;

    public function envelope(): Envelope
    {
        return new Envelope(
            subject: "You're on the Utterbar waitlist",
        );
    }

    public function content(): Content
    {
        return new Content(
            view: 'emails.waitlist-confirmation',
            text: 'emails.waitlist-confirmation-text',
        );
    }
}
