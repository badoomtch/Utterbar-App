@php
    $minimum = [
        'Windows 10 (22H2) or Windows 11',
        'x86_64 processor',
        '4 GB RAM',
        '500 MB free disk space',
    ];

    $recommended = [
        '8 GB RAM or more',
        'DirectX 12-capable GPU (any vendor) for GPU acceleration',
        'NVIDIA CUDA-capable card for the fp16 model (Pro only)',
        'Additional 1.5 GB disk space for the Pro model',
    ];

    $otherPlatforms = [
        [
            'name'   => 'macOS',
            'status' => 'Later',
            'note'   => 'Planned after Windows 1.0 ships. A Mac version needs an audio-capture and hotkey rewrite &mdash; the core is portable, the system integration isn\'t.',
        ],
        [
            'name'   => 'Linux',
            'status' => 'Later',
            'note'   => 'On the roadmap, further out than Mac. Technically possible once the core is portable; not a 1.0 priority.',
        ],
    ];
@endphp

@php
    $downloadDesc = $latest
        ? "Download Utterbar v{$latest->version} for Windows 10 and 11. Small installer, local-only dictation, free forever. Pro unlocks advanced features, £39.99 lifetime."
        : 'Download Utterbar for Windows 10 and 11. Small installer, local-only dictation, free forever. Pro unlocks advanced features, £39.99 lifetime.';
@endphp

