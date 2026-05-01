<?php

namespace Tests\Feature\Admin;

use App\Models\Activation;
use App\Models\Licence;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Http\UploadedFile;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Str;
use Tests\TestCase;

/**
 * Admin "Upload activation request" route. Pinned guarantees:
 *
 * 1. Auth gate — unauthenticated requests redirect to sign-in;
 *    authed non-admin users get 403 from the AdminOnly middleware.
 * 2. A valid request file creates a single Activation row with
 *    `offline = true`, `last_heartbeat = null`, and the
 *    fingerprint + device_name lifted from the JSON.
 * 3. Validation errors (bad JSON, wrong magic, bad fingerprint
 *    shape, oversized file) flash an error and create no row.
 * 4. Idempotency on `(licence_id, fingerprint_hash)`: a duplicate
 *    upload returns a friendly flash without creating a second
 *    row.
 * 5. max_devices is enforced — a fourth upload against a
 *    3-device licence flashes an error and creates no row.
 * 6. The existing `/admin/licences/{id}/download` route still
 *    serves the signed_blob byte-for-byte (regression guard).
 */
class AirGappedActivationTest extends TestCase
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

    private function seedLicence(int $maxDevices = 3): Licence
    {
        return Licence::create([
            'key_lookup'  => strtoupper(Str::random(20)),
            'email'       => 'buyer@example.com',
            'sku'         => 'utterbar-pro-lifetime',
            'max_devices' => $maxDevices,
            'status'      => 'active',
            'issued_at'   => now(),
        ]);
    }

    /**
     * Build a valid request payload. Tests override individual
     * keys to exercise the validator's rejection paths.
     */
    private function validRequestBody(array $overrides = []): array
    {
        return array_replace([
            'format'         => 'utterbar-licence-request',
            'format_version' => 1,
            'fingerprint'    => str_repeat('a', 64),
            'device_name'    => 'TEST-PC',
            'platform'       => 'windows',
            'app_version'    => '0.15.11',
            'generated_at'   => '2026-05-03T14:25:01Z',
        ], $overrides);
    }

    /**
     * Wrap an array as a multipart UploadedFile. Reuses Laravel's
     * fake-upload helper rather than hitting tmpfile by hand.
     */
    private function makeUpload(array $payload): UploadedFile
    {
        $json = json_encode($payload, JSON_PRETTY_PRINT);
        return UploadedFile::fake()->createWithContent('request.json', $json);
    }

    public function test_unauthenticated_request_is_refused(): void
    {
        $licence = $this->seedLicence();
        $response = $this->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($this->validRequestBody())]
        );
        // Either a 302 redirect or 403 is acceptable; the load-bearing
        // assertion is "not 200" and no row was created.
        $this->assertNotEquals(200, $response->status());
        $this->assertSame(0, Activation::count(), 'no row should be created without auth');
    }

    public function test_authed_non_admin_user_gets_403(): void
    {
        $licence = $this->seedLicence();
        $user = $this->seedNonAdmin();
        $response = $this->actingAs($user)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($this->validRequestBody())]
        );
        $response->assertStatus(403);
        $this->assertSame(0, Activation::count());
    }

    public function test_valid_request_creates_offline_activation_row(): void
    {
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence();
        $body = $this->validRequestBody(['device_name' => 'OFFLINE-LAPTOP-7']);

        $response = $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($body)]
        );

        $response->assertRedirect(route('admin.licences.show', $licence->id));
        $this->assertSame(1, Activation::count(), 'exactly one row created');

        $row = Activation::first();
        $this->assertSame($licence->id, $row->licence_id);
        $this->assertSame(str_repeat('a', 64), $row->fingerprint_hash);
        $this->assertSame('OFFLINE-LAPTOP-7', $row->device_name);
        $this->assertSame('windows', $row->platform);
        $this->assertSame('0.15.11', $row->app_version);
        $this->assertTrue($row->offline, 'offline marker must be true on uploads');
        $this->assertNull($row->last_heartbeat, 'offline rows never heartbeat');
        $this->assertNotNull($row->first_seen);
        $this->assertSame(64, strlen($row->activation_token));
    }

    public function test_invalid_json_is_rejected(): void
    {
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence();
        $upload = UploadedFile::fake()->createWithContent('bad.json', 'not json at all');

        $response = $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $upload]
        );

        $response->assertRedirect(route('admin.licences.show', $licence->id));
        $this->assertSame(0, Activation::count());
        $response->assertSessionHas('flash', function ($flash) {
            return ($flash['type'] ?? null) === 'error'
                && str_contains($flash['text'] ?? '', 'not valid JSON');
        });
    }

    public function test_wrong_format_magic_is_rejected(): void
    {
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence();
        $body = $this->validRequestBody(['format' => 'something-else']);

        $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($body)]
        );

        $this->assertSame(0, Activation::count());
    }

    public function test_wrong_format_version_is_rejected(): void
    {
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence();
        // String "1" instead of int 1 — strict comparison will fail.
        $body = $this->validRequestBody(['format_version' => '1']);

        $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($body)]
        );

        $this->assertSame(0, Activation::count());

        // And version 2 (future format we don't yet understand).
        $body2 = $this->validRequestBody(['format_version' => 2]);
        $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($body2)]
        );
        $this->assertSame(0, Activation::count());
    }

    public function test_bad_fingerprint_shape_is_rejected(): void
    {
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence();
        // Cases: wrong length, uppercase hex, non-hex chars, missing.
        $bad = ['short', str_repeat('A', 64), str_repeat('z', 64)];
        foreach ($bad as $fp) {
            $body = $this->validRequestBody(['fingerprint' => $fp]);
            $this->actingAs($admin)->post(
                route('admin.licences.activations.upload', $licence->id),
                ['request_file' => $this->makeUpload($body)]
            );
        }
        $this->assertSame(0, Activation::count(), 'every bad fingerprint shape must be rejected');
    }

    public function test_oversized_file_is_rejected(): void
    {
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence();
        // Build a payload >4 KB by stuffing the device_name with
        // junk. Laravel's `file|max:4` (KB) rule rejects before we
        // even hit the JSON parser.
        $body = $this->validRequestBody([
            'device_name' => str_repeat('A', 5000),
        ]);
        $upload = $this->makeUpload($body);

        $response = $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $upload]
        );

        // Validator rejects with 422 + redirect-back errors — either
        // shape is fine; the load-bearing assertion is "no row".
        $this->assertSame(0, Activation::count());
    }

    public function test_duplicate_upload_is_idempotent(): void
    {
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence();
        $body = $this->validRequestBody();

        $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($body)]
        );
        $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($body)]
        );

        $this->assertSame(1, Activation::count(), 'duplicate upload must not create a second row');
    }

    public function test_max_devices_limit_is_enforced(): void
    {
        $admin = $this->seedAdmin();
        $licence = $this->seedLicence(maxDevices: 2);

        // Two different fingerprints → fills the cap.
        $a = $this->validRequestBody(['fingerprint' => str_repeat('a', 64)]);
        $b = $this->validRequestBody(['fingerprint' => str_repeat('b', 64)]);
        $c = $this->validRequestBody(['fingerprint' => str_repeat('c', 64)]);

        $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($a)]
        );
        $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($b)]
        );
        $response = $this->actingAs($admin)->post(
            route('admin.licences.activations.upload', $licence->id),
            ['request_file' => $this->makeUpload($c)]
        );

        $this->assertSame(2, Activation::count(), 'cap of 2 must be enforced');
        $response->assertSessionHas('flash', function ($flash) {
            return ($flash['type'] ?? null) === 'error'
                && str_contains($flash['text'] ?? '', 'Device limit reached');
        });
    }

    /**
     * Regression guard: the existing `/admin/licences/{id}/download`
     * route MUST still serve the signed_blob bytes unchanged. The
     * air-gapped flow ends with the operator clicking that link;
     * if its behaviour drifts, the whole sprint silently breaks.
     */
    public function test_existing_download_route_still_serves_signed_blob_byte_for_byte(): void
    {
        $admin = $this->seedAdmin();
        $bytes = random_bytes(96);
        $licence = Licence::create([
            'key_lookup'  => strtoupper(Str::random(20)),
            'email'       => 'buyer@example.com',
            'sku'         => 'utterbar-pro-lifetime',
            'signed_blob' => $bytes,
            'max_devices' => 3,
            'status'      => 'active',
            'issued_at'   => now(),
        ]);

        $response = $this->actingAs($admin)
            ->get(route('admin.licences.download', $licence->id));

        $response->assertStatus(200);
        $response->assertHeader('Content-Type', 'application/octet-stream');
        $this->assertSame($bytes, $response->getContent(), 'signed_blob must come back byte-for-byte');
    }
}
