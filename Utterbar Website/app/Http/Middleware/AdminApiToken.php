<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

/**
 * Bearer-token auth for admin API endpoints.
 *
 * Compares the `Authorization: Bearer <token>` header against ADMIN_API_TOKEN
 * from .env. Use this for machine-to-machine admin ops (e.g. the release-push
 * script on the app builder's laptop) where a session cookie isn't available.
 *
 * Timing-safe comparison via hash_equals.
 */
class AdminApiToken
{
    public function handle(Request $request, Closure $next): Response
    {
        $expected = (string) env('ADMIN_API_TOKEN', '');
        if ($expected === '') {
            // Env var unset — refuse all requests rather than auth-bypass.
            return response()->json([
                'error' => 'admin_api_disabled',
                'message' => 'Admin API is not configured on this server.',
            ], 503);
        }

        $header = (string) $request->header('Authorization', '');
        if (!str_starts_with($header, 'Bearer ')) {
            return $this->unauth();
        }

        $presented = substr($header, 7);
        if (!hash_equals($expected, $presented)) {
            return $this->unauth();
        }

        return $next($request);
    }

    private function unauth(): Response
    {
        return response()->json([
            'error' => 'unauthorized',
            'message' => 'Invalid or missing admin API token.',
        ], 401);
    }
}
