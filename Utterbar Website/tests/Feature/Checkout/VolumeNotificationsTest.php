<?php

namespace Tests\Feature\Checkout;

use App\Mail\LicenceDelivery;
use App\Models\Licence;
use App\Services\Notify\AdminNotify;
use App\Services\Pricing\VolumePricing;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Mail;
use Illuminate\Support\Str;
use Tests\TestCase;

/**
 * AdminNotify and LicenceDelivery copy branches for volume rows.
 *
 * Single-user copy is preserved exactly — each volume-tier
 * assertion has a paired single-user assertion that the existing
 * subject/body hasn't drifted.
 */
class VolumeNotificationsTest extends TestCase
{
    use RefreshDatabase;

    protected function setUp(): void
    {
        parent::setUp();

        config([
            'services.licence.sku_string'        => 'utterbar-pro-lifetime',
            'services.licence.sku_string_volume' => 'utterbar-pro-volume',
            'services.stripe.pro_currency'       => 'gbp',
            'services.stripe.pro_amount'         => 3999,
            'services.stripe.volume_per_seat_pence'           => 2900,
            'services.stripe.volume_per_seat_business_pence'  => 1900,
            'services.admin.notify_email'        => 'ops@utterbar.example',
        ]);
    }

    private function seedLicence(string $sku, int $maxDevices, ?string $email = null): Licence
    {
        return Licence::create([
            'key_lookup'  => strtoupper(Str::random(20)),
            'email'       => $email ?? 'buyer@example.com',
            'sku'         => $sku,
            'max_devices' => $maxDevices,
            'status'      => 'active',
            'issued_at'   => now(),
            'stripe_checkout_session_id' => 'cs_test_' . bin2hex(random_bytes(8)),
            'stripe_payment_intent_id'   => 'pi_test_' . bin2hex(random_bytes(8)),
        ]);
    }

    /**
     * AdminNotify uses Mail::raw, which under Mail::fake() does NOT
     * route through the Mailable assertion API. The cleanest way to
     * assert the rendered subject/body is to capture the call
     * arguments via a Mailer mock against the underlying facade. We
     * use the mail "array" transport (already wired by
     * phpunit.xml's MAIL_MAILER=array) and read the resulting
     * captured mails via Mail::mailer's transport.
     */
    private function capturedMessages(): array
    {
        // The "array" transport swaps the SMTP send for an in-memory
        // collection of Symfony SentMessage instances. After
        // sending, pull them off the transport. `messages()` returns
        // an Illuminate Collection in current Laravel; we coerce to
        // a plain array so the test asserts against a stable shape.
        /** @var \Illuminate\Mail\Mailer $mailer */
        $mailer = Mail::mailer();
        $transport = $mailer->getSymfonyTransport();
        if (! method_exists($transport, 'messages')) {
            $this->fail('mail transport is not the array transport; phpunit.xml should set MAIL_MAILER=array');
        }
        $msgs = $transport->messages();
        return is_array($msgs) ? $msgs : iterator_to_array($msgs);
    }

    public function test_admin_notify_for_single_user_uses_existing_copy(): void
    {
        $licence = $this->seedLicence('utterbar-pro-lifetime', 3);

        AdminNotify::newLicence($licence);

        $msgs = $this->capturedMessages();
        $this->assertCount(1, $msgs, 'AdminNotify should produce exactly one message');
        $envelope = $msgs[0]->getOriginalMessage();
        $subject = $envelope->getSubject();
        $body    = $envelope->getTextBody();

        $this->assertStringContainsString('[Utterbar] New purchase ·', $subject);
        $this->assertStringContainsString($licence->email, $subject);
        $this->assertStringContainsString('GBP 39.99', $subject);
        // Single-user body opens with the canonical line.
        $this->assertStringContainsString('Someone just bought Utterbar Pro.', $body);
        // Single-user body must NOT mention seats / max devices /
        // fleet deployment — those are volume-only.
        $this->assertStringNotContainsString('Seats:', $body);
        $this->assertStringNotContainsString('Fleet deployment', $body);
    }

