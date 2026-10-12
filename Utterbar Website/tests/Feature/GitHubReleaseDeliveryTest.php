<?php

namespace Tests\Feature;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;
use Tests\TestCase;

class GitHubReleaseDeliveryTest extends TestCase
{
    public function test_download_redirects_to_github_latest_release(): void
    {
        $this->get('/download')
            ->assertRedirect(config('site.github_latest_release_url'));
    }

    public function test_changelog_redirects_to_github_releases(): void
    {
        $this->get('/changelog')
            ->assertRedirect(config('site.github_releases_url'));
    }

    public function test_self_hosted_updater_endpoint_is_removed(): void
    {
        $this->get('/api/updater/check')->assertNotFound();
    }

    public function test_homepage_falls_back_to_the_configured_version_when_github_is_unavailable(): void
    {
        Cache::forget('github.latest-stable-release');
        Http::fake([
            config('site.github_latest_release_api_url') => Http::response([], 503),
        ]);

        $this->get('/')
            ->assertOk()
            ->assertSee('Currently: v'.config('site.version'), false);
    }

    public function test_homepage_rejects_a_release_without_the_expected_installer(): void
    {
        Cache::forget('github.latest-stable-release');
        Http::fake([
            config('site.github_latest_release_api_url') => Http::response([
                'tag_name' => 'v9.9.9',
                'assets' => [
                    ['name' => 'source-code.zip', 'size' => 100],
                ],
            ]),
        ]);

        $this->get('/')
            ->assertOk()
            ->assertSee('Currently: v'.config('site.version'), false)
            ->assertDontSee('Currently: v9.9.9', false);
    }
}
