<?php

namespace App\Mail;

use App\Models\ContactMessage;
use Illuminate\Bus\Queueable;
use Illuminate\Mail\Mailable;
use Illuminate\Mail\Mailables\Content;
use Illuminate\Mail\Mailables\Envelope;
use Illuminate\Queue\SerializesModels;

class ContactSubmission extends Mailable
{
    use Queueable, SerializesModels;

    public function __construct(public ContactMessage $submission) {}

    public function envelope(): Envelope
    {
        $subject = $this->submission->subject
            ? '[Utterbar] ' . $this->submission->subject
            : '[Utterbar] New contact message';

        return new Envelope(
            subject: $subject,
            replyTo: [
                new \Illuminate\Mail\Mailables\Address(
                    $this->submission->email,
                    $this->submission->name ?: $this->submission->email
                ),
            ],
        );
    }

    public function content(): Content
    {
        return new Content(
            view: 'emails.contact-submission',
            text: 'emails.contact-submission-text',
            with: [
                'submission' => $this->submission,
            ],
        );
    }
}
