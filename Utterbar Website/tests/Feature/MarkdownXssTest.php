<?php

namespace Tests\Feature;

use App\Support\Markdown;
use Tests\TestCase;

/**
 * Markdown XSS protection. The output of `Markdown::renderEmbedded` is
 * consumed by Blade's `{!! !!}` raw-print on the changelog, download,
 * and blog pages — Blade does NOT auto-escape that output. Anything
 * CommonMark emits goes straight into the page. Pre-fix, CommonMark
 * passed raw HTML through verbatim and accepted `javascript:`-style
 * URLs in links, so a release-notes file with `<script>alert(1)</script>`
 * would have executed in the browser.
 *
 * The fix is `html_input => 'strip'` + `allow_unsafe_links => false`
 * passed to `Str::markdown` inside `renderEmbedded`. These tests pin
 * that behaviour absence-style: the dangerous tokens MUST NOT appear
 * in the rendered output.
 */
class MarkdownXssTest extends TestCase
{
    public function test_script_tag_is_stripped(): void
    {
        // `html_input=strip` removes the <script> wrapper. CommonMark's
        // strip mode keeps the inner *text* but the browser never
        // executes plain text — the asserted protection is the
        // absence of the executable wrapper, not the absence of
        // arbitrary substrings that happened to appear inside it.
        $out = Markdown::renderEmbedded('Hello <script>alert(1)</script> world');
        $this->assertStringNotContainsString('<script', $out, 'opening script tag must be absent');
        $this->assertStringNotContainsString('</script>', $out, 'closing script tag must be absent');
    }

    public function test_event_handler_attributes_are_stripped(): void
    {
        $out = Markdown::renderEmbedded(
            'Hover this: <a href="https://utterbar.com" onclick="alert(1)">link</a>'
        );
        // `html_input=strip` removes the entire raw-HTML node, so we
        // assert absence of both the on-handler and the JS payload.
        $this->assertStringNotContainsString('onclick', $out);
        $this->assertStringNotContainsString('alert(1)', $out);
    }

    public function test_img_onerror_payload_is_stripped(): void
    {
        $out = Markdown::renderEmbedded('<img src=x onerror=alert(1)>');
        $this->assertStringNotContainsString('onerror', $out);
        $this->assertStringNotContainsString('alert(1)', $out);
        // `html_input=strip` removes the entire <img> element since it
        // came in as raw HTML inline. Markdown image syntax (![alt](url))
        // is unaffected — see the legitimate-formatting test below.
        $this->assertStringNotContainsString('<img', $out);
    }

    public function test_javascript_url_in_markdown_link_is_neutralised(): void
    {
        // CommonMark *parses* the link, but `allow_unsafe_links=false`
        // makes it drop the href value. The link text still renders.
        $out = Markdown::renderEmbedded('[click](javascript:alert(1))');
        $this->assertStringNotContainsString('javascript:', $out);
        $this->assertStringNotContainsString('alert(1)', $out);
    }

    public function test_data_url_in_markdown_link_is_neutralised(): void
    {
        $out = Markdown::renderEmbedded('[exfil](data:text/html,<script>alert(1)</script>)');
        $this->assertStringNotContainsString('data:text/html', $out);
        $this->assertStringNotContainsString('<script', $out);
    }

    public function test_vbscript_url_is_neutralised(): void
    {
        $out = Markdown::renderEmbedded('[old](vbscript:msgbox(1))');
        $this->assertStringNotContainsString('vbscript:', $out);
    }

    public function test_iframe_is_stripped(): void
    {
        $out = Markdown::renderEmbedded('<iframe src="https://attacker.example"></iframe>');
        $this->assertStringNotContainsString('<iframe', $out);
        $this->assertStringNotContainsString('attacker.example', $out);
    }

    /**
     * Sanity: legitimate Markdown formatting still works. If the
     * sanitiser were over-aggressive and stripped headings, links,
     * lists, code, or emphasis, the changelog and blog would render
     * as plain text. Pin that the safe shapes survive.
     */
    public function test_safe_markdown_still_renders(): void
    {
        $out = Markdown::renderEmbedded(
            "# Top heading\n\n## Sub heading\n\n"
            . "Paragraph with **bold**, *italics*, and `code`.\n\n"
            . "[Utterbar home](https://utterbar.com)\n\n"
            . "- list item 1\n- list item 2\n\n"
            . "```\nfn main() {}\n```\n"
        );
        // h1 is demoted to h2 by renderEmbedded — pin both shifted
        // forms to surface a future regression in either direction.
        $this->assertStringContainsString('<h2>Top heading</h2>', $out);
        $this->assertStringContainsString('<h3>Sub heading</h3>', $out);
        $this->assertStringContainsString('<strong>bold</strong>', $out);
        $this->assertStringContainsString('<em>italics</em>', $out);
        $this->assertStringContainsString('<code>code</code>', $out);
        $this->assertStringContainsString('href="https://utterbar.com"', $out);
        $this->assertStringContainsString('<li>list item 1</li>', $out);
        $this->assertStringContainsString('<pre>', $out);
    }

    /**
     * Empty + null inputs. The early-return short-circuit should stay
     * in place so we don't pay the CommonMark setup cost for empty
     * markdown frontmatter / missing release notes.
     */
    public function test_empty_input_returns_empty_string(): void
    {
        $this->assertSame('', Markdown::renderEmbedded(''));
        $this->assertSame('', Markdown::renderEmbedded(null));
    }
}
