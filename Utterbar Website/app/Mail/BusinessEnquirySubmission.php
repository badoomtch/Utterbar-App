<?php

namespace App\Mail;

use App\Models\BusinessEnquiry;
use Illuminate\Bus\Queueable;
use Illuminate\Mail\Mailable;
use Illuminate\Mail\Mailables\Address;
use Illuminate\Mail\Mailables\Content;
use Illuminate\Mail\Mailables\Envelope;
use Illuminate\Queue\SerializesModels;

class BusinessEnquirySubmission extends Mailable
{
    use Queueable, SerializesModels;

    public function __construct(public BusinessEnquiry $enquiry) {}

    public function envelope(): Envelope
    {
        $subject = sprintf(
            '[Utterbar B2B] %s · %d seats · %s',
            $this->enquiry->company,
            (int) $this->enquiry->seat_count,
            $this->enquiry->tierLabel(),
        );

        return new Envelope(
            subject: $subject,
            replyTo: [
                new Address(
                    $this->enquiry->email,
                    $this->enquiry->name ?: $this->enquiry->email
                ),
            ],
        );
    }

    public function content(): Content
    {
        return new Content(
            view: 'emails.business-enquiry',
            text: 'emails.business-enquiry-text',
            with: [
                'enquiry' => $this->enquiry,
            ],
        );
    }
}
