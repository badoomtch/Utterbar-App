<?php

namespace Tests\Feature;

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
}
