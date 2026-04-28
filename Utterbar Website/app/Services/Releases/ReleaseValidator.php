<?php

namespace App\Services\Releases;

use Illuminate\Support\Facades\Storage;

/**
 * Shared validation for publish-time release creation.
 *
 * Both `Admin\ReleaseController` (browser form) and
 * `Api\AdminReleaseApiController` (Bearer-token API) end up writing
 * the same row to `releases`, but pre-fix they ran subtly different
 * validation: the API path verified updater_signature when a key was
 * configured, the browser form did not, and neither path recomputed
 * the SHA-256 from R2 — both trusted whatever the caller pasted into
 * the `sha256` field.
 *
 * Both bugs are fixed here in one place so the two controllers can't
 * drift apart again. The behaviour:
 *
 *   1. The pasted `sha256` is treated as a *guard* against
 *      uploaded-the-wrong-file — we recompute the hash from the R2
 *      object, compare to the pasted value, and the canonical hash
 *      we persist is the one we computed. A typo in the pasted SHA
 *      surfaces as a clear validation error rather than a release
 *      row whose hash doesn't match what's in the bucket (which the
 *      desktop updater would refuse forever).
 *
 *   2. The updater signature is verified against the *computed* SHA
 *      (the one we just hashed), not the pasted SHA. If the operator
 *      signed a slightly different blob, we'd still catch the
 *      mismatch — closes the loophole where a wrong pasted SHA could
 *      drag the signature along with it and produce a row that
 *      signed-and-verified but didn't match the bucket.
 *
 *   3. The browser form is required to send an updater_signature on
 *      the same conditions as the API — when UPDATER_PUBLIC_KEY_B64
 *      is configured server-side, both paths must produce
 *      signed-row publishes only.
 */
class ReleaseValidator
{
    /**
     * Result of a publish-time validation pass.
     *
     * On success: ok=true, fileSize + canonicalSha + downloadUrl
     * are populated. Caller persists those.
     *
     * On failure: ok=false, error is a stable code (see constants
     * below) and message is human copy. Caller maps to a 422 with
     * the appropriate field tag so the UI can highlight it.
     */
    public bool $ok;
    public ?string $error;
    public ?string $message;
    public ?int $fileSize;
    public ?string $canonicalSha;
    public ?string $downloadUrl;
    public ?string $errorField;

    public const ERR_FILE_NOT_IN_BUCKET     = 'file_not_in_bucket';
    public const ERR_SHA_MISMATCH           = 'sha_mismatch';
    public const ERR_R2_HASH_FAILED         = 'r2_hash_failed';
    public const ERR_UPDATER_SIG_REQUIRED   = 'updater_signature_required';
    public const ERR_UPDATER_SIG_INVALID    = 'updater_signature_invalid';
    public const ERR_R2_KEY_SHAPE           = 'r2_key_shape';

    private function __construct(
        bool $ok,
        ?string $error = null,
        ?string $message = null,
        ?string $errorField = null,
        ?int $fileSize = null,
        ?string $canonicalSha = null,
        ?string $downloadUrl = null,
    ) {
        $this->ok = $ok;
        $this->error = $error;
        $this->message = $message;
        $this->errorField = $errorField;
        $this->fileSize = $fileSize;
        $this->canonicalSha = $canonicalSha;
        $this->downloadUrl = $downloadUrl;
    }

    public static function ok(int $fileSize, string $canonicalSha, string $downloadUrl): self
    {
        return new self(
            ok: true,
            fileSize: $fileSize,
            canonicalSha: $canonicalSha,
            downloadUrl: $downloadUrl,
        );
    }

    public static function fail(string $error, string $message, ?string $errorField = null): self
    {
        return new self(
            ok: false,
            error: $error,
            message: $message,
            errorField: $errorField,
        );
    }

