<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class BusinessEnquiry extends Model
{
    protected $guarded = [];

    protected $casts = [
        'seat_count'   => 'int',
        'requirements' => 'array',
        'emailed_at'   => 'datetime',
    ];

    /**
     * Map seat count to the visible tier label. Cheap helper used by
     * the admin listing so the operator sees "Business (100)" rather
     * than having to read the number and translate it back. Mirrors
     * the bands published on /pricing.
     */
    public function tierLabel(): string
    {
        $n = (int) $this->seat_count;
        if ($n >= 250) return 'Enterprise';
        if ($n >= 25)  return 'Business';
        if ($n >= 3)   return 'Team';
        return 'Personal';
    }
}
