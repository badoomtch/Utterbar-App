<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/**
 * Per-row purchase amount on the licences table.
 *
 * Until now the admin dashboard's Gross / Net of refunds tiles
 * computed revenue as `licences_total * services.stripe.pro_amount`.
 * That's correct for single-user £39.99 rows but quietly wrong for
 * volume rows (which charge £29 or £19 per seat × seat count) and
 * for any future SKU. Storing the actual checkout amount per row
 * means the dashboard can SUM honestly without the controller
 * having to reverse-engineer pricing from sku + max_devices.
 *
 * Both columns are nullable on purpose:
 *   * Pre-existing rows minted before this migration have no amount
 *     to backfill; we deliberately do NOT extrapolate. The admin
 *     dashboard reports them as "Untracked rows" so the gross
 *     figure stays truthful.
 *   * Admin manually-minted rows (LicenceController::store) carry
 *     null because no Stripe payment backs them — there's no
 *     amount to assert.
 *
 * Forward fills: the Stripe webhook mint paths (single-user and
 * volume) write amount_pence + currency at insert time, so any new
 * row from this migration onward is fully accounted for.
 *
 * Storage: `unsignedInteger` (32-bit) covers up to ~£42M per row,
 * which is more than enough headroom for any plausible volume
 * order (the self-serve cap is 100 seats × £29 = £2900). Keeping
 * it 32-bit rather than 64-bit avoids paying for headroom we'll
 * never use; the per-deal Stripe limit stops us long before the
 * column overflows.
 */
return new class extends Migration {
    public function up(): void
    {
        Schema::table('licences', function (Blueprint $t) {
            $t->unsignedInteger('amount_pence')->nullable()->after('max_devices');
            // ISO 4217 currency code (lowercase to match how Stripe
            // returns it). 3 chars sufficient; column is nullable
            // and pairs with amount_pence.
            $t->string('currency', 3)->nullable()->after('amount_pence');
        });
    }

    public function down(): void
    {
        Schema::table('licences', function (Blueprint $t) {
            $t->dropColumn(['amount_pence', 'currency']);
        });
    }
};
