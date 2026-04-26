<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/**
 * Audit log for the bearer-token admin API. Recorded by
 * App\Http\Middleware\AdminApiToken on every request that reaches the
 * /api/admin/* surface. Stores enough to triage a token leak: which
 * IP made the call, what the bearer-token prefix was (first 8 chars
 * only — never the full secret), what method+path was hit, what the
 * response status was. Body is NOT recorded — that's potentially
 * sensitive (release SHA-256s, R2 keys) and the request line is
 * enough to reconstruct intent.
 *
 * Existed-before-the-leak: a regular `SELECT * FROM admin_api_log
 * WHERE created_at > '<rotation timestamp>'` produces the abuse log.
 * Without this table the only forensic source is Cloudways access
 * logs which rotate weekly and don't carry the bearer prefix.
 */
return new class extends Migration {
    public function up(): void
    {
        Schema::create('admin_api_log', function (Blueprint $t) {
            $t->id();
            // First 8 chars of the bearer token presented. Truncated so
            // a DB dump doesn't leak the whole token; long enough to
            // uniquely identify which token was in use across rotations.
            $t->string('token_prefix', 16);
            // 'ok' if the token was accepted, 'reject' if it was wrong
            // or missing. Brute-force attempts surface as a column of
            // 'reject' rows from one IP.
            $t->string('outcome', 16);
            $t->string('method', 8);
            $t->string('path', 191);
            $t->string('ip', 45)->nullable();
            $t->string('user_agent', 191)->nullable();
            $t->unsignedSmallInteger('status_code')->nullable();
            $t->timestamps();

            $t->index('token_prefix');
            $t->index(['outcome', 'created_at']);
            $t->index('ip');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('admin_api_log');
    }
};
