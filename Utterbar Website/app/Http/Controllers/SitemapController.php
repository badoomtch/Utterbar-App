<?php

namespace App\Http\Controllers;

use App\Support\BlogPost;
use Illuminate\Http\Response;

/**
 * GET /sitemap.xml
 *
 * Marketing pages, docs, and legal pages that are real public destinations.
 *
 * Pages that sit behind auth (/account, /admin, /sign-in/*) or are
 * transactional (/thanks, /updates/confirm, /updates/unsubscribe) are
 * deliberately omitted — they're also blocked in robots.txt.
 *
 * Notable absences and why:
 *   • /legal/eula — placeholder text; excluded until the full EULA ships
 *     with 1.0. Tagged noindex on the page itself for the same reason.
 *   • Release fragment URLs (/download#v0.7.0) — fragments aren't
 *     separate pages to a crawler. /download has a fresh `lastmod`
 *     pointing at the latest release; that's all crawlers can use.
 */
class SitemapController extends Controller
{
    public function index(): Response
    {
        $now = now()->toIso8601String();

        $entries = [
            ['path' => '/',                       'priority' => '1.0', 'changefreq' => 'weekly',  'lastmod' => $now],
            ['path' => '/features',               'priority' => '0.9', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/how-it-works',           'priority' => '0.9', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/pricing',                'priority' => '0.9', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/business',               'priority' => '0.9', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/business/contact',       'priority' => '0.6', 'changefreq' => 'yearly',  'lastmod' => $now],
            ['path' => '/download',               'priority' => '1.0', 'changefreq' => 'weekly',  'lastmod' => $now],
            ['path' => '/privacy',                'priority' => '0.8', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/roadmap',                'priority' => '0.6', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/changelog',              'priority' => '0.7', 'changefreq' => 'weekly',  'lastmod' => $now],
            ['path' => '/contact',                'priority' => '0.5', 'changefreq' => 'yearly',  'lastmod' => $now],
            ['path' => '/press',                  'priority' => '0.5', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/compare',                'priority' => '0.7', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/compare/wispr-flow',     'priority' => '0.8', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/compare/dragon',         'priority' => '0.8', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/compare/voice-access',   'priority' => '0.8', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/compare/superwhisper',   'priority' => '0.8', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/what-is-local-dictation','priority' => '0.7', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/docs',                   'priority' => '0.7', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/docs/getting-started',   'priority' => '0.7', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/docs/modes',             'priority' => '0.6', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/docs/vocabulary',        'priority' => '0.6', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/docs/shortcuts',         'priority' => '0.6', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/docs/privacy-controls',  'priority' => '0.6', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/docs/troubleshooting',   'priority' => '0.6', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/docs/faq',               'priority' => '0.7', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/docs/it-deployment',     'priority' => '0.7', 'changefreq' => 'monthly', 'lastmod' => $now],
            ['path' => '/legal/privacy',          'priority' => '0.4', 'changefreq' => 'yearly',  'lastmod' => $now],
            ['path' => '/legal/terms',            'priority' => '0.4', 'changefreq' => 'yearly',  'lastmod' => $now],
            ['path' => '/blog',                   'priority' => '0.6', 'changefreq' => 'weekly',  'lastmod' => $now],
        ];

        // Blog posts — added at the bottom so the static list above stays
        // easy to read. Each post's lastmod is its publish date.
        foreach (BlogPost::all() as $post) {
            $entries[] = [
                'path'       => '/blog/' . $post->slug,
                'priority'   => '0.6',
                'changefreq' => 'monthly',
                'lastmod'    => $post->publishedAt()->toIso8601String(),
            ];
        }

        $xml = '<?xml version="1.0" encoding="UTF-8"?>' . "\n";
        $xml .= '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">' . "\n";
        foreach ($entries as $e) {
            $xml .= "  <url>\n";
            $xml .= '    <loc>'       . htmlspecialchars(url($e['path']), ENT_XML1) . "</loc>\n";
            $xml .= '    <lastmod>'   . $e['lastmod']                               . "</lastmod>\n";
            $xml .= '    <changefreq>'. $e['changefreq']                            . "</changefreq>\n";
            $xml .= '    <priority>'  . $e['priority']                              . "</priority>\n";
            $xml .= "  </url>\n";
        }
        $xml .= '</urlset>' . "\n";

        return response($xml, 200, [
            'Content-Type'  => 'application/xml; charset=utf-8',
            'Cache-Control' => 'public, max-age=3600',
        ]);
    }
}
