<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    public function up(): void
    {
        Schema::create('update_subscribers', function (Blueprint $t) {
            $t->id();
            $t->string('email')->unique();
            // Where they subscribed from — 'app' (settings), 'website' (footer),
            // 'waitlist_migrated' (from the pre-launch waitlist).
            $t->string('source', 32)->default('app');
            // Single opt-in: subscribed_at is set on creation. confirmed_at is
            // set when they click the confirmation link — gates us from blasting
            // a newsletter to an unconfirmed email.
            $t->timestamp('subscribed_at');
            $t->timestamp('confirmed_at')->nullable();
            $t->timestamp('unsubscribed_at')->nullable();
            // Per-subscriber secret for confirmation + unsubscribe links.
            $t->string('token', 64)->unique();
            $t->string('ip', 45)->nullable();
            $t->string('user_agent', 255)->nullable();
            $t->timestamps();

            $t->index(['unsubscribed_at', 'confirmed_at']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('update_subscribers');
    }
};
