@php
    $freeFeatures = [
        'Unlimited dictation, any length',
        'Parakeet v3 (English, compact) &mdash; ~460 MB, runs on any modern CPU',
        'Whisper base (multilingual, 30+ languages) &mdash; ~145 MB',
        'Zipformer (English, lightweight) &mdash; ~220 MB, smallest footprint',
        'All seven hotkey slots',
        'All three built-in modes',
        'Microphone, system-audio (WASAPI loopback), mixed',
        '50 vocabulary words + 25 replacements',
        'History (last 100 transcriptions)',
        'Three themes &times; six accents, reduce-motion, sound cues',
        'Media auto-pause, full privacy controls',
    ];

    $proFeatures = [
        'Everything in Free',
        'Whisper small (multilingual, high accuracy) &mdash; ~610 MB, stronger non-English transcription',
        'Unlimited custom modes',
        'Unlimited vocabulary + replacements',
        'File transcription (WAV, MP3, M4A, FLAC, OGG, MP4, MOV, MKV &rarr; plain-text transcript)',
        'Batch transcription &mdash; drop a folder, walk away',
        'Full history + audio retention (1&ndash;90 day slider)',
        'Re-transcribe any recording with any mode',
        'Tip-jar isn&rsquo;t shown to Pro users (you&rsquo;ve already paid)',
        'Priority support',
    ];

    $faqs = [
        ['q' => 'Is dictation actually free forever?', 'a' => 'Yes. Every word you speak into Utterbar is transcribed, free, forever, unlimited length. The paid tier is for power-user ergonomics \u2014 file transcription, unlimited custom modes, full history with re-transcription \u2014 not for the basic act of talking.'],
        ['q' => 'Why a one-time price and not a subscription?', 'a' => 'Subscriptions are fine for software that keeps getting materially better every month. A dictation app that works is a dictation app that works \u2014 charging rent on it feels icky. Pay once; it\'s yours.'],
        ['q' => 'Do I need an account to use the free tier?', 'a' => 'No account, no sign-in, no email. Download, install, talk. Pro unlocks with a licence key that arrives by email after purchase.'],
        ['q' => 'What if your servers go down forever?', 'a' => 'Pro keeps working. Licence keys are verified locally (Ed25519-signed). The app doesn\'t phone home \u2014 if I disappeared tomorrow, your copy still runs.'],
        ['q' => 'How many devices?', 'a' => 'Three per licence. Desktop, laptop, spare. Need more, email me.'],
        ['q' => 'Can I self-host?', 'a' => 'It\'s already self-hosted. It runs on your PC. There\'s no server.'],
        ['q' => 'Discounts?', 'a' => 'Students and OSS maintainers: drop me a line at <a href="mailto:' . config('site.email.contact') . '" style="color: var(--accent);">' . config('site.email.contact') . '</a>.'],
    ];
@endphp

@php
    $pricingDesc = 'Utterbar pricing: free forever for unlimited local dictation. Pro is a one-time £39.99 lifetime licence — three devices, all Pro features, no subscription.';
@endphp

