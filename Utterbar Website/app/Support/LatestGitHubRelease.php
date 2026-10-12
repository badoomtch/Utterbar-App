<?php

namespace App\Support;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Throwable;

final class LatestGitHubRelease
{
    /**
     * @return array{version: string, sizeLabel: ?string}
     */
    public function get(): array
    {
        $fallback = [
            'version' => (string) config('site.version'),
            'sizeLabel' => null,
        ];

        try {
            return Cache::remember(
                'github.latest-stable-release',
                now()->addHour(),
                fn (): array => $this->fetch() ?? $fallback,
            );
        } catch (Throwable $exception) {
            Log::warning('Latest GitHub release lookup failed', [
                'exception' => $exception::class,
            ]);

            return $fallback;
        }
    }

    /**
     * @return array{version: string, sizeLabel: string}|null
     */
    private function fetch(): ?array
    {
        $response = Http::acceptJson()
            ->withUserAgent('Utterbar-Website')
            ->timeout(5)
            ->get((string) config('site.github_latest_release_api_url'));

        if (! $response->successful()) {
            return null;
        }

        $tag = $response->json('tag_name');
        if (! is_string($tag) || preg_match('/^v?(\d+\.\d+\.\d+)$/', $tag, $matches) !== 1) {
            return null;
        }

        $version = $matches[1];
        $assetName = "Utterbar_{$version}_x64-setup.exe";
        $assets = $response->json('assets');

        if (! is_array($assets)) {
            return null;
        }

        $asset = collect($assets)->first(
            fn (mixed $candidate): bool => is_array($candidate)
                && ($candidate['name'] ?? null) === $assetName
                && is_int($candidate['size'] ?? null)
                && $candidate['size'] > 0,
        );

        if (! is_array($asset)) {
            return null;
        }

        return [
            'version' => $version,
            'sizeLabel' => number_format($asset['size'] / 1_048_576, 1).' MB',
        ];
    }
}