<x-layouts.app title="Download" :description="$downloadDesc">

    {{-- Hero --}}
    <section class="relative overflow-hidden">
        <div class="ub-hero-glow" aria-hidden="true"></div>
        <div class="relative ub-container pt-20 md:pt-28 pb-12">
            <div class="max-w-4xl">
                <h1 class="ub-fg-hi" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Download for <x-hl>Windows</x-hl>.
                </h1>
                <p class="ub-lead mt-8 max-w-xl">
                    Windows-native. Local-only. Small installer.
                    @if (!$latest)
                        The public 1.0 build drops when it&rsquo;s worth installing &mdash; waitlist members see it first.
                    @endif
                </p>
            </div>
        </div>
    </section>

    {{-- Featured Windows download block --}}
    <section class="pt-12 md:pt-16 pb-16 md:pb-20">
        <div class="ub-container">
            <article class="ub-card ub-card-rail max-w-5xl" style="padding: 2.5rem 2rem;">
                <div class="grid grid-cols-1 lg:grid-cols-12 gap-10 lg:gap-14">

                    {{-- Left: version + download CTA --}}
                    <div class="lg:col-span-5">
                        <p class="ub-eyebrow">Windows</p>
                        <div class="mt-4 flex items-baseline gap-3 flex-wrap">
                            <span class="font-mono" style="font-size: 2.25rem; font-weight: 600; color: var(--fg-hi); letter-spacing: -0.02em; line-height: 1;">
                                v{{ $latest?->version ?? config('site.version') }}
                            </span>
                            @if ($latest?->tag)
                                <span class="font-mono text-[10px] uppercase tracking-wider px-2.5 py-1 rounded-full" style="background: color-mix(in oklab, var(--accent) 15%, transparent); color: var(--accent); border: 1px solid color-mix(in oklab, var(--accent) 30%, transparent); white-space: nowrap;">
                                    {{ $latest->tag }}
                                </span>
                            @endif
                        </div>
                        <p class="mt-2 font-mono text-xs" style="color: var(--muted); letter-spacing: 0.04em;">
                            @if ($latest)
                                Released {{ $latest->published_at?->format('F Y') }}
                            @else
                                Pre-release
                            @endif
                        </p>

                        <div class="mt-8">
                            @if ($latest)
                                <a href="{{ $latest->downloadUrl() }}" class="ub-btn ub-btn-cta">
                                    Download Utterbar v{{ $latest->version }}
                                </a>
                            @else
                                <a href="{{ route('waitlist') }}" class="ub-btn ub-btn-cta">
                                    Get Utterbar for Windows
                                </a>
                            @endif
                        </div>

                        @if ($latest)
                            <p class="mt-4 font-mono text-xs" style="color: var(--muted); letter-spacing: 0.04em;">
                                {{ $latest->sizeLabel() }} &middot; Windows installer (.exe)
                            </p>
                            @if ($latest->sha256)
                                <div class="mt-3" x-data="{ copied: false }">
                                    <p class="font-mono text-[10px] uppercase tracking-wider mb-1" style="color: var(--muted);">SHA-256</p>
                                    <button type="button"
                                            @click="navigator.clipboard.writeText('{{ $latest->sha256 }}'); copied = true; setTimeout(() => copied = false, 1600)"
                                            class="font-mono text-[11px] text-left"
                                            style="color: var(--fg); word-break: break-all; background: transparent; border: none; padding: 0; cursor: pointer;">
                                        <span x-show="!copied">{{ $latest->sha256 }}</span>
                                        <span x-show="copied" style="color: var(--accent);">Copied to clipboard</span>
                                    </button>
                                </div>
                            @endif
                        @else
                            <p class="mt-4 font-mono text-xs" style="color: var(--muted); letter-spacing: 0.04em;">
                                Windows installer (.exe)
                            </p>
                            <p class="mt-6 text-sm" style="color: var(--muted); line-height: 1.55;">
                                Pre-release. The public 1.0 build drops when it&rsquo;s worth installing &mdash; waitlist members see it first.
                            </p>
                        @endif
                    </div>

                    {{-- Right: system requirements --}}
                    <div class="lg:col-span-7">
                        <p class="ub-eyebrow mb-5">System requirements</p>

                        <div class="space-y-7">
                            <div>
                                <p class="text-sm font-semibold mb-3" style="color: var(--fg-hi); letter-spacing: -0.005em;">Minimum</p>
                                <ul class="space-y-2.5">
                                    @foreach ($minimum as $item)
                                        <li class="relative text-base" style="color: var(--fg); line-height: 1.65; padding-left: 1.5rem;">
                                            <span class="absolute" style="left: 0; top: 0.7em; width: 5px; height: 5px; border-radius: 50%; background: var(--accent);"></span>
                                            {{ $item }}
                                        </li>
                                    @endforeach
                                </ul>
                            </div>

                            <div>
                                <p class="text-sm font-semibold mb-3" style="color: var(--fg-hi); letter-spacing: -0.005em;">Recommended</p>
                                <ul class="space-y-2.5">
                                    @foreach ($recommended as $item)
                                        <li class="relative text-base" style="color: var(--fg); line-height: 1.65; padding-left: 1.5rem;">
                                            <span class="absolute" style="left: 0; top: 0.7em; width: 5px; height: 5px; border-radius: 50%; background: var(--accent-3);"></span>
                                            {{ $item }}
                                        </li>
                                    @endforeach
                                </ul>
                            </div>
                        </div>
                    </div>

                </div>
            </article>
        </div>
    </section>

    {{-- Other platforms — smaller row --}}
    <section class="pb-20 md:pb-24">
        <div class="ub-container">
            <div class="max-w-5xl">
                <p class="ub-eyebrow mb-6">Other platforms</p>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    @foreach ($otherPlatforms as $p)
                        <div class="ub-card" style="padding: 1.5rem 1.75rem;">
                            <div class="flex items-baseline justify-between mb-2">
                                <h3 class="text-xl" style="letter-spacing: -0.02em; font-weight: 600;">{{ $p['name'] }}</h3>
                                <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted);">
                                    {{ $p['status'] }}
                                </span>
                            </div>
                            <p class="text-sm" style="color: var(--muted); line-height: 1.55;">
                                {!! $p['note'] !!}
                            </p>
                        </div>
                    @endforeach
                </div>
            </div>
        </div>
    </section>

    {{-- Changelog --}}
    @if ($releasesList->isNotEmpty())
    <section class="py-24 md:py-32 border-t" style="border-color: var(--border);">
        <div class="ub-container">
            <div class="max-w-4xl">
                <p class="ub-eyebrow">Releases</p>
                <h2 class="mt-5" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    What shipped, <x-hl>when</x-hl>.
                </h2>
                <p class="ub-lead mt-6 max-w-2xl">
                    Every public build. Older builds are kept for anyone who wants them.
                </p>
            </div>

            <div class="mt-16 md:mt-20 max-w-4xl">
                @foreach ($releasesList as $i => $r)
                    <article class="grid grid-cols-12 gap-y-6 gap-x-8 md:gap-x-12 py-16 md:py-20 {{ $i !== 0 ? 'border-t' : '' }}" style="{{ $i !== 0 ? 'border-color: var(--border);' : '' }}">
                        <div class="col-span-12 md:col-span-3">
                            <div>
                                <span class="font-mono block" style="font-size: 1.5rem; font-weight: 600; color: var(--fg-hi); letter-spacing: -0.02em;">
                                    v{{ $r->version }}
                                </span>
                                @if ($r->tag)
                                    <span class="inline-block mt-3 font-mono text-[10px] uppercase tracking-wider px-3 py-1 rounded-full" style="background: color-mix(in oklab, var(--accent) 15%, transparent); color: var(--accent); border: 1px solid color-mix(in oklab, var(--accent) 30%, transparent); white-space: nowrap;">
                                        {{ $r->tag }}
                                    </span>
                                @endif
                            </div>
                            <p class="mt-3 font-mono text-xs" style="color: var(--muted); letter-spacing: 0.04em;">
                                {{ $r->published_at?->format('F Y') }}
                            </p>
                            <a href="{{ $r->downloadUrl() }}"
                               class="inline-flex items-center mt-4 text-xs font-mono"
                               style="color: var(--accent); letter-spacing: 0.04em;">
                                Download &rarr;
                            </a>
                        </div>
                        <div class="col-span-12 md:col-span-9">
                            <div class="ub-markdown" style="color: var(--fg); line-height: 1.65; font-size: 1rem;">
                                {!! \Illuminate\Support\Str::markdown($r->notes_markdown) !!}
                            </div>
                        </div>
                    </article>
                @endforeach
            </div>
        </div>
    </section>
    @endif

    <x-prompt-cta
        :heading="'Get the first build.'"
        :sub="'Waitlist members see the public 1.0 release before anyone else. One email when it drops.'"
    />
</x-layouts.app>
