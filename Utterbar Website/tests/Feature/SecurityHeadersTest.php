<?php

namespace Tests\Feature;

use Illuminate\Foundation\Testing\RefreshDatabase;
use PHPUnit\Framework\Attributes\DataProvider;
use Tests\TestCase;

/**
 * Pins the baseline browser-hardening headers added by
 * App\Http\Middleware\SecurityHeaders.
 *
 * The middleware is registered globally in bootstrap/app.php, so every
 * route (web, api, /up health) should carry the headers. We sample
 * three different surface types:
 *
 *   - GET /              : marketing Blade page
 *   - GET /sign-in       : auth Blade page
 *   - GET /api/pricing   : JSON API endpoint consumed by the Tauri app
 *
 * If any of these stops carrying the expected headers, browsers lose
 * the click-jacking / sniffing / referrer / permission protections we
 * advertise.
 *
 * CSP is asserted to arrive in Report-Only form. The middleware does
 * not yet emit Strict-Transport-Security (HTTP -> HTTPS redirect at
 * the edge isn't enabled yet), so we also pin its absence so a
 * future code path doesn't sneak HSTS in before the redirect lands.
 */
class SecurityHeadersTest extends TestCase
{
    use RefreshDatabase;

    /** @return array<string, array{string}> */
    public static function sampledRoutes(): array
    {
        return [
            'home page'      => ['/'],
            'sign-in page'   => ['/sign-in'],
            'pricing api'    => ['/api/pricing'],
        ];
    }

    #[DataProvider('sampledRoutes')]
    public function test_baseline_security_headers_are_present(string $path): void
    {
        // /api/pricing reads Stripe config; pin it so the route doesn't 500.
        config([
            'services.stripe.pro_amount'   => 3999,
            'services.stripe.pro_currency' => 'gbp',
        ]);

        $response = $this->get($path);
        $response->assertStatus(200);

        $response->assertHeader('X-Frame-Options', 'DENY');
        $response->assertHeader('X-Content-Type-Options', 'nosniff');
        $response->assertHeader('Referrer-Policy', 'strict-origin-when-cross-origin');
        $response->assertHeader('Permissions-Policy', 'camera=(), microphone=(), geolocation=(), payment=()');
    }

    #[DataProvider('sampledRoutes')]
    public function test_csp_is_report_only_not_enforced(string $path): void
    {
        config([
            'services.stripe.pro_amount'   => 3999,
            'services.stripe.pro_currency' => 'gbp',
        ]);

        $response = $this->get($path);
        $response->assertStatus(200);

        $response->assertHeader('Content-Security-Policy-Report-Only');
        $this->assertFalse(
            $response->headers->has('Content-Security-Policy'),
            "Content-Security-Policy must not be set yet; this commit is Report-Only."
        );

        $csp = (string) $response->headers->get('Content-Security-Policy-Report-Only');

        // Spot-check directives we explicitly contracted on.
        $this->assertStringContainsString("default-src 'self'", $csp);
        $this->assertStringContainsString("base-uri 'self'", $csp);
        $this->assertStringContainsString("object-src 'none'", $csp);
        $this->assertStringContainsString("frame-ancestors 'none'", $csp);
        $this->assertStringContainsString("form-action 'self' https://checkout.stripe.com", $csp);
        $this->assertStringContainsString('frame-src https://checkout.stripe.com', $csp);
        $this->assertStringContainsString("script-src 'self' 'unsafe-inline' https://www.googletagmanager.com https://instant.page", $csp);
        $this->assertStringContainsString("style-src 'self' 'unsafe-inline'", $csp);
        $this->assertStringContainsString('img-src \'self\' data: blob: https:', $csp);
        $this->assertStringContainsString("font-src 'self' data:", $csp);
        $this->assertStringContainsString('connect-src \'self\' https://www.google-analytics.com https://*.google-analytics.com', $csp);

        // Pin the absence of upgrade-insecure-requests for now: HTTP -> HTTPS
        // redirect is not enabled at Cloudflare yet, so forcing upgrades
        // would produce broken links until that edge change lands.
        $this->assertStringNotContainsString('upgrade-insecure-requests', $csp);
    }

    #[DataProvider('sampledRoutes')]
    public function test_hsts_is_not_emitted_yet(string $path): void
    {
        config([
            'services.stripe.pro_amount'   => 3999,
            'services.stripe.pro_currency' => 'gbp',
        ]);

        $response = $this->get($path);
        $response->assertStatus(200);

        // HSTS is only meaningful once HTTP returns a redirect to HTTPS at
        // the edge. Until that's done in Cloudflare, we deliberately do not
        // emit Strict-Transport-Security from the app.
        $this->assertFalse(
            $response->headers->has('Strict-Transport-Security'),
            "Strict-Transport-Security must not be emitted until Cloudflare 'Always Use HTTPS' is enabled."
        );
    }
}
