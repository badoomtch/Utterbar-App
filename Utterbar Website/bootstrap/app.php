<?php

use Illuminate\Foundation\Application;
use Illuminate\Foundation\Configuration\Exceptions;
use Illuminate\Foundation\Configuration\Middleware;

return Application::configure(basePath: dirname(__DIR__))
    ->withRouting(
        web: __DIR__.'/../routes/web.php',
        commands: __DIR__.'/../routes/console.php',
        health: '/up',
    )
    ->withMiddleware(function (Middleware $middleware): void {
        // Stripe webhook verifies via Stripe-Signature HMAC; licence API is
        // called from the Tauri app (desktop, no session cookies) — both are
        // CSRF-exempt by design and authenticated via their own mechanisms.
        $middleware->validateCsrfTokens(except: [
            'api/stripe/webhook',
            'api/licence/*',
            'api/updates/*',
            'api/admin/*',
        ]);

        // Unauthenticated visitors to auth-protected routes land on /sign-in,
        // not Laravel's default /login.
        $middleware->redirectGuestsTo(fn () => route('sign-in.show'));

        // Baseline browser-hardening headers (XFO, nosniff, referrer-policy,
        // permissions-policy, Report-Only CSP) on every response. Appended
        // to the global stack so it covers /up + every web/api route.
        // CSP is Report-Only in this commit; HSTS is intentionally not
        // emitted yet. See SecurityHeaders.php for the why.
        $middleware->append(\App\Http\Middleware\SecurityHeaders::class);

        // Route middleware aliases.
        $middleware->alias([
            'admin'     => \App\Http\Middleware\AdminOnly::class,
            'admin.api' => \App\Http\Middleware\AdminApiToken::class,
        ]);
    })
    ->withExceptions(function (Exceptions $exceptions): void {
        //
    })->create();
