@php
    $excerpt = $post->excerpt ?: 'A note from building Utterbar.';
@endphp

<x-layouts.app
    :title="$post->title"
    :description="$excerpt"
    ogType="article">

    {{-- Article schema for the post — gives Google a clearer signal for
         rich-result eligibility on /blog/* pages. --}}
    @push('head')
    <script type="application/ld+json">{!! json_encode([
        '@context'      => 'https://schema.org',
        '@type'         => 'BlogPosting',
        'headline'      => $post->title,
        'datePublished' => $post->publishedAt()->toIso8601String(),
        'dateModified'  => $post->publishedAt()->toIso8601String(),
        'author'        => ['@type' => 'Person', 'name' => 'Jay Parry'],
        'publisher'     => ['@type' => 'Organization', 'name' => 'Utterbar'],
        'mainEntityOfPage' => $post->url(),
        'description'   => $excerpt,
    ], JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) !!}</script>
    @endpush

    <article class="pt-20 md:pt-28 pb-20">
        <div class="ub-container">
            <div class="max-w-3xl">
                <p class="font-mono text-xs" style="color: var(--muted); letter-spacing: 0.06em; text-transform: uppercase;">
                    {{ $post->publishedAt()->format('j F Y') }}
                    @if ($post->draft)
                        <span style="color: var(--accent);">&middot; draft</span>
                    @endif
                </p>
                <h1 class="ub-fg-hi mt-4" style="font-size: clamp(2.25rem, 5vw, 3.75rem); letter-spacing: -0.035em; line-height: 1.1;">
                    {{ $post->title }}
                </h1>
                @if ($post->excerpt)
                    <p class="ub-lead mt-6 max-w-2xl">{{ $post->excerpt }}</p>
                @endif

                <div class="ub-markdown mt-12" style="color: var(--fg); line-height: 1.7; font-size: 1.0625rem;">
                    {!! $post->html() !!}
                </div>

                <div class="mt-16 pt-8" style="border-top: 1px solid var(--border);">
                    <a href="{{ route('blog') }}" class="inline-flex items-center font-mono text-xs" style="color: var(--accent); letter-spacing: 0.04em;">
                        &larr; All posts
                    </a>
                </div>
            </div>
        </div>
    </article>
</x-layouts.app>
