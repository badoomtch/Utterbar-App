@php
    $assets = [
        [
            'slug'  => 'utterbar-logo-dark',
            'name'  => 'Full lockup — dark',
            'desc'  => 'Glyph + wordmark. Use on dark surfaces.',
            'bg'    => 'var(--bg-0)',
            'pad'   => '2.25rem 2rem',
            'pngs'  => [1024, 2048, 4096],
        ],
        [
            'slug'  => 'utterbar-logo-light',
            'name'  => 'Full lockup — light',
            'desc'  => 'Glyph + wordmark. Use on light surfaces.',
            'bg'    => '#F5F5F4',
            'pad'   => '2.25rem 2rem',
            'pngs'  => [1024, 2048, 4096],
        ],
        [
            'slug'  => 'utterbar-mark',
            'name'  => 'Mark only',
            'desc'  => 'Glyph alone, transparent background. Works anywhere.',
            'bg'    => 'var(--bg-0)',
            'pad'   => '2.5rem 2rem',
            'pngs'  => [512, 1024, 2048],
        ],
        [
            'slug'  => 'utterbar-wordmark',
            'name'  => 'Wordmark — dark',
            'desc'  => 'The name alone. White "Utter" for dark surfaces.',
            'bg'    => 'var(--bg-0)',
            'pad'   => '2.5rem 2rem',
            'pngs'  => [512, 1024, 2048],
        ],
        [
            'slug'  => 'utterbar-wordmark-light',
            'name'  => 'Wordmark — light',
            'desc'  => 'The name alone. Dark "Utter" for light surfaces.',
            'bg'    => '#F5F5F4',
            'pad'   => '2.5rem 2rem',
            'pngs'  => [512, 1024, 2048],
        ],
        [
            'slug'  => 'utterbar-app-icon',
            'name'  => 'App icon',
            'desc'  => '256×256 squircle. The Windows app tile.',
            'bg'    => 'var(--bg-1)',
            'pad'   => '2rem',
            'pngs'  => [256, 512, 1024],
        ],
    ];

    $palette = [
        ['label' => 'Accent cyan',     'hex' => '#22D3EE', 'role' => 'Primary CTA + highlights'],
        ['label' => 'Accent cyan hi',  'hex' => '#67E8F9', 'role' => 'Hover + gradient top'],
        ['label' => 'Accent cyan deep','hex' => '#0891B2', 'role' => 'Gradient deep stop'],
        ['label' => 'Wave top',        'hex' => '#00F6FF', 'role' => 'Waveform gradient top'],
        ['label' => 'Wave bottom',     'hex' => '#00A3FF', 'role' => 'Waveform gradient bottom'],
        ['label' => 'Background',      'hex' => '#0A0E17', 'role' => 'Canvas'],
        ['label' => 'Surface',         'hex' => '#10151F', 'role' => 'Cards + footer'],
        ['label' => 'Foreground hi',   'hex' => '#FFFFFF', 'role' => 'Headings'],
        ['label' => 'Foreground',      'hex' => '#E8E8EB', 'role' => 'Body text'],
        ['label' => 'Muted',           'hex' => '#8A8A94', 'role' => 'Captions + meta'],
    ];
@endphp

