<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Support\Facades\Schema;

/**
 * Waitlist removed 2026-04-24. The table was only ever an email holding
 * pen for "tell me when 1.0 ships" — now that the app is shipping openly,
 * that funnel is dead weight. The changelog-by-email flow lives on the
 * separate `update_subscribers` table and is untouched.
 */
return new class extends Migration {
    public function up(): void
    {
        Schema::dropIfExists('waitlist_entries');
    }

    public function down(): void
    {
        // Deliberately no-op. Re-creating an empty table would be misleading
        // — the data that used to live here is gone. If the waitlist concept
        // ever returns, write a fresh migration.
    }
};
