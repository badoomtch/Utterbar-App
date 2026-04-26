<?php

namespace App\Support;

use Illuminate\Support\Carbon;
use Illuminate\Support\Facades\Cache;

/**
 * File-backed blog posts.
 *
 * Posts live under `resources/blog/` as `YYYY-MM-DD-slug.md`. The first
 * lines may declare YAML-ish front matter (title, excerpt, draft) bounded
 * by `---` markers. Anything after is the post body in Markdown.
 *
 * No DB, no CMS, no admin UI. Edit a file, redeploy. Cached so the file
 * scan is one disk hit per page render rather than per-post.
 */
class BlogPost
{
    public string $slug;
    public string $date;        // YYYY-MM-DD
    public string $title;
    public string $excerpt;
    public string $body;        // raw markdown
    public bool   $draft;

    public function url(): string
    {
        return route('blog.show', ['slug' => $this->slug]);
    }

    public function publishedAt(): Carbon
    {
        return Carbon::parse($this->date);
    }

    public function html(): string
    {
        // Demote h1 → h2 etc. so the post body never collides with the
        // page H1 (which the layout owns).
        return Markdown::renderEmbedded($this->body);
    }

    /**
     * All published posts, newest first. Drafts (front-matter
     * `draft: true`) are excluded in production. Cached for 60s in
     * production so a deploy refresh feels instant; uncached in dev.
     */
    public static function all(): array
    {
        $loader = function () {
            $dir = resource_path('blog');
            if (!is_dir($dir)) {
                return [];
            }
            $files = glob($dir . '/*.md') ?: [];
            $posts = [];
            foreach ($files as $f) {
                $p = self::fromFile($f);
                if ($p === null) continue;
                if ($p->draft && app()->environment('production')) continue;
                $posts[] = $p;
            }
            usort($posts, fn ($a, $b) => strcmp($b->date, $a->date));
            return $posts;
        };

        return app()->environment('production')
            ? Cache::remember('blog.posts.all', 60, $loader)
            : $loader();
    }

    public static function findBySlug(string $slug): ?self
    {
        foreach (self::all() as $post) {
            if ($post->slug === $slug) return $post;
        }
        return null;
    }

    private static function fromFile(string $path): ?self
    {
        $base = basename($path, '.md');
        if (!preg_match('/^(\d{4}-\d{2}-\d{2})-(.+)$/', $base, $m)) {
            return null;
        }
        $date = $m[1];
        $slug = $m[2];

        $raw = @file_get_contents($path);
        if ($raw === false) {
            return null;
        }

        // Front matter: optional, bounded by --- on separate lines at
        // the top. Tiny parser — `key: value` per line, no nested
        // structures. Anything richer would be over-engineered for a
        // single-author blog.
        $title = $slug;
        $excerpt = '';
        $draft = false;
        $body = $raw;

        if (preg_match('/^---\s*\n(.+?)\n---\s*\n(.*)$/s', $raw, $fm)) {
            $body = $fm[2];
            foreach (preg_split('/\r?\n/', $fm[1]) as $line) {
                if (!preg_match('/^([a-z_]+):\s*(.*?)\s*$/i', $line, $kv)) continue;
                $key = strtolower($kv[1]);
                $val = trim($kv[2], " \t\"'");
                if ($key === 'title')   $title = $val;
                if ($key === 'excerpt') $excerpt = $val;
                if ($key === 'draft')   $draft = in_array(strtolower($val), ['true', '1', 'yes'], true);
            }
        }

        $post = new self();
        $post->slug    = $slug;
        $post->date    = $date;
        $post->title   = $title;
        $post->excerpt = $excerpt;
        $post->body    = $body;
        $post->draft   = $draft;
        return $post;
    }
}
