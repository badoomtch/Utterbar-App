<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class SignInToken extends Model
{
    protected $guarded = [];

    protected $casts = [
        'expires_at' => 'datetime',
        'used_at'    => 'datetime',
    ];

    /**
     * Hash a raw token for storage / lookup. SHA-256 hex — 64 chars.
     *
     * The raw token is a 64-char random string with ~380 bits of entropy; a
     * plain cryptographic hash is sufficient here. We do NOT use bcrypt
     * because bcrypt salts per-hash, which would prevent the constant-time
     * lookup-by-hash pattern — and bcrypt's slowness is protection against
     * low-entropy password guessing, which doesn't apply to random tokens.
     */
    public static function hashToken(string $raw): string
    {
        return hash('sha256', $raw);
    }

    public function isValid(): bool
    {
        return $this->used_at === null && $this->expires_at->isFuture();
    }
}
