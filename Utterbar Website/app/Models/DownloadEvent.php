<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * Single deduplicated download event. See migration for the uniqueness
 * model: (release_id, ip_hash, ua_hash, event_date) is unique, so re-
 * downloads from the same IP/UA on the same day collapse to one row.
 */
class DownloadEvent extends Model
{
    protected $guarded = [];

    protected $casts = [
        'event_date' => 'date',
    ];

    public function release(): BelongsTo
    {
        return $this->belongsTo(Release::class);
    }
}
