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
        $html = (string) Str::markdown($markdown);
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
