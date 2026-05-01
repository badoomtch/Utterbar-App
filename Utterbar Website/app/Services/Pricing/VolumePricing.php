<?php

namespace App\Services\Pricing;

/**
 * Single source of truth for volume self-serve checkout pricing.
 *
 * Two tiers, both flat per-seat:
 *   5 to 24 seats  -> volume_per_seat_pence            (default £29)
 *   25 to 100 seats -> volume_per_seat_business_pence  (default £19)
 *
 * Quantity above 100 is intentionally out of scope for self-serve
 * checkout — those buyers go through /business and we mint manually
 * (invoice, signed installer, the usual enterprise plumbing). The
 * controller refuses anything outside [MIN_SEATS, MAX_SEATS] so a
 * malicious quantity can't slip past the boundary.
 *
 * The boundary value (`BUSINESS_TIER_THRESHOLD = 25`) lives here and
 * NOWHERE ELSE in the code base. Checkout, webhook, UI preview, and
 * tests all read it from this class. If the boundary moves, this is
 * the only file that needs editing.
 */
class VolumePricing
{
    /** Minimum quantity accepted by self-serve volume checkout. */
    public const MIN_SEATS = 5;

    /** Maximum quantity accepted by self-serve volume checkout. */
    public const MAX_SEATS = 100;

    /**
     * Quantities >= this value are billed at the business per-seat
     * rate; below it, the standard volume rate applies. The bound
     * is inclusive on the business side: 25 seats is the cheaper
     * tier.
     */
    public const BUSINESS_TIER_THRESHOLD = 25;

    /**
     * Per-seat price in pence for the given quantity. Reads pence
     * amounts from `services.stripe.volume_per_seat_pence` and
     * `services.stripe.volume_per_seat_business_pence`. The caller
     * is responsible for asserting the quantity is in
     * [MIN_SEATS, MAX_SEATS] first; this method computes the rate
     * but does not validate range — the controller's `validate()`
     * pass owns that.
     */
    public static function unitPence(int $quantity): int
    {
        if ($quantity >= self::BUSINESS_TIER_THRESHOLD) {
            return (int) config('services.stripe.volume_per_seat_business_pence', 1900);
        }
        return (int) config('services.stripe.volume_per_seat_pence', 2900);
    }

    /**
     * Total price in pence for the given quantity at its tier rate.
     * Used by both the Stripe line-item construction and the UI's
     * "Estimated total" preview so server and client agree byte-for-
     * byte. Server-side computation; the request never controls
     * either factor.
     */
    public static function totalPence(int $quantity): int
    {
        return self::unitPence($quantity) * $quantity;
    }

    /**
     * Currency code for the volume tier. Reuses the existing single-
     * user currency setting — there's no business reason for the two
     * surfaces to diverge.
     */
    public static function currency(): string
    {
        return strtolower((string) config('services.stripe.pro_currency', 'gbp'));
    }

    /**
     * SKU string the webhook uses to recognise a volume checkout. The
     * single-user path uses `services.licence.sku_string`; this is
     * its volume sibling. The webhook keys both off these config
     * values, never on hardcoded strings.
     */
    public static function sku(): string
    {
        // `config(..., default)` only returns the default when the key
        // is missing, not when the key is present-but-null. We coerce
        // explicitly so a null/empty value (e.g. an env var set to a
        // blank string) still falls back to the documented default.
        $value = config('services.licence.sku_string_volume');
        if (!is_string($value) || $value === '') {
            return 'utterbar-pro-volume';
        }
        return $value;
    }

    /**
     * True when `$quantity` is a real PHP int inside the self-serve
     * range. Strict on type: numeric strings ("10"), floats (5.0),
     * null, arrays, and booleans all return false. The Laravel
     * `integer` validation rule on the request side is more
     * permissive (it accepts numeric strings that parse as ints),
     * so the controller casts validated input to a real int via
     * `(int) $validated['quantity']` before reaching this helper.
     * The helper exists so callers that don't go through the request
     * pipeline (UI preview logic, the webhook re-validation, future
     * artisan commands) can apply the same boundary check without
     * worrying about coercion drift.
     */
    public static function isValidQuantity(mixed $quantity): bool
    {
        if (!is_int($quantity)) {
            // Reject anything non-int (float, string, null, array).
            // The controller is the only request-path caller, and it
            // casts validated input to a real int before getting
            // here, so this branch is defence in depth for non-
            // request callers (webhook handler, UI preview helpers,
            // future tooling).
            return false;
        }
        return $quantity >= self::MIN_SEATS && $quantity <= self::MAX_SEATS;
    }
}
