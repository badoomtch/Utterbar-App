<?php

namespace App\Http\Controllers;

use App\Mail\LicenceDelivery;
use App\Models\Licence;
use App\Services\Licence\LicenceKeys;
use App\Services\Licence\LicenceSigner;
use App\Services\Notify\AdminNotify;
use App\Services\Pricing\VolumePricing;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Illuminate\Support\Carbon;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Mail;
use Stripe\Webhook;

class StripeWebhookController extends Controller
{
    /**
     * POST /api/stripe/webhook
     *
     * Stripe hits this on every subscribed event. We verify the Stripe-Signature
     * header via the webhook secret, then idempotently process supported events.
     */
    public function handle(Request $request): Response
    {
        $secret = config('services.stripe.webhook_secret');
        if (!$secret) {
            Log::error('Stripe webhook hit but STRIPE_WEBHOOK_SECRET is not configured.');
            return response('webhook secret not configured', 500);
        }

        $payload = $request->getContent();
        $sigHeader = $request->header('Stripe-Signature');

        try {
            $event = Webhook::constructEvent($payload, $sigHeader, $secret);
        } catch (\UnexpectedValueException $e) {
            Log::warning('Stripe webhook: invalid payload', ['err' => $e->getMessage()]);
            return response('invalid payload', 400);
        } catch (\Stripe\Exception\SignatureVerificationException $e) {
            Log::warning('Stripe webhook: signature mismatch');
            return response('invalid signature', 400);
        }

        // Idempotency: record the event. Unique constraint on stripe_event_id means
        // duplicate deliveries short-circuit here.
        $row = DB::table('stripe_webhook_events')->where('stripe_event_id', $event->id)->first();
        if ($row && $row->processed_at) {
            return response('already processed', 200);
        }

        if (!$row) {
            try {
                DB::table('stripe_webhook_events')->insert([
                    'stripe_event_id' => $event->id,
                    'type'            => $event->type,
                    'payload'         => json_encode($event->toArray()),
                    'created_at'      => now(),
                    'updated_at'      => now(),
                ]);
            } catch (\Throwable $e) {
                // Race between concurrent deliveries — safe to treat as a duplicate.
                return response('already seen', 200);
            }
        }

        try {
            switch ($event->type) {
                case 'checkout.session.completed':
                    $this->handleCheckoutCompleted($event->data->object);
                    break;
                case 'charge.refunded':
                    $this->handleChargeRefunded($event->data->object);
                    break;
                case 'charge.dispute.created':
                    $this->handleChargeDispute($event->data->object);
                    break;
                default:
                    // Ignore — we only subscribe to what we handle, but be defensive.
                    Log::info('Stripe webhook: ignoring event type', ['type' => $event->type]);
            }

            // Success — mark processed and clear any prior error from a failed retry.
            DB::table('stripe_webhook_events')
                ->where('stripe_event_id', $event->id)
                ->update(['processed_at' => now(), 'error' => null, 'updated_at' => now()]);

        } catch (\Throwable $e) {
            Log::error('Stripe webhook handler threw', [
                'event' => $event->id,
                'type'  => $event->type,
                'err'   => $e->getMessage(),
                'trace' => $e->getTraceAsString(),
            ]);
            DB::table('stripe_webhook_events')
                ->where('stripe_event_id', $event->id)
                ->update(['error' => substr($e->getMessage(), 0, 250), 'updated_at' => now()]);
            // Return 500 so Stripe retries.
            return response('handler error', 500);
        }

        return response('ok', 200);
    }

