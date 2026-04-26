@php
    $features = [
        ['num' => '01', 'title' => 'Press, speak, done.',    'body' => 'Hold one key. Talk. Release. Your words land where the cursor is.'],
        ['num' => '02', 'title' => 'Modes for the job.',     'body' => 'Plain voice-to-text, cleaned-up chat voice, or verbatim meeting capture. Switch mode in a click or a hotkey.'],
        ['num' => '03', 'title' => 'Hears everything.',      'body' => 'Your mic, the audio your PC is playing, or both. No virtual cable to install.'],
        ['num' => '04', 'title' => 'Learns your words.',     'body' => 'Names, acronyms, product terms. Your custom list stops them being mangled.'],
        ['num' => '05', 'title' => 'Mutes your music.',      'body' => 'Whatever is playing pauses the moment you press the hotkey. Resumes on stop.'],
        ['num' => '06', 'title' => 'No cloud. Ever.',        'body' => 'No account, no telemetry, no phone-home. Your voice never leaves the PC.'],
    ];

    $faqs = [
        ['q' => 'Does Utterbar send my voice to the cloud?', 'a' => 'No. Audio is captured, transcribed, and binned on your machine. The only network calls it ever makes are model downloads on first run \u2014 and those are logged so you can read them.'],
        ['q' => 'Which Windows versions work?', 'a' => 'Windows 10 22H2 and Windows 11, x86_64. WebView2 is required (already on Windows 11; auto-bootstrapped on Windows 10).'],
        ['q' => 'Do I need a GPU?', 'a' => 'No. The default Parakeet v3 build runs comfortably on any modern CPU. GPU acceleration (DirectML on any DX12 card, CUDA on NVIDIA) is supported and speeds things up if you have one, but it isn\'t required.'],
        ['q' => 'What languages are supported?', 'a' => 'English on Parakeet (the default, fastest), plus around 99 languages on Whisper (free for the base model, Pro for the higher-accuracy small build). Auto-detect is on by default; pin a specific language in Settings \u2192 Models for a small speed gain.'],
        ['q' => 'How does it compare to the Mac-native ones?', 'a' => 'Same philosophy, different platform. The Mac alternatives are great if you\'re on Mac. Utterbar is Windows-native from day one \u2014 no Python shim, no Electron wrapper, smaller installer. Different camp, same conviction about local.'],
        ['q' => 'Why not a more common speech model?', 'a' => 'The one I picked is faster and more accurate on English than the common alternatives at the same size. Other models come in later for multilingual support.'],
        ['q' => 'Can I use this in video-call and chat apps?', 'a' => 'Only for typing into the chat box. Utterbar transcribes your voice into text; it won\'t (and will never) inject audio into a call. Your colleagues hear nothing from Utterbar.'],
        ['q' => 'Does it work offline?', 'a' => 'Yes. Pull your ethernet cable mid-sentence and nothing changes. The app doesn\'t authenticate against anything.'],
        ['q' => 'Will there be a macOS version?', 'a' => 'Eventually, after Windows 1.0 ships. Mac already has decent local-only options; I\'d rather make Windows actually good first.'],
        ['q' => 'What happens to my data when I uninstall?', 'a' => '<code>%APPDATA%\\Utterbar\\</code> stays on disk so you can re-install without losing history. Delete the folder yourself for a clean wipe.'],
        ['q' => 'Prove it really runs local.', 'a' => 'Start the app. Unplug your ethernet. Keep dictating. That\'s the proof. If you want harder evidence, sniff the network during a session \u2014 there isn\'t any after the model is downloaded.'],
    ];
@endphp

@php
    // Title is the single most important on-page SEO signal for the home
    // page. Lead with the query buyers actually type ("local dictation
    // for Windows"), not a tagline. Layout wraps with " — Utterbar".
    $homeTitle = 'Local dictation for Windows';
    $homeDescription = 'Local-only Windows dictation. Press a hotkey, speak, your words land anywhere. Free forever, no cloud, no account. Pro unlocks power features, £39.99 lifetime.';
