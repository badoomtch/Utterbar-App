<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/**
 * download_events tracks unique installer downloads per release per day.
 *
 * The "uniqueness" key is (release_id, ip_hash, ua_hash, day). We store
 * SHA-256 hashes of (IP || pepper) and (User-Agent || pepper), not raw
 * IPs or UAs — UK GDPR Article 5 data minimisation. With the pepper
 * (APP_KEY-derived) the hashes are non-reversible without the key, and
 * the per-day rotation means a redownload tomorrow IS counted as a new
 * event (which is the right behaviour for "download activity over time").
 *
 * Country is stored when Cloudflare passes it via CF-IPCountry. Two-letter
 * ISO; null when the request doesn't come through Cloudflare (local dev,
 * direct origin hit, etc.).
 */
return new class extends Migration {
    public function up(): void
    {
        Schema::create('download_events', function (Blueprint $t) {
            $t->id();
            $t->foreignId('release_id')
                ->constrained('releases')
                ->cascadeOnDelete();
            // SHA-256 hex (64 chars). Pepper-keyed so the same IP from the
            // same UA on different days is the same hash AND can't be
            // reversed without the pepper.
            $t->string('ip_hash', 64);
            $t->string('ua_hash', 64);
            // ISO-3166-1 alpha-2 country code from Cloudflare. Null when
            // not available. Used only for aggregate stats — never indexed
            // alongside the hash, so no pinpoint geolocation per visitor.
            $t->string('country', 2)->nullable();
            // The day the event landed, in UTC. Compound-unique with the
            // hashes so reinstalls within the same day collapse to one
            // event. Different days = separate events (deliberate).
            $t->date('event_date');
            $t->timestamps();

            $t->unique(
                ['release_id', 'ip_hash', 'ua_hash', 'event_date'],
                'download_events_uniq',
            );
            $t->index(['event_date']);
            $t->index(['release_id', 'event_date']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('download_events');
    }
};
