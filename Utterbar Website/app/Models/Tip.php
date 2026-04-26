<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class Tip extends Model
{
    protected $guarded = [];

    protected $casts = [
        'paid_at'      => 'datetime',
        'amount_pence' => 'int',
    ];
}
