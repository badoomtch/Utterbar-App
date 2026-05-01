@php
    $freeFeatures = [
        'Unlimited dictations, no daily quota (single-shot capped at 5 minutes; Meeting Mode chunks longer recordings)',
        'Parakeet v3 (English, compact). ~460 MB, runs on any modern CPU',
        'Whisper base (multilingual, 30+ languages). ~145 MB',
        'All seven hotkey slots',
        'All three built-in modes',
        'Microphone, system-audio (WASAPI loopback), mixed',
        '50 vocabulary words and 25 replacements',
        'History (last 100 transcriptions)',
        'Three themes &times; six accents, reduce-motion, sound cues',
        'Media auto-pause, full privacy controls',
    ];

    $proFeatures = [
        'Everything in Free',
        'Whisper small (multilingual, higher accuracy). ~610 MB, the stronger non-English engine',
        'Unlimited custom modes',
        'Unlimited vocabulary and replacements',
        'File transcription (WAV, MP3, M4A, FLAC, OGG, MP4, MOV, MKV). Plain-text plus .srt and .vtt',
        'Batch transcription. Drop a folder, walk away',
        'Full history with audio retention (1&ndash;90 day slider)',
        'Re-transcribe any recording with any mode',
        'No tip jar (you&rsquo;ve already paid)',
        'Priority support',
    ];

    $faqs = [
        ['q' => 'Is dictation actually free forever?', 'a' => 'Yes. Every word you speak into Utterbar is transcribed, free, forever, unlimited length. The paid tier is for power-user ergonomics: file transcription, unlimited custom modes, full history with re-transcription. The basic act of talking stays free.'],
        ['q' => 'Why a one-time price and not a subscription?', 'a' => 'Subscriptions are fine for software that keeps getting materially better every month. A dictation app that works is a dictation app that works. Charging rent on it feels off. Pay once, it\'s yours.'],
        ['q' => 'Do I need an account to use the free tier?', 'a' => 'No account, no sign-in, no email. Download, install, talk. Pro unlocks with a licence key that arrives by email after purchase.'],
        ['q' => 'What if your servers go down forever?', 'a' => 'Pro keeps working. Licence keys are verified locally (Ed25519-signed). The app doesn\'t phone home. If I disappeared tomorrow, your copy still runs.'],
        ['q' => 'How many devices?', 'a' => 'Three per licence. Desktop, laptop, spare. Need more, email me.'],
        ['q' => 'Can I self-host?', 'a' => 'It\'s already self-hosted. It runs on your PC. There\'s no server.'],
        ['q' => 'Discounts?', 'a' => 'Students and OSS maintainers: drop me a line at <a href="mailto:' . config('site.email.contact') . '" style="color: var(--accent);">' . config('site.email.contact') . '</a>.'],
    ];
@endphp