<x-layouts.app title="Pricing" :description="$pricingDesc">
    @php
        $pricingSchema = [
            '@context'    => 'https://schema.org',
            '@type'       => 'Product',
            'name'        => 'Utterbar Pro',
            'description' => 'Lifetime Pro licence for Utterbar. Unlocks unlimited custom modes, unlimited vocabulary, file transcription, full audio history, and Whisper small for higher-accuracy multilingual transcription.',
            'brand'       => ['@type' => 'Brand', 'name' => 'Utterbar'],
            'url'         => url('/pricing'),
            'image'       => url('/og/og-default.png'),
            'offers'      => [
                '@type'           => 'Offer',
                'url'             => url('/pricing'),
                'price'           => '39.99',
                'priceCurrency'   => 'GBP',
                'availability'    => 'https://schema.org/InStock',
                'priceValidUntil' => now()->addYear()->format('Y-m-d'),
                'seller'          => ['@id' => url('/#organization')],
            ],
        ];
    @endphp
    @push('head')
        <script type="application/ld+json">{!! json_encode($pricingSchema, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) !!}</script>
    @endpush

    {{-- Hero / intro --}}
    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <h1 class="ub-fg-hi" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Free <x-hl>forever</x-hl>.<br>Pro once, for life.
                </h1>
                <p class="ub-lead mt-8 max-w-xl">
                    The popular dictation apps are all good, all charging monthly to do a thing your PC could do on its own.
                    Utterbar does it for free. Power-user features are a one-time purchase, if you want them.
                </p>
            </div>
        </div>
    </section>

    {{-- Unified pricing block, centered --}}
    <section class="pb-6">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto rounded-2xl overflow-hidden" style="background: var(--bg-1);">
                <div class="grid grid-cols-1 md:grid-cols-2 items-stretch">

                    {{-- FREE --}}
                    <div class="p-8 md:p-10 relative flex flex-col">
                        <div class="flex items-baseline justify-between">
                            <h2 class="text-2xl md:text-3xl">Free</h2>
                            <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted);">forever</span>
                        </div>

                        <div class="mt-6 flex items-baseline gap-2">
                            <span class="text-5xl md:text-6xl font-extrabold tracking-tight" style="color: var(--fg-hi);">&pound;0</span>
                        </div>
                        <p class="mt-2 text-sm" style="color: var(--muted);">No account. No sign-in. No trial clock.</p>

                        <ul class="mt-8 space-y-3 flex-1">
                            @foreach ($freeFeatures as $f)
                                <li class="flex items-start gap-2.5 text-sm">
                                    <span class="mt-0.5 shrink-0" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span>
                                    <span style="color: var(--fg);">{!! $f !!}</span>
                                </li>
                            @endforeach
                        </ul>

                        <div class="mt-10">
                            <a href="{{ route('download') }}" class="ub-btn ub-btn-ghost w-full">Download Utterbar</a>
                        </div>
                    </div>

                    {{-- PRO --}}
                    <div class="p-8 md:p-10 relative flex flex-col"
                         style="background:
                            radial-gradient(120% 80% at 100% 0%, color-mix(in oklab, var(--accent) 12%, transparent), transparent 60%),
                            color-mix(in oklab, var(--bg-2) 80%, var(--bg-1));">
                        <div class="flex items-baseline justify-between">
                            <h2 class="text-2xl md:text-3xl ub-gradient-text">Pro</h2>
                            <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--accent);">lifetime licence</span>
                        </div>

                        <div class="mt-6 flex items-baseline gap-2 flex-wrap">
                            <span class="text-5xl md:text-6xl font-extrabold tracking-tight ub-gradient-text">&pound;39.99</span>
                            <span class="text-base" style="color: var(--muted);">once, for life</span>
                        </div>
                        <p class="mt-2 font-mono text-xs" style="color: var(--muted);">GBP &middot; your local currency at checkout &middot; three devices</p>

                        <ul class="mt-8 space-y-3 flex-1">
                            @foreach ($proFeatures as $f)
                                <li class="flex items-start gap-2.5 text-sm">
                                    <span class="mt-0.5 shrink-0" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span>
                                    <span style="color: var(--fg);">{!! $f !!}</span>
                                </li>
                            @endforeach
                        </ul>

                        {{-- Lifetime + source-if-sunset commitment. Understated on purpose. --}}
                        <p class="mt-8 pt-6 text-xs leading-relaxed" style="color: var(--muted); border-top: 1px solid var(--border);">
                            <span style="color: var(--fg);">Lifetime means lifetime.</span>
                            You keep Pro forever, on every future Utterbar release. And if this project ever shuts down, the source goes public &mdash; GPL, on GitHub, the day the lights go out.
                        </p>

                        <div class="mt-8" x-data="{ busy: false, err: '' }">
                            @if (config('services.stripe.enabled'))
                                <button type="button" class="ub-btn ub-btn-cta w-full"
                                        :disabled="busy"
                                        @click="
                                            busy = true; err = '';
                                            try {
                                                const res = await fetch('{{ route('checkout.create') }}', {
                                                    method: 'POST',
                                                    headers: {
                                                        'Accept': 'application/json',
                                                        'Content-Type': 'application/json',
                                                        'X-CSRF-TOKEN': '{{ csrf_token() }}',
                                                        'X-Requested-With': 'XMLHttpRequest',
                                                    },
                                                    credentials: 'same-origin',
                                                    body: '{}',
                                                });
                                                const data = await res.json().catch(() => ({}));
                                                if (!res.ok || !data.url) { err = data.message || 'Checkout failed. Try again?'; busy = false; return; }
                                                window.location.href = data.url;
                                            } catch (e) { err = 'Network error.'; busy = false; }">
                                    <span x-show="!busy">Get Utterbar Pro</span>
                                    <span x-show="busy">One moment&hellip;</span>
                                </button>
                                <p x-show="err" x-text="err" class="mt-3 text-sm text-center" style="color: var(--danger);"></p>
                            @else
                                <a href="{{ route('contact') }}" class="ub-btn ub-btn-cta w-full">Contact us</a>
                            @endif
                        </div>
                    </div>
                </div>
            </div>

            <p class="mt-6 text-center font-mono text-xs max-w-xl mx-auto" style="color: var(--muted);">
                Lifetime licence &middot; three devices &middot; refund within 14 days, no questions.
            </p>

            {{-- Tip jar for the Free tier — sits below the pricing tiles
                 so anyone choosing Free still has a visible way to chip
                 in. Same component as home + download. Auto-hides when
                 STRIPE_TIP_PRICE_ID isn't set. --}}
            <div class="mt-12 max-w-3xl mx-auto">
                <x-tip-jar />
            </div>
        </div>
    </section>

    {{-- What Pro unlocks — file transcription demo + Files route still --}}
    <section class="py-20 md:py-24">
        <div class="ub-container">
            <div class="max-w-3xl mx-auto text-center mb-12 md:mb-14">
                <p class="ub-eyebrow">Pro in practice</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.1;">
                    File transcription, the whole thing.
                </h2>
                <p class="ub-lead mt-5">
                    Drop in meetings, podcasts, old lectures. Same local pipeline, same privacy guarantees. The transcript lands as a plain-text file next to the source.
                </p>
            </div>
            {{-- Single video, full width within the wider container.
                 Previous layout had a still and the same video side by
                 side which made both small and unreadable; the still
                 didn't add anything the video doesn't already show. --}}
            <div class="max-w-4xl mx-auto">
                <x-video-embed id="feature-file-transcription" />
            </div>
        </div>
    </section>

    <x-section eyebrow="Questions" title="Pricing FAQ." align="center">
        <div class="max-w-3xl mx-auto">
            <x-faq-accordion :items="$faqs" />
        </div>
    </x-section>

    <x-prompt-cta
        :heading="'Changelog in your inbox.'"
        :sub="'Drop your email if you&rsquo;d like release notes and the odd behind-the-scenes note. One-click unsubscribe, no marketing.'"
    />
</x-layouts.app>
