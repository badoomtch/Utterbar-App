<!DOCTYPE html>
<html lang="en-GB" class="scroll-smooth">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="theme-color" content="#0A0E17">
    <meta name="color-scheme" content="dark">

    {{-- Paint-early: prevent any white flash before CSS resolves --}}
    <style>html,body{background-color:#0A0E17;color:#E8E8EB;}</style>

    @php
        $t = $title ?? null;
        $pageTitle = $t ? "$t — " . config('site.name') : config('site.name') . ' — ' . config('site.tagline');
        $desc = $description ?? config('site.description');
        $canonical = url()->current();
        $ogImage = isset($ogImage) ? url($ogImage) : url('/og/og-default.png?v=1');
        $ogType = $ogType ?? 'website';
        $noIndex = $noIndex ?? false;
    @endphp

    <title>{{ $pageTitle }}</title>
    <meta name="description" content="{{ $desc }}">
    <link rel="canonical" href="{{ $canonical }}">
    <meta name="author" content="Jay Parry">

    {{-- Block indexing on auth'd / transactional surfaces. Everywhere else is indexable + followed by default. --}}
    @if ($noIndex)
        <meta name="robots" content="noindex, nofollow">
    @else
        <meta name="robots" content="index, follow, max-image-preview:large, max-snippet:-1">
    @endif

    {{-- Open Graph --}}
    <meta property="og:type"         content="{{ $ogType }}">
    <meta property="og:title"        content="{{ $pageTitle }}">
    <meta property="og:description"  content="{{ $desc }}">
    <meta property="og:url"          content="{{ $canonical }}">
    <meta property="og:site_name"    content="{{ config('site.name') }}">
    <meta property="og:locale"       content="en_GB">
    <meta property="og:image"        content="{{ $ogImage }}">
    <meta property="og:image:width"  content="1200">
    <meta property="og:image:height" content="630">
    <meta property="og:image:alt"    content="Utterbar — dictation that stays on your PC">

    {{-- Twitter Card --}}
    <meta name="twitter:card"        content="summary_large_image">
    <meta name="twitter:title"       content="{{ $pageTitle }}">
    <meta name="twitter:description" content="{{ $desc }}">
    <meta name="twitter:image"       content="{{ $ogImage }}">
    <meta name="twitter:image:alt"   content="Utterbar — dictation that stays on your PC">

    {{-- Favicons + app icons --}}
    <link rel="icon"             href="/favicon.svg?v=3" type="image/svg+xml">
    <link rel="apple-touch-icon" href="/brand/png/utterbar-app-icon-256.png">
    <link rel="mask-icon"        href="/favicon.svg?v=3" color="#22D3EE">

    {{-- Site-wide Organization + WebSite structured data --}}
    @php
        $siteSchema = [
            '@context' => 'https://schema.org',
            '@graph' => [
                [
                    '@type' => 'Organization',
                    '@id'   => url('/#organization'),
                    'name'  => config('site.name'),
                    'url'   => url('/'),
                    'logo'  => url('/brand/png/utterbar-app-icon-256.png'),
                    'founder' => [
                        '@type' => 'Person',
                        'name'  => 'Jay Parry',
                    ],
                    'contactPoint' => [
                        '@type'             => 'ContactPoint',
                        'email'             => config('site.email.contact'),
                        'contactType'       => 'customer support',
                        'areaServed'        => 'Worldwide',
                        'availableLanguage' => ['en'],
                    ],
                ],
                [
                    '@type'       => 'WebSite',
                    '@id'         => url('/#website'),
                    'url'         => url('/'),
                    'name'        => config('site.name'),
                    'description' => config('site.description'),
                    'inLanguage'  => 'en-GB',
                    'publisher'   => ['@id' => url('/#organization')],
                ],
            ],
        ];
    @endphp
    <script type="application/ld+json">{!! json_encode($siteSchema, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) !!}</script>

    @stack('head')

    @vite(['resources/css/app.css', 'resources/js/app.js'])

    {{-- instant.page — pre-fetches links on hover so clicks feel immediate --}}
    <script src="//instant.page/5.2.0" type="module" integrity="sha384-jnZyxPjiipYXnSU0ygqeac2q7CVYMbh84q0uHVRRxEtvFPiQYbXWUorga2aqZJ0z"></script>
</head>
<body class="min-h-screen antialiased">
    <a href="#main" class="ub-skip">Skip to content</a>

    <x-nav />

    <main id="main" class="pt-20">
        {{ $slot }}
    </main>

    <x-footer />

    @stack('scripts')
</body>
</html>
