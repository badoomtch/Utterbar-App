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
 * CSP is asserted to arrive in Report-Only form. Strict-Transport-
 * Security is asserted at a deliberately short 1-day max-age and
 * without `preload`; raising those is a separate decision after the
 * soak window proves clean.
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

        // Pin the absence of upgrade-insecure-requests. Cloudflare's
        // 301 already upgrades HTTP navigations to HTTPS, so the
        // directive is not needed for that. Adding it would also force
        // embedded HTTP sub-resources to upgrade in-place, which can
        // break third-party assets that don't serve HTTPS. Revisit
        // separately once mixed-content behaviour has been reviewed.
        $this->assertStringNotContainsString('upgrade-insecure-requests', $csp);
    }

    #[DataProvider('sampledRoutes')]
    public function test_hsts_short_max_age_present(string $path): void
    {
        config([
            'services.stripe.pro_amount'   => 3999,
            'services.stripe.pro_currency' => 'gbp',
        ]);

        $response = $this->get($path);
        $response->assertStatus(200);

        // Cloudflare's "Always Use HTTPS" toggle is on and verified
        // redirecting plain HTTP to HTTPS, so HSTS is meaningful. The
        // 1-day max-age is a deliberate soak value: if a subdomain HTTPS
        // regression slips through, the worst case is 24 hours of clients
        // unable to fall back to HTTP. Raise to 1 year (31536000) only
        // after a clean week. No `preload` here.
        $response->assertHeader(
            'Strict-Transport-Security',
            'max-age=86400; includeSubDomains'
        );

        $hsts = (string) $response->headers->get('Strict-Transport-Security');
        $this->assertStringNotContainsString(
            'preload',
            $hsts,
            'preload must not be set until every subdomain is committed to HTTPS-only forever; submission is irrevocable for ~6 months.'
        );
    }
}
