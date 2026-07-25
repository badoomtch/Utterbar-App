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
        ['q' => 'Does Utterbar send my voice to the cloud?', 'a' => 'No. Audio is captured, transcribed, and binned on your machine. The only network calls it ever makes are model downloads on first run, and those are logged so you can read them.'],
        ['q' => 'Which Windows versions work?', 'a' => 'Windows 10 22H2 and Windows 11, x86_64. WebView2 is required (already on Windows 11; auto-bootstrapped on Windows 10).'],
        ['q' => 'Do I need a GPU?', 'a' => 'No. The default Parakeet build runs comfortably on any modern CPU. If you have a DirectX 12 graphics card (NVIDIA, AMD, or Intel), Utterbar uses it via DirectML for Whisper Medium, Whisper Large, and Parakeet. CUDA is not bundled.'],
        ['q' => 'What languages does it support?', 'a' => 'Over 30 out of the box: English, French, German, Spanish, Italian, Portuguese, Dutch, Polish, Turkish, Russian, Arabic, Hindi, Japanese, Korean, Mandarin, and more. English uses Parakeet by default for speed; everything else uses Whisper. Pick a default in Settings > Models, or leave it on auto-detect.'],
        ['q' => 'Does it work with Claude / ChatGPT / Gemini / Copilot?', 'a' => 'Yes. Utterbar isn\'t an integration. It types into whichever app has the cursor, so any chat input becomes a voice input. Click the input, hold the hotkey, ask. The text lands in the box as if you\'d typed it. No API key, no extension, no per-assistant setup.'],
        ['q' => 'How does it compare to the Mac-native ones?', 'a' => 'Same philosophy, different platform. The Mac alternatives are great if you\'re on Mac. Utterbar is Windows-native from day one. No Python shim, no Electron wrapper, a smaller installer. Different camp, same conviction about local.'],
        ['q' => 'Why two engines instead of one?', 'a' => 'Each one is best at something different. Parakeet is fastest and most accurate on English; Whisper covers 30+ other languages. The app picks sensibly by default; you can override in Settings > Models.'],
        ['q' => 'Can I use this in video-call and chat apps?', 'a' => 'Only for typing into the chat box. Utterbar transcribes your voice into text; it won\'t (and will never) inject audio into a call. Your colleagues hear nothing from Utterbar.'],
        ['q' => 'Does it work offline?', 'a' => 'Yes. Pull your ethernet cable mid-sentence and nothing changes. The app doesn\'t authenticate against anything.'],
        ['q' => 'Will there be a macOS version?', 'a' => 'Eventually, after Windows 1.0 ships. Mac already has decent local-only options; I\'d rather make Windows actually good first.'],
        ['q' => 'What happens to my data when I uninstall?', 'a' => '<code>%APPDATA%\\Utterbar\\</code> stays on disk so you can re-install without losing history. Delete the folder yourself for a clean wipe.'],
        ['q' => 'Prove it really runs local.', 'a' => 'Start the app. Unplug your ethernet. Keep dictating. That\'s the proof. If you want harder evidence, sniff the network during a session: there\'s nothing to see once the model has finished downloading.'],
    ];
@endphp

