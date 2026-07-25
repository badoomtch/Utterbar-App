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
        // The headless admin release API is called by the publish script
        // (no session cookies) and authenticated by its own bearer token,
        // so it is CSRF-exempt by design.
        $middleware->validateCsrfTokens(except: [
            'api/admin/*',
        ]);

        // Baseline browser-hardening headers (XFO, nosniff, referrer-policy,
        // permissions-policy, Report-Only CSP) on every response. Appended
        // to the global stack so it covers /up + every web/api route.
        // CSP is Report-Only in this commit; HSTS is intentionally not
        // emitted yet. See SecurityHeaders.php for the why.
        $middleware->append(\App\Http\Middleware\SecurityHeaders::class);

        // Route middleware aliases.
        $middleware->alias([
            'admin.api' => \App\Http\Middleware\AdminApiToken::class,
        ]);
    })
    ->withExceptions(function (Exceptions $exceptions): void {
        //
    })->create();