    /**
     * Validate a pending publish.
     *
     * @param  array{version:string,channel:string,r2_key:string,sha256:string,updater_signature?:string|null}  $input
     */
    public static function validate(array $input): self
    {
        $version    = (string) $input['version'];
        $channel    = (string) $input['channel'];
        $r2Key      = ltrim((string) $input['r2_key'], '/');
        $pastedSha  = strtolower((string) $input['sha256']);
        $updaterSig = $input['updater_signature'] ?? null;

        // Shape-check the r2_key BEFORE any R2 round-trip. Without
        // this, a release row could point at any pre-existing object
        // in the bucket — even one outside `releases/<v>/`. The
        // operator could legitimately have authority to upload, but
        // pinning the row to a specific bucket layout (and to the
        // submitted version) means the publish flow can't mint a
        // release that references e.g. an old asset, a debug build
        // hidden under another prefix, or an object the operator
        // mis-typed. Shape rules:
        //   - Path is literally `releases/<X.Y.Z>/<filename>` —
        //     no nested directories beyond the version folder.
        //   - The version folder must match the submitted `version`.
        //   - Filename: ASCII alphanumerics, `_`, `-`, `.`. No `..`,
        //     no `/`, no `\`, no query string, no control chars.
        //   - Filename must end `.exe` (Windows installer is the
        //     only release artifact today; widening this requires
        //     an explicit allow-list extension here).
        $shapeError = self::r2KeyShapeError($r2Key, $version);
        if ($shapeError !== null) {
            return self::fail(self::ERR_R2_KEY_SHAPE, $shapeError, 'r2_key');
        }

        $disk = Storage::disk('r2');

        if (!$disk->exists($r2Key)) {
            return self::fail(
                self::ERR_FILE_NOT_IN_BUCKET,
                "No file at R2 key: {$r2Key}. Upload it first.",
                'r2_key',
            );
        }

        $size = (int) $disk->size($r2Key);

        // Recompute the SHA from R2. Stream-read so we don't hold a
        // ~10 MB installer in memory; sha256 hashing is incremental.
        $stream = $disk->readStream($r2Key);
        if (!is_resource($stream)) {
            return self::fail(
                self::ERR_R2_HASH_FAILED,
                "Couldn't read {$r2Key} from R2 to verify its SHA-256. Try again.",
                'r2_key',
            );
        }
        $ctx = hash_init('sha256');
        while (!feof($stream)) {
            $chunk = fread($stream, 1 << 16);
            if ($chunk === false) {
                fclose($stream);
                return self::fail(
                    self::ERR_R2_HASH_FAILED,
                    "Read error while computing SHA-256 of {$r2Key}.",
                    'r2_key',
                );
            }
            hash_update($ctx, $chunk);
        }
        fclose($stream);
        $canonicalSha = hash_final($ctx);

        // Pasted SHA is a guard, not the canonical value. If they
        // disagree, the operator either uploaded the wrong file or
        // pasted the wrong hash — either way, refuse.
        if (!hash_equals($canonicalSha, $pastedSha)) {
            return self::fail(
                self::ERR_SHA_MISMATCH,
                "Pasted SHA-256 does not match the file in R2. "
                . "Pasted: {$pastedSha}; computed: {$canonicalSha}. "
                . "Either re-upload the matching installer or correct the SHA.",
                'sha256',
            );
        }

        // Build the canonical download URL the same way
        // `UpdaterApiController` does (single source of truth:
        // `r2.url` config). This is the URL the desktop client
        // verifies the signature against.
        $cdnBase = (string) (
            config('filesystems.disks.r2.url')
            ?: env('CLOUDFLARE_R2_PUBLIC_URL', 'https://dl.utterbar.com')
        );
        $downloadUrl = rtrim($cdnBase, '/') . '/' . ltrim($r2Key, '/');

        // Updater signature gating: required when the public key is
        // configured (production); accepted-but-not-required when no
        // key is configured (local dev / CI).
        $hasUpdaterKey = (bool) config('services.updater.public_key_b64');
        if ($hasUpdaterKey) {
            if (!is_string($updaterSig) || $updaterSig === '') {
                return self::fail(
                    self::ERR_UPDATER_SIG_REQUIRED,
                    'Releases must include an updater_signature when '
                    . 'UPDATER_PUBLIC_KEY_B64 is set. Use scripts/sign-update-metadata.sh.',
                    'updater_signature',
                );
            }
            $signedOk = UpdateMetadataSigner::verify($updaterSig, [
                'version'      => $version,
                'channel'      => $channel,
                'size_bytes'   => $size,
                // Verify against the *computed* SHA, never the pasted
                // one — closes the loophole where a wrong pasted SHA
                // could drag the signature along with it.
                'sha256'       => $canonicalSha,
                'download_url' => $downloadUrl,
            ]);
            if (!$signedOk) {
                return self::fail(
                    self::ERR_UPDATER_SIG_INVALID,
                    'updater_signature does not verify against the canonical metadata blob '
                    . '(sha verified against the value computed from R2). '
                    . 'Check that the operator key matches UPDATER_PUBLIC_KEY_B64 and that '
                    . 'nothing changed (sha256, r2_key, size) between signing and upload.',
                    'updater_signature',
                );
            }
        }

        return self::ok($size, $canonicalSha, $downloadUrl);
    }

    /**
     * Pure-string shape check for the `r2_key` field. Returns a
     * human-readable error message when the key doesn't match the
     * expected `releases/<version>/<filename>.exe` shape, or null
     * when the shape is good.
     *
     * Public so tests (and any future direct caller) can exercise
     * the rule without standing up a fake R2 disk. The actual
     * publish flow always goes through `validate()` which calls
     * this internally as the first gate.
     *
     * Shape rules (see `validate()` for rationale):
     *   - exactly three path segments separated by `/`: literal
     *     "releases", a version folder, and a filename;
     *   - version folder MUST equal the submitted `$version`;
     *   - filename MUST be `^[A-Za-z0-9._-]+\.exe$` (no slashes,
     *     no `..`, no query string, no control chars).
     */
    public static function r2KeyShapeError(string $r2Key, string $version): ?string
    {
        // Refuse anything containing a parent-dir traversal or
        // backslashes outright. `r2_key` is normalised to forward-
        // slashes by the time it reaches us, but defence-in-depth.
        if (str_contains($r2Key, '..') || str_contains($r2Key, '\\')) {
            return "r2_key must not contain '..' or backslashes: {$r2Key}";
        }
        // Refuse query strings and control chars.
        if (preg_match('/[\?\#\x00-\x1f]/', $r2Key)) {
            return "r2_key must not contain query strings or control characters: {$r2Key}";
        }

        $parts = explode('/', $r2Key);
        if (count($parts) !== 3) {
            return "r2_key must be exactly 'releases/<version>/<filename>': {$r2Key}";
        }
        [$prefix, $folder, $filename] = $parts;

        if ($prefix !== 'releases') {
            return "r2_key must start with 'releases/': {$r2Key}";
        }
        if ($folder !== $version) {
            return "r2_key version folder ({$folder}) must match submitted version ({$version}): {$r2Key}";
        }
        // Filename: ASCII alphanumerics + `_`, `-`, `.`. Must end `.exe`.
        if (!preg_match('/^[A-Za-z0-9._-]+\.exe$/', $filename)) {
            return "r2_key filename must be alphanumerics + ._- ending in .exe (got: {$filename})";
        }

        return null;
    }
}
