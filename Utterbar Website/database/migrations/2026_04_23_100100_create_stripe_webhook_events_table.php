<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    public function up(): void
    {
        Schema::create('stripe_webhook_events', function (Blueprint $t) {
            $t->id();
            $t->string('stripe_event_id')->unique();
            $t->string('type');
            $t->json('payload');
            $t->timestamp('processed_at')->nullable();
            $t->string('error')->nullable();
            $t->timestamps();
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('stripe_webhook_events');
    }
};
