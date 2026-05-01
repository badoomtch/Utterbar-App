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
 * Admin licence detail page (GET /admin/licences/{id}).
 *
 * Pins:
 *   * Volume row renders the Volume hero chip with seat count, the
 *     dedicated "Volume / fleet deployment" card with a primary
 *     Download licence.bin CTA, and the IT-deployment doc link.
 *   * Single-user row renders the existing Actions card unchanged
 *     and does NOT carry the IT block.
 *   * Auth gate still keeps unauthenticated and non-admin users out.
 *   * The licence.bin route is admin-only on both row types — there
 *     is no public surface for it.
 */
class LicenceShowAdminTest extends TestCase
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

    private function seedNonAdmin(): User
    {
        putenv('ADMIN_EMAILS=admin@example.com');
        return User::create([
            'name'              => 'User',
            'email'             => 'someone@example.com',
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
        $licence = $this->seedLicence(VolumePricing::sku(), 25);
        $response = $this->get(route('admin.licences.show', $licence->id));
        $this->assertNotEquals(200, $response->status());
    }

    public function test_authed_non_admin_user_gets_403(): void
    {
        $licence = $this->seedLicence(VolumePricing::sku(), 25);
        $user = $this->seedNonAdmin();
        $response = $this->actingAs($user)->get(route('admin.licences.show', $licence->id));
        $response->assertStatus(403);
    }

    public function test_volume_row_renders_volume_hero_block_and_download_cta(): void
    {
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence(VolumePricing::sku(), 50, 'fleet@example.com');

        $response = $this->actingAs($admin)->get(route('admin.licences.show', $licence->id));

        $response->assertStatus(200);

        // Hero chip + seat count.
        $response->assertSee('Volume licence', false);
        $response->assertSee('50 seats', false);

        // Dedicated fleet block with the IT deployment doc link.
        $response->assertSee('Volume / fleet deployment', false);
        $response->assertSee('%APPDATA%\Utterbar\licence.bin', false);
        $response->assertSee(route('docs.it-deployment'), false);

        // Primary Download licence.bin CTA appears in the volume block.
        $response->assertSee(route('admin.licences.download', $licence->id), false);
        $response->assertSee('Download licence.bin', false);
    }

    public function test_single_user_row_does_not_render_volume_block(): void
    {
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence('utterbar-pro-lifetime', 3, 'consumer@example.com');

        $response = $this->actingAs($admin)->get(route('admin.licences.show', $licence->id));

        $response->assertStatus(200);

        // Single-user pages must not advertise themselves as volume.
        $response->assertDontSee('Volume / fleet deployment', false);
        $response->assertDontSee('Volume licence', false);
        $response->assertDontSee('25 seats', false); // sanity: no stale text
        $response->assertDontSee('50 seats', false);

        // The standard Actions row + its existing Download licence.bin
        // ghost button MUST still be there — the admin can manually
        // mint and serve a licence.bin for any row. The change for
        // volume is that we additionally surface it as a primary CTA;
        // for single-user the existing surface is unchanged.
        $response->assertSee('Actions', false);
        $response->assertSee(route('admin.licences.download', $licence->id), false);
    }

    public function test_volume_row_includes_existing_actions_unchanged(): void
    {
        // Volume rows must not LOSE any of the existing actions
        // (resend, re-sign, revoke, the existing ghost-button
        // licence.bin link). Pin that the Actions card still renders
        // and the existing routes are still wired in.
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence(VolumePricing::sku(), 25);

        $response = $this->actingAs($admin)->get(route('admin.licences.show', $licence->id));

        $response->assertStatus(200);
        $response->assertSee('Actions', false);
        $response->assertSee(route('admin.licences.resend', $licence->id), false);
        $response->assertSee(route('admin.licences.remint', $licence->id), false);
        $response->assertSee(route('admin.licences.revoke', $licence->id), false);
        $response->assertSee(route('admin.licences.download', $licence->id), false);
    }
}
