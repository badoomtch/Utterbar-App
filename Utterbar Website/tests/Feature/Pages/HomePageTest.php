<?php

namespace Tests\Feature\Pages;

use Illuminate\Foundation\Testing\RefreshDatabase;
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
 * RefreshDatabase needed because the homepage queries the releases
 * table to populate the version chip in the pill mock.
 */
class HomePageTest extends TestCase
{
    use RefreshDatabase;

    public function test_home_renders(): void
    {
        $this->get(route('home'))->assertStatus(200);
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
}
