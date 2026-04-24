<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    public function up(): void
    {
        Schema::create('sign_in_tokens', function (Blueprint $t) {
            $t->id();
            // 86 chars for 64 bytes base64url. We store 64-char tokens below.
            $t->string('token', 96)->unique();
            $t->string('email');
            $t->timestamp('expires_at');
            $t->timestamp('used_at')->nullable();
            $t->string('ip', 45)->nullable();
            $t->string('user_agent', 255)->nullable();
            $t->timestamps();

            $t->index(['email', 'created_at']);
            $t->index('expires_at');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('sign_in_tokens');
    }
};
