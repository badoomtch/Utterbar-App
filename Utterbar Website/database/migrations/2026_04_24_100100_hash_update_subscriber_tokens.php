<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

/**
 * Store update-subscriber confirm/unsubscribe tokens as SHA-256 hex, not
 * plaintext.
 *
 * Same rationale as the sign-in-token migration: a DB dump should not grant
 * replay of confirm / unsubscribe URLs. Unlike sign-in tokens these don't
 * expire — they're valid for the lifetime of the subscription — so the
 * compromise window is larger, making this change proportionally more
 * important.
 *
 * Migration strategy:
 *   1. Add `token_hash` (char(64), nullable, unique-indexed).
 *   2. Backfill: hash every existing row's `token` into `token_hash`.
 *      Live confirm/unsubscribe links in already-sent emails keep working
 *      because the plaintext in the URL re-hashes to the same value.
 *   3. Drop the plaintext `token` column.
 *
 * Rolling back re-adds a NULLABLE `token` column but cannot recover the
 * plaintext — all live URLs become useless. Documented so the down
 * migration isn't mistaken for a safe rollback.
 */
return new class extends Migration {
    public function up(): void
    {
        Schema::table('update_subscribers', function (Blueprint $t) {
            $t->char('token_hash', 64)->nullable()->after('token');
        });

        DB::table('update_subscribers')->orderBy('id')->chunkById(500, function ($rows) {
            foreach ($rows as $row) {
                if ($row->token === null || $row->token === '') {
                    continue;
                }
                DB::table('update_subscribers')
                    ->where('id', $row->id)
                    ->update(['token_hash' => hash('sha256', $row->token)]);
            }
        });

        Schema::table('update_subscribers', function (Blueprint $t) {
            $t->unique('token_hash');
        });

        Schema::table('update_subscribers', function (Blueprint $t) {
            $t->dropUnique(['token']);
            $t->dropColumn('token');
        });
    }

    public function down(): void
    {
        Schema::table('update_subscribers', function (Blueprint $t) {
            $t->string('token', 64)->nullable()->after('id');
        });
        Schema::table('update_subscribers', function (Blueprint $t) {
            $t->dropUnique(['token_hash']);
            $t->dropColumn('token_hash');
        });
    }
};
