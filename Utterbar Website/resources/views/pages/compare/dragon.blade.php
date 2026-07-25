@php
    // Comparison rows. Sourced from Dragon's own product pages
    // (dragon.nuance.com), Microsoft acquisition documents, and
    // verified UK reseller listings on 2026-05-01.
    //
    // We deliberately stay soft on Dragon Professional v16's exact
    // price because Nuance/Microsoft no longer publishes a price
    // directly on dragon.nuance.com — the "Buy" button routes to
    // "Contact us" and UK customers go through resellers (Speech
    // Computing, Posturite, Voicepower, etc.). Reseller pricing
    // clusters around £600–£700 GBP / $699 USD; we cite the latter
    // and link the reseller landscape rather than asserting a
    // single figure.
    $rows = [
        [
            'topic'      => 'Where transcription happens',
            'utterbar'   => 'On your PC. Audio never leaves your machine.',
            'competitor' => 'Dragon Professional v16 runs on-device. The Anywhere/Mobile/Medical line is cloud-processed, with audio sent to Nuance/Microsoft servers.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Pricing model',
            'utterbar'   => 'Free and open source. Every feature included, no account.',
            'competitor' => 'Dragon Professional v16: one-time, around $699 USD via UK/US resellers (no direct GBP price published). Dragon Anywhere: $14.99/month or $149.99/year subscription.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'How to get it',
            'utterbar'   => 'Download free from utterbar.com. No account, no card.',
            'competitor' => 'No direct GBP retail. UK customers go through resellers (Speech Computing, Posturite, Voicepower, Speak-IT). dragon.nuance.com routes UK buyers to a contact form.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'Devices',
            'utterbar'   => 'Install on as many devices as you like. No licence, no seat cap.',
            'competitor' => 'Dragon Professional v16 single-user licence: per Nuance KB, installable on up to two computers for the same user. Verify the EULA at purchase time.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'Specialised vocabularies',
            'utterbar'   => 'User-defined words and replacements, applied locally before output. No built-in legal/medical pack.',
            'competitor' => 'Dragon Legal v16 and Dragon Medical One ship with deeply trained legal and clinical vocabularies, voice macros, and templates. Genuine strength for those professions.',
            'edge'       => 'competitor',
        ],
        [
            'topic'      => 'Command and control',
            'utterbar'   => 'Dictation focused. Hotkey to start; text appears in the focused app. No mouse-grid or full system voice control.',
            'competitor' => 'Dragon Professional includes voice commands for app control, keyboard shortcuts, and a mouse-grid pointer. Mature feature with decades of refinement.',
            'edge'       => 'competitor',
        ],
        [
            'topic'      => 'System-audio capture (transcribe meetings)',
            'utterbar'   => 'Yes, native WASAPI loopback. Transcribe a Teams or Meet call without a virtual cable.',
            'competitor' => 'Microphone only on Dragon desktop. Meetings need a separate transcription tool.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'File transcription',
            'utterbar'   => 'Yes, included. Drop a recording, get a plain-text transcript next to the source file.',
            'competitor' => 'Not a headline feature on Dragon Professional v16. Dragon Anywhere is dictation-only.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'Languages',
            'utterbar'   => '30+ out of the box (Whisper for non-English; Parakeet for English). All processed on your PC.',
            'competitor' => 'Dragon Professional v16 ships per-language editions. UK English, US English, French, German, Italian, Spanish, Dutch most prominent.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Modern installer footprint',
            'utterbar'   => 'About 19 MB installer; pulls models on first run. Updates are differential and quiet.',
            'competitor' => 'Dragon Professional v16 is a substantial install (typical guidance is 8&nbsp;GB free disk; speech models are part of the box).',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'Platforms',
            'utterbar'   => 'Windows 10 / 11.',
            'competitor' => 'Windows 10 / 11 (optimised for Windows 11). No macOS desktop product since 2018. iOS/Android via Dragon Anywhere subscription.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Telemetry',
            'utterbar'   => 'None. The app does not phone home.',
            'competitor' => 'Dragon Professional v16 (desktop) is on-device. Anywhere/Mobile/Medical send speech to Nuance/Microsoft servers; the Anywhere Mobile page states processing happens in the United States.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'Update cadence',
            'utterbar'   => 'Several beta releases a month, signed updater, opt-in beta channel.',
            'competitor' => 'Dragon Professional v16 ships as a major version. Reseller and customer notes report no major desktop release since 2023.',
            'edge'       => 'utterbar',
        ],
    ];

    $compareCrumbs = [
        '@context'        => 'https://schema.org',
        '@type'           => 'BreadcrumbList',
        'itemListElement' => [
            ['@type' => 'ListItem', 'position' => 1, 'name' => 'Home',    'item' => url('/')],
            ['@type' => 'ListItem', 'position' => 2, 'name' => 'Compare', 'item' => route('compare.index')],
            ['@type' => 'ListItem', 'position' => 3, 'name' => 'Dragon',  'item' => url()->current()],
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
    title="Utterbar vs Dragon (NaturallySpeaking, Professional v16)"
    description="Comparing Utterbar (local-only, Windows-native, free and open source) with the Microsoft-owned Dragon dictation line (Professional v16 on-device, Anywhere/Mobile/Medical cloud). Honest table, no spin.">

    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <p class="ub-eyebrow">Comparison</p>
                <h1 class="ub-fg-hi mt-4" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Utterbar vs <x-hl>Dragon</x-hl>.
                </h1>
                <p class="ub-lead mt-8 max-w-2xl">
                    Dragon has run Windows dictation for thirty years. It still does some things better than anyone. Here&rsquo;s where Utterbar fits, and where it doesn&rsquo;t.
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
                            Windows-native, free and open source. On-device speech recognition with Whisper and NVIDIA Parakeet. Dictation, modes, vocabulary, file transcription, meetings.
                        </p>
                        <ul class="mt-6 space-y-2.5 text-sm">
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">Direct GBP purchase, no reseller dance</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">Modern installer; first run pulls models on demand</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">Active beta channel; signed updates every few weeks</span></li>
                        </ul>
                    </div>
                    <div class="p-8 md:p-10" style="border-left: 1px solid var(--border);">
                        <div class="flex items-baseline justify-between mb-2">
                            <h2 class="text-2xl" style="color: var(--muted);">Dragon Professional v16</h2>
                            <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted);">Local · perpetual · USD</span>
                        </div>
                        <p class="mt-4" style="color: var(--muted); line-height: 1.55;">
                            The mature incumbent. Windows-only desktop, on-device, around $699 via resellers. Dragon Legal and Dragon Medical One add specialised vocabularies; Dragon Anywhere is a separate cloud subscription on iOS/Android.
                        </p>
                        <ul class="mt-6 space-y-2.5 text-sm">
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">Deep legal and medical vocabularies</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">Voice command-and-control across the OS</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">Decades of accuracy refinement</span></li>
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
                    competitor="Dragon"
                    sourceUrl="https://dragon.nuance.com/en-gb/dragon-professional"
                    sourceLabel="dragon.nuance.com"
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
                        Dragon and Utterbar live in the same corner of the market: serious Windows dictation, on-device, no subscription rent for the core flow. The honest split is what you&rsquo;re paying for and how often you need it.
                    </p>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">Pick Dragon if&hellip;</h3>
                    <ul class="space-y-3 text-base" style="color: var(--fg);">
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You work in law, medicine, or law enforcement and you need a vocabulary built for that work, with templates and macros that have been refined for decades. Dragon Legal, Dragon Medical, and Dragon Law Enforcement exist for a reason.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You want full voice-driven control of Windows: launching apps, clicking with a mouse-grid, navigating menus, scripted command sequences. Dragon does this; we don&rsquo;t. We&rsquo;re a dictation tool, not a desktop accessibility shell.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You already have Dragon installed, your workflows are wired into it, and the cost of switching outweighs the per-seat saving.</span></li>
                    </ul>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">Pick Utterbar if&hellip;</h3>
                    <ul class="space-y-3 text-base" style="color: var(--fg);">
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You want to dictate quickly across every app on your PC, not run the OS by voice. The hotkey-and-paste flow is what you actually do all day; the rest of Dragon&rsquo;s feature surface is overhead you&rsquo;ll never use.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You&rsquo;re in the UK and want to pay in GBP without going through a reseller and a quote. We sell direct from a single page, and the licence arrives by email immediately.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You&rsquo;re used to modern apps that ship updates in weeks, not years, and you don&rsquo;t want a tool that has gone quiet since 2023.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You need to transcribe meetings (system audio) or files (recordings, podcasts) in addition to dictation. That&rsquo;s a different surface from Dragon&rsquo;s.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You don&rsquo;t want to pay several hundred pounds up front for a tool you&rsquo;d use a few times a day. Free is a different commitment level.</span></li>
                    </ul>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">A note on the Dragon line under Microsoft</h3>
                    <p class="text-base" style="color: var(--fg);">
                        Microsoft acquired Nuance, the maker of Dragon, in 2022. Dragon Home was discontinued in 2023; the consumer-friendly tier is gone. The remaining desktop product is Dragon Professional v16 (with Legal and Law Enforcement editions), and the cloud line is Dragon Anywhere, Dragon Professional Anywhere, and Dragon Medical One. If you&rsquo;re a UK consumer comparison-shopping in 2026, the realistic Dragon options are reseller-priced Professional v16 or a per-month Dragon Anywhere subscription.
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
