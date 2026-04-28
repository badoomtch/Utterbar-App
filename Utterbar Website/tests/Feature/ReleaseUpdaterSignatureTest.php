<?php

namespace Tests\Feature;

use App\Models\Release;
use App\Models\User;
use App\Services\Releases\UpdateMetadataSigner;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Storage;
use Illuminate\Support\Str;
use Tests\TestCase;

/**
 * Updater-signature enforcement on the publish path.
 *
 * `ReleasePublishValidationTest` covers shape, SHA recompute, channel
 * rules, and stable+announce confirmation, but skips signature
 * verification because that requires a live Ed25519 keypair. This
 * file generates a real keypair, configures
 * `services.updater.public_key_b64`, signs the canonical metadata
 * blob, and asserts the validator's signature gating across both
 * publish entry points.
 *
 * Coverage:
 *   - API rejects missing signature when key configured
 *   - browser rejects missing signature when key configured
 *   - API rejects invalid signature
 *   - browser rejects invalid signature
 *   - valid signature succeeds and the row's `updater_signature`
 *     column carries the operator's b64 string
 *   - signature is verified against the SERVER-COMPUTED SHA, not the
 *     pasted SHA (a signature signed over the wrong SHA must be
 *     refused even when the pasted SHA matches the computed one)
 */
class ReleaseUpdaterSignatureTest extends TestCase
{
    use RefreshDatabase;

    private const ADMIN_TOKEN = 'test-updater-sig-admin-token';
    private const ADMIN_EMAIL = 'admin-updater-sig@example.com';

    /** Raw 32-byte Ed25519 secret key (operator-local). */
    private string $secretKey;
    /** Raw 32-byte Ed25519 public key (server-side). */
    private string $publicKey;

    protected function setUp(): void
    {
        parent::setUp();
        putenv('ADMIN_API_TOKEN=' . self::ADMIN_TOKEN);
        $_ENV['ADMIN_API_TOKEN'] = self::ADMIN_TOKEN;
        putenv('ADMIN_EMAILS=' . self::ADMIN_EMAIL);
        $_ENV['ADMIN_EMAILS'] = self::ADMIN_EMAIL;
        Storage::fake('r2');

        // Generate a real Ed25519 keypair for this test. libsodium
        // returns a packed 64-byte sign-keypair; split into seed-
        // derived secret + public halves and base64-encode the public
        // for the config override.
        $kp = sodium_crypto_sign_keypair();
        $this->secretKey = sodium_crypto_sign_secretkey($kp);
        $this->publicKey = sodium_crypto_sign_publickey($kp);
        config()->set(
            'services.updater.public_key_b64',
            base64_encode($this->publicKey),
        );
    }

    private function adminUser(): User
    {
        return User::firstOrCreate(
            ['email' => self::ADMIN_EMAIL],
            [
                'name'              => 'Admin',
                'password'          => Hash::make(Str::random(64)),
                'email_verified_at' => now(),
            ],
        );
    }

    /**
     * Seed deterministic bytes into the fake R2 disk and return both
     * the body fields and the canonical SHA the server will compute.
     * Pulling this out keeps each test focused on the signature
     * variation rather than the seeding boilerplate.
     */
    private function seedAndBaseBody(array $override = []): array
    {
        $r2Key = $override['r2_key'] ?? 'releases/0.6.2/Utterbar-Setup-0.6.2.exe';
        $bytes = "fake-installer-{$r2Key}";
        Storage::disk('r2')->put($r2Key, $bytes);
        $sha = hash('sha256', $bytes);

        $body = array_merge([
            'version'        => '0.6.2',
            'channel'        => 'stable',
            'r2_key'         => $r2Key,
            'sha256'         => $sha,
            'notes_markdown' => 'sig test',
        ], $override);

        // Stable + announce gate: the validator refuses without an
        // explicit confirm flag. Tests here never want to be tripped
        // by that branch — pre-set it so we exercise the signature
        // path specifically.
        if (($body['channel'] ?? null) === 'stable' && ($body['announce'] ?? false)) {
            $body['confirm_stable_announce'] = true;
        }

        return ['body' => $body, 'sha' => $sha];
    }

    /**
     * Compute the canonical-blob signature for `$body`, encoded
     * base64. Tests use this for the happy-path signature; for the
     * negative cases the signature is simply replaced.
     */
    private function signFor(array $body, string $canonicalSha): string
    {
        $blob = UpdateMetadataSigner::canonicalBlob([
            'version'      => $body['version'],
            'channel'      => $body['channel'],
            'size_bytes'   => Storage::disk('r2')->size($body['r2_key']),
            'sha256'       => $canonicalSha,
            'download_url' => $this->expectedDownloadUrl($body['r2_key']),
        ]);
        $sig = sodium_crypto_sign_detached($blob, $this->secretKey);
        return base64_encode($sig);
    }

    private function expectedDownloadUrl(string $r2Key): string
    {
        $cdnBase = (string) (
            config('filesystems.disks.r2.url')
            ?: env('CLOUDFLARE_R2_PUBLIC_URL', 'https://dl.utterbar.com')
        );
        return rtrim($cdnBase, '/') . '/' . ltrim($r2Key, '/');
    }

