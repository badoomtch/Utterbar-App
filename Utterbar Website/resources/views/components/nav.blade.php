@php
    $links = [
        ['label' => 'Features', 'href' => route('features')],
        ['label' => 'How it works', 'href' => route('how-it-works')],
        ['label' => 'Privacy', 'href' => route('privacy')],
        ['label' => 'Pricing', 'href' => route('pricing')],
        ['label' => 'Docs', 'href' => route('docs.index')],
    ];
@endphp

<header x-data="{ open: false }" class="fixed inset-x-0 top-0 z-40" style="backdrop-filter: blur(14px); background: color-mix(in oklab, var(--bg-0) 80%, transparent); border-bottom: 1px solid var(--border);">
    <div class="ub-container flex items-center justify-between h-20">
        <a href="{{ route('home') }}" class="flex items-center gap-2 no-underline" style="color: inherit;">
            <x-logo size="nav" />
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
