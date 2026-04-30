<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

/**
 * Adds baseline browser-hardening headers to every response.
 *
 * Covers: clickjacking (XFO + CSP frame-ancestors), MIME sniffing,
 * referrer leakage, gratuitous Permissions-Policy capability access,
 * and a starter Content-Security-Policy in REPORT-ONLY mode.
 *
 * CSP is Report-Only because the site currently ships inline <script>
 * blocks (cookie consent, gtag config, JSON-LD, page-level interactive
 * scripts) and inline <style> blocks (paint-early background, alpine
 * cloak helper, component styles). Allowing 'unsafe-inline' for
 * script-src and style-src keeps the page functional but means this
 * commit does NOT yet provide full XSS containment.
 *
 * Follow-up work (separate commit) should:
 *   1. Refactor inline scripts to use a per-request CSP nonce, or move
 *      them into Vite-bundled external files.
 *   2. Drop 'unsafe-inline' from script-src once nonces are in place.
 *   3. Switch the header from Content-Security-Policy-Report-Only to
 *      Content-Security-Policy after a clean soak in browser dev
 *      consoles.
 *
 * Strict-Transport-Security is intentionally NOT emitted yet. HTTP
 * currently returns 200 because Cloudflare's "Always Use HTTPS" toggle
 * is off; HSTS is only meaningful once the redirect is in place.
 *
 * upgrade-insecure-requests is intentionally NOT used in the CSP for
 * the same reason: HTTP isn't redirected to HTTPS at the edge yet.
 *
 * Existing headers (e.g. set by a controller for a specific response)
 * are not overwritten. Each header is gated by a `$response->headers->has(...)`
 * check before `set()`, so a downstream caller can opt out per-response
 * by setting the header itself before this middleware runs.
 */
class SecurityHeaders
{
    /**
     * @var array<string, string>
     */
    private const STATIC_HEADERS = [
        'X-Frame-Options'        => 'DENY',
        'X-Content-Type-Options' => 'nosniff',
        'Referrer-Policy'        => 'strict-origin-when-cross-origin',
        'Permissions-Policy'     => 'camera=(), microphone=(), geolocation=(), payment=()',
    ];

    public function handle(Request $request, Closure $next): Response
    {
        $response = $next($request);

        foreach (self::STATIC_HEADERS as $name => $value) {
            if (!$response->headers->has($name)) {
                $response->headers->set($name, $value);
            }
        }

        if (!$response->headers->has('Content-Security-Policy-Report-Only')
            && !$response->headers->has('Content-Security-Policy')) {
            $response->headers->set(
                'Content-Security-Policy-Report-Only',
                $this->buildCsp()
            );
        }

        return $response;
    }

    /**
     * Build the Report-Only CSP. Kept in one place so the test asserts
     * against the same string.
     */
    private function buildCsp(): string
    {
        $directives = [
            "default-src 'self'",
            "base-uri 'self'",
            "object-src 'none'",
            "frame-ancestors 'none'",
            "form-action 'self' https://checkout.stripe.com",
            "frame-src https://checkout.stripe.com",
            "script-src 'self' 'unsafe-inline' https://www.googletagmanager.com https://instant.page",
            "style-src 'self' 'unsafe-inline'",
            "img-src 'self' data: blob: https:",
            "font-src 'self' data:",
            "connect-src 'self' https://www.google-analytics.com https://*.google-analytics.com",
        ];

        return implode('; ', $directives);
    }
}
