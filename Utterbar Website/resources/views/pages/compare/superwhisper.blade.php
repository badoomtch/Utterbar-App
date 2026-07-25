@php
    // Comparison rows. Sourced 2026-05-01 from superwhisper.com
    // (homepage, /windows page, /privacy page, /changelog).
    //
    // Important context: Superwhisper is NOT macOS-only. The home
    // page lists "Available on Mac, Windows and iOS" and there is
    // a dedicated /windows download page. This is a cross-platform
    // sibling product, not a Mac-only foil. The honest comparison
    // is feature/pricing/positioning on the same Windows machine.
    //
    // Pricing: only $8.49/month was directly verified on the
    // homepage in our fact-check pass. Yearly and lifetime price
    // points exist (and are widely reported) but the homepage
    // pricing widget did not render their literal figures cleanly
    // in the fetch we did, so we cite only $8.49/mo here and link
    // to superwhisper.com for the current full pricing matrix.
    $rows = [
        [
            'topic'      => 'Platforms',
            'utterbar'   => 'Windows 10 / 11.',
            'competitor' => 'Mac, Windows, and iOS. The Windows build is officially shipped, not an experimental side project.',
            'edge'       => 'competitor',
        ],
        [
            'topic'      => 'Pricing model',
            'utterbar'   => 'Free and open source. Every feature included, no account.',
            'competitor' => 'Free tier (does not expire) + Pro from $8.49/month, plus yearly and lifetime tiers. USD only on the public site. 30-day refund.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Where transcription happens',
            'utterbar'   => 'On your PC. No cloud option exists. Audio stays on the machine.',
            'competitor' => 'Local-first. Offline models run on-device; cloud models are an opt-in for older or lower-spec hardware. Privacy policy says no audio is retained on Superwhisper&rsquo;s servers.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Speech models',
            'utterbar'   => 'NVIDIA Parakeet-tdt for English (fast); Whisper Small / Medium / Large for everything else.',
            'competitor' => 'Whisper Large on-device (Mac), with cloud variants available. Other model variants are not enumerated on the public homepage.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Languages',
            'utterbar'   => '30+ out of the box, all processed on your PC.',
            'competitor' => '100+ languages and dialects, plus translation to English from any of them.',
            'edge'       => 'competitor',
        ],
        [
            'topic'      => 'File transcription',
            'utterbar'   => 'Yes, included. Drop a recording, get a plain-text transcript next to the source file.',
            'competitor' => 'Yes. Audio and video upload supported.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Meeting transcription',
            'utterbar'   => 'Yes. Native WASAPI loopback captures system audio; Meeting Mode chunks long recordings.',
            'competitor' => 'Meeting assistant feature exists. The system-audio capture mechanism is not detailed on the public site.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Custom vocabulary',
            'utterbar'   => 'Yes. Custom words and replacements, applied locally before output.',
            'competitor' => 'Yes. Specialised terms feature called out on the homepage.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Modes',
            'utterbar'   => 'Three built-in modes (voice / message / meeting); user can create more.',
            'competitor' => 'Predefined modes (voice, message, email) plus user-defined.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'AI post-processing',
            'utterbar'   => 'Local, deterministic. No LLM rewrites of your speech.',
            'competitor' => '&ldquo;Super Mode&rdquo; uses an LLM (configurable: GPT, Claude, Llama, Gemini, etc.) to reshape transcripts. Bring your own key or use Superwhisper&rsquo;s.',
            'edge'       => 'competitor',
        ],
        [
            'topic'      => 'Devices',
            'utterbar'   => 'Install on as many devices as you like. No licence, no seat cap.',
            'competitor' => 'Per-user, no stated device cap. &ldquo;Activate on as many of your devices as you like.&rdquo;',
            'edge'       => 'competitor',
        ],
        [
            'topic'      => 'Telemetry',
            'utterbar'   => 'None. The app does not phone home.',
            'competitor' => 'Privacy policy: &ldquo;we do not collect any usage data when you use Superwhisper.&rdquo; Honest cousin.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Update cadence',
            'utterbar'   => 'Several beta releases a month, signed updater, opt-in beta channel.',
            'competitor' => 'Active. Multiple releases per week (changelog shows 2.13.x patches every 2&ndash;3 days at the time of writing).',
            'edge'       => 'tie',
        ],
    ];

    $compareCrumbs = [
        '@context'        => 'https://schema.org',
        '@type'           => 'BreadcrumbList',
        'itemListElement' => [
            ['@type' => 'ListItem', 'position' => 1, 'name' => 'Home',    'item' => url('/')],
            ['@type' => 'ListItem', 'position' => 2, 'name' => 'Compare', 'item' => route('compare.index')],
            ['@type' => 'ListItem', 'position' => 3, 'name' => 'Superwhisper', 'item' => url()->current()],
        ],
    ];
    $compareProduct = [
        '@context'           => 'https://schema.org',
        '@type'              => 'SoftwareApplication',
        'name'               => 'Utterbar',
        'operatingSystem'    => 'Windows 10, Windows 11',
        'applicationCategory' => 'BusinessApplication',
        'offers' => [
            '@type'         => 'Offer',
            'price'         => '0',
            'priceCurrency' => 'GBP',
            'url'           => route('pricing'),
        ],
        'url' => url('/'),
    ];
@endphp

@push('head')
<script type="application/ld+json">{!! json_encode($compareCrumbs, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) !!}</script>
<script type="application/ld+json">{!! json_encode($compareProduct, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) !!}</script>
@endpush

