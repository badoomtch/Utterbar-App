<?php

namespace App\Mail;

use App\Models\Licence;
use Illuminate\Bus\Queueable;
use Illuminate\Mail\Mailable;
use Illuminate\Mail\Mailables\Content;
use Illuminate\Mail\Mailables\Envelope;
use Illuminate\Queue\SerializesModels;

class LicenceDelivery extends Mailable
{
    use Queueable, SerializesModels;

    public function __construct(public Licence $licence) {}

    public function envelope(): Envelope
    {
        return new Envelope(
            subject: 'Your Utterbar Pro licence — ' . $this->licence->displayKey(),
        );
    }

    public function content(): Content
    {
        return new Content(
            view: 'emails.licence-delivery',
            text: 'emails.licence-delivery-text',
            with: [
                'displayKey'   => $this->licence->displayKey(),
                'email'        => $this->licence->email,
                'issuedAt'     => $this->licence->issued_at,
                'maxDevices'   => $this->licence->max_devices,
                'deepLinkKey'  => $this->licence->key_lookup,  // no dashes for the URL
            ],
        );
    }
}
