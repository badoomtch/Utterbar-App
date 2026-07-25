<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Models\Release;
use App\Services\Releases\ReleaseValidator;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Storage;

/**
 * Headless release publishing for the Tauri app's release script. Auth is
 * a bearer token (admin.api middleware). This is the only privileged
 * publish surface: the app is free and open source, but only releases
 * signed with the operator's updater key are served.
 *
 * Workflow from the builder side:
 *   1. Upload Utterbar-Setup.exe to R2 at releases/{version}/Utterbar-Setup.exe
 *   2. Compute SHA-256 locally
 *   3. POST /api/admin/releases with a JSON body (schema below)
 *
 * See RELEASE.md for the builder-side guide.
 */
class AdminReleaseApiController extends Controller
{
    /**
     * POST /api/admin/releases
     */
    public function store(Request $request): JsonResponse
    {
        // Version: strict three-segment semver (X.Y.Z). Prerelease suffixes
        // are rejected because `/api/updater/check` cannot compare them —
        // use the beta channel for pre-GA builds. Admin form + this API
        // intentionally share the same regex so publishing rules stay
        // consistent across both entry points.
        //
        // SHA-256: REQUIRED. The desktop updater refuses to install any
        // release without a hash (see updater.rs install_update), so a
        // release row with null sha256 cannot be auto-installed. We
        // reject at publish rather than silently persist a dead row.
        $data = $request->validate([
            'version'            => ['required', 'string', 'max:32', 'regex:/^\d+\.\d+\.\d+$/'],
            'channel'            => ['nullable', 'in:stable,beta'],
            'r2_key'             => ['nullable', 'string', 'max:255'],
            'tag'                => ['nullable', 'string', 'max:64'],
            'sha256'             => ['required', 'string', 'regex:/^[A-Fa-f0-9]{64}$/'],
            'notes_markdown'     => ['required', 'string'],
            // Base64-encoded Ed25519 signature (64 raw bytes = 88 chars b64) over
            // the canonical metadata blob. Required when an updater public key
            // is configured — the publish script signs locally and sends the
            // signature here so the server never needs the private key.
            'updater_signature'  => ['nullable', 'string', 'max:128'],
        ], [
            'version.regex' => 'Version must be three numeric segments, e.g. 0.6.2. Pre-release suffixes (-rc.1, -beta) are not supported; use the beta channel for pre-GA builds.',
            'sha256.required' => 'SHA-256 is required. Releases without a hash cannot be auto-installed by the desktop updater.',
            'sha256.regex'  => 'SHA-256 must be 64 hex chars.',
        ]);

        $channel = $data['channel'] ?? 'stable';

        // Default r2_key convention: releases/{version}/Utterbar-Setup-{version}.exe
        // (the versioned filename is canon so a user dragging the installer
        // anywhere can still tell which build it is). publish-release.sh
        // always sends r2_key explicitly — this default is only hit by
        // manual POSTs that omitted it.
        $r2Key = $data['r2_key'] ?? "releases/{$data['version']}/Utterbar-Setup-{$data['version']}.exe";
        $r2Key = ltrim($r2Key, '/');

        if (Release::where('version', $data['version'])->where('channel', $channel)->exists()) {
            return response()->json([
                'error'   => 'version_exists',
                'message' => "A {$channel} release with version {$data['version']} already exists.",
            ], 409);
        }

        $result = ReleaseValidator::validate([
            'version'           => $data['version'],
            'channel'           => $channel,
            'r2_key'            => $r2Key,
            'sha256'            => $data['sha256'],
            'updater_signature' => $data['updater_signature'] ?? null,
        ]);
        if (!$result->ok) {
            return response()->json([
                'error'   => $result->error,
                'message' => $result->message,
                'meta'    => $result->errorField === 'r2_key' ? ['r2_key' => $r2Key] : null,
            ], 422);
        }

        $release = Release::create([
            'version'           => $data['version'],
            'channel'           => $channel,
            'r2_key'            => $r2Key,
            'file_size_bytes'   => $result->fileSize,
            // Persist the SHA we *computed* from R2, not what the
            // operator pasted. Pre-fix, a wrong pasted SHA (typo,
            // wrong file uploaded) created a row whose hash didn't
            // match the bucket — which the desktop updater then
            // refused forever. Recomputing makes the row trustworthy
            // by construction.
            'sha256'            => $result->canonicalSha,
            'updater_signature' => $data['updater_signature'] ?? null,
            'tag'               => $data['tag'] ?? null,
            'notes_markdown'    => $data['notes_markdown'],
            'published_at'      => now(),
        ]);
        Log::info('release published via API', [
            'id' => $release->id, 'version' => $release->version, 'channel' => $release->channel,
        ]);

        return response()->json([
            'ok'             => true,
            'id'             => $release->id,
            'version'        => $release->version,
            'channel'        => $release->channel,
            'download_url'   => $release->downloadUrl(),
            'size_bytes'     => $release->file_size_bytes,
        ], 201);
    }

    /**
     * DELETE /api/admin/releases/{id}?delete_file=1
     */
    public function destroy(Request $request, int $id): JsonResponse
    {
        $release = Release::find($id);
        if (!$release) {
            return response()->json(['error' => 'not_found', 'message' => 'No release with that id.'], 404);
        }

        $deleteFile = $request->boolean('delete_file');
        if ($deleteFile) {
            try {
                Storage::disk('r2')->delete($release->r2_key);
            } catch (\Throwable $e) {
                Log::warning('R2 delete failed during release destroy', ['key' => $release->r2_key, 'err' => $e->getMessage()]);
            }
        }

        $release->delete();

        return response()->json([
            'ok' => true,
            'deleted_file' => $deleteFile,
            'version' => $release->version,
        ]);
    }
}
