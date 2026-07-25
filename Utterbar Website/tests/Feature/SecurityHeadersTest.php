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
 *   - GET /                    : marketing Blade page
 *   - GET /download            : Blade page with a DB read
 *   - GET /api/updater/check   : JSON API endpoint consumed by the Tauri app
 *
 * If any of these stops carrying the expected headers, browsers lose
 * the click-jacking / sniffing / referrer / permission protections we
 * advertise.
 *
 * CSP is asserted to arrive in Report-Only form. Strict-Transport-
 * Security is asserted at one year and without `preload`; adding
 * `preload` is a separate, effectively-irrevocable decision.
 */
class SecurityHeadersTest extends TestCase
{
    use RefreshDatabase;

    /** @return array<string, array{string}> */
    public static function sampledRoutes(): array
    {
        return [
            'home page'      => ['/'],
            'download page'  => ['/download'],
            'updater api'    => ['/api/updater/check'],
        ];
    }

    #[DataProvider('sampledRoutes')]
    public function test_baseline_security_headers_are_present(string $path): void
    {
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
        $this->assertStringContainsString("form-action 'self'", $csp);
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
    public function test_hsts_one_year_max_age_present(string $path): void
    {
        $response = $this->get($path);
        $response->assertStatus(200);

        // Cloudflare's "Always Use HTTPS" toggle is on and the HTTP -> HTTPS
        // redirect is verified across the apex, www, and dl. subdomains, so
        // HSTS is meaningful. max-age is one year (31536000). `preload`
        // remains deliberately omitted: submission to the HSTS preload
        // list is effectively irrevocable for ~6 months.
        $response->assertHeader(
            'Strict-Transport-Security',
            'max-age=31536000; includeSubDomains'
        );

        $hsts = (string) $response->headers->get('Strict-Transport-Security');
        $this->assertStringNotContainsString(
            'preload',
            $hsts,
            'preload must not be set until every subdomain is committed to HTTPS-only forever; submission is irrevocable for ~6 months.'
        );
    }
}
