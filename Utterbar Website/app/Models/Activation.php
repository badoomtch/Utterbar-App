<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class Activation extends Model
{
    protected $guarded = [];

    protected $casts = [
        'first_seen'     => 'datetime',
        'last_heartbeat' => 'datetime',
        // Air-gapped marker (added 2026-05-03). True when the row
        // was created via /admin/licences/{id}/activations/upload
        // rather than the online /api/licence/activate endpoint.
        'offline'        => 'bool',
    ];

    public function licence(): BelongsTo
    {
        return $this->belongsTo(Licence::class);
    }

    /**
     * Serialise for API responses. Exposes only what the app needs; never the
     * activation token (that's the caller's to keep) or internal ids.
     */
    public function toPublicArray(?string $highlightFingerprint = null): array
    {
        return [
            'fingerprint'    => $this->fingerprint_hash,
            'device_name'    => $this->device_name,
            'platform'       => $this->platform,
            'app_version'    => $this->app_version,
            'first_seen'     => $this->first_seen?->toIso8601ZuluString(),
            'last_heartbeat' => $this->last_heartbeat?->toIso8601ZuluString(),
            'this_device'    => $highlightFingerprint !== null
                                && hash_equals($highlightFingerprint, $this->fingerprint_hash),
        ];
    }
}