@php
    $pricingDesc = 'Utterbar pricing. Free forever for unlimited local dictation. Pro is a one-time £39.99 lifetime licence: three devices, all Pro features, no subscription.';
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
                            You keep Pro forever, on every future Utterbar release. And if this project ever shuts down, the source goes public: GPL, on GitHub, the day the lights go out.
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

            {{-- Tip jar for the Free tier. Sits below the pricing tiles
                 so anyone choosing Free still has a visible way to chip
                 in. Same component as home + download. Auto-hides when
                 STRIPE_TIP_PRICE_ID isn't set. --}}
            <div class="mt-12 max-w-3xl mx-auto">
                <x-tip-jar />
            </div>
        </div>
    </section>

    {{-- Volume / business pricing. Three visible tiers, fourth is
         contact-us. Lives BELOW the consumer Free/Pro block so the
         home buyer doesn't have to wade through team pricing to get
         to "download for £0". --}}
    <section class="py-20 md:py-24" style="background: var(--bg-1);">
        <div class="ub-container">
            <div class="max-w-3xl mb-12">
                <p class="ub-eyebrow">Buying for a team?</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Volume pricing, <x-hl>still lifetime</x-hl>.
                </h2>
                <p class="ub-lead mt-6 max-w-2xl">
                    Same product, per seat, in plain GBP. Pay by card or invoice. Volume discounts kick in at three seats and again at twenty-five. No subscription, no per-user accounts to provision.
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-5 max-w-5xl">
                <div class="ub-card">
                    <p class="ub-eyebrow mb-3">Team</p>
                    <div class="flex items-baseline gap-1">
                        <span class="text-4xl font-extrabold tracking-tight" style="color: var(--fg-hi);">&pound;29</span>
                        <span class="text-sm" style="color: var(--muted);">per seat</span>
                    </div>
                    <p class="mt-3 text-sm" style="color: var(--muted);">5 to 24 seats, lifetime.</p>
                </div>
                <div class="ub-card relative" style="background:
                        radial-gradient(120% 80% at 100% 0%, color-mix(in oklab, var(--accent) 12%, transparent), transparent 60%),
                        var(--bg-2);">
                    <p class="ub-eyebrow mb-3" style="color: var(--accent);">Business</p>
                    <div class="flex items-baseline gap-1">
                        <span class="text-4xl font-extrabold tracking-tight ub-gradient-text">&pound;19</span>
                        <span class="text-sm" style="color: var(--muted);">per seat</span>
                    </div>
                    <p class="mt-3 text-sm" style="color: var(--muted);">25 to 100 seats, lifetime.</p>
                </div>
                <div class="ub-card">
                    <p class="ub-eyebrow mb-3">Enterprise</p>
                    <div class="flex items-baseline gap-1">
                        <span class="text-2xl font-extrabold tracking-tight" style="color: var(--fg-hi);">Talk to us</span>
                    </div>
                    <p class="mt-3 text-sm" style="color: var(--muted);">100+ seats, custom quote.</p>
                </div>
            </div>

            {{-- Self-serve volume checkout. Quantity selector backed by
                 the same VolumePricing helper the controller and webhook
                 read, so the displayed total is exactly what Stripe
                 will charge. Hidden when Stripe is disabled (sandbox /
                 dev) — same gate as the consumer Pro button. --}}
            @php
                $vp = \App\Services\Pricing\VolumePricing::class;
                $minSeats = $vp::MIN_SEATS;
                $maxSeats = $vp::MAX_SEATS;
                $threshold = $vp::BUSINESS_TIER_THRESHOLD;
                $teamPence = (int) config('services.stripe.volume_per_seat_pence', 2900);
                $bizPence  = (int) config('services.stripe.volume_per_seat_business_pence', 1900);
                // £29 / £19 with no decimals when whole pounds.
                $teamGbp = number_format($teamPence / 100, $teamPence % 100 ? 2 : 0);
                $bizGbp  = number_format($bizPence / 100, $bizPence % 100 ? 2 : 0);
            @endphp

            @if (config('services.stripe.enabled'))
                <div class="mt-10 ub-card max-w-2xl"
                     x-data="{
                        qty: {{ $minSeats }},
                        min: {{ $minSeats }},
                        max: {{ $maxSeats }},
                        threshold: {{ $threshold }},
                        teamPence: {{ $teamPence }},
                        bizPence: {{ $bizPence }},
                        busy: false,
                        err: '',
                        unitPence() { return this.qty >= this.threshold ? this.bizPence : this.teamPence; },
                        unitGbp()   { const p = this.unitPence(); return (p / 100).toFixed(p % 100 ? 2 : 0); },
                        totalGbp()  { const p = this.unitPence() * this.qty; return (p / 100).toFixed(p % 100 ? 2 : 0); },
                        clamp() {
                            let n = parseInt(this.qty, 10);
                            if (!Number.isFinite(n)) n = this.min;
                            if (n < this.min) n = this.min;
                            if (n > this.max) n = this.max;
                            this.qty = n;
                        },
                        async checkout() {
                            this.clamp();
                            this.busy = true; this.err = '';
                            try {
                                const res = await fetch('{{ route('checkout.create.volume') }}', {
                                    method: 'POST',
                                    headers: {
                                        'Accept': 'application/json',
                                        'Content-Type': 'application/json',
                                        'X-CSRF-TOKEN': '{{ csrf_token() }}',
                                        'X-Requested-With': 'XMLHttpRequest',
                                    },
                                    credentials: 'same-origin',
                                    body: JSON.stringify({ quantity: this.qty }),
                                });
                                const data = await res.json().catch(() => ({}));
                                if (!res.ok || !data.url) {
                                    this.err = data.message || 'Checkout failed. Try again?';
                                    this.busy = false;
                                    return;
                                }
                                window.location.href = data.url;
                            } catch (e) {
                                this.err = 'Network error.';
                                this.busy = false;
                            }
                        }
                     }">
                    <p class="ub-eyebrow mb-3">Buy now (5 to {{ $maxSeats }} seats)</p>
                    <p class="mt-2 text-sm" style="color: var(--muted);">
                        One licence covers your selected device allowance.
                        {{ $minSeats }} to {{ $threshold - 1 }} seats are &pound;{{ $teamGbp }} per seat.
                        {{ $threshold }} to {{ $maxSeats }} seats are &pound;{{ $bizGbp }} per seat.
                        Need more than {{ $maxSeats }}? <a href="{{ route('business.contact') }}" style="color: var(--accent);">Get a quote</a>.
                    </p>

                    <div class="mt-5 flex flex-wrap items-center gap-4">
                        <label class="flex items-center gap-2 text-sm" style="color: var(--fg);">
                            <span class="ub-eyebrow" style="margin: 0;">Seats</span>
                            <input type="number"
                                   x-model.number="qty"
                                   @input="clamp()"
                                   @change="clamp()"
                                   :min="min"
                                   :max="max"
                                   step="1"
                                   class="ub-input font-mono"
                                   style="width: 6rem; text-align: right;">
                        </label>
                        <div class="text-sm" style="color: var(--muted);">
                            &pound;<span x-text="unitGbp()"></span> per seat
                            &middot;
                            <span style="color: var(--fg-hi);">total &pound;<span x-text="totalGbp()"></span></span>
                            <span class="font-mono text-xs" style="color: var(--muted); margin-left: 0.25rem;">GBP</span>
                        </div>
                    </div>

                    <div class="mt-5">
                        <button type="button"
                                class="ub-btn ub-btn-cta"
                                :disabled="busy"
                                @click="checkout()">
                            <span x-show="!busy">Buy <span x-text="qty"></span> seats &mdash; &pound;<span x-text="totalGbp()"></span></span>
                            <span x-show="busy">One moment&hellip;</span>
                        </button>
                        <p x-show="err" x-text="err" class="mt-3 text-sm" style="color: var(--danger);"></p>
                    </div>

                    <p class="mt-5 text-xs" style="color: var(--muted);">
                        Lifetime licence. One key per order. Each device activates once.
                        Need to deploy across a fleet without users pasting the key?
                        Your IT admin can download a signed
                        <code class="font-mono">licence.bin</code> from the admin licence
                        page after purchase &mdash; see
                        <a href="{{ route('docs.it-deployment') }}" style="color: var(--accent);">IT deployment</a>.
                    </p>
                </div>
            @endif

            <div class="mt-10 flex flex-wrap items-center gap-4">
                <a href="{{ route('business.contact') }}" class="ub-btn ub-btn-ghost">Need {{ $maxSeats }}+ seats? Get a quote</a>
                <a href="{{ route('business') }}" class="ub-btn ub-btn-ghost">See the business page</a>
                <a href="{{ route('docs.it-deployment') }}" class="ub-btn ub-btn-ghost">IT deployment notes</a>
            </div>
        </div>
    </section>

    {{-- What Pro unlocks. File transcription demo + Files route still. --}}
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
