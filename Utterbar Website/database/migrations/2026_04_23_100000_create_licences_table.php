<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    public function up(): void
    {
        Schema::create('licences', function (Blueprint $t) {
            $t->id();
            // User-facing short token, dashes stripped. 20 chars of Crockford base32.
            $t->string('key_lookup', 20)->unique();
            // 96 bytes = 32-byte payload + 64-byte Ed25519 signature.
            // Stored as BINARY(96) to keep byte-exact bytes without charset games.
            $t->binary('signed_blob')->nullable();
            $t->string('email');
            $t->string('stripe_customer_id')->nullable();
            $t->string('stripe_payment_intent_id')->unique()->nullable();
            $t->string('stripe_checkout_session_id')->unique()->nullable();
            $t->string('sku')->default('utterbar-pro-lifetime');
            $t->smallInteger('max_devices')->default(3);
            $t->enum('status', ['active', 'revoked', 'refunded'])->default('active');
            $t->string('revoked_reason')->nullable();
            $t->timestamp('issued_at')->nullable();
            $t->timestamp('revoked_at')->nullable();
            $t->timestamps();

            $t->index('email');
            $t->index('status');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('licences');
    }
};