@endphp

<x-layouts.app :title="$homeTitle" :description="$homeDescription">
    @php
        $homeSchema = [
            '@context' => 'https://schema.org',
            '@graph' => [
                [
                    '@type'               => 'SoftwareApplication',
                    '@id'                 => url('/#software'),
                    'name'                => config('site.name'),
                    'url'                 => url('/'),
                    'description'         => $homeDescription,
                    'applicationCategory' => 'ProductivityApplication',
                    'operatingSystem'     => 'Windows 10, Windows 11',
                    'image'               => url('/og/og-default.png'),
                    'downloadUrl'         => url('/download'),
                    'softwareVersion'     => ($latestRelease ?? null)?->version ?? config('site.version'),
                    'offers' => [
                        [
                            '@type'         => 'Offer',
                            'name'          => 'Utterbar Free',
                            'price'         => '0',
                            'priceCurrency' => 'GBP',
                            'availability'  => 'https://schema.org/InStock',
                        ],
                        [
                            '@type'         => 'Offer',
                            'name'          => 'Utterbar Pro (Lifetime)',
                            'price'         => '39.99',
                            'priceCurrency' => 'GBP',
                            'availability'  => 'https://schema.org/InStock',
                            'url'           => url('/pricing'),
                        ],
                    ],
                    'author'    => ['@id' => url('/#organization')],
                    'publisher' => ['@id' => url('/#organization')],
                ],
                [
                    '@type'      => 'FAQPage',
                    '@id'        => url('/#faq'),
                    'mainEntity' => collect($faqs)->map(fn ($f) => [
                        '@type'          => 'Question',
                        'name'           => strip_tags(str_replace(['\\u2014', '&mdash;', '&rsquo;'], ['—', '—', "'"], $f['q'])),
                        'acceptedAnswer' => [
                            '@type' => 'Answer',
                            'text'  => strip_tags(str_replace(['\\u2014', '&mdash;', '&rsquo;'], ['—', '—', "'"], $f['a'])),
                        ],
                    ])->values()->all(),
                ],
            ],
        ];
    @endphp
    @push('head')
        <script type="application/ld+json">{!! json_encode($homeSchema, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) !!}</script>
    @endpush

    {{-- Hero: brand plate, headline, punchy subhead --}}
    <section class="relative overflow-hidden">
        <div class="ub-hero-glow" aria-hidden="true"></div>
        <div class="relative ub-container pt-20 md:pt-28 pb-24 md:pb-32">
            <div class="max-w-5xl">

                {{-- Brand plate: the wordmark, scaled as a display element --}}
                <div style="font-size: clamp(2.5rem, 5.5vw, 4.5rem); letter-spacing: -0.045em; line-height: 1; font-weight: 600;">
                    <x-wordmark />
                </div>

                {{-- Headline. Keeps the brand voice ("stays on your PC")
                     while also carrying the primary keyword phrase
                     ("Local dictation for Windows") for SEO. The H1 is
                     a single sentence visually but split for emphasis. --}}
                <h1 class="mt-6" style="font-size: clamp(2.75rem, 6.25vw, 5.25rem); letter-spacing: -0.045em; line-height: 1.0; font-weight: 800;">
                    <span class="ub-fg-hi">Local dictation for Windows.</span>
                    <span class="block mt-2" style="color: var(--muted); font-weight: 600;">Stays on your PC.</span>
                </h1>

                {{-- Punchy subhead: cyan full-stops, thin weight --}}
                <p class="mt-6" style="font-size: clamp(1.5rem, 3.2vw, 2.5rem); letter-spacing: -0.03em; line-height: 1.15; font-weight: 300; color: var(--fg);">
                    No cloud<span style="color: var(--accent);">.</span> No nonsense<span style="color: var(--accent);">.</span>
                </p>

                <div class="mt-12 flex flex-wrap gap-3">
                    <a href="{{ route('download') }}" class="ub-btn ub-btn-cta">
                        Download for Windows
                        <x-icon name="arrow-right" size="16" stroke="2" />
                    </a>
                    <a href="{{ route('pricing') }}" class="ub-btn ub-btn-ghost">
                        See pricing
                    </a>
                </div>
                @if ($latestRelease ?? null)
                    <p class="mt-5 font-mono text-xs" style="color: var(--muted); letter-spacing: 0.04em;">
                        Currently: v{{ $latestRelease->version }} &middot; {{ $latestRelease->sizeLabel() }} &middot; Windows 10/11
                    </p>
                @endif
            </div>

            {{-- Interactive pill widget: hover/tap to expand, drag to move --}}
            <div class="mt-20 md:mt-24 flex justify-center">
                <div x-data="heroPill"
                     class="ub-pill-widget"
                     :class="{ 'is-dragging': dragging }"
                     :style="`transform: translate(${x}px, ${y}px) rotate(${tilt}deg);`"
                     @pointerdown="onDown($event)">

                    {{-- "drag me" hint — bobs, fades on first interaction --}}
                    <div class="ub-pill-hint" :class="{ 'is-gone': hasInteracted }" aria-hidden="true">
                        <div class="ub-pill-hint-inner">
                            <span class="ub-pill-hint-label">Drag me</span>
                            <svg class="ub-pill-hint-arrow" viewBox="0 0 48 62" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M 38 6 C 44 20, 40 38, 20 52" />
                                <path d="M 22 42 L 20 52 L 30 50" />
                            </svg>
                        </div>
                    </div>

                    {{-- Idle-sway wrapper — gently rocks the pill so it reads as "loose / movable" --}}
                    <div class="ub-pill-sway" :class="{ 'is-settled': hasInteracted }">

                    <div class="ub-pill"
                         :class="{ 'is-expanded': expanded, 'is-dragging': dragging }"
                         @mouseenter="onEnter()"
                         @mouseleave="onLeave()"
                         @click="onTap($event)">

                        {{-- Left: glyph contents (hole-punch + waveform) + wordmark --}}
                        <div class="ub-pill-left">
                            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 60" width="88" height="44" aria-hidden="true">
                                <defs>
                                    <linearGradient id="pill-widget-wave" x1="0%" y1="0%" x2="0%" y2="100%" color-interpolation="linearRGB">
                                        <stop offset="0%"   stop-color="#00f6ff"/>
                                        <stop offset="50%"  stop-color="#00ccf5"/>
                                        <stop offset="100%" stop-color="#00a3ff"/>
                                    </linearGradient>
                                </defs>
                                <circle cx="20" cy="30" r="8" fill="#ffffff"/>
                                <g fill="url(#pill-widget-wave)">
                                    <rect x="44"  y="25" width="4" height="10" rx="2"/>
                                    <rect x="54"  y="21" width="4" height="18" rx="2"/>
                                    <rect x="64"  y="16" width="4" height="28" rx="2"/>
                                    <path d="M 74 10 A 2 2 0 0 1 78 10 V 50 L 74 44 Z"/>
                                    <rect x="84"  y="16" width="4" height="28" rx="2"/>
                                    <rect x="94"  y="21" width="4" height="18" rx="2"/>
                                    <rect x="104" y="25" width="4" height="10" rx="2"/>
                                </g>
                            </svg>
                            <span class="ub-pill-wordmark">Utter<span class="ub-wm-bar">bar</span></span>
                        </div>

                        {{-- Right: revealed on hover/tap --}}
                        <div class="ub-pill-right">
                            <span class="ub-pill-version">v{{ ($latestRelease ?? null)?->version ?? config('site.version') }}</span>
                            <div class="ub-pill-wave" aria-hidden="true">
                                <span></span><span></span><span></span><span></span><span></span><span></span><span></span>
                            </div>
                            <a href="{{ route('download') }}" class="ub-pill-cta" @click.stop>
                                Download
                                <x-icon name="arrow-right" size="14" stroke="2.2" />
                            </a>
                        </div>
                    </div>
                    </div>{{-- /.ub-pill-sway --}}
                </div>
            </div>

        </div>
    </section>

    @push('scripts')
    <script>
        document.addEventListener('alpine:init', () => {
            window.Alpine.data('heroPill', () => ({
                x: 0, y: 0, tilt: 0,
                startX: 0, startY: 0,
                originX: 0, originY: 0,
                dragging: false,
                expanded: false,
                touchMode: false,
                pointerId: null,
                moveThreshold: 4,
                moved: false,
                hasInteracted: false,

                onEnter() {
                    if (!this.touchMode) this.expanded = true;
                    this.hasInteracted = true;
                },
                onLeave() { if (!this.touchMode && !this.dragging) this.expanded = false; },
                onTap(e) {
                    // Only toggle on tap if we didn't move (i.e. not a drag)
                    if (this.moved) { e.preventDefault(); return; }
                    if (this.touchMode) this.expanded = !this.expanded;
                    this.hasInteracted = true;
                },

                onDown(e) {
                    // Ignore if clicking the internal CTA link
                    if (e.target.closest('a')) return;
                    this.touchMode = e.pointerType === 'touch';
                    this.dragging = true;
                    this.moved = false;
                    this.pointerId = e.pointerId;
                    this.startX = e.clientX;
                    this.startY = e.clientY;
                    this.originX = this.x;
                    this.originY = this.y;
                    this.hasInteracted = true;

                    // Horizontal drag bounds: clamp to viewport (pill can't escape left/right).
                    const rect = this.$el.getBoundingClientRect();
                    const navH = 100;   // nav bar (80px) + 20px breathing room
                    const pad  = 20;    // safety margin
                    this.minX = pad - rect.left + this.x;
                    this.maxX = window.innerWidth  - pad - rect.right  + this.x;

                    // Vertical bounds: document-space, so pill can be dragged down
                    // past the viewport and onto the rest of the page. Top is the
                    // fixed nav (doc-space: navH from page top). Bottom is the top
                    // of <footer> (doc-space: footer's document-top position).
                    const scrollY   = window.scrollY || document.documentElement.scrollTop || 0;
                    const pillDocTop    = rect.top    + scrollY;
                    const pillDocBottom = rect.bottom + scrollY;
                    const footer = document.querySelector('footer');
                    const footerDocTop = footer
                        ? footer.getBoundingClientRect().top + scrollY
                        : document.documentElement.scrollHeight;  // fallback

                    // Translation-y range: pill's doc position = pillDocTop + (y - originY).
                    // Constrain pillDocTop + translation >= navH (no going under the nav at doc top)
                    // and pillDocBottom + translation <= footerDocTop - pad (no entering footer).
                    this.minY = (navH - pillDocTop)         + this.y;
                    this.maxY = (footerDocTop - pillDocBottom - pad) + this.y;

                    e.target.setPointerCapture?.(e.pointerId);
                    document.addEventListener('pointermove', this.onMoveBound = this.onMove.bind(this));
                    document.addEventListener('pointerup', this.onUpBound = this.onUp.bind(this));
                    document.addEventListener('pointercancel', this.onUpBound);
                },

                onMove(e) {
                    if (!this.dragging) return;
                    const dx = e.clientX - this.startX;
                    const dy = e.clientY - this.startY;
                    if (!this.moved && Math.hypot(dx, dy) > this.moveThreshold) this.moved = true;
                    const nextX = this.originX + dx;
                    const nextY = this.originY + dy;
                    this.x = Math.max(this.minX, Math.min(this.maxX, nextX));
                    this.y = Math.max(this.minY, Math.min(this.maxY, nextY));
                    // Tiny tilt as it moves, for physicality
                    this.tilt = Math.max(-4, Math.min(4, dx * 0.02));

                    // Auto-scroll the page when dragging near the viewport edges
                    // so the user can drag the pill all the way down to the footer
                    // without lifting the pointer.
                    const edge = 80;
                    const speed = 12;
                    if (e.clientY > window.innerHeight - edge) {
                        window.scrollBy(0, speed);
                    } else if (e.clientY < edge) {
                        window.scrollBy(0, -speed);
                    }
                },

                onUp(e) {
                    this.dragging = false;
                    // Snap tilt back to 0
                    this.tilt = 0;
                    document.removeEventListener('pointermove', this.onMoveBound);
                    document.removeEventListener('pointerup', this.onUpBound);
                    document.removeEventListener('pointercancel', this.onUpBound);
                    // Small delay so the click handler can check `moved`
                    setTimeout(() => { this.moved = false; }, 50);
                },
            }));
        });
    </script>
    @endpush

    {{-- Position strip: one quiet sentence of conviction, no box --}}
    <section class="py-20 md:py-24">
        <div class="ub-container">
            <p class="max-w-3xl text-xl md:text-2xl leading-relaxed" style="font-weight: 300;">
                <span class="ub-fg-hi ub-neon-underline" style="font-weight: 500;">Most dictation apps charge rent on your voice.</span>
                <span style="color: var(--muted);"> Utterbar runs entirely on your machine, never opens a socket after install, and the core is free forever. You pay once for the power-user features, or not at all.</span>
            </p>
        </div>
    </section>

    {{-- What the pill looks like: state-cycle loop --}}
    <section class="py-20 md:py-28">
        <div class="ub-container">
            <div class="max-w-3xl mb-10 md:mb-12">
                <p class="ub-eyebrow">The pill</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Four <x-hl>states</x-hl>, one pill.
                </h2>
                <p class="ub-lead mt-6 max-w-xl">
                    Idle when you&rsquo;re not using it, warming when it wakes, listening while you talk, processing while it thinks. That&rsquo;s the whole UI.
                </p>
            </div>
            <div class="max-w-4xl mx-auto">
                <x-video-embed id="hero-pill-loop" />
            </div>
        </div>
    </section>

    {{-- Features: 3x2 grid, punchy one-liners --}}
    <section class="py-20 md:py-28">
        <div class="ub-container">
            <div class="max-w-3xl mb-14 md:mb-16">
                <h2 style="font-size: clamp(2rem, 4.5vw, 3.5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Six <x-hl>reasons</x-hl> it&rsquo;s different.
                </h2>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
                @foreach ($features as $f)
                    <x-feature-card :num="$f['num']" :title="$f['title']" :body="$f['body']" />
                @endforeach
            </div>
        </div>
    </section>

    {{-- How it works --}}
    <x-section id="how" eyebrow="How it works" title="Three steps. No ceremony.">
        <x-how-it-works-stepper />

        {{-- One utterance, three shapes. The stepper teaches the flow; this shows what the modes actually DO with your voice. --}}
        <div class="mt-16 md:mt-20 max-w-5xl mx-auto">
            <x-video-embed id="feature-modes" />
        </div>
    </x-section>

    {{-- The app itself --}}
    <section class="py-20 md:py-28">
        <div class="ub-container">
            <div class="max-w-3xl mb-10 md:mb-12">
                <p class="ub-eyebrow">The app</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    A real Windows <x-hl>app</x-hl>, not a wrapper.
                </h2>
                <p class="ub-lead mt-6 max-w-xl">
                    Dashboard for session metrics, modes you actually use, a vocabulary that learns your names. Dark theme out of the box, native feel throughout.
                </p>
            </div>
            <div class="max-w-3xl mx-auto">
                <x-still id="press-03-main-shell-dashboard" />
            </div>
        </div>
    </section>

    {{-- Vocabulary showcase --}}
    <section class="py-20 md:py-28">
        <div class="ub-container">
            <div class="max-w-3xl mb-10 md:mb-12">
                <p class="ub-eyebrow">Learns your words</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Names, acronyms, product terms &mdash; <x-hl>fixed</x-hl>.
                </h2>
                <p class="ub-lead mt-6 max-w-xl">
                    Add your custom words and replacements once. Transcripts come out with &ldquo;J parrie&rdquo; as &ldquo;Jay Parry&rdquo; every single time, locally, on-device.
                </p>
            </div>
            <div class="max-w-5xl mx-auto">
                <x-video-embed id="feature-vocabulary" />
            </div>
        </div>
    </section>

    {{-- Files showcase --}}
    <section class="py-20 md:py-28">
        <div class="ub-container">
            <div class="max-w-3xl mb-10 md:mb-12">
                <p class="ub-eyebrow">Transcribe files too</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Drop a meeting in. Get <x-hl>clean text</x-hl> out.
                </h2>
                <p class="ub-lead mt-6 max-w-xl">
                    Existing recordings, podcasts, interviews &mdash; all transcribed on-device to a plain-text file next to the source. Same pipeline as the floating bar, just for files.
                </p>
            </div>
            <div class="max-w-5xl mx-auto">
                <x-video-embed id="feature-file-transcription" />
            </div>
        </div>
    </section>

    {{-- Pricing teaser --}}
    <x-section eyebrow="Pricing" title="Free forever. Pro once, for life." align="center">
        <p class="max-w-2xl mx-auto text-lg mb-10 text-center" style="color: var(--muted);">
            The act of speaking isn't a premium feature, and I won't charge for it. Free covers everything a normal person needs. Pro is for power users who want file transcription, unlimited custom modes, and full audio history.
        </p>

        <div class="max-w-4xl mx-auto rounded-2xl overflow-hidden" style="background: var(--bg-1);">
            <div class="grid grid-cols-1 md:grid-cols-2">

                {{-- FREE --}}
                <div class="p-8 md:p-10">
                    <div class="flex items-baseline justify-between">
                        <h3 class="text-2xl">Free</h3>
                        <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted);">forever</span>
                    </div>
                    <div class="mt-5 flex items-baseline gap-2">
                        <span class="text-4xl md:text-5xl font-extrabold tracking-tight" style="color: var(--fg-hi);">£0</span>
                    </div>
                    <p class="mt-2 text-sm" style="color: var(--muted);">No account. No trial clock.</p>
                    <ul class="mt-6 space-y-2.5">
                        @foreach (['Unlimited dictation, any length', 'Every built-in mode, deterministic post-processing', 'Mic, system-audio, and mixed capture', 'Vocabulary, history, privacy controls'] as $f)
                            <li class="flex items-start gap-2.5 text-sm">
                                <span class="mt-0.5 shrink-0" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span>
                                <span style="color: var(--fg);">{{ $f }}</span>
                            </li>
                        @endforeach
                    </ul>
                </div>

                {{-- PRO --}}
                <div class="p-8 md:p-10 relative"
                     style="background:
                        radial-gradient(120% 80% at 100% 0%, color-mix(in oklab, var(--accent) 12%, transparent), transparent 60%),
                        color-mix(in oklab, var(--bg-2) 80%, var(--bg-1));">
                    <div class="flex items-baseline justify-between">
                        <h3 class="text-2xl ub-gradient-text">Pro</h3>
                        <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--accent);">lifetime licence</span>
                    </div>
                    <div class="mt-5 flex items-baseline gap-2 flex-wrap">
                        <span class="text-4xl md:text-5xl font-extrabold tracking-tight ub-gradient-text">&pound;39.99</span>
                        <span class="text-sm" style="color: var(--muted);">once, for life</span>
                    </div>
                    <p class="mt-2 font-mono text-xs" style="color: var(--muted);">GBP &middot; lifetime licence &middot; three devices</p>
                    <ul class="mt-6 space-y-2.5">
                        @foreach (['Everything in Free', 'Unlimited custom modes + vocabulary', 'Whisper small (multilingual, high accuracy)', 'File transcription, full audio history, re-transcribe'] as $f)
                            <li class="flex items-start gap-2.5 text-sm">
                                <span class="mt-0.5 shrink-0" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span>
                                <span style="color: var(--fg);">{{ $f }}</span>
                            </li>
                        @endforeach
                    </ul>
                </div>
            </div>
        </div>

        <div class="mt-8 text-center">
            <a href="{{ route('pricing') }}" class="ub-btn ub-btn-ghost">See the full pricing</a>
        </div>
    </x-section>

    {{-- FAQ --}}
    <x-section eyebrow="FAQ" title="The questions I always get.">
        <x-faq-accordion :items="$faqs" />
    </x-section>

    {{-- Founder note: editorial pull-quote, rail with bloom, no panel --}}
    <section class="py-24 md:py-32">
        <div class="ub-container">
            <div class="max-w-3xl mx-auto relative">
                <div class="absolute top-0 bottom-0 left-0 w-[2px]" style="background: var(--accent); box-shadow: 0 0 24px color-mix(in oklab, var(--accent) 40%, transparent);"></div>
                <div class="pl-10 md:pl-14">
                    <p class="font-mono mb-8" style="font-size: 0.75rem; letter-spacing: 0.12em; color: var(--muted); font-weight: 500; text-transform: uppercase;">
                        A note from the person building this
                    </p>
                    <p style="font-size: clamp(1.5rem, 3vw, 2.25rem); line-height: 1.3; letter-spacing: -0.02em; font-weight: 400; color: var(--fg-hi);">
                        I&rsquo;ve used dictation software for years. The cloud ones are fast but creepy; the local ones are fiddly and Mac-only. Utterbar is the app I wanted to exist on Windows &mdash; opinionated, native, local, and not renting me my own voice back.
                    </p>
                    <p class="mt-6" style="font-size: 1.25rem; line-height: 1.55; color: var(--muted); font-weight: 300;">
                        Free stays free because the core loop isn&rsquo;t a feature &mdash; it&rsquo;s the whole point. Pro exists so I can keep building it. If you&rsquo;ve read this far, that&rsquo;s genuinely appreciated.
                    </p>
                    <p class="mt-10 flex items-center gap-3 text-sm" style="color: var(--muted);">
                        <span style="color: var(--fg-hi); font-weight: 500;">&mdash; Jay</span>
                        <span style="color: var(--border-strong);">&middot;</span>
                        <span class="font-mono" style="font-size: 0.75rem; letter-spacing: 0.08em;">BUILDING UTTERBAR</span>
                    </p>
                </div>
            </div>
        </div>
    </section>

    {{-- Final CTA — with onboarding walkthrough --}}
    <section class="py-24 md:py-32">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto">
                <div class="max-w-2xl mb-12 md:mb-16 text-center mx-auto">
                    <p class="ub-eyebrow">Getting started</p>
                    <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                        Set up in under <x-hl>two minutes</x-hl>.
                    </h2>
                    <p class="ub-lead mt-6">
                        Install, pick a mic, press the hotkey. That&rsquo;s it.
                    </p>
                </div>
                <div class="mt-6 flex flex-wrap gap-3 justify-center">
                    <a href="{{ route('download') }}" class="ub-btn ub-btn-cta">
                        Download for Windows
                        <x-icon name="arrow-right" size="16" stroke="2" />
                    </a>
                    <a href="{{ route('pricing') }}" class="ub-btn ub-btn-ghost">
                        See pricing
                    </a>
                </div>

                {{-- Tip jar — quiet support surface. Auto-hides if Stripe
                     tipping isn't configured (see config/services.php). --}}
                <div class="mt-12 max-w-2xl mx-auto">
                    <x-tip-jar />
                </div>
            </div>
        </div>
    </section>
</x-layouts.app>
