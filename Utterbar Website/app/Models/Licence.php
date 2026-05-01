<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

class Licence extends Model
{
    protected $guarded = [];

    protected $casts = [
        'issued_at'    => 'datetime',
        'revoked_at'   => 'datetime',
        'max_devices'  => 'int',
        // amount_pence is nullable; cast to int when present so SUMs
        // and arithmetic in the dashboard don't have to coerce.
        'amount_pence' => 'int',
    ];

    public function activations(): HasMany
    {
        return $this->hasMany(Activation::class);
    }

    /**
     * Format the canonical key_lookup back into the dashed display form:
     * UB-XXXX-XXXX-XXXX-XXXX-XXXX
     */
    public function displayKey(): string
    {
        $chunks = str_split($this->key_lookup, 4);
        return 'UB-' . implode('-', $chunks);
    }

    public function isActive(): bool
    {
        return $this->status === 'active';
    }

    public function availableDeviceSlots(): int
    {
        return max(0, $this->max_devices - $this->activations()->count());
    }
}