    /**
     * Mint a licence for a completed checkout, sign it, and email the key.
     */
    protected function handleCheckoutCompleted(object $session): void
    {
        // For cards the session should be paid by the time we see this event.
        // For async methods (bank transfer, etc.) Stripe fires async_payment_succeeded
        // separately — we'll add that handler when we support those methods.
        if (($session->payment_status ?? null) !== 'paid') {
            Log::info('checkout.session.completed ignored: payment_status not paid', [
                'session' => $session->id,
                'payment_status' => $session->payment_status ?? null,
            ]);
            return;
        }

        $singleUserSku = config('services.licence.sku_string', 'utterbar-pro-lifetime');
        $volumeSku     = VolumePricing::sku();
        $sku           = $session->metadata->sku ?? null;

        // Tip checkouts — record a row in `tips` and bail out before
        // the licence-minting path. Idempotent on stripe_session_id so
        // a webhook retry collapses to one row.
        if ($sku === 'utterbar-tip') {
            $this->recordTip($session);
            return;
        }

        // Volume self-serve checkout. Same minting plumbing as the
        // single-user path, but `max_devices` comes from the
        // session metadata (set by CheckoutController::createVolumeSession,
        // server-controlled, never request-controlled). The metadata
        // is the source of truth for the seat count: the line-item
        // quantity on Stripe's side drives the line, but quantity is
        // also duplicated into metadata at session create time so
        // the webhook never depends on parsing line items.
        if ($sku === $volumeSku) {
            $this->mintVolumeLicence($session);
            return;
        }

        if ($sku !== $singleUserSku) {
            Log::info('checkout.session.completed ignored: SKU mismatch', [
                'session' => $session->id, 'sku' => $sku, 'expected' => $singleUserSku,
            ]);
            return;
        }

        // Don't double-mint for a replayed session.
        if (Licence::where('stripe_checkout_session_id', $session->id)->exists()) {
            Log::info('checkout.session.completed: licence already exists', ['session' => $session->id]);
            return;
        }

        $email = $session->customer_details->email ?? $session->customer_email ?? null;
        if (!$email) {
            throw new \RuntimeException('checkout.session.completed has no email to deliver the licence to.');
        }

        $licence = DB::transaction(function () use ($session, $email, $singleUserSku) {
            $licence = Licence::create([
                'key_lookup'                  => LicenceKeys::generateLookup(),
                'email'                       => $email,
                'stripe_customer_id'          => is_string($session->customer) ? $session->customer : null,
                'stripe_payment_intent_id'    => is_string($session->payment_intent) ? $session->payment_intent : null,
                'stripe_checkout_session_id'  => $session->id,
                'sku'                         => $singleUserSku,
                'max_devices'                 => (int) config('services.licence.max_devices', 3),
                'status'                      => 'active',
                'issued_at'                   => Carbon::now(),
                // Per-row purchase amount (added 2026-05-02). The
                // single-user price is config-driven; the webhook
                // captures it at mint time so a future price change
                // doesn't retroactively rewrite historic dashboard
                // figures. See AdminController for the SUM that
                // turns these into the Gross / Net of refunds tiles.
                'amount_pence'                => (int) config('services.stripe.pro_amount', 4900),
                'currency'                    => strtolower((string) config('services.stripe.pro_currency', 'gbp')),
            ]);

            // Sign and persist. We sign after insert so key_id = licences.id.
            $licence->signed_blob = LicenceSigner::sign($licence);
            $licence->save();

            return $licence;
        });

        Log::info('licence minted', [
            'licence_id' => $licence->id,
            'email'      => $licence->email,
            'session'    => $session->id,
        ]);

        // Email delivery is best-effort: if it fails, the customer's money is
        // already taken and the licence is in the DB. We don't want to return
        // 500 and have Stripe retry forever — we log loudly and the operator
        // can resend via `php artisan licence:resend {email}` without re-mint.
        try {
            Mail::to($licence->email)->send(new LicenceDelivery($licence));
        } catch (\Throwable $e) {
            Log::error('licence minted but email FAILED — resend manually', [
                'licence_id' => $licence->id,
                'email'      => $licence->email,
                'key'        => $licence->displayKey(),
                'err'        => $e->getMessage(),
            ]);
        }

        // Fire-and-forget admin notification. Failures don't affect the webhook.
        AdminNotify::newLicence($licence);
    }

