<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

/**
 * Gates access to admin-only routes via an email allow-list.
 *
 * Allow-list source: ADMIN_EMAILS env var, comma-separated. Case-insensitive.
 * Set in .env, never committed. Failing this check returns 403.
 *
 * Requires the request to already be authenticated (chain after 'auth').
 */
class AdminOnly
{
    public function handle(Request $request, Closure $next): Response
    {
        $user = $request->user();
        if (!$user) {
            abort(403);
        }

        $allow = array_filter(array_map(
            fn ($e) => strtolower(trim($e)),
            explode(',', (string) env('ADMIN_EMAILS', ''))
        ));

        if (empty($allow) || !in_array(strtolower($user->email), $allow, true)) {
            abort(403);
        }

        return $next($request);
    }
}
