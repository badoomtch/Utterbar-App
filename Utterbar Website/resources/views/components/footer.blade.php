@php
    $cols = [
        'Product' => [
            ['Features', route('features')],
            ['How it works', route('how-it-works')],
            ['Privacy', route('privacy')],
            ['Pricing', route('pricing')],
            ['Compare', route('compare.index')],
            ['Changelog', route('changelog')],
            ['Roadmap', route('roadmap')],
        ],
        'Docs' => [
            ['Getting started', route('docs.getting-started')],
            ['Modes', route('docs.modes')],
            ['Vocabulary', route('docs.vocabulary')],
            ['Shortcuts', route('docs.shortcuts')],
            ['Troubleshooting', route('docs.troubleshooting')],
            ['IT deployment', route('docs.it-deployment')],
        ],
        // EULA omitted while it remains a placeholder pending the 1.0
        // installer.
        'Company' => [
            ['Press', route('press')],
            ['Contact', route('contact')],
            ['Privacy policy', route('legal.privacy')],
            ['Terms', route('legal.terms')],
            // "Cookies" reopens the consent banner. Implemented as a
            // <button> via JS rather than a route — the consent state
            // is purely client-side, so revisiting it doesn't need a
            // server round-trip. The handler is defined in
            // components/cookie-consent.blade.php.
            ['Cookies', '#', ['onclick' => 'utterbarRevisitCookieConsent(); return false;']],
        ],
    ];
@endphp

<footer class="relative overflow-hidden mt-32" style="background: var(--bg-1); border-top: 1px solid var(--border);">
    {{-- Atmospheric cyan wash, top-right corner --}}
    <div aria-hidden="true" class="absolute inset-0 pointer-events-none"
         style="background: radial-gradient(50% 70% at 100% 0%, color-mix(in oklab, var(--accent) 18%, transparent), transparent 55%);"></div>

    <div class="relative ub-container py-20 md:py-24">

        {{-- Top band: brand on the left, final CTA on the right --}}
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-12 lg:gap-16 items-start">

            {{-- Brand plate --}}
            <div class="lg:col-span-7">
                <div style="font-size: clamp(2.5rem, 5vw, 4rem); letter-spacing: -0.045em; line-height: 1; font-weight: 600;">
                    <x-wordmark />
                </div>
                <p class="mt-6 text-lg max-w-md" style="color: var(--muted); font-weight: 300; line-height: 1.55;">
                    Windows-native dictation. Local only. No account, no cloud, no subscription tax on the act of talking.
                </p>
            </div>

            {{-- Final CTA moment --}}
            <div class="lg:col-span-5 relative" style="padding-left: 1.25rem;">
                <div class="absolute top-0 bottom-0 left-0 w-[2px]" style="background: var(--accent); box-shadow: 0 0 20px color-mix(in oklab, var(--accent) 45%, transparent);"></div>
                <p class="ub-eyebrow mb-4">Ready when you are</p>
                <h3 class="text-2xl md:text-3xl" style="letter-spacing: -0.025em; line-height: 1.15; font-weight: 700;">
                    Download Utterbar for Windows.
                </h3>
                <p class="mt-4 text-sm" style="color: var(--muted);">
                    Small installer. Windows 10 or 11. Free and open source, every feature included.
                </p>
                <div class="mt-6">
                    <a href="{{ route('download') }}" class="ub-btn ub-btn-cta">
                        Download
                        <x-icon name="arrow-right" size="16" stroke="2" />
                    </a>
                </div>
            </div>
        </div>

        {{-- Divider --}}
        <div class="mt-16 md:mt-20" style="height: 1px; background: var(--border);"></div>

        {{-- Link columns --}}
        <div class="mt-12 grid grid-cols-2 md:grid-cols-3 gap-10 md:gap-16">
            @foreach ($cols as $title => $items)
                <div>
                    <p class="ub-eyebrow mb-5">{{ $title }}</p>
                    <ul class="space-y-3">
                        @foreach ($items as $item)
                            @php
                                [$label, $href] = $item;
                                // Optional third element: extra HTML attribute map for
                                // links that need a JS handler (e.g. Cookies → reopen
                                // consent banner). Standard links pass two elements.
                                $extra = $item[2] ?? [];
                                $attrs = collect($extra)->map(fn ($v, $k) => $k . '="' . e($v) . '"')->implode(' ');
                            @endphp
                            <li><a href="{{ $href }}" class="ub-footer-link" {!! $attrs !!}>{{ $label }}</a></li>
                        @endforeach
                    </ul>
                </div>
            @endforeach
        </div>

        {{-- Waveform divider — identical to the logo glyph's internal waveform (vertical gradient) --}}
        <div class="mt-16 flex justify-center opacity-60" aria-hidden="true">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="40 8 72 44" width="220" height="40" color-interpolation="linearRGB">
                <defs>
                    <linearGradient id="fw-wave" x1="0%" y1="0%" x2="0%" y2="100%" color-interpolation="linearRGB">
                        <stop offset="0%"   stop-color="#00f6ff"/>
                        <stop offset="25%"  stop-color="#00e2fb"/>
                        <stop offset="50%"  stop-color="#00ccf5"/>
                        <stop offset="75%"  stop-color="#00b5ed"/>
                        <stop offset="100%" stop-color="#00a3ff"/>
                    </linearGradient>
                </defs>
                <g fill="url(#fw-wave)">
                    <rect x="44"  y="25" width="4" height="10" rx="2"/>
                    <rect x="54"  y="21" width="4" height="18" rx="2"/>
                    <rect x="64"  y="16" width="4" height="28" rx="2"/>
                    <path d="M 74 10 A 2 2 0 0 1 78 10 V 50 L 74 44 Z"/>
                    <rect x="84"  y="16" width="4" height="28" rx="2"/>
                    <rect x="94"  y="21" width="4" height="18" rx="2"/>
                    <rect x="104" y="25" width="4" height="10" rx="2"/>
                </g>
            </svg>
        </div>

        {{-- Copyright + metadata --}}
        <div class="mt-10 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
            <p class="text-xs" style="color: var(--muted);">
                © {{ date('Y') }} Utterbar &middot; Built by
                <a href="https://ddraig.dev" target="_blank" rel="noopener"
                   class="inline-flex items-center gap-1.5 align-middle"
                   style="color: var(--fg); text-decoration: none; border-bottom: 1px solid var(--border);">
                    <img src="{{ asset('brand/ddraig-icon.png') }}" alt=""
                         width="14" height="14"
                         style="display: inline-block; vertical-align: -2px;" />
                    Ddraig Dev
                </a>
                &middot; Built in Wales
                <img src="{{ asset('brand/welsh-dragon.png') }}" alt="Welsh dragon"
                     width="14" height="14"
                     style="display: inline-block; vertical-align: -2px; margin-left: 4px;" />
            </p>
            <p class="font-mono text-[11px]" style="color: var(--fg-dim); letter-spacing: 0.08em;">
                LOCAL-ONLY &middot; NO ACCOUNT &middot; NO TELEMETRY
            </p>
        </div>
    </div>
</footer>
