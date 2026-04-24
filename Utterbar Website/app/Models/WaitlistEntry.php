<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class WaitlistEntry extends Model
{
    protected $table = 'waitlist_entries';

    protected $fillable = [
        'email',
        'use_case',
        'ip',
        'user_agent',
    ];
}
