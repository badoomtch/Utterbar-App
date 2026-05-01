<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/**
 * Air-gapped activation marker on the activations table.
 *
 * The desktop app exports a JSON request file from a machine that
 * cannot reach the website; an admin uploads it at
 * /admin/licences/{id}/activations/upload, which records an
 * activations row identical in shape to an online activation
 * EXCEPT that this column is true and last_heartbeat stays null
 * (offline machines never call /api/licence/heartbeat).
 *
 * Surfaces in the admin UI as a small "Offline" chip alongside the
 * existing device list so the operator can tell at a glance which
 * activations the licence-server-side revoke flow can reach (online
 * = will go inactive on next heartbeat 410) and which it cannot
 * (offline = inactive only after the user picks up a new licence
 * file from the admin).
 *
 * Default false for two reasons:
 *  1. Existing online activations are not offline.
 *  2. The online /api/licence/activate endpoint doesn't set the
 *     column at all; the default keeps that path working without a
 *     controller edit.
 */
return new class extends Migration {
    public function up(): void
    {
        Schema::table('activations', function (Blueprint $t) {
            $t->boolean('offline')->default(false)->after('platform');
        });
    }

    public function down(): void
    {
        Schema::table('activations', function (Blueprint $t) {
            $t->dropColumn('offline');
        });
    }
};
