<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/**
 * tips records voluntary "Tip the developer" payments.
 *
 * Populated by StripeWebhookController on `checkout.session.completed`
 * when the session metadata sku is "utterbar-tip" (Pro purchases use
 * "utterbar-pro-lifetime" and skip this table; tips skip the licence
 * minting path). Idempotent on stripe_session_id so a webhook retry
 * doesn't double-record.
 *
 * We store amount + currency + the email Stripe captured (when the user
 * filled it in) so the admin dashboard can show "who tipped, how much"
 * without round-tripping to Stripe. Privacy: an email here is the same
 * data Stripe already holds; we don't capture anything extra.
 */
return new class extends Migration {
    public function up(): void
    {
        Schema::create('tips', function (Blueprint $t) {
            $t->id();
            $t->string('stripe_session_id')->unique();
            $t->string('stripe_payment_intent_id')->nullable();
            $t->unsignedInteger('amount_pence');
            $t->string('currency', 8)->default('gbp');
            $t->string('email')->nullable();
            $t->string('country', 2)->nullable();
            $t->timestamp('paid_at');
            $t->timestamps();

            $t->index('paid_at');
            $t->index('email');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('tips');
    }
};
