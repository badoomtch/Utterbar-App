<?php

namespace Tests\Feature\Admin;

use App\Models\Licence;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Str;
use Tests\TestCase;

/**
 * Admin "Download licence.bin" route. Two guarantees:
 *
 *   1. Auth is enforced. Unauthenticated requests redirect; authed
 *      non-admin users get 403 from the AdminOnly email-allowlist
 *      middleware. We verify both shapes; the redirect destination
 *      isn't asserted because Laravel's default redirect target is
 *      not pinned by this controller.
 *
 *   2. An admin downloading a licence with a populated signed_blob
 *      gets the raw bytes back as application/octet-stream with a
 *      predictable filename. The bytes are byte-for-byte the column
 *      contents — no encoding, no wrapper.
 *
 * The route is `GET /admin/licences/{id}/download`, named
 * `admin.licences.download`. The download serves the existing
 * `signed_blob` column on the Licence model (a 96-byte Ed25519-
 * signed blob in production); these tests use 96 random bytes as
 * a fixture because the test doesn't exercise verification.
 */
class LicenceDownloadTest extends TestCase
{
    use RefreshDatabase;

    private function seedUser(string $email): User
    {
        return User::create([
            'name'              => 'Test',
            'email'             => $email,
            'password'          => Hash::make(Str::random(64)),
            'email_verified_at' => now(),
        ]);
    }

    private function seedLicenceWithBlob(string $blob): Licence
    {
        return Licence::create([
            'key_lookup'   => strtoupper(Str::random(20)),
            'email'        => 'buyer@example.com',
            'sku'          => 'utterbar-pro-lifetime',
            'signed_blob'  => $blob,
            'max_devices'  => 3,
            'status'       => 'active',
            'issued_at'    => now(),
        ]);
    }

    private function seedLicenceWithoutBlob(): Licence
    {
        return Licence::create([
            'key_lookup'  => strtoupper(Str::random(20)),
            'email'       => 'buyer@example.com',
            'sku'         => 'utterbar-pro-lifetime',
            'signed_blob' => null,
            'max_devices' => 3,
            'status'      => 'active',
            'issued_at'   => now(),
        ]);
    }

    public function test_unauthenticated_request_is_refused(): void
    {
        $licence = $this->seedLicenceWithBlob(random_bytes(96));

        $response = $this->get(route('admin.licences.download', $licence->id));

        // Laravel redirects unauthenticated requests to the sign-in
        // page by default. Either a 302 redirect or 403 is acceptable
        // here — the load-bearing assertion is "not 200".
        $this->assertNotEquals(200, $response->status());
    }

    public function test_authed_non_admin_user_gets_403(): void
    {
        $user = $this->seedUser('not-admin@example.com');
        $licence = $this->seedLicenceWithBlob(random_bytes(96));

        // Force the env so the AdminOnly middleware is active and
        // doesn't accidentally let everyone through.
        config(['app.admin_emails_override' => null]);
        putenv('ADMIN_EMAILS=admin@example.com');

        $response = $this->actingAs($user)
            ->get(route('admin.licences.download', $licence->id));

        $response->assertStatus(403);
    }

    public function test_admin_download_returns_raw_bytes(): void
    {
        $admin = $this->seedUser('admin@example.com');
        $bytes = random_bytes(96);
        $licence = $this->seedLicenceWithBlob($bytes);

        putenv('ADMIN_EMAILS=admin@example.com');

        $response = $this->actingAs($admin)
            ->get(route('admin.licences.download', $licence->id));

        $response->assertStatus(200);
        $response->assertHeader('Content-Type', 'application/octet-stream');
        $response->assertHeader(
            'Content-Disposition',
            'attachment; filename="utterbar-licence-' . $licence->id . '.bin"'
        );
        $this->assertSame(
            $bytes,
            $response->getContent(),
            'admin download must serve signed_blob byte-for-byte (no encoding, no wrapper)',
        );
        $response->assertHeader('Content-Length', (string) strlen($bytes));
    }

    public function test_admin_download_404s_when_signed_blob_is_missing(): void
    {
        $admin = $this->seedUser('admin@example.com');
        $licence = $this->seedLicenceWithoutBlob();

        putenv('ADMIN_EMAILS=admin@example.com');

        $response = $this->actingAs($admin)
            ->get(route('admin.licences.download', $licence->id));

        $response->assertStatus(404);
    }
}