@php
    // Title is the single most important on-page SEO signal for the home
    // page. Lead with the query buyers actually type ("local dictation
    // for Windows"), not a tagline. Layout wraps with " — Utterbar".
    $homeTitle = 'Local dictation for Windows';
    $homeDescription = 'Local-only Windows dictation. Press a hotkey, speak, your words land anywhere. Free and open source, no cloud, no account, every feature included.';
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
                        '@type'         => 'Offer',
                        'name'          => 'Utterbar',
                        'price'         => '0',
                        'priceCurrency' => 'GBP',
                        'availability'  => 'https://schema.org/InStock',
                        'url'           => url('/download'),
                    ],
                    'author'    => ['@id' => url('/#organization')],
                    'publisher' => ['@id' => url('/#organization')],
                ],
                [
                    '@type'      => 'FAQPage',
                    '@id'        => url('/#faq'),
                    'mainEntity' => collect($faqs)->map(fn ($f) => [
                        '@type'          => 'Question',
                        'name'           => strip_tags(str_replace(['&rsquo;'], ["'"], $f['q'])),
                        'acceptedAnswer' => [
                            '@type' => 'Answer',
                            'text'  => strip_tags(str_replace(['&rsquo;'], ["'"], $f['a'])),
                        ],
                    ])->values()->all(),
                ],
            ],
        ];
    @endphp
    @push('head')
        <script type="application/ld+json">{!! json_encode($homeSchema, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) !!}</script>
    @endpush

    {{-- Hero: brand plate, headline, punchy subhead, interactive pill.
         Carries the design's full hero ambient layer: aurora blobs (three
         drifting cyan radial gradients), floating particles (10 cyan dots
         rising from below), a soft fade at the bottom edge so the hero
         bleeds into the next section without a hard line. All ambient
         layers respect prefers-reduced-motion. --}}
    <section class="ub-hero">
        {{-- Animated cyan aurora: three drifting radial blobs --}}
        <div class="ub-hero-aurora" aria-hidden="true">
            <div class="ub-aurora-blob ub-aurora-b1"></div>
            <div class="ub-aurora-blob ub-aurora-b2"></div>
            <div class="ub-aurora-blob ub-aurora-b3"></div>
        </div>
        {{-- Floating "voice" particles drifting up. Each <span> picks up
             its own left position + delay + drift via :nth-child rules in
             the stylesheet. --}}
        <div class="ub-hero-particles" aria-hidden="true">
            <span></span><span></span><span></span><span></span><span></span>
            <span></span><span></span><span></span><span></span><span></span>
        </div>

        <div class="ub-container ub-hero-inner">
            <div class="ub-hero-copy">

                {{-- Brand plate: the wordmark, scaled as a display element.
                     `:wave="true"` enables the hover-only animated waveform
                     reveal (matches the same affordance on the nav brand).
                     Pure SVG <animate>, no JS. --}}
                <div style="font-size: clamp(2.5rem, 5.5vw, 4.5rem); letter-spacing: -0.045em; line-height: 1; font-weight: 600;">
                    <x-wordmark :wave="true" />
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

                <p class="mt-4 max-w-xl" style="font-size: clamp(1rem, 1.5vw, 1.125rem); line-height: 1.5; color: var(--muted);">
                    Speaks 30+ languages. None of them ever leaves your PC.
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

            {{-- Interactive pill widget: hover/tap to expand, drag to move,
                 and click "Try me" to play a short dictation demo that
                 streams text into a faux field below the pill. --}}
            <div class="ub-hero-pill-slot">
            <div x-data="heroPill" x-init="autoExpandOnce()" class="ub-pill-anchor">
                <div class="ub-pill-widget"
                     :class="{ 'is-dragging': dragging }"
                     :style="`transform: translate(${x}px, ${y}px) rotate(${tilt}deg);`"
                     @pointerdown="onDown($event)">

                    {{-- "drag me" hint, fades on first interaction --}}
                    <div class="ub-pill-hint" :class="{ 'is-gone': hasInteracted }" aria-hidden="true">
                        <div class="ub-pill-hint-inner">
                            <span class="ub-pill-hint-label">Drag me</span>
                            <svg class="ub-pill-hint-arrow" viewBox="0 0 48 62" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M 38 6 C 44 20, 40 38, 20 52" />
                                <path d="M 22 42 L 20 52 L 30 50" />
                            </svg>
                        </div>
                    </div>

                    {{-- Idle-sway wrapper: gentle rock so it reads as "loose / movable" --}}
                    <div class="ub-pill-sway" :class="{ 'is-settled': hasInteracted }">

                        <div class="ub-pill"
                             :class="{ 'is-expanded': expanded || recording, 'is-dragging': dragging }"
                             @mouseenter="onEnter()"
                             @mouseleave="onLeave()"
                             @click="onTap($event)">

                            {{-- Left: glyph + wordmark --}}
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

                            {{-- Right: state chip + mode chips + waveform + Try-me button.
                                 Mirrors the real desktop floating bar's
                                 cluster: a state chip (idle / Recording /
                                 Inserted), a tight row of mode chips (Voice
                                 / Message / Meeting) matching the real
                                 BarModeSwitch, and the seven-bar waveform.
                                 Recording flips state to "Recording" + red
                                 dot; the demo end flashes "Inserted" for a
                                 beat before settling back to the version
                                 chip. --}}
                            <div class="ub-pill-right">
                                <span class="ub-pill-version"
                                      :class="{ 'is-rec': recording, 'is-success': !recording && demoText }"
                                      x-text="stateLabel()"></span>
                                <div class="ub-pill-modes" aria-hidden="true">
                                    <template x-for="m in modes" :key="m.id">
                                        <button type="button"
                                                class="ub-pill-mode"
                                                :class="{ 'is-active': m.id === activeMode }"
                                                :aria-pressed="(m.id === activeMode).toString()"
                                                :aria-label="m.label"
                                                @click.stop="cycleMode(m.id)">
                                            <span class="ub-pill-mode-icon" x-html="m.icon"></span>
                                        </button>
                                    </template>
                                </div>
                                <div class="ub-pill-wave" :class="{ 'is-rec': recording }" aria-hidden="true">
                                    <span></span><span></span><span></span><span></span><span></span><span></span><span></span>
                                </div>
                                <button type="button"
                                        class="ub-pill-cta"
                                        :class="{ 'is-rec': recording }"
                                        :disabled="recording"
                                        @click.stop="startDemo()">
                                    <span x-show="!recording">Try me <span aria-hidden="true">&rsaquo;</span></span>
                                    <span x-show="recording" x-cloak>Listening...</span>
                                </button>
                            </div>
                        </div>
                    </div>{{-- /.ub-pill-sway --}}

                    {{-- Faux text field that streams the demo result. Sits
                         below the pill, fades in once recording starts and
                         decays after a beat. aria-hidden when empty so
                         screen readers do not announce a phantom field. --}}
                    <div class="ub-pill-field" :class="{ 'is-on': demoText }" :aria-hidden="(! demoText).toString()">
                        <div class="ub-pill-field-chrome">
                            <span class="ub-pill-field-dot"></span>
                            <span class="ub-pill-field-label">Reply to Marcus.docx</span>
                        </div>
                        <div class="ub-pill-field-body">
                            <span x-text="demoText"></span><template x-if="recording"><span class="ub-pill-caret"></span></template>
                        </div>
                    </div>

                </div>{{-- /.ub-pill-widget --}}
            </div>{{-- /.ub-pill-anchor --}}
            </div>{{-- /.ub-hero-pill-slot --}}

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

                // Demo state. Click "Try me" -> recording=true, the SCRIPT
                // streams into demoText character-by-character, recording
                // flips off when the script ends, and the field decays a
                // beat later by clearing demoText.
                recording: false,
                demoText: '',
                _demoScript: "Hey Marcus, quick reply on the Q3 forecast: I am fine pushing the revision to Friday, but let's keep the headline figure exactly as we agreed on the call.",
                _demoTimer: null,
                _demoEndTimer: null,
                _demoFadeTimer: null,
                _autoExpandTimer1: null,
                _autoExpandTimer2: null,

                // Mode chips. Matches the desktop app's BarModeSwitch:
                // three built-in mode icons sitting in the recording
                // cluster, one active (cyan-gradient fill), the rest
                // muted. Tapping a chip cycles the active one in this
                // demo. The chips are decorative on the hero — they
                // demonstrate the surface, they don't drive different
                // demo text. Icons are inline SVGs to avoid an icon-
                // pack dependency on the home page.
                activeMode: 'message',
                modes: [
                    { id: 'voice',   label: 'Voice mode',   icon: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><rect x="6" y="2" width="4" height="8" rx="2"/><path d="M3.5 7v1a4.5 4.5 0 0 0 9 0V7"/><path d="M8 12.5v1.5"/></svg>' },
                    { id: 'message', label: 'Message mode', icon: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M2.5 4.5h11a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H6L3 14v-2.5h-.5a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1Z"/></svg>' },
                    { id: 'meeting', label: 'Meeting mode', icon: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><circle cx="5" cy="6" r="2"/><circle cx="11" cy="6" r="2"/><path d="M2 13c.5-2 1.8-3 3-3s2.5 1 3 3"/><path d="M8 13c.5-2 1.8-3 3-3s2.5 1 3 3"/></svg>' },
                ],
                cycleMode(id) {
                    this.activeMode = id;
                    this.hasInteracted = true;
                },
                // Single source of truth for the state-chip label so
                // the markup stays simple. Mirrors the real desktop
                // app's StatePill: "Recording" while the demo runs,
                // "Inserted" briefly after it ends (the success
                // window where demoText is still on screen but
                // recording has flipped off), otherwise the version
                // chip.
                stateLabel() {
                    if (this.recording) return '● Recording';
                    if (this.demoText) return 'Inserted';
                    return 'v{{ ($latestRelease ?? null)?->version ?? config('site.version') }}';
                },

                autoExpandOnce() {
                    // Auto-expand 900ms after page load so the "Try me"
                    // button is discoverable without hovering. Then collapse
                    // again 2.6s later unless the user is hovering.
                    this._autoExpandTimer1 = setTimeout(() => {
                        if (!this.recording) this.expanded = true;
                    }, 900);
                    this._autoExpandTimer2 = setTimeout(() => {
                        if (!this._isHovering && !this.recording) this.expanded = false;
                    }, 3500);
                },

                _isHovering: false,

                onEnter() {
                    this._isHovering = true;
                    if (!this.touchMode) this.expanded = true;
                    this.hasInteracted = true;
                },
                onLeave() {
                    this._isHovering = false;
                    if (!this.touchMode && !this.dragging && !this.recording) this.expanded = false;
                },
                onTap(e) {
                    // Only toggle on tap if we didn't move (i.e. not a drag)
                    if (this.moved) { e.preventDefault(); return; }
                    if (this.touchMode) this.expanded = !this.expanded;
                    this.hasInteracted = true;
                },

                startDemo() {
                    if (this.recording) return;
                    this.recording = true;
                    this.expanded = true;
                    this.hasInteracted = true;
                    this.demoText = '';
                    if (this._demoTimer) clearInterval(this._demoTimer);
                    if (this._demoEndTimer) clearTimeout(this._demoEndTimer);
                    if (this._demoFadeTimer) clearTimeout(this._demoFadeTimer);
                    let i = 0;
                    this._demoTimer = setInterval(() => {
                        i += Math.random() > 0.6 ? 2 : 1;
                        if (i >= this._demoScript.length) {
                            i = this._demoScript.length;
                            clearInterval(this._demoTimer);
                            this._demoTimer = null;
                            // Brief beat at the end before recording flips off.
                            this._demoEndTimer = setTimeout(() => {
                                this.recording = false;
                                // Hold the result on screen for 4s then fade.
                                this._demoFadeTimer = setTimeout(() => {
                                    this.demoText = '';
                                }, 4000);
                            }, 350);
                        }
                        this.demoText = this._demoScript.slice(0, i);
                    }, 28);
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

    {{-- Conviction strip: design's exact sizing. Lead fragment sits on
         its own line in fg-hi 500-weight; the rest reads in muted at a
         clamp(1.25rem, 2.2vw, 1.75rem) display size with -0.015em
         tracking. Not the bigger glow underline ours used to have. --}}
    <section class="ub-conviction">
        <div class="ub-container">
            <p>
                <span class="ub-conviction-lead">Most dictation apps charge rent on your voice.</span>
                Utterbar runs entirely on your machine, never opens a socket after install, and is free and open source. Every feature is on for everyone, with nothing to buy.
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

    {{-- Speed race: looping typing-vs-Utterbar comparison. Sits after
         how-it-works because the user has just learned the loop and is
         primed for "and here is why it is faster". --}}
    <section class="py-20 md:py-28">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto">
                <x-speed-race />
            </div>
        </div>
    </section>

    {{-- Network proof: live activity-monitor mock that proves the
         local-only claim. Sits after the speed race because the user
         has just been sold on speed; the next objection is "but is it
         really private?", and this section answers it visually. --}}
    <section id="proof" class="py-20 md:py-28">
        <div class="ub-container">
            <div class="max-w-3xl mb-10 md:mb-12">
                <p class="ub-eyebrow">The proof</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Watch it <x-hl>not</x-hl> phone home.
                </h2>
                <p class="ub-lead mt-6 max-w-2xl">
                    Open your network monitor. Start dictating. Dictate for hours. The only network calls Utterbar ever makes are model downloads on first run, and those are logged so you can read them.
                </p>
                <p class="mt-4 max-w-2xl text-sm" style="color: var(--muted);">
                    Pull your ethernet cable mid-sentence. Nothing changes. The app does not authenticate against anything.
                </p>
            </div>
            <div class="max-w-5xl mx-auto">
                <x-network-proof />
            </div>
        </div>
    </section>

    {{-- Talk-to-AI showcase. Sits high on the page because for a large
         slice of buyers this is the single concrete reason to install
         the app — type-replacement-into-LLM-chat is a daily friction
         that voice removes entirely. The section frames it as
         "whichever assistant you already use" so neither vendor lock-in
         nor extension-installation enters the buyer's head. --}}
    <section class="py-20 md:py-28">
        <div class="ub-container">
            <div class="max-w-3xl mb-10 md:mb-12">
                <p class="ub-eyebrow">For everyone who lives in a chat box</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Talk to <x-hl>AI</x-hl> at the speed of speech.
                </h2>
                <p class="ub-lead mt-6 max-w-2xl">
                    Claude, ChatGPT, Gemini, Copilot, the local model you run on your own machine. Whichever one you already use, Utterbar dictates straight into the chat box. People speak about three times faster than they type. Longer prompts, fuller context, fewer sore wrists.
                </p>
                <p class="mt-4 max-w-2xl text-sm" style="color: var(--muted);">
                    No browser extension. No API key. No integration. Click the input, hold the hotkey, ask the question. The reply streams back.
                </p>
            </div>
            <div class="max-w-5xl mx-auto">
                <x-talk-to-ai />
            </div>
        </div>
    </section>

    {{-- Comparison grid: cloud dictation vs Utterbar. Sits after the
         Talk-to-AI section because the buyer has now been shown what
         it does and is starting to weigh it against the cloud tools
         they already know. The grid answers "why this and not those". --}}
    <section class="py-20 md:py-28">
        <div class="ub-container">
            <div class="max-w-3xl mb-10 md:mb-12">
                <p class="ub-eyebrow">vs the cloud incumbents</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Different <x-hl>camp.</x-hl> Same conviction.
                </h2>
            </div>
            <div class="max-w-5xl mx-auto">
                <x-compare-grid />
            </div>
        </div>
    </section>

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
            <div class="max-w-5xl mx-auto">
                <x-app-mock />
            </div>
        </div>
    </section>

    {{-- Vocabulary showcase --}}
    <section class="py-20 md:py-28">
        <div class="ub-container">
            <div class="max-w-3xl mb-10 md:mb-12">
                <p class="ub-eyebrow">Learns your words</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Names, acronyms, product terms. <x-hl>Fixed</x-hl>.
                </h2>
                <p class="ub-lead mt-6 max-w-xl">
                    Add your custom words and replacements once. From then on, &ldquo;J parrie&rdquo; comes out as &ldquo;Jay Parry&rdquo;. The fix happens on your machine, after the ASR runs, before the text is pasted.
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
                    Old recordings, podcasts, voice memos, an interview from last Tuesday. Drop the file in, get a plain-text transcript next to it. Same engine as the floating bar, longer audio.
                </p>
            </div>
            <div class="max-w-5xl mx-auto">
                <x-video-embed id="feature-file-transcription" />
            </div>
        </div>
    </section>

    {{-- Pricing: single "everything's free" panel. The paid tier is
         gone; every feature is on for everyone. CTA drives to
         /download; a secondary link goes to the /pricing page which
         now explains the free + open-source model. --}}
    <section id="pricing" class="ub-pricing-section">
        <div class="ub-container">
            <div class="ub-pricing-head">
                <p class="ub-eyebrow">Pricing</p>
                <h2 class="ub-pricing-title">
                    Free and <x-hl>open source</x-hl>.
                </h2>
                <p class="ub-pricing-lead">
                    The act of speaking isn&rsquo;t a premium feature.
                </p>
            </div>

            <div class="ub-pricing-wrap">
                {{-- FREE — the whole app --}}
                <div class="ub-tier ub-tier-free">
                    <div class="ub-tier-head">
                        <h3>Everything</h3>
                        <span class="ub-tier-badge">free</span>
                    </div>
                    <div class="ub-tier-price">
                        <span class="amount">&pound;0</span>
                    </div>
                    <p class="ub-tier-sub">No account. No trial clock. No limits. No paid tier.</p>
                    <ul>
                        @foreach (['Unlimited dictations', '30+ languages, all on-device', 'Every mode, unlimited custom modes', 'File transcription, full history, meetings', 'Vocabulary, replacements, privacy controls'] as $f)
                            <li>
                                <x-icon name="check" class="ub-tier-check" size="16" stroke="2" />
                                <span>{{ $f }}</span>
                            </li>
                        @endforeach
                    </ul>
                    <a href="{{ route('download') }}" class="ub-btn ub-btn-cta ub-tier-cta">Download free</a>
                </div>
            </div>

            <div class="ub-pricing-foot">
                <a href="{{ route('pricing') }}" class="ub-btn ub-btn-ghost">Why it&rsquo;s free</a>
            </div>
        </div>
    </section>

    {{-- FAQ: centred header + centred accordion column. The
         accordion component itself is unchanged; we just centre the
         section head and constrain the wrapper so the items sit
         centred under the headline. Question copy stays BRIEF
         verbatim (defined at the top of this file in $faqs). --}}
    <section id="faq" class="ub-faq-section">
        <div class="ub-container">
            <div class="ub-faq-head">
                <p class="ub-eyebrow">FAQ</p>
                <h2 class="ub-faq-title">
                    The questions <x-hl>I always</x-hl> get.
                </h2>
            </div>
            <div class="ub-faq-wrap">
                <x-faq-accordion :items="$faqs" />
            </div>
        </div>
    </section>

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
                        I&rsquo;ve used dictation software for years. The cloud ones are fast but creepy. The local ones are fiddly, and almost all of them are on Mac. Utterbar is the Windows app I wanted to exist: opinionated, native, local, not renting me my own voice back.
                    </p>
                    <p class="mt-6" style="font-size: 1.25rem; line-height: 1.55; color: var(--muted); font-weight: 300;">
                        Free stays free because the core loop isn&rsquo;t a feature, it&rsquo;s the whole product. Pro exists so I can keep building. If you&rsquo;ve read this far, thank you.
                    </p>
                    <p class="mt-10 flex items-center gap-3 text-sm" style="color: var(--muted);">
                        <span style="color: var(--fg-hi); font-weight: 500;">Jay</span>
                        <span style="color: var(--border-strong);">&middot;</span>
                        <span class="font-mono" style="font-size: 0.75rem; letter-spacing: 0.08em;">BUILDING UTTERBAR</span>
                    </p>
                </div>
            </div>
        </div>
    </section>

    {{-- Final CTA: design's .final-cta shell. Centred bg-1 panel
         with a soft cyan radial bloom from the bottom edge, the
         "Set up in under two minutes" headline, two CTAs, and the
         release meta line. Tip-jar sits below the panel inside the
         same outer section so the support surface stays in the same
         visual neighbourhood. --}}
    <section id="download" class="ub-final-cta-section">
        <div class="ub-container">
            <div class="ub-final-cta">
                <div class="ub-final-cta-inner">
                    <p class="ub-eyebrow">Get Utterbar</p>
                    <h2 class="ub-final-cta-title">
                        Set up in under <x-hl>two minutes</x-hl>.
                    </h2>
                    <p class="ub-final-cta-sub">
                        Install. Pick a mic. Press the hotkey. That&rsquo;s it.
                    </p>
                    <div class="ub-final-cta-ctas">
                        <a href="{{ route('download') }}" class="ub-btn ub-btn-cta">
                            Download for Windows
                            <x-icon name="arrow-right" size="16" stroke="2" />
                        </a>
                        <a href="{{ route('pricing') }}" class="ub-btn ub-btn-ghost">
                            See pricing
                        </a>
                    </div>
                    @if ($latestRelease ?? null)
                        <p class="ub-final-cta-meta">
                            v{{ $latestRelease->version }} &middot; {{ $latestRelease->sizeLabel() }} &middot; WIN10/11 &middot; SHA-256 verified
                        </p>
                    @endif
                </div>
            </div>

        </div>
    </section>
</x-layouts.app>
