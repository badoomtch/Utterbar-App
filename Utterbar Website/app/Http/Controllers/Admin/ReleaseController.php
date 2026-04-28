<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Mail\UpdateAnnouncement;
use App\Models\Release;
use App\Models\UpdateSubscriber;
use App\Services\Releases\ReleaseValidator;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Mail;
use Illuminate\Support\Facades\Storage;

class ReleaseController extends Controller
{
    public function index(Request $request)
    {
        $releases = Release::orderByDesc('published_at')->get();
        return view('pages.admin.releases.index', compact('releases'));
    }

    public function create(Request $request)
    {
        return view('pages.admin.releases.create', [
            'default' => [
                'version'       => '',
                'channel'       => 'stable',
                'r2_key'        => '',
                'tag'           => '',
                'sha256'        => '',
                'notes'         => '',
                'announce'      => false,
            ],
        ]);
    }

    public function store(Request $request): RedirectResponse
    {
        // Version: strict three-segment semver (X.Y.Z). Prereleases like
        // `0.7.0-rc.1` are rejected deliberately — the beta channel is the
        // supported mechanism for pre-GA builds, and `/api/updater/check`
        // parses strict X.Y.Z only. Allowing prereleases here while the
        // updater can't compare them was the 0.6.x mismatch.
        //
        // SHA-256: REQUIRED for every published release. The desktop
        // updater won't install without a hash (it verifies the download
        // before spawning the installer), so a release row missing its
        // hash is a release that can't actually be auto-installed — we
        // refuse to create one rather than leave a trap for users.
        //
        // updater_signature: same gating as the API path — required when
        // UPDATER_PUBLIC_KEY_B64 is configured. Pre-fix the browser form
        // skipped this check entirely, which let an admin mint signed-key
        // releases that the desktop updater would refuse forever (the
        // updater filters out unsigned rows on a signed-key environment).
        $data = $request->validate([
            'version' => ['required', 'string', 'max:32', 'regex:/^\d+\.\d+\.\d+$/'],
            'channel' => ['required', 'in:stable,beta'],
            'r2_key'  => ['required', 'string', 'max:255'],
            'tag'     => ['nullable', 'string', 'max:64'],
            'sha256'  => ['required', 'string', 'regex:/^[A-Fa-f0-9]{64}$/'],
            'notes'   => ['required', 'string'],
            'announce' => ['nullable', 'boolean'],
            'announce_subject' => ['nullable', 'string', 'max:200'],
            'confirm_stable_announce' => ['nullable', 'boolean'],
            'updater_signature' => ['nullable', 'string', 'max:128'],
        ], [
            'version.regex' => 'Version must be three numeric segments, e.g. 0.6.2. Pre-release suffixes (-rc.1, -beta) are not supported; use the beta channel for pre-GA builds.',
            'sha256.required' => 'SHA-256 is required — releases without a hash cannot be auto-installed by the desktop updater.',
            'sha256.regex'  => 'SHA-256 must be 64 hex chars.',
        ]);

        // Uniqueness per (version, channel) — catch before R2 lookup so we don't
        // waste the round-trip.
        if (Release::where('version', $data['version'])->where('channel', $data['channel'])->exists()) {
            return back()->withInput()->withErrors([
                'version' => "A {$data['channel']} release with version {$data['version']} already exists.",
            ]);
        }

        // Stable + announce destructive-flag confirmation. The browser
        // form must require the operator to tick a "yes, I really want
        // to email every active subscriber" box before we run the
        // SHA-recompute and persist. Refused before the R2 round-trip.
        if ($data['channel'] === 'stable' && $request->boolean('announce')
            && !$request->boolean('confirm_stable_announce')
        ) {
            return back()->withInput()->withErrors([
                'confirm_stable_announce' =>
                    'Stable + announce sends email to every active subscriber. '
                    . 'Tick the confirmation box before publishing.',
            ]);
        }

        $r2Key = ltrim($data['r2_key'], '/');

        // Shared validator: existence check, server-recompute SHA from
        // R2, compare to pasted SHA, verify updater_signature against
        // the *computed* SHA (when keyed). The canonical SHA we
        // persist is the one we computed — never the pasted one.
        $result = ReleaseValidator::validate([
            'version'           => $data['version'],
            'channel'           => $data['channel'],
            'r2_key'            => $r2Key,
            'sha256'            => $data['sha256'],
            'updater_signature' => $data['updater_signature'] ?? null,
        ]);
        if (!$result->ok) {
            return back()->withInput()->withErrors([
                $result->errorField ?? 'r2_key' => $result->message,
            ]);
        }

        $release = Release::create([
            'version'           => $data['version'],
            'channel'           => $data['channel'],
            'r2_key'            => $r2Key,
            'file_size_bytes'   => $result->fileSize,
            // Persist the computed SHA, not the pasted one — see
            // ReleaseValidator for the rationale.
            'sha256'            => $result->canonicalSha,
            'updater_signature' => $data['updater_signature'] ?? null,
            // `tag` is nullable in the validator so it may be absent from
            // $data entirely when the form leaves the field blank — use
            // coalesce rather than indexing.
            'tag'               => ($data['tag'] ?? '') !== '' ? $data['tag'] : null,
            'notes_markdown'    => $data['notes'],
            'published_at'      => now(),
        ]);

        $flash = ['type' => 'ok', 'text' => "Release v{$release->version} ({$release->channel}) published."];

        if ($request->boolean('announce') && $release->channel === 'stable') {
            $subject = trim((string) $request->input('announce_subject'))
                ?: "Utterbar {$release->version} is out";
            $sent = $this->blastAnnouncement($release, $subject);
            $release->announced_at = now();
            $release->save();
            $flash['text'] .= " Announced to {$sent} subscriber(s).";
        }

        return redirect()->route('admin.releases.index')->with('flash', $flash);
    }

    public function destroy(Request $request, int $id): RedirectResponse
    {
        $release = Release::findOrFail($id);

        $deleteFile = $request->boolean('delete_file');
        if ($deleteFile) {
            try {
                Storage::disk('r2')->delete($release->r2_key);
            } catch (\Throwable $e) {
                Log::warning('Failed to delete R2 object', [
                    'key' => $release->r2_key, 'err' => $e->getMessage(),
                ]);
            }
        }

        $release->delete();

        return redirect()->route('admin.releases.index')->with('flash', [
            'type' => 'ok',
            'text' => "Release deleted" . ($deleteFile ? ' (also removed from R2).' : '.'),
        ]);
    }

    /**
     * Send the release announcement to every active subscriber.
     */
    private function blastAnnouncement(Release $release, string $subject): int
    {
        $subs = UpdateSubscriber::active()->get();
        $sent = 0;
        foreach ($subs as $sub) {
            // Staged rotation: in-memory new token primes the Mailable URL;
            // DB only advances if the send succeeds. See
            // UpdateSubscriber::prepareRotation() for rationale.
            $sub->prepareRotation();
            try {
                Mail::to($sub->email)->send(new UpdateAnnouncement(
                    subscriber:      $sub,
                    release:         $release->version,
                    announceSubject: $subject,
                    notesMd:         $release->notes_markdown,
                ));
                $sub->commitRotation();
                $sent++;
            } catch (\Throwable $e) {
                $sub->discardRotation();
                Log::warning('Release announcement email failed', [
                    'email' => $sub->email, 'err' => $e->getMessage(),
                ]);
            }
        }
        return $sent;
    }
}
