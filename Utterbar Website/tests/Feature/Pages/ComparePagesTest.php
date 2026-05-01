<?php

namespace Tests\Feature\Pages;

use Tests\TestCase;

/**
 * Smoke + content tests for the /compare pages. Each route must
 * return 200, the comparison table must render, and the page must
 * carry the canonical £39.99 GBP / Utterbar one-time pricing
 * mention so a future template refactor can't quietly drop the
 * commercial anchor.
 *
 * Each new /compare/<competitor> page extends the same shape:
 *   * route returns 200
 *   * page renders the competitor's name in the headline
 *   * page renders the £39.99 anchor
 *   * page renders at least one row of the <x-compare-table>
 *
 * The table component is the load-bearing primitive — if it ever
 * breaks, every comparison page goes silent at the same time.
 * That regression would cost real conversion, so pin it here.
 */
class ComparePagesTest extends TestCase
{
    public function test_compare_index_renders(): void
    {
        $this->get(route('compare.index'))->assertStatus(200);
    }

    public function test_compare_wispr_flow_renders(): void
    {
        $response = $this->get(route('compare.wispr-flow'));
        $response->assertStatus(200);
        $response->assertSee('Wispr Flow', false);
    }

    public function test_compare_wispr_flow_shows_utterbar_pricing_anchor(): void
    {
        $response = $this->get(route('compare.wispr-flow'));
        $response->assertStatus(200);
        $response->assertSee('£39.99', false);
    }

    public function test_compare_wispr_flow_uses_extracted_compare_table(): void
    {
        // The table refactor moves the inline 13-row block into
        // <x-compare-table>. The component renders the competitor
        // name in a footer label under each competitor cell, so a
        // working refactor must surface "Wispr Flow" at least
        // twice on the page (the column footers + at least one
        // body mention). Pricing-test pattern: assert the rendered
        // shape, not the implementation detail.
        $response = $this->get(route('compare.wispr-flow'));
        $count = substr_count($response->getContent(), 'Wispr Flow');
        $this->assertGreaterThan(2, $count, 'Wispr Flow must appear in headline + table column labels.');
    }

    public function test_compare_dragon_renders(): void
    {
        $response = $this->get(route('compare.dragon'));
        $response->assertStatus(200);
        $response->assertSee('Dragon', false);
    }

    public function test_compare_dragon_shows_utterbar_pricing_anchor(): void
    {
        $response = $this->get(route('compare.dragon'));
        $response->assertStatus(200);
        $response->assertSee('£39.99', false);
    }

    public function test_compare_dragon_uses_extracted_compare_table(): void
    {
        $response = $this->get(route('compare.dragon'));
        $count = substr_count($response->getContent(), 'Dragon');
        $this->assertGreaterThan(2, $count, 'Dragon must appear in headline + table column labels.');
    }

    public function test_compare_voice_access_renders(): void
    {
        $response = $this->get(route('compare.voice-access'));
        $response->assertStatus(200);
        $response->assertSee('Voice Access', false);
    }

    public function test_compare_voice_access_shows_utterbar_pricing_anchor(): void
    {
        $response = $this->get(route('compare.voice-access'));
        $response->assertStatus(200);
        $response->assertSee('£39.99', false);
    }

    public function test_compare_voice_access_does_not_overclaim_voice_access_lacks_vocabulary(): void
    {
        // Microsoft does not document a Dragon-style persistent
        // personal dictionary in Voice Access; we said so in the
        // comparison row but soft-phrased it ("not documented in
        // the Microsoft docs we reviewed"). Pin the soft phrasing
        // so a future content edit can't quietly harden the claim
        // into "Voice Access has no custom vocabulary" without
        // re-checking Microsoft's docs.
        $response = $this->get(route('compare.voice-access'));
        $response->assertSee('not documented', false);
    }

    public function test_compare_superwhisper_renders(): void
    {
        $response = $this->get(route('compare.superwhisper'));
        $response->assertStatus(200);
        $response->assertSee('Superwhisper', false);
    }

    public function test_compare_superwhisper_shows_utterbar_pricing_anchor(): void
    {
        $response = $this->get(route('compare.superwhisper'));
        $response->assertStatus(200);
        $response->assertSee('£39.99', false);
    }

    public function test_compare_superwhisper_does_not_describe_competitor_as_mac_only(): void
    {
        // Superwhisper is NOT Mac-only — they ship on Mac, Windows,
        // and iOS. A previous version of the website's
        // alternatives table called Superwhisper "Mac-first
        // heritage; Windows is newer", which under-described
        // their cross-platform reach. Pin the corrected
        // multi-platform framing so a future edit can't quietly
        // re-introduce the Mac-only mistake.
        $response = $this->get(route('compare.superwhisper'));
        $response->assertSee('Mac, Windows', false);
    }
}
