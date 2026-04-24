<?php

namespace App\Http\Controllers;

use App\Models\Release;
use Illuminate\Http\Response;

/**
 * GET /sitemap.xml
 *
 * Static marketing pages + docs + legal, plus dynamic entries for every
 * published stable release (so the changelog gets crawled and older versions
 * remain indexable).
 *
 * Pages that sit behind auth (/account, /admin, /sign-in/*) or are
 * transactional (/thanks, /updates/confirm, /updates/unsubscribe) are
 * deliberately omitted — they're also blocked in robots.txt.
 */
class SitemapController extends Controller
{
    public function index(): Response
    {
        $now  = now()->toIso8601String();

        $entries = [];

        // Static marketing + docs + legal pages.
        $static = [
            ['path' => '/',                       'priority' => '1.0', 'changefreq' => 'weekly'],
            ['path' => '/features',               'priority' => '0.9', 'changefreq' => 'monthly'],
            ['path' => '/how-it-works',           'priority' => '0.9', 'changefreq' => 'monthly'],
            ['path' => '/pricing',                'priority' => '0.9', 'changefreq' => 'monthly'],
            ['path' => '/download',               'priority' => '1.0', 'changefreq' => 'weekly'],
            ['path' => '/privacy',                'priority' => '0.8', 'changefreq' => 'monthly'],
            ['path' => '/roadmap',                'priority' => '0.6', 'changefreq' => 'monthly'],
            ['path' => '/changelog',              'priority' => '0.7', 'changefreq' => 'weekly'],
            ['path' => '/contact',                'priority' => '0.5', 'changefreq' => 'yearly'],
            ['path' => '/press',                  'priority' => '0.5', 'changefreq' => 'monthly'],
            ['path' => '/docs',                   'priority' => '0.7', 'changefreq' => 'monthly'],
            ['path' => '/docs/getting-started',   'priority' => '0.7', 'changefreq' => 'monthly'],
            ['path' => '/docs/modes',             'priority' => '0.6', 'changefreq' => 'monthly'],
            ['path' => '/docs/vocabulary',        'priority' => '0.6', 'changefreq' => 'monthly'],
            ['path' => '/docs/shortcuts',         'priority' => '0.6', 'changefreq' => 'monthly'],
            ['path' => '/docs/privacy-controls',  'priority' => '0.6', 'changefreq' => 'monthly'],
            ['path' => '/docs/troubleshooting',   'priority' => '0.6', 'changefreq' => 'monthly'],
            ['path' => '/docs/faq',               'priority' => '0.7', 'changefreq' => 'monthly'],
            ['path' => '/legal/privacy',          'priority' => '0.4', 'changefreq' => 'yearly'],
            ['path' => '/legal/terms',            'priority' => '0.4', 'changefreq' => 'yearly'],
            ['path' => '/legal/eula',             'priority' => '0.4', 'changefreq' => 'yearly'],
        ];

        foreach ($static as $s) {
            $entries[] = [
                'loc'        => url($s['path']),
                'lastmod'    => $now,
                'changefreq' => $s['changefreq'],
                'priority'   => $s['priority'],
            ];
        }

        // Every stable release gets included so search engines can see the
        // changelog + indexable versioned release metadata.
        Release::stable()->orderByDesc('published_at')->get()->each(function (Release $r) use (&$entries) {
            $entries[] = [
                'loc'        => url('/download') . '#v' . $r->version,
                'lastmod'    => ($r->published_at ?? $r->created_at)->toIso8601String(),
                'changefreq' => 'yearly',
                'priority'   => '0.5',
            ];
        });

        $xml = '<?xml version="1.0" encoding="UTF-8"?>' . "\n";
        $xml .= '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">' . "\n";
        foreach ($entries as $e) {
            $xml .= "  <url>\n";
            $xml .= '    <loc>'       . htmlspecialchars($e['loc'],      ENT_XML1) . "</loc>\n";
            $xml .= '    <lastmod>'   . $e['lastmod']                              . "</lastmod>\n";
            $xml .= '    <changefreq>'. $e['changefreq']                           . "</changefreq>\n";
            $xml .= '    <priority>'  . $e['priority']                             . "</priority>\n";
            $xml .= "  </url>\n";
        }
        $xml .= '</urlset>' . "\n";

        return response($xml, 200, [
            'Content-Type'  => 'application/xml; charset=utf-8',
            'Cache-Control' => 'public, max-age=3600',
        ]);
    }
}
