<?php

namespace Tests\Feature\Pages;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;
use Tests\TestCase;

/**
 * Smoke + content tests for the homepage hero. Specifically pins the
 * post-2026-05-01 hero pill alignment with the desktop bar:
 *   * Mode chip cluster is rendered (Voice / Message / Meeting).
 *   * State chip uses real product phase language ("Recording",
 *     "Inserted") instead of just a version label.
 *
 * These were not previously asserted; the hero pill is the most
 * visible product surface on the marketing site, so the cost of a
 * silent regression here is high.
 *
 * GitHub's latest ordinary release supplies the version displayed
 * in the hero. The API is faked here so these tests stay deterministic.
 */
class HomePageTest extends TestCase
{
    protected function setUp(): void
    {
        parent::setUp();

        Cache::forget('github.latest-stable-release');
        Http::fake([
            config('site.github_latest_release_api_url') => Http::response([
                'tag_name' => 'v0.26.0',
                'assets' => [
                    [
                        'name' => 'Utterbar_0.26.0_x64-setup.exe',
                        'size' => 20_380_803,
                    ],
                ],
            ]),
        ]);
    }

    public function test_home_renders(): void
    {
        $this->get(route('home'))->assertStatus(200);
    }

    public function test_home_displays_the_latest_github_release_and_source_link(): void
    {
        $response = $this->get(route('home'));

        $response->assertOk();
        $response->assertSee('Currently: v0.26.0', false);
        $response->assertSee('19.4 MB', false);
        $response->assertSee(config('site.github_repository_url'), false);
    }

    public function test_home_hero_renders_mode_chips_for_voice_message_meeting(): void
    {
        // The hero pill mock now mirrors the desktop app's
        // BarModeSwitch with three icon-only chips. Modes are
        // populated from an Alpine `modes` array; their accessible
        // labels render as `aria-label` strings on the buttons.
        $response = $this->get(route('home'));
        $response->assertStatus(200);
        $response->assertSee('Voice mode', false);
        $response->assertSee('Message mode', false);
        $response->assertSee('Meeting mode', false);
    }

    public function test_home_hero_uses_real_product_phase_language(): void
    {
        // The Alpine state-chip label cycles through real desktop
        // phase strings: "Recording" while the demo runs, "Inserted"
        // briefly after it ends. Pin the literal strings so a future
        // edit can't quietly soften them back to generic copy
        // ("Listening...", "Done") that contradicts the desktop bar
        // it's meant to mirror.
        $response = $this->get(route('home'));
        $response->assertSee('Recording', false);
        $response->assertSee('Inserted', false);
    }

    public function test_app_mock_uses_svg_safe_icon_visibility(): void
    {
        $source = file_get_contents(resource_path('views/components/app-mock.blade.php'));

        $this->assertIsString($source);
        $svgStart = strpos($source, '<svg class="icon"');
        $svgEnd = strpos($source, '</svg>', $svgStart);
        $this->assertIsInt($svgStart);
        $this->assertIsInt($svgEnd);

        $iconSvg = substr($source, $svgStart, $svgEnd - $svgStart);
        $this->assertStringNotContainsString('<template x-if=', $iconSvg);
        $this->assertSame(5, substr_count($iconSvg, '<g x-show='));
    }
}