<x-layouts.app
    title="Utterbar vs Superwhisper"
    description="Two local-first dictation apps. Superwhisper is cross-platform (Mac, Windows, iOS) with optional cloud-LLM rewrites and subscription pricing from $8.49/month. Utterbar is Windows-only, free and open source, with strict on-device processing.">

    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <p class="ub-eyebrow">Comparison</p>
                <h1 class="ub-fg-hi mt-4" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Utterbar vs <x-hl>Superwhisper</x-hl>.
                </h1>
                <p class="ub-lead mt-8 max-w-2xl">
                    Two local-first dictation apps with overlapping values. Superwhisper is the cross-platform incumbent in this corner of the market. Utterbar is the Windows-focused sibling. Both are honest about privacy. Here&rsquo;s what differs.
                </p>
            </div>
        </div>
    </section>

    {{-- Top-line summary card --}}
    <section class="pb-10">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto rounded-2xl overflow-hidden" style="background: var(--bg-1);">
                <div class="grid grid-cols-1 md:grid-cols-2">
                    <div class="p-8 md:p-10 relative"
                         style="background: radial-gradient(120% 80% at 0% 0%, color-mix(in oklab, var(--accent) 12%, transparent), transparent 60%); border-left: 2px solid var(--accent);">
                        <div class="flex items-baseline justify-between mb-2">
                            <h2 class="text-2xl"><x-wordmark /></h2>
                            <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--accent);">Local · free · open source</span>
                        </div>
                        <p class="mt-4" style="color: var(--fg); line-height: 1.55;">
                            Windows-native, free and open source. Strict on-device processing. The app has nowhere to send your audio.
                        </p>
                        <ul class="mt-6 space-y-2.5 text-sm">
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">Whisper / Parakeet on-device, no cloud option</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">Free and open source, no account, no device cap</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">No LLM rewrites of your speech</span></li>
                        </ul>
                    </div>
                    <div class="p-8 md:p-10" style="border-left: 1px solid var(--border);">
                        <div class="flex items-baseline justify-between mb-2">
                            <h2 class="text-2xl" style="color: var(--muted);">Superwhisper</h2>
                            <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted);">Local-first · cross-platform · USD</span>
                        </div>
                        <p class="mt-4" style="color: var(--muted); line-height: 1.55;">
                            Mac, Windows, iOS. Local-first with optional cloud models. Subscription from $8.49/month, plus yearly and lifetime tiers. 100+ languages with translation. Strong privacy policy.
                        </p>
                        <ul class="mt-6 space-y-2.5 text-sm">
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">Cross-platform; same account everywhere</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">Optional LLM post-processing (BYOK)</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">100+ languages with translation</span></li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    </section>

    {{-- Detailed comparison table --}}
    <section class="py-12 md:py-16">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto">
                <h2 class="text-2xl md:text-3xl mb-8" style="letter-spacing: -0.025em;">The full comparison.</h2>
                <x-compare-table
                    :rows="$rows"
                    competitor="Superwhisper"
                    sourceUrl="https://superwhisper.com"
                    sourceLabel="superwhisper.com"
                    sourceDate="2026-05-01"
                />
            </div>
        </div>
    </section>

    {{-- Honest essay --}}
    <section class="py-16 md:py-24">
        <div class="ub-container">
            <div class="max-w-3xl mx-auto">
                <h2 class="text-2xl md:text-3xl mb-8" style="letter-spacing: -0.025em;">Which one should you pick?</h2>

                <div class="space-y-6 text-lg leading-relaxed" style="color: var(--fg);">
                    <p>
                        Superwhisper is a great product. We&rsquo;re not going to pretend otherwise. Their privacy policy is genuinely strong, the team ships often, the cross-platform reach is real, and the LLM post-processing angle is something we deliberately don&rsquo;t do. The honest split is values and platform.
                    </p>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">Pick Superwhisper if&hellip;</h3>
                    <ul class="space-y-3 text-base" style="color: var(--fg);">
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You work across Mac, Windows, and iOS and want one app on all of them with one account. Cross-platform is genuinely Superwhisper&rsquo;s; we are deliberately Windows-only.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You want the option of an LLM post-processing pass after the transcript lands. Reshape into bullets, fix tone, translate, summarise. Bring your own GPT/Claude/Llama key, or use theirs. We don&rsquo;t do this and don&rsquo;t plan to.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You dictate in many languages. 100+ vs our 30+. If you regularly switch between, say, Polish and Greek, Superwhisper&rsquo;s coverage is wider.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You prefer subscription billing or a USD lifetime tier. Both are on offer.</span></li>
                    </ul>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">Pick Utterbar if&hellip;</h3>
                    <ul class="space-y-3 text-base" style="color: var(--fg);">
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You&rsquo;re on Windows and you want a Windows-native app, not a cross-platform tool that happens to run there. The floating bar, the WASAPI loopback, the DirectML acceleration, the signed NSIS installer: those are built on Windows primitives.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You&rsquo;d rather not pay at all. Utterbar is free and open source, with no card, no account, and every feature included.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You want strict on-device processing as a property of the architecture, not a default that can be flipped. Utterbar has no cloud option. The app cannot send your audio anywhere because the code does not exist.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You don&rsquo;t want LLM rewrites. Some users find them magical; others find them subtly wrong. Our default is honest transcription only.</span></li>
                    </ul>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">If you have both Mac and Windows</h3>
                    <p class="text-base" style="color: var(--fg);">
                        You probably want Superwhisper for the Mac and Utterbar for the Windows machine. Both are local-first; both have strong privacy stances; both are actively developed. There is no shame in running the right tool on each platform.
                    </p>
                </div>
            </div>
        </div>
    </section>

    <x-prompt-cta
        :heading="'See for yourself.'"
        :sub="'Free and open source, no account needed. Every feature is included.'"
    />
</x-layouts.app>
