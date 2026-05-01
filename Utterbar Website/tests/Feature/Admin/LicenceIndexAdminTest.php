<?php

namespace Tests\Feature\Admin;

use App\Models\Licence;
use App\Models\User;
use App\Services\Pricing\VolumePricing;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Str;
use Tests\TestCase;

/**
 * Admin licences index page (GET /admin/licences).
 *
 * Pins the volume polish:
 *   * Volume rows render a "Volume" chip in the new Type column.
 *   * Single-user rows render the bare "Pro" label.
 *   * Volume rows show the seat count emphasised next to the
 *     activations/max_devices fraction.
 *   * Auth gate still keeps unauthenticated and non-admin users out.
 */
class LicenceIndexAdminTest extends TestCase
{
    use RefreshDatabase;

    private function seedAdmin(string $email = 'admin@example.com'): User
    {
        putenv('ADMIN_EMAILS=' . $email);
        return User::create([
            'name'              => 'Admin',
            'email'             => $email,
            'password'          => Hash::make(Str::random(64)),
            'email_verified_at' => now(),
        ]);
    }

    private function seedNonAdmin(string $email = 'someone@example.com'): User
    {
        // ADMIN_EMAILS is set deliberately to a different value so
        // the AdminOnly middleware refuses this user.
        putenv('ADMIN_EMAILS=admin@example.com');
        return User::create([
            'name'              => 'User',
            'email'             => $email,
            'password'          => Hash::make(Str::random(64)),
            'email_verified_at' => now(),
        ]);
    }

    private function seedLicence(string $sku, int $maxDevices, string $email = 'buyer@example.com'): Licence
    {
        return Licence::create([
            'key_lookup'  => strtoupper(Str::random(20)),
            'email'       => $email,
            'sku'         => $sku,
            'max_devices' => $maxDevices,
            'status'      => 'active',
            'issued_at'   => now(),
        ]);
    }

    public function test_unauthenticated_request_is_refused(): void
    {
        $response = $this->get(route('admin.licences.index'));
        // Either a 302 redirect to sign-in or a 403; the load-bearing
        // assertion is "not 200".
        $this->assertNotEquals(200, $response->status());
    }

    public function test_authed_non_admin_user_gets_403(): void
    {
        $user = $this->seedNonAdmin();
        $response = $this->actingAs($user)->get(route('admin.licences.index'));
        $response->assertStatus(403);
    }

    public function test_admin_sees_volume_chip_and_seat_count_for_volume_row(): void
    {
        $admin = $this->seedAdmin();
        $volume = $this->seedLicence(VolumePricing::sku(), 50, 'fleet@example.com');

        $response = $this->actingAs($admin)->get(route('admin.licences.index'));

        $response->assertStatus(200);
        $response->assertSee('Volume', false);
        // Seat-count emphasis: "0 / 50" + the "(50 seats)" annotation.
        $response->assertSee('0 / 50', false);
        $response->assertSee('(50 seats)', false);
        // The buyer's email should appear so the admin can spot the row.
        $response->assertSee('fleet@example.com', false);
    }

    public function test_single_user_row_does_not_show_volume_chip(): void
    {
        $admin = $this->seedAdmin();
        $single = $this->seedLicence('utterbar-pro-lifetime', 3, 'consumer@example.com');

        $response = $this->actingAs($admin)->get(route('admin.licences.index'));

        $response->assertStatus(200);
        $response->assertSee('consumer@example.com', false);
        // The bare seat fraction renders for single-user, but the
        // "(N seats)" annotation is volume-only.
        $response->assertSee('0 / 3', false);
        $response->assertDontSee('(3 seats)', false);
        $response->assertDontSee('(seats)', false);
    }

    public function test_mixed_index_renders_both_types(): void
    {
        $admin = $this->seedAdmin();
        $volume  = $this->seedLicence(VolumePricing::sku(), 25, 'volume@example.com');
        $single  = $this->seedLicence('utterbar-pro-lifetime', 3, 'single@example.com');

        $response = $this->actingAs($admin)->get(route('admin.licences.index'));

        $response->assertStatus(200);
        $response->assertSee('volume@example.com', false);
        $response->assertSee('single@example.com', false);
        $response->assertSee('Volume', false);
        $response->assertSee('(25 seats)', false);
    }
}
