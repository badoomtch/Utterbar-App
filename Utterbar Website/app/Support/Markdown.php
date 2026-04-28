<?php

namespace App\Support;

use Illuminate\Support\Str;

/**
 * Markdown rendering helpers.
 */
class Markdown
{
    /**
     * Render Markdown for content that lives BELOW a page-level <h1>.
     *
     * Each release-notes file (e.g. `# Utterbar 0.7.5`) is authored as
     * a standalone document with a top-level H1. When that body is
     * embedded inside a page that already has its own H1 (the
     * Changelog or Download page), the result is a multi-H1 document
     * with a broken outline. We demote every heading by one level so
     * `#` becomes `<h2>`, `##` becomes `<h3>`, etc. — keeping the
     * outline valid no matter how the original Markdown was authored.
     *
     * h6 stays at h6 (HTML's deepest semantic heading) rather than
     * silently dropping content.
     */
    public static function renderEmbedded(?string $markdown): string
    {
        if ($markdown === null || $markdown === '') {
            return '';
        }
        // Hard-strip raw HTML and refuse unsafe URL schemes
        // (`javascript:`, `data:`, `vbscript:`).
        //
        // The output of this method is consumed by Blade's `{!! !!}`
        // raw-print on the changelog, download, and blog pages
        // (changelog.blade.php, download.blade.php, blog/show.blade.php).
        // `{!! !!}` does NOT auto-escape, so anything CommonMark
        // emits goes straight into the page. Pre-fix, the default
        // CommonMark config left raw HTML pass-through enabled —
        // a release-notes file or blog post with `<script>` would
        // execute in the browser.
        //
        // `html_input => 'strip'` removes inline HTML at parse time
        // (so `<script>` becomes nothing rather than the escaped
        // string `&lt;script&gt;`). `allow_unsafe_links => false`
        // makes CommonMark drop the href on `[x](javascript:...)`
        // / `[x](data:...)` / `[x](vbscript:...)` rather than emit
        // them.
        $options = [
            'html_input'         => 'strip',
            'allow_unsafe_links' => false,
        ];
        $html = (string) Str::markdown($markdown, $options);
        // Walk h5 → h6, h4 → h5, ..., h1 → h2. Order matters: rewrite
        // the deepest first so we don't double-shift earlier matches.
        for ($from = 5; $from >= 1; $from--) {
            $to = $from + 1;
            $html = preg_replace(
                "#<(/)?h{$from}(\b[^>]*)>#i",
                "<\\1h{$to}\\2>",
                $html,
            );
        }
        return $html;
    }
}
