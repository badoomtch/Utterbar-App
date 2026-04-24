<?php

namespace App\Mail;

use Illuminate\Bus\Queueable;
use Illuminate\Mail\Mailable;
use Illuminate\Mail\Mailables\Content;
use Illuminate\Mail\Mailables\Envelope;
use Illuminate\Queue\SerializesModels;

class SignInLink extends Mailable
{
    use Queueable, SerializesModels;

    public function __construct(public string $token, public string $email) {}

    public function envelope(): Envelope
    {
        return new Envelope(
            subject: 'Your Utterbar sign-in link',
        );
    }

    public function content(): Content
    {
        return new Content(
            view: 'emails.sign-in-link',
            text: 'emails.sign-in-link-text',
            with: [
                'url'   => route('sign-in.consume', ['token' => $this->token]),
                'email' => $this->email,
                'ttl'   => 20,
            ],
        );
    }
}
