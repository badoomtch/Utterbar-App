<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Symfony\Component\HttpFoundation\Response;

/**
 * Bearer-token auth for admin API endpoints.
 *
 * Compares the `Authorization: Bearer <token>` header against
 * ADMIN_API_TOKEN from .env. Use this for machine-to-machine admin ops
 * (e.g. the release-push script on the app builder's laptop) where a
 * session cookie isn't available.
 *
 * Timing-safe comparison via hash_equals.
 *
 * Audit logging (added in v0.9.0 hardening pass): every request that
 * reaches this middleware writes a row to `admin_api_log` recording the
 * token-prefix, outcome (ok / reject), method, path, IP, and response
 * status. The token's first 8 characters are used as a non-PII
 * identifier for rotation tracking; the full token is never stored.
 * Body is NOT recorded — request line is enough for triage and avoids
 * leaking release SHA-256s / R2 keys into a separate persistence
 * surface.
 */
class AdminApiToken
{
    public function handle(Request $request, Closure $next): Response
    {
        $expected = (string) env('ADMIN_API_TOKEN', '');
        $header = (string) $request->header('Authorization', '');
        $presented = str_starts_with($header, 'Bearer ') ? substr($header, 7) : '';
        $tokenPrefix = $presented === '' ? '(none)' : substr($presented, 0, 8);

        if ($expected === '') {
            // Env var unset — refuse all requests rather than auth-bypass.
            $this->log($request, $tokenPrefix, 'reject', 503);
            return response()->json([
                'error' => 'admin_api_disabled',
                'message' => 'Admin API is not configured on this server.',
            ], 503);
        }

        if ($presented === '' || !hash_equals($expected, $presented)) {
            $this->log($request, $tokenPrefix, 'reject', 401);
            return $this->unauth();
        }

        $response = $next($request);
        $this->log($request, $tokenPrefix, 'ok', $response->getStatusCode());
        return $response;
    }

    private function unauth(): Response
    {
        return response()->json([
            'error' => 'unauthorized',
            'message' => 'Invalid or missing admin API token.',
        ], 401);
    }

    /**
     * Persist an audit row. Best-effort: a DB write failure must NEVER
     * block the auth flow itself, so we wrap in try/catch and log to
     * the file logger as a fallback. The migration is the source of
     * truth for the schema.
     */
    private function log(Request $request, string $tokenPrefix, string $outcome, int $statusCode): void
    {
        try {
            DB::table('admin_api_log')->insert([
                'token_prefix' => $tokenPrefix,
                'outcome'      => $outcome,
                'method'       => substr($request->method(), 0, 8),
                'path'         => substr($request->path(), 0, 191),
                'ip'           => substr((string) $request->ip(), 0, 45),
                'user_agent'   => substr((string) $request->userAgent(), 0, 191),
                'status_code'  => $statusCode,
                'created_at'   => now(),
                'updated_at'   => now(),
            ]);
        } catch (\Throwable $e) {
            Log::warning('admin_api_log write failed', [
                'error'   => $e->getMessage(),
                'outcome' => $outcome,
                'path'    => $request->path(),
            ]);
        }
    }
}
