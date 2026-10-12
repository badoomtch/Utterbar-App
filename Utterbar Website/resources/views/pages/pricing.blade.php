@php
    // Everything Utterbar does, all included. There is no paid tier any
    // more: the app is free and open source, every feature on for
    // everyone. Kept as one honest list rather than a Free/Pro split.
    $included = [
        'Unlimited dictations, no daily quota (single-shot capped at 5 minutes; Meeting Mode chunks longer recordings)',
        'Parakeet v3 (English, compact) and Whisper (multilingual, 30+ languages), all on-device',
        'All seven hotkey slots and unlimited custom modes',
        'Microphone, system-audio (WASAPI loopback), and mixed capture',
        'Unlimited custom vocabulary and replacements',
        'File transcription (WAV, MP3, M4A, FLAC, OGG, MP4, MOV, MKV) to plain text plus .srt and .vtt',
        'Batch transcription. Drop a folder, walk away',
        'Full history with audio retention (1&ndash;90 day slider) and re-transcription with any mode',
        'Three themes &times; six accents, reduce-motion, sound cues',
        'Media auto-pause and full privacy controls',
    ];

    $faqs = [
        ['q' => 'Is it really free?', 'a' => 'Yes. Utterbar is free and open source, with every feature included. No trial, no paid tier, no upsell, no card. Download, install, talk.'],
        ['q' => 'What is the catch?', 'a' => 'There is not one. The app runs entirely on your PC and the source is open, so you can read exactly what it does. There is nothing to buy and nothing to subscribe to.'],
        ['q' => 'Do I need an account?', 'a' => 'No account, no sign-in, no email. Nothing to create, nothing to log in to.'],
        ['q' => 'What if the website goes down?', 'a' => 'Utterbar keeps working. Everything happens on your machine, while installers and updates are published through GitHub Releases. Offline, the app runs exactly the same.'],
        ['q' => 'Can I build from source?', 'a' => 'Yes. The complete source is public on <a href="' . config('site.github_repository_url') . '" target="_blank" rel="noopener" style="color: var(--accent);">GitHub</a> under the MIT licence, so you can inspect it, build it yourself, and contribute changes.'],
        ['q' => 'How can I support it?', 'a' => 'Use it, tell people, and open issues or pull requests. If you have a question, email <a href="mailto:' . config('site.email.contact') . '" style="color: var(--accent);">' . config('site.email.contact') . '</a>.'],
    ];

    $pricingDesc = 'Utterbar is free and open source. Unlimited local dictation, every feature included, no account, no subscription, no cloud.';
@endphp

<x-layouts.app title="Pricing" :description="$pricingDesc">
    @php
        // SoftwareApplication with a zero-price offer: honest "free"
        // signal for search engines now that there is no paid SKU.
        $pricingSchema = [
            '@context'            => 'https://schema.org',
            '@type'               => 'SoftwareApplication',
            'name'                => 'Utterbar',
            'operatingSystem'     => 'Windows 10, Windows 11',
            'applicationCategory' => 'UtilitiesApplication',
            'description'         => 'Windows-native, local-only dictation. Free and open source, with every feature included.',
            'url'                 => url('/pricing'),
            'image'               => url('/og/og-default.png'),
            'offers'              => [
                '@type'         => 'Offer',
                'url'           => url('/download'),
                'price'         => '0',
                'priceCurrency' => 'GBP',
                'availability'  => 'https://schema.org/InStock',
            ],
        ];
    @endphp
    @push('head')
        <script type="application/ld+json">{!! json_encode($pricingSchema, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) !!}</script>
    @endpush

    {{-- Hero --}}
    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-3xl">
                <p class="ub-eyebrow mb-5">Pricing</p>
                <h1 style="font-size: clamp(2.75rem, 6vw, 4.5rem); letter-spacing: -0.04em; line-height: 1.02; font-weight: 700;">
                    Free and open source.
                </h1>
                <p class="mt-6 text-xl" style="color: var(--muted); font-weight: 300; line-height: 1.5;">
                    Every feature, on for everyone. No account, no cloud, no subscription tax on the act of talking. Download it and speak.
                </p>
                <div class="mt-8 flex flex-wrap items-center gap-4">
                    <a href="{{ route('download') }}" class="ub-btn ub-btn-cta">
                        Download for Windows
                        <x-icon name="arrow-right" size="16" stroke="2" />
                    </a>
                    <a href="{{ config('site.github_repository_url') }}" target="_blank" rel="noopener" class="ub-btn ub-btn-ghost">
                        <x-icon name="github" size="17" stroke="1.8" />
                        View source
                    </a>
                </div>
                <p class="mt-4 font-mono text-[11px]" style="color: var(--fg-dim); letter-spacing: 0.08em;">
                    LOCAL-ONLY &middot; NO ACCOUNT &middot; NO TELEMETRY
                </p>
            </div>
        </div>
    </section>

    {{-- Everything included --}}
    <section class="py-16 md:py-20 border-t" style="border-color: var(--border);">
        <div class="ub-container">
            <div class="max-w-3xl">
                <h2 class="text-2xl md:text-3xl" style="letter-spacing: -0.025em; line-height: 1.15; font-weight: 700;">
                    Everything is included.
                </h2>
                <p class="mt-4 text-lg" style="color: var(--muted); font-weight: 300;">
                    There used to be a paid tier. There is not any more. Here is the whole app, free.
                </p>
                <ul class="mt-8 space-y-4">
                    @foreach ($included as $item)
                        <li class="flex items-start gap-3">
                            <span class="mt-1 shrink-0" style="color: var(--accent);">
                                <x-icon name="check" size="18" stroke="2.5" />
                            </span>
                            <span class="text-base" style="color: var(--fg); font-weight: 300; line-height: 1.5;">{!! $item !!}</span>
                        </li>
                    @endforeach
                </ul>
            </div>
        </div>
    </section>

    {{-- FAQ --}}
    <section class="py-16 md:py-24 border-t" style="border-color: var(--border);">
        <div class="ub-container">
            <div class="max-w-3xl">
                <h2 class="text-2xl md:text-3xl mb-10" style="letter-spacing: -0.025em; line-height: 1.15; font-weight: 700;">
                    Questions.
                </h2>
                <div class="space-y-8">
                    @foreach ($faqs as $faq)
                        <div>
                            <h3 class="text-lg" style="font-weight: 700; letter-spacing: -0.01em;">{{ $faq['q'] }}</h3>
                            <p class="mt-2 text-base" style="color: var(--muted); font-weight: 300; line-height: 1.55;">{!! $faq['a'] !!}</p>
                        </div>
                    @endforeach
                </div>

                <div class="mt-14">
                    <a href="{{ route('download') }}" class="ub-btn ub-btn-cta">
                        Download Utterbar
                        <x-icon name="arrow-right" size="16" stroke="2" />
                    </a>
                </div>
            </div>
        </div>
    </section>
</x-layouts.app>
