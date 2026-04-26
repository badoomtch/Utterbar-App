<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/**
 * Adds the operator-signed updater signature to releases.
 *
 * The desktop client embeds an Ed25519 public key at build time and verifies
 * this signature against a deterministic metadata blob (see
 * App\Services\Releases\UpdateMetadataSigner) before downloading any
 * installer. A release without a signature is treated as not-installable
 * by the updater. Existing rows pre-dating this column will be nullable
 * until reminted.
 */
return new class extends Migration {
    public function up(): void
    {
        Schema::table('releases', function (Blueprint $t) {
            $t->string('updater_signature', 128)->nullable()->after('sha256');
        });
    }

    public function down(): void
    {
        Schema::table('releases', function (Blueprint $t) {
            $t->dropColumn('updater_signature');
        });
    }
};
