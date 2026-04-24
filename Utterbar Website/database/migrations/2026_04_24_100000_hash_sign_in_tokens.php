<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

/**
 * Store magic-link tokens as SHA-256 hex, not plaintext.
 *
 * The plaintext `token` column leaked the ability to sign in as any user whose
 * token row was readable (database dump, read replica compromise, exported
 * backup, admin shoulder-surfing the admin dashboard). Tokens already have
 * ~380 bits of entropy so a hash preimage attack is not the threat model;
 * the threat model is "anyone who reads the row can replay the link."
 *
 * Migration strategy:
 *   1. Add `token_hash` (char(64), unique, indexed).
 *   2. Backfill: for each existing row, hash the plaintext into `token_hash`.
 *      This preserves any currently-live tokens (TTL 20 min) so users
 *      mid-flight don't have to request a new link.
 *   3. Drop the plaintext `token` column.
 *
 * Rolling back reinstates `token` as NULLABLE (we can't recover plaintext
 * from a hash), which means rolling back invalidates all live tokens — the
 * user must request a new link. Documented so the down migration isn't
 * mistaken for a safe rollback.
 */
return new class extends Migration {
    public function up(): void
    {
        Schema::table('sign_in_tokens', function (Blueprint $t) {
            // 64 chars for the hex-encoded SHA-256 output. Indexed + unique
            // so lookup by hash is O(log n) and prevents collisions.
            $t->char('token_hash', 64)->nullable()->after('token');
        });

        // Backfill live tokens. Uses a PHP-side loop so we get hash_hmac/hash
        // rather than relying on SQL-side crypto that may differ across MySQL
        // / SQLite (tests use SQLite).
        DB::table('sign_in_tokens')->orderBy('id')->chunkById(500, function ($rows) {
            foreach ($rows as $row) {
                if ($row->token === null || $row->token === '') {
                    continue;
                }
                DB::table('sign_in_tokens')
                    ->where('id', $row->id)
                    ->update(['token_hash' => hash('sha256', $row->token)]);
            }
        });

        Schema::table('sign_in_tokens', function (Blueprint $t) {
            $t->unique('token_hash');
        });

        // Drop the plaintext column. Anything still sitting on the old
        // unhashed value after this line is unrecoverable — but because
        // of the 20-min TTL the window is acceptable.
        Schema::table('sign_in_tokens', function (Blueprint $t) {
            // Drop the unique constraint on the old column before dropping
            // the column itself (MySQL refuses otherwise).
            $t->dropUnique(['token']);
            $t->dropColumn('token');
        });
    }

    public function down(): void
    {
        Schema::table('sign_in_tokens', function (Blueprint $t) {
            $t->string('token', 96)->nullable()->after('id');
        });
        // Cannot restore plaintext from hash; all live tokens are invalidated.
        Schema::table('sign_in_tokens', function (Blueprint $t) {
            $t->dropUnique(['token_hash']);
            $t->dropColumn('token_hash');
        });
    }
};