    private function postApi(array $body)
    {
        return $this->postJson('/api/admin/releases', $body, [
            'Authorization' => 'Bearer ' . self::ADMIN_TOKEN,
        ]);
    }

    private function postBrowser(array $body)
    {
        $form = $body;
        if (array_key_exists('notes_markdown', $form)) {
            $form['notes'] = $form['notes_markdown'];
            unset($form['notes_markdown']);
        }
        return $this->actingAs($this->adminUser())->post('/admin/releases', $form);
    }

    /* ----- Missing signature is refused on both paths when keyed ----- */

    public function test_api_rejects_publish_with_missing_signature_when_keyed(): void
    {
        ['body' => $body] = $this->seedAndBaseBody();
        // No `updater_signature` field.
        $resp = $this->postApi($body);
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'updater_signature_required');
        $this->assertSame(0, Release::count());
    }

    public function test_browser_rejects_publish_with_missing_signature_when_keyed(): void
    {
        ['body' => $body] = $this->seedAndBaseBody();
        $resp = $this->postBrowser($body);
        $resp->assertSessionHasErrors(['updater_signature']);
        $this->assertSame(0, Release::count());
    }

    /* ----- Invalid signature is refused on both paths ----- */

    public function test_api_rejects_publish_with_invalid_signature(): void
    {
        ['body' => $body] = $this->seedAndBaseBody();
        // 88 b64 chars (= 64 raw bytes) so the field-level length
        // validator passes; bytes themselves don't verify.
        $body['updater_signature'] = base64_encode(str_repeat("\x00", SODIUM_CRYPTO_SIGN_BYTES));
        $resp = $this->postApi($body);
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'updater_signature_invalid');
        $this->assertSame(0, Release::count());
    }

    public function test_browser_rejects_publish_with_invalid_signature(): void
    {
        ['body' => $body] = $this->seedAndBaseBody();
        $body['updater_signature'] = base64_encode(str_repeat("\x00", SODIUM_CRYPTO_SIGN_BYTES));
        $resp = $this->postBrowser($body);
        $resp->assertSessionHasErrors(['updater_signature']);
        $this->assertSame(0, Release::count());
    }

    /* ----- Valid signature succeeds and persists ----- */

    public function test_api_accepts_publish_with_valid_signature(): void
    {
        ['body' => $body, 'sha' => $sha] = $this->seedAndBaseBody();
        $body['updater_signature'] = $this->signFor($body, $sha);
        $resp = $this->postApi($body);
        $resp->assertStatus(201);

        $row = Release::first();
        $this->assertNotNull($row);
        $this->assertSame($body['updater_signature'], $row->updater_signature);
    }

    public function test_browser_accepts_publish_with_valid_signature(): void
    {
        ['body' => $body, 'sha' => $sha] = $this->seedAndBaseBody();
        $body['updater_signature'] = $this->signFor($body, $sha);
        $resp = $this->postBrowser($body);
        $resp->assertRedirect(route('admin.releases.index'));

        $row = Release::first();
        $this->assertNotNull($row);
        $this->assertSame($body['updater_signature'], $row->updater_signature);
    }

    /* ----- Signature must verify against the canonical SHA, not a stray one ----- */

    /**
     * The signature is verified against `ReleaseValidator`'s
     * computed SHA. Because the validator rejects pasted-vs-
     * computed SHA mismatches BEFORE it reaches the signature
     * branch, this test can't separately distinguish "verified
     * against pasted" from "verified against computed" when the
     * two are equal — the SHA-mismatch gate would have refused
     * either case anyway.
     *
     * What this test DOES prove: a signature that was never
     * signed over the canonical metadata blob fails verification.
     * Construct a signature over a deliberately-wrong-SHA blob
     * with the otherwise-correct version/channel/size/url, send it
     * with the correct pasted SHA (so SHA recompute passes), and
     * confirm the signature verification step refuses it. That
     * pins the contract: the validator does its own canonical-
     * blob construction and won't accept any signature that
     * doesn't verify against it.
     */
    public function test_api_refuses_signature_signed_over_a_different_sha(): void
    {
        ['body' => $body, 'sha' => $sha] = $this->seedAndBaseBody();
        $this->assertNotEmpty($sha);

        $wrongSha = str_repeat('b', 64);
        $blob = UpdateMetadataSigner::canonicalBlob([
            'version'      => $body['version'],
            'channel'      => $body['channel'],
            'size_bytes'   => Storage::disk('r2')->size($body['r2_key']),
            'sha256'       => $wrongSha,
            'download_url' => $this->expectedDownloadUrl($body['r2_key']),
        ]);
        $body['updater_signature'] = base64_encode(
            sodium_crypto_sign_detached($blob, $this->secretKey),
        );

        $resp = $this->postApi($body);
        $resp->assertStatus(422);
        $resp->assertJsonPath('error', 'updater_signature_invalid');
        $this->assertSame(0, Release::count());
    }
}
