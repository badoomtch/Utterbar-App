<?php

namespace App\Mail;

use App\Models\Licence;
use App\Services\Pricing\VolumePricing;
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
        // Volume rows get a slightly different subject so the buyer
        // can spot a fleet licence in their inbox at a glance. The
        // single-user subject is preserved exactly.
        $isVolume = $this->licence->sku === VolumePricing::sku();
        $subject = $isVolume
            ? sprintf(
                'Your Utterbar Pro volume licence (%d seats): %s',
                (int) $this->licence->max_devices,
                $this->licence->displayKey(),
            )
            : 'Your Utterbar Pro licence: ' . $this->licence->displayKey();

        return new Envelope(subject: $subject);
    }

    public function content(): Content
    {
        $isVolume = $this->licence->sku === VolumePricing::sku();

        return new Content(
            view: 'emails.licence-delivery',
            text: 'emails.licence-delivery-text',
            with: [
                'displayKey'   => $this->licence->displayKey(),
                'email'        => $this->licence->email,
                'issuedAt'     => $this->licence->issued_at,
                'maxDevices'   => $this->licence->max_devices,
                'deepLinkKey'  => $this->licence->key_lookup,  // no dashes for the URL
                // Volume flag drives the conditional fleet-deployment
                // copy in both blade templates. Falsy for single-user
                // rows so the existing one-key-three-devices wording
                // is untouched.
                'isVolume'     => $isVolume,
            ],
        );
    }
}
