@php
    $links = [
        ['label' => 'Features', 'href' => route('features')],
        ['label' => 'How it works', 'href' => route('how-it-works')],
        ['label' => 'Privacy', 'href' => route('privacy')],
        ['label' => 'Pricing', 'href' => route('pricing')],
        // 'Business' deliberately omitted from the primary nav. The
        // 99% of visitors are individuals and a B2B link in the header
        // is clutter for them. IT buyers reach /business via the
        // volume tiers block on /pricing, the "For business" footer
        // link, and direct sales conversations.
        ['label' => 'Compare', 'href' => route('compare.index')],
        ['label' => 'Docs', 'href' => route('docs.index')],
    ];
@endphp

<header x-data="{ open: false }" class="fixed inset-x-0 top-0 z-40" style="backdrop-filter: blur(14px); background: color-mix(in oklab, var(--bg-0) 80%, transparent); border-bottom: 1px solid var(--border);">
    <div class="ub-container flex items-center justify-between h-20">
        <a href="{{ route('home') }}" class="ub-nav-brand flex items-center gap-2 no-underline" style="color: inherit;" aria-label="Utterbar home">
            <x-logo size="nav" />
            {{-- Inline reveal-on-hover waveform. Pure SVG <animate>, no JS.
                 Hidden by default; the .ub-nav-brand:hover rule slides it
                 in. Lives next to the wordmark, sized to read against the
                 28px nav font without competing with the speech-pill mark. --}}
            <svg class="ub-nav-brand-wave" viewBox="0 0 60 24" aria-hidden="true" focusable="false">
                <defs>
                    <linearGradient id="ub-nav-wave-grad" x1="0%" y1="0%" x2="100%" y2="0%">
                        <stop offset="0%" stop-color="#00e5ff"/>
                        <stop offset="100%" stop-color="#00a3ff"/>
                    </linearGradient>
                </defs>
                <g fill="url(#ub-nav-wave-grad)">
                    <rect x="2" y="9" width="3" height="6" rx="1.5"><animate attributeName="height" values="6;14;6" dur="1.1s" begin="0s" repeatCount="indefinite"/><animate attributeName="y" values="9;5;9" dur="1.1s" begin="0s" repeatCount="indefinite"/></rect>
                    <rect x="10" y="6" width="3" height="12" rx="1.5"><animate attributeName="height" values="12;20;12" dur="1.0s" begin="0.1s" repeatCount="indefinite"/><animate attributeName="y" values="6;2;6" dur="1.0s" begin="0.1s" repeatCount="indefinite"/></rect>
                    <rect x="18" y="3" width="3" height="18" rx="1.5"><animate attributeName="height" values="18;22;18" dur="0.9s" begin="0.05s" repeatCount="indefinite"/><animate attributeName="y" values="3;1;3" dur="0.9s" begin="0.05s" repeatCount="indefinite"/></rect>
                    <rect x="26" y="1" width="3" height="22" rx="1.5"><animate attributeName="height" values="22;14;22" dur="1.2s" begin="0.2s" repeatCount="indefinite"/><animate attributeName="y" values="1;5;1" dur="1.2s" begin="0.2s" repeatCount="indefinite"/></rect>
                    <rect x="34" y="3" width="3" height="18" rx="1.5"><animate attributeName="height" values="18;10;18" dur="1.0s" begin="0.15s" repeatCount="indefinite"/><animate attributeName="y" values="3;7;3" dur="1.0s" begin="0.15s" repeatCount="indefinite"/></rect>
                    <rect x="42" y="6" width="3" height="12" rx="1.5"><animate attributeName="height" values="12;18;12" dur="0.95s" begin="0.25s" repeatCount="indefinite"/><animate attributeName="y" values="6;3;6" dur="0.95s" begin="0.25s" repeatCount="indefinite"/></rect>
                    <rect x="50" y="9" width="3" height="6" rx="1.5"><animate attributeName="height" values="6;14;6" dur="1.1s" begin="0.3s" repeatCount="indefinite"/><animate attributeName="y" values="9;5;9" dur="1.1s" begin="0.3s" repeatCount="indefinite"/></rect>
                </g>
            </svg>
        </a>

        <nav aria-label="Primary" class="hidden md:flex items-center gap-8">
            @foreach ($links as $link)
                @php $active = request()->url() === $link['href']; @endphp
                <a href="{{ $link['href'] }}" class="ub-nav-link {{ $active ? 'is-active' : '' }}" @if($active) aria-current="page" @endif>
                    {{ $link['label'] }}
                </a>
            @endforeach
        </nav>

        <div class="hidden md:flex items-center gap-3">
            <a href="{{ route('download') }}" class="ub-btn ub-btn-cta">Download</a>
        </div>

        <button type="button" @click="open = !open" class="md:hidden inline-flex items-center justify-center p-2 rounded-md" aria-label="Toggle menu" :aria-expanded="open.toString()" style="color: var(--fg);">
            <template x-if="!open"><x-icon name="menu" size="22" /></template>
            <template x-if="open"><x-icon name="x" size="22" /></template>
        </button>
    </div>

    <div x-show="open" x-cloak x-transition.opacity class="md:hidden border-t" style="border-color: var(--border); background: var(--bg-0);">
        <nav aria-label="Mobile" class="ub-container py-4 flex flex-col gap-3">
            @foreach ($links as $link)
                <a href="{{ $link['href'] }}" class="py-2 text-base font-medium" style="color: var(--fg);">{{ $link['label'] }}</a>
            @endforeach
            <a href="{{ route('download') }}" class="ub-btn ub-btn-cta mt-2">Download</a>
        </nav>
    </div>
</header>

<style>[x-cloak]{display:none !important;}</style>
