<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Model;

class Release extends Model
{
    protected $guarded = [];

    protected $casts = [
        'published_at'     => 'datetime',
        'announced_at'     => 'datetime',
        'file_size_bytes'  => 'int',
    ];

    public function scopeStable(Builder $q): Builder
    {
        return $q->where('channel', 'stable');
    }

    public function scopeBeta(Builder $q): Builder
    {
        return $q->where('channel', 'beta');
    }

    /**
     * Public download URL via the dl.utterbar.com custom domain.
     */
    public function downloadUrl(): string
    {
        return rtrim((string) config('filesystems.disks.r2.url'), '/') . '/' . ltrim($this->r2_key, '/');
    }

    /**
     * Formatted size for display, e.g. "58.4 MB".
     */
    public function sizeLabel(): string
    {
        $b = (int) $this->file_size_bytes;
        if ($b <= 0) return '—';
        if ($b < 1024)                     return $b . ' B';
        if ($b < 1024 * 1024)              return number_format($b / 1024, 1) . ' KB';
        if ($b < 1024 * 1024 * 1024)       return number_format($b / (1024 * 1024), 1) . ' MB';
        return number_format($b / (1024 * 1024 * 1024), 2) . ' GB';
    }
}
