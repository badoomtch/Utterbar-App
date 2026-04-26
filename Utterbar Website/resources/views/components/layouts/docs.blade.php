@php
    $docLinks = [
        ['Getting started',  route('docs.getting-started')],
        ['Modes',            route('docs.modes')],
        ['Vocabulary',       route('docs.vocabulary')],
        ['Shortcuts',        route('docs.shortcuts')],
        ['Privacy controls', route('docs.privacy-controls')],
        ['Troubleshooting',  route('docs.troubleshooting')],
        ['FAQ',              route('docs.faq')],
    ];

    // BreadcrumbList schema. Helps Google render the docs hierarchy in
    // SERPs as a breadcrumb trail. Two-level: Home → Docs → {current}.
    $crumbs = [
        ['name' => 'Home',     'item' => url('/')],
        ['name' => 'Docs',     'item' => route('docs.index')],
    ];
    if (! request()->routeIs('docs.index') && isset($title)) {
        $crumbs[] = ['name' => $title, 'item' => url()->current()];
    }
    $breadcrumbSchema = [
        '@context'        => 'https://schema.org',
        '@type'           => 'BreadcrumbList',
        'itemListElement' => collect($crumbs)->values()->map(fn ($c, $i) => [
            '@type'    => 'ListItem',
            'position' => $i + 1,
            'name'     => $c['name'],
            'item'     => $c['item'],
        ])->all(),
    ];
@endphp

<x-layouts.app :title="$title ?? 'Docs'" :description="$description ?? null">
    @push('head')
        <script type="application/ld+json">{!! json_encode($breadcrumbSchema, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) !!}</script>
    @endpush

    <div class="ub-container pt-12 md:pt-16 pb-20 md:pb-24">
        <div class="grid grid-cols-1 md:grid-cols-12 gap-12 md:gap-16">
            <aside class="md:col-span-3 md:sticky md:top-28 self-start">
                <p class="ub-eyebrow mb-6">Docs</p>
                <nav class="flex flex-col">
                    @php $isOverview = request()->routeIs('docs.index'); @endphp
                    <a href="{{ route('docs.index') }}"
                       class="ub-docs-link {{ $isOverview ? 'is-active' : '' }}"
                       @if($isOverview) aria-current="page" @endif>
                        Overview
                    </a>
                    @foreach ($docLinks as [$label, $href])
                        @php $active = url()->current() === $href; @endphp
                        <a href="{{ $href }}"
                           class="ub-docs-link {{ $active ? 'is-active' : '' }}"
                           @if($active) aria-current="page" @endif>
                            {{ $label }}
                        </a>
                    @endforeach
                </nav>

                <div class="mt-10 relative" style="padding-left: 1rem;">
                    <div class="absolute top-0 bottom-0 left-0 w-[2px]" style="background: var(--accent); box-shadow: 0 0 12px color-mix(in oklab, var(--accent) 45%, transparent);"></div>
                    <p class="ub-eyebrow mb-2">v{{ config('site.version') }}</p>
                    <p class="text-sm" style="color: var(--muted); line-height: 1.55;">
                        Live docs for the current shipping build. Updated with each release.
                    </p>
                </div>
            </aside>

            <article class="md:col-span-9 max-w-3xl">
                {{ $slot }}
            </article>
        </div>
    </div>
</x-layouts.app>