    /**
     * Volume self-serve mint. Same shape as single-user except
     * `max_devices` comes from the session metadata (which the
     * checkout controller set from the server-side validated
     * quantity). The signed blob format is identical — `max_devices`
     * is NOT carried in the 32-byte payload — so the desktop app
     * verifies the same way and learns the seat count via the
     * server's /activate response.
     *
     * Idempotent: a duplicate webhook delivery short-circuits on
     * the `stripe_checkout_session_id` exists check.
     */
    protected function mintVolumeLicence(object $session): void
    {
        $sku = VolumePricing::sku();

        // Don't double-mint for a replayed session.
        if (Licence::where('stripe_checkout_session_id', $session->id)->exists()) {
            Log::info('checkout.session.completed: volume licence already exists', [
                'session' => $session->id,
            ]);
            return;
        }

        // Pull the quantity from metadata. The checkout controller
        // wrote it; Stripe round-trips it byte-for-byte. We re-validate
        // here against the helper's range so a tampered Stripe Dashboard
        // edit (or a future code path that bypasses the controller)
        // can't slip a quantity outside [MIN_SEATS, MAX_SEATS] into a
        // licence row.
        $rawQuantity = $session->metadata->quantity ?? null;
        if (!is_numeric($rawQuantity)) {
            throw new \RuntimeException(
                "volume checkout.session.completed missing or non-numeric metadata.quantity (session {$session->id})"
            );
        }
        $quantity = (int) $rawQuantity;
        if ((string) $quantity !== (string) $rawQuantity || !VolumePricing::isValidQuantity($quantity)) {
            throw new \RuntimeException(
                "volume checkout.session.completed has out-of-range metadata.quantity={$rawQuantity} (session {$session->id})"
            );
        }

        $email = $session->customer_details->email ?? $session->customer_email ?? null;
        if (!$email) {
            throw new \RuntimeException(
                'volume checkout.session.completed has no email to deliver the licence to.'
            );
        }

        $licence = DB::transaction(function () use ($session, $email, $sku, $quantity) {
            $licence = Licence::create([
                'key_lookup'                  => LicenceKeys::generateLookup(),
                'email'                       => $email,
                'stripe_customer_id'          => is_string($session->customer) ? $session->customer : null,
                'stripe_payment_intent_id'    => is_string($session->payment_intent) ? $session->payment_intent : null,
                'stripe_checkout_session_id'  => $session->id,
                'sku'                         => $sku,
                'max_devices'                 => $quantity,
                'status'                      => 'active',
                'issued_at'                   => Carbon::now(),
                // Per-row purchase amount (added 2026-05-02). The
                // VolumePricing helper is the single source of truth
                // for the team / business tier rate, so the webhook
                // captures the same total the buyer was charged at
                // checkout. The metadata also carries unit_pence and
                // total can be reconstructed, but we read fresh from
                // the helper here so a future price change doesn't
                // retroactively rewrite the dashboard figure.
                'amount_pence'                => VolumePricing::totalPence($quantity),
                'currency'                    => VolumePricing::currency(),
            ]);

            // Sign and persist after insert so key_id = licences.id.
            $licence->signed_blob = LicenceSigner::sign($licence);
            $licence->save();

            return $licence;
        });

        Log::info('volume licence minted', [
            'licence_id'  => $licence->id,
            'email'       => $licence->email,
            'session'     => $session->id,
            'sku'         => $sku,
            'max_devices' => $quantity,
        ]);

        try {
            Mail::to($licence->email)->send(new LicenceDelivery($licence));
        } catch (\Throwable $e) {
            Log::error('volume licence minted but email FAILED — resend manually', [
                'licence_id'  => $licence->id,
                'email'       => $licence->email,
                'key'         => $licence->displayKey(),
                'max_devices' => $quantity,
                'err'         => $e->getMessage(),
            ]);
        }

        AdminNotify::newLicence($licence);
    }

    protected function handleChargeRefunded(object $charge): void
    {
        $pi = is_string($charge->payment_intent) ? $charge->payment_intent : null;
        if (!$pi) return;

        $licence = Licence::where('stripe_payment_intent_id', $pi)->first();
        if (!$licence) return;

        $licence->status = 'refunded';
        $licence->revoked_reason = 'refunded';
        $licence->revoked_at = now();
        $licence->save();

        Log::info('licence refunded', ['licence_id' => $licence->id]);
    }

    protected function handleChargeDispute(object $dispute): void
    {
        $pi = is_string($dispute->payment_intent) ? $dispute->payment_intent : null;
        if (!$pi) return;

        $licence = Licence::where('stripe_payment_intent_id', $pi)->first();
        if (!$licence) return;

        $licence->status = 'revoked';
        $licence->revoked_reason = 'chargeback';
        $licence->revoked_at = now();
        $licence->save();

        Log::warning('licence frozen due to chargeback', ['licence_id' => $licence->id]);
    }

    /**
     * Record a "Tip the developer" checkout. Idempotent on
     * stripe_session_id; a duplicate webhook retry inserts nothing.
     * Stripe's session object is the source of all values; we just
     * mirror what we need for the admin dashboard.
     */
    private function recordTip(object $session): void
    {
        $sessionId = (string) $session->id;
        if (\App\Models\Tip::where('stripe_session_id', $sessionId)->exists()) {
            Log::info('tip already recorded', ['session' => $sessionId]);
            return;
        }

        $amountTotal = (int) ($session->amount_total ?? 0);
        $currency = strtolower((string) ($session->currency ?? 'gbp'));
        $email = $session->customer_details->email ?? $session->customer_email ?? null;
        $country = null;
        if (isset($session->customer_details->address->country)) {
            $country = strtoupper((string) $session->customer_details->address->country);
            if (strlen($country) !== 2) {
                $country = null;
            }
        }
        $pi = is_string($session->payment_intent) ? $session->payment_intent : null;

        \App\Models\Tip::create([
            'stripe_session_id'        => $sessionId,
            'stripe_payment_intent_id' => $pi,
            'amount_pence'             => $amountTotal,
            'currency'                 => $currency,
            'email'                    => $email,
            'country'                  => $country,
            'paid_at'                  => now(),
        ]);

        Log::info('tip recorded', [
            'session' => $sessionId,
            'amount'  => $amountTotal,
            'currency' => $currency,
        ]);
    }
}
