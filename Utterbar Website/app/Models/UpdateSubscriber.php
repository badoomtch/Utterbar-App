<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Model;

class UpdateSubscriber extends Model
{
    protected $guarded = [];

    protected $casts = [
        'subscribed_at'   => 'datetime',
        'confirmed_at'    => 'datetime',
        'unsubscribed_at' => 'datetime',
    ];

    /**
     * Ephemeral plaintext token. Populated only when we mint or re-mint a
     * token in the controller; read by the Mail classes so the magic URL
     * has the raw value. Never persisted — only the SHA-256 of this sits
     * in `token_hash`.
     */
    public ?string $rawToken = null;

    /**
     * Hash a raw subscriber token for storage / lookup. See
     * SignInToken::hashToken for the reasoning behind plain SHA-256.
     */
    public static function hashToken(string $raw): string
    {
        return hash('sha256', $raw);
    }

    /**
     * Rotate the stored token and return the fresh plaintext. Saves the
     * hash to the DB immediately; the caller is expected to use the
     * returned string (via `$sub->rawToken` on the same in-memory model)
     * to build the URL for an outgoing email.
     *
     * Rotating per outgoing announcement means that unsubscribe links from
     * an earlier email stop working once a later email goes out — which
     * lowers our token-leak surface area and guarantees the "most recent
     * email's unsubscribe URL" is always the working one.
     *
     * **Prefer `prepareRotation()` + `commitRotation()` in new code** so a
     * failed email send doesn't leave the subscriber without a working
     * unsubscribe link. This method is retained for any callers that
     * genuinely need an immediate persisted rotation (none today).
     */
    public function rotateTokenAndSave(): string
    {
        $raw = \Illuminate\Support\Str::random(64);
        $this->token_hash = self::hashToken($raw);
        $this->save();
        $this->rawToken = $raw;
        return $raw;
    }

    /**
     * Stage a token rotation without persisting. Returns the fresh raw
     * token and primes `$this->rawToken` so the Mailable can read it when
     * building the URL. The previous `token_hash` is preserved in
     * `$previousTokenHash` so the caller can decide to either commit
     * (persist the new hash, drop the old one) or discard (revert to the
     * old hash in memory) based on whether the send succeeded.
     *
     * Rationale: `Mail::send()` can fail (SMTP error, Resend 5xx, mailbox
     * rejection mid-blast). Under `rotateTokenAndSave()`, a failed send
     * left the subscriber with a DB row whose `token_hash` matched a
     * plaintext URL that was never actually delivered — they'd lose the
     * ability to click the unsubscribe link from their most recent
     * successful email. Staged rotation means the DB only moves forward
     * when delivery has actually succeeded.
     */
    public function prepareRotation(): string
    {
        $raw = \Illuminate\Support\Str::random(64);
        $this->previousTokenHash = $this->token_hash;
        $this->token_hash = self::hashToken($raw);
        $this->rawToken = $raw;
        return $raw;
    }

    /**
     * Persist a previously-prepared rotation. Call ONLY after the email
     * send that consumed `$this->rawToken` has succeeded — otherwise
     * invoke `discardRotation()` to revert the in-memory model.
     */
    public function commitRotation(): void
    {
        $this->previousTokenHash = null;
        $this->save();
    }

    /**
     * Roll back a prepared-but-not-committed rotation. Restores
     * `$this->token_hash` to the pre-prepare value so the row is left in
     * its original state if the model is then saved, and clears
     * `$this->rawToken` so no stale plaintext leaks into later code.
     */
    public function discardRotation(): void
    {
        if ($this->previousTokenHash !== null) {
            $this->token_hash = $this->previousTokenHash;
            $this->previousTokenHash = null;
        }
        $this->rawToken = null;
    }

    /**
     * Snapshot of the token_hash at the moment `prepareRotation()` was
     * called. Used by `discardRotation()` to roll back. Non-persistent.
     */
    public ?string $previousTokenHash = null;

    public function isActive(): bool
    {
        return $this->confirmed_at !== null && $this->unsubscribed_at === null;
    }

    public function scopeActive(Builder $q): Builder
    {
        return $q->whereNotNull('confirmed_at')->whereNull('unsubscribed_at');
    }
}