<x-layouts.app title="Press & brand kit" description="Utterbar press kit: logos, wordmark, colours, founder bio, press contact. Everything you need to write about the local-only Windows dictation app.">

    {{-- Hero --}}
    <section class="relative overflow-hidden">
        <div class="ub-hero-glow" aria-hidden="true"></div>
        <div class="relative ub-container pt-20 md:pt-28 pb-12">
            <div class="max-w-4xl">
                <h1 class="ub-fg-hi" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Everything you need <x-hl>to write about</x-hl> Utterbar.
                </h1>
                <p class="ub-lead mt-8 max-w-xl">
                    Logos, colours, a founder bio, and a contact that replies within a day or two.
                </p>
                <div class="mt-8 flex flex-wrap gap-3">
                    <a href="/brand/utterbar-brand-kit.zip" class="ub-btn ub-btn-cta" download>
                        Download brand kit (.zip)
                        <x-icon name="arrow-right" size="16" stroke="2" />
                    </a>
                    <a href="mailto:{{ config('site.email.contact') }}" class="ub-btn ub-btn-ghost">
                        Email press
                    </a>
                </div>
            </div>
        </div>
    </section>

    {{-- Product stills — editorial-use PNGs for journalists + reviewers --}}
    <section class="pb-16">
        <div class="ub-container">
            <div class="flex items-end justify-between mb-6 flex-wrap gap-2">
                <p class="ub-eyebrow">Product stills</p>
                <p class="text-xs font-mono" style="color: var(--muted); letter-spacing: 0.04em;">
                    2560 &times; 1600 PNG &middot; CC BY with credit &ldquo;Utterbar&rdquo;
                </p>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <x-still id="press-01-floating-bar-idle" />
                <x-still id="press-02-floating-bar-listening" />
                <x-still id="press-03-main-shell-dashboard" class="md:col-span-2" />
                <x-still id="press-04-vocabulary" />
                <x-still id="press-05-file-transcription" />
            </div>
            <p class="mt-4 text-xs" style="color: var(--muted);">
                Right-click &rarr; &ldquo;Save image as&hellip;&rdquo; to download any still at full resolution.
            </p>
        </div>
    </section>

    {{-- Logos --}}
    <section class="pb-16">
        <div class="ub-container">
            <p class="ub-eyebrow mb-6">Logos</p>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                @foreach ($assets as $a)
                    <div class="ub-card" style="padding: 0; overflow: hidden;">
                        <div class="flex items-center justify-center"
                             style="background: {{ $a['bg'] }}; padding: {{ $a['pad'] }}; min-height: 200px; border-bottom: 1px solid var(--border);">
                            <img src="/brand/{{ $a['slug'] }}.svg" alt="{{ $a['name'] }}"
                                 style="max-width: 100%; max-height: 140px; height: auto;" />
                        </div>
                        <div style="padding: 1.25rem 1.5rem;">
                            <p class="font-semibold" style="color: var(--fg-hi); letter-spacing: -0.01em;">{{ $a['name'] }}</p>
                            <p class="text-sm mt-1" style="color: var(--muted);">{{ $a['desc'] }}</p>

                            <div class="mt-4 flex flex-wrap items-center gap-2">
                                <a href="/brand/{{ $a['slug'] }}.svg" download
                                   class="inline-flex items-center gap-1.5 rounded-lg"
                                   style="padding: 0.5rem 0.85rem; font-size: 0.8rem; font-weight: 600; letter-spacing: 0.02em;
                                          background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg-hi);">
                                    SVG
                                </a>
                                <span class="font-mono text-[11px] ml-1 mr-0.5" style="color: var(--muted); letter-spacing: 0.06em;">PNG</span>
                                @foreach ($a['pngs'] as $w)
                                    <a href="/brand/png/{{ $a['slug'] }}-{{ $w }}.png" download
                                       class="inline-flex items-center rounded-lg font-mono"
                                       style="padding: 0.5rem 0.75rem; font-size: 0.75rem; font-weight: 500; letter-spacing: 0.04em;
                                              background: transparent; border: 1px solid var(--border-strong); color: var(--fg);">
                                        {{ $w }}
                                    </a>
                                @endforeach
                            </div>
                        </div>
                    </div>
                @endforeach
            </div>
        </div>
    </section>

    {{-- Colours --}}
    <section class="pb-16" x-data="{ copied: '' }">
        <div class="ub-container">
            <p class="ub-eyebrow mb-6">Colours</p>
            <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
                @foreach ($palette as $c)
                    <button type="button"
                            @click="navigator.clipboard.writeText('{{ $c['hex'] }}'); copied = '{{ $c['hex'] }}'; setTimeout(() => copied = '', 1400)"
                            class="text-left ub-card"
                            style="padding: 0; overflow: hidden; cursor: pointer;">
                        <div style="background: {{ $c['hex'] }}; height: 72px; border-bottom: 1px solid var(--border);"></div>
                        <div style="padding: 0.75rem 0.9rem;">
                            <p class="font-mono text-[11px]" style="color: var(--fg-hi); letter-spacing: 0.04em;">
                                <span x-show="copied !== '{{ $c['hex'] }}'">{{ $c['hex'] }}</span>
                                <span x-show="copied === '{{ $c['hex'] }}'" style="color: var(--accent);">Copied</span>
                            </p>
                            <p class="text-[11px] mt-0.5" style="color: var(--muted);">{{ $c['label'] }}</p>
                        </div>
                    </button>
                @endforeach
            </div>
            <p class="mt-4 text-xs" style="color: var(--muted);">Tap a swatch to copy the hex.</p>
        </div>
    </section>

    {{-- Typography --}}
    <section class="pb-16">
        <div class="ub-container">
            <p class="ub-eyebrow mb-6">Typography</p>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="ub-card">
                    <p class="ub-eyebrow">Display + body</p>
                    <p class="mt-4" style="font-family: var(--font-sans); font-size: 2.75rem; font-weight: 600; letter-spacing: -0.04em; line-height: 1.05; color: var(--fg-hi);">
                        Instrument Sans
                    </p>
                    <p class="mt-3 text-sm" style="color: var(--muted);">
                        Variable weight. Weight 600 for headings and the wordmark, 400 for body. Letter-spacing −0.04em on display sizes. SIL Open Font License.
                    </p>
                    <p class="mt-4 font-mono text-xs" style="color: var(--muted);">fonts.google.com/specimen/Instrument+Sans</p>
                </div>
                <div class="ub-card">
                    <p class="ub-eyebrow">Mono</p>
                    <p class="mt-4" style="font-family: var(--font-mono); font-size: 2rem; font-weight: 500; letter-spacing: -0.02em; line-height: 1.1; color: var(--fg-hi);">
                        Geist Mono
                    </p>
                    <p class="mt-3 text-sm" style="color: var(--muted);">
                        Used sparingly for tags, version strings, and keyboard chips. Open Font License.
                    </p>
                    <p class="mt-4 font-mono text-xs" style="color: var(--muted);">vercel.com/font</p>
                </div>
            </div>
        </div>
    </section>

    {{-- Usage --}}
    <section class="pb-16">
        <div class="ub-container">
            <p class="ub-eyebrow mb-6">Usage</p>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="ub-card">
                    <p class="font-semibold" style="color: var(--accent);">Do</p>
                    <ul class="mt-4 space-y-2 text-sm" style="color: var(--fg);">
                        <li>— Use the dark lockup on dark surfaces, the light lockup on light surfaces.</li>
                        <li>— Leave clear space around the lockup equal to the height of the hole-punch circle.</li>
                        <li>— Keep the mark at least 32px wide, the full lockup at least 160px wide.</li>
                        <li>— Place the logo on a clean background or a brand surface.</li>
                    </ul>
                </div>
                <div class="ub-card">
                    <p class="font-semibold" style="color: var(--danger, #f87171);">Don't</p>
                    <ul class="mt-4 space-y-2 text-sm" style="color: var(--fg);">
                        <li>— Recolour the glyph, the waveform, or the wordmark.</li>
                        <li>— Stretch, skew, rotate, or add drop shadows.</li>
                        <li>— Place the logo on busy imagery without a background plate.</li>
                        <li>— Recreate the wordmark in another typeface.</li>
                    </ul>
                </div>
            </div>
        </div>
    </section>

    {{-- Facts for writers --}}
    <section class="pb-16">
        <div class="ub-container">
            <p class="ub-eyebrow mb-6">Facts for writers</p>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="ub-card">
                    <h3 class="text-xl" style="letter-spacing: -0.02em;">One-liner</h3>
                    <p class="mt-3" style="color: var(--muted);">{{ config('site.description') }}</p>
                </div>
                <div class="ub-card">
                    <h3 class="text-xl" style="letter-spacing: -0.02em;">Tagline</h3>
                    <p class="mt-3" style="color: var(--muted);">{{ config('site.tagline') }}</p>
                </div>
                <div class="ub-card">
                    <h3 class="text-xl" style="letter-spacing: -0.02em;">Positioning</h3>
                    <p class="mt-3" style="color: var(--muted);">
                        A Windows-native dictation app that runs entirely on the user's PC. No account, no cloud transcription, no subscription tax on the act of talking. Press a hotkey, speak, and the words land wherever the cursor is.
                    </p>
                </div>
                <div class="ub-card">
                    <h3 class="text-xl" style="letter-spacing: -0.02em;">Founder bio</h3>
                    <p class="mt-3" style="color: var(--muted);">
                        Jay Parry is a British software engineer building Utterbar from the conviction that dictation tools have no business in the cloud. He writes Rust in the morning and TypeScript after lunch.
                    </p>
                </div>
                @php $latest = \App\Models\Release::stable()->orderByDesc('published_at')->first(); @endphp
                <div class="ub-card">
                    <h3 class="text-xl" style="letter-spacing: -0.02em;">Stage</h3>
                    <p class="mt-3" style="color: var(--muted);">
                        Shipping. Current public build: v{{ $latest?->version ?? config('site.version') }}. Download at
                        <a href="{{ route('download') }}" style="color: var(--accent);">{{ config('site.domain') }}/download</a>.
                    </p>
                </div>
                <div class="ub-card">
                    <h3 class="text-xl" style="letter-spacing: -0.02em;">Press contact</h3>
                    <p class="mt-3" style="color: var(--muted);">
                        Email <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a>.
                    </p>
                    <p class="mt-2 text-sm" style="color: var(--muted);">Usually within a day, always within two.</p>
                </div>
            </div>
        </div>
    </section>

    {{-- Final CTA --}}
    <section class="pb-24">
        <div class="ub-container">
            <div class="ub-card ub-card-rail flex flex-col md:flex-row items-start md:items-center justify-between gap-6" style="padding: 2rem 2.25rem;">
                <div>
                    <p class="ub-eyebrow mb-2">Grab the kit</p>
                    <p class="text-xl" style="color: var(--fg-hi); letter-spacing: -0.02em;">
                        All five assets plus a README, in one zip.
                    </p>
                </div>
                <a href="/brand/utterbar-brand-kit.zip" class="ub-btn ub-btn-cta" download>
                    Download .zip
                    <x-icon name="arrow-right" size="16" stroke="2" />
                </a>
            </div>
        </div>
    </section>

</x-layouts.app>
