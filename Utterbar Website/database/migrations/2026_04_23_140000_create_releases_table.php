<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    public function up(): void
    {
        Schema::create('releases', function (Blueprint $t) {
            $t->id();
            // Semver-ish — whatever CARGO_PKG_VERSION produces. Stored as a string.
            $t->string('version', 32);
            // stable | beta. Stable releases show on /download + /releases.
            // Beta releases are for internal use / trusted testers — not rendered
            // publicly in v1 (can be later).
            $t->string('channel', 16)->default('stable');
            // R2 object key. Public URL is {CLOUDFLARE_R2_PUBLIC_URL}/{r2_key}.
            $t->string('r2_key');
            $t->unsignedBigInteger('file_size_bytes')->nullable();
            // Optional SHA-256 hex (64 chars). Shown on the download page for
            // users who want to verify. Operator pastes it from local hash.
            $t->string('sha256', 64)->nullable();
            // Optional eyebrow pill on the download page ("latest preview", etc.)
            $t->string('tag', 64)->nullable();
            // Markdown. Rendered on /releases and /download changelog sections.
            $t->text('notes_markdown');
            $t->timestamp('published_at')->useCurrent();
            // When an announcement email was fired (if any).
            $t->timestamp('announced_at')->nullable();
            $t->timestamps();

            $t->unique(['version', 'channel']);
            $t->index(['channel', 'published_at']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('releases');
    }
};