    public function test_admin_notify_for_volume_uses_volume_copy(): void
    {
        $quantity = 50; // business tier: £19/seat × 50 = £950
        $licence = $this->seedLicence(VolumePricing::sku(), $quantity);

        AdminNotify::newLicence($licence);

        $msgs = $this->capturedMessages();
        $this->assertCount(1, $msgs);
        $envelope = $msgs[0]->getOriginalMessage();
        $subject = $envelope->getSubject();
        $body    = $envelope->getTextBody();

        $this->assertStringContainsString('[Utterbar] New volume purchase ·', $subject);
        $this->assertStringContainsString($licence->email, $subject);
        $this->assertStringContainsString('GBP 950.00', $subject);
        $this->assertStringContainsString("{$quantity} seats", $subject);

        $this->assertStringContainsString('volume Utterbar Pro licence', $body);
        $this->assertStringContainsString("Seats:", $body);
        $this->assertStringContainsString("{$quantity} (per-seat GBP 19.00)", $body);
        $this->assertStringContainsString("Max devices:  {$quantity}", $body);
        $this->assertStringContainsString('Fleet deployment', $body);
        $this->assertStringContainsString('licence.bin', $body);
        $this->assertStringContainsString('docs/it-deployment', $body);
        $this->assertStringContainsString(VolumePricing::sku(), $body);
    }

    public function test_admin_notify_for_volume_at_team_tier_uses_team_rate(): void
    {
        // 10 seats × £29 = £290 (team tier).
        $licence = $this->seedLicence(VolumePricing::sku(), 10);

        AdminNotify::newLicence($licence);

        $msgs = $this->capturedMessages();
        $this->assertCount(1, $msgs);
        $subject = $msgs[0]->getOriginalMessage()->getSubject();
        $body    = $msgs[0]->getOriginalMessage()->getTextBody();

        $this->assertStringContainsString('GBP 290.00', $subject);
        $this->assertStringContainsString('10 seats', $subject);
        $this->assertStringContainsString('per-seat GBP 29.00', $body);
    }

    public function test_licence_delivery_envelope_is_volume_specific(): void
    {
        $licence = $this->seedLicence(VolumePricing::sku(), 25);

        $mail = new LicenceDelivery($licence);
        $envelope = $mail->envelope();

        $this->assertStringContainsString('volume licence (25 seats)', $envelope->subject);
        $this->assertStringContainsString($licence->displayKey(), $envelope->subject);
    }

    public function test_licence_delivery_envelope_for_single_user_is_unchanged(): void
    {
        $licence = $this->seedLicence('utterbar-pro-lifetime', 3);

        $mail = new LicenceDelivery($licence);
        $envelope = $mail->envelope();

        $this->assertSame(
            'Your Utterbar Pro licence — ' . $licence->displayKey(),
            $envelope->subject,
            'single-user subject must not regress when the volume branch lands'
        );
    }

    public function test_licence_delivery_text_body_carries_it_block_for_volume(): void
    {
        $licence = $this->seedLicence(VolumePricing::sku(), 25);
        $mail = new LicenceDelivery($licence);

        // Render the text view with the same data the mailable feeds it.
        $rendered = view('emails.licence-delivery-text', [
            'displayKey'  => $licence->displayKey(),
            'email'       => $licence->email,
            'issuedAt'    => $licence->issued_at,
            'maxDevices'  => $licence->max_devices,
            'deepLinkKey' => $licence->key_lookup,
            'isVolume'    => true,
        ])->render();

        $this->assertStringContainsString('For your IT team', $rendered);
        $this->assertStringContainsString('volume licence covering 25 devices', $rendered);
        $this->assertStringContainsString('licence.bin', $rendered);
        $this->assertStringContainsString('docs/it-deployment', $rendered);
    }

    public function test_licence_delivery_text_body_omits_it_block_for_single_user(): void
    {
        $licence = $this->seedLicence('utterbar-pro-lifetime', 3);

        $rendered = view('emails.licence-delivery-text', [
            'displayKey'  => $licence->displayKey(),
            'email'       => $licence->email,
            'issuedAt'    => $licence->issued_at,
            'maxDevices'  => $licence->max_devices,
            'deepLinkKey' => $licence->key_lookup,
            'isVolume'    => false,
        ])->render();

        $this->assertStringNotContainsString('For your IT team', $rendered);
        $this->assertStringNotContainsString('licence.bin', $rendered);
        $this->assertStringContainsString('Good on up to 3 devices.', $rendered);
    }
}
