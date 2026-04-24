<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    public function up(): void
    {
        Schema::create('activations', function (Blueprint $t) {
            $t->id();
            $t->foreignId('licence_id')->constrained()->cascadeOnDelete();
            // sha256 hex of (machine_guid || hostname || mac_lo64) — 64 lowercase hex chars.
            $t->string('fingerprint_hash', 64);
            // User-editable, defaults to hostname on the app side.
            $t->string('device_name', 100);
            $t->string('platform', 32)->default('windows');
            $t->string('app_version', 32)->nullable();
            // Opaque 64-char token returned to the app; currently informational
            // only — endpoint auth is key + fingerprint per spec §5.3.
            $t->string('activation_token', 64)->unique();
            $t->timestamp('first_seen')->nullable();
            $t->timestamp('last_heartbeat')->nullable();
            $t->string('last_ip', 45)->nullable();
            $t->string('last_user_agent', 255)->nullable();
            $t->timestamps();

            // One activation row per (licence, device).
            $t->unique(['licence_id', 'fingerprint_hash']);
            $t->index('last_heartbeat');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('activations');
    }
};
