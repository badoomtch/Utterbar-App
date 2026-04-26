<?php

namespace App\Services\Notify;

use App\Models\Licence;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Mail;

/**
 * Plain-text admin/operator notifications for "something interesting just happened
 * on the site". Sent to config('services.admin.notify_email') if set; silently
 * skipped otherwise. Delivery failures are logged and never break the request
 * that triggered them.
 *
 * Deliberately plain-text and terse — these are tool-tier alerts, not marketing.
 */
class AdminNotify
{
    public static function newLicence(Licence $licence): void
    {
        $amountPence = (int) config('services.stripe.pro_amount', 4900);
        $currency    = strtoupper((string) config('services.stripe.pro_currency', 'gbp'));
        $amount      = sprintf('%s %0.2f', $currency, $amountPence / 100);

        $subject = "[Utterbar] New purchase · {$licence->email} · {$amount}";
        $body = <<<BODY
Someone just bought Utterbar Pro.

Email:     {$licence->email}
Amount:    {$amount}
Key:       {$licence->displayKey()}
Licence:   https://utterbar.com/admin
Issued:    {$licence->issued_at}

Stripe:
  Session:    {$licence->stripe_checkout_session_id}
  PaymentInt: {$licence->stripe_payment_intent_id}
BODY;

        self::send($subject, $body, replyTo: $licence->email);
    }

    /* internals */

    private static function send(string $subject, string $body, ?string $replyTo = null): void
    {
        $to = config('services.admin.notify_email');
        if (!$to) {
            // No admin email configured — silent no-op. Not an error.
            return;
        }

        try {
            Mail::raw($body, function ($m) use ($to, $subject, $replyTo) {
                $m->to($to)->subject($subject);
                if ($replyTo) {
                    $m->replyTo($replyTo);
                }
            });
        } catch (\Throwable $e) {
            Log::warning('AdminNotify: delivery failed', [
                'subject' => $subject,
                'err'     => $e->getMessage(),
            ]);
        }
    }
}
