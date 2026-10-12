@php
    // Comparison rows. Sourced 2026-05-01 from Microsoft Support
    // pages on Voice Access:
    //   * Set up Voice Access (set-up + first-run language download)
    //   * Use Voice Access to control your PC, author text with your
    //     voice (on-device speech recognition; "works without the
    //     internet" after the language pack is installed)
    //   * Voice Access FAQs
    //   * What's new in Voice Access (multi-language rollout)
    //   * Use voice to create Voice Access shortcuts (custom commands
    //     in Windows 11 23H2+)
    //   * Use voice recognition in Windows (Voice Access supersedes
    //     Windows Speech Recognition on Windows 11 22H2+)
    //
    // Where Microsoft does not document a feature (file transcription,
    // meeting transcription, large-language custom vocabulary), we say
    // so directly rather than over-claim absence.
    $rows = [
        [
            'topic'      => 'Cost',
            'utterbar'   => 'Free and open source. Every feature included, no account.',
            'competitor' => 'Free. Built into Windows 11.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Where it lives',
            'utterbar'   => 'A floating bar that sits above your apps. Press a hotkey to dictate; text lands in the focused field.',
            'competitor' => 'A docked bar at the top of the screen, plus voice control for the entire OS (clicking, scrolling, app switching). Accessibility-first, dictation-second.',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Where transcription happens',
            'utterbar'   => 'On your PC. Audio never leaves your machine.',
            'competitor' => 'On-device after first-run language-pack download. Microsoft says it &ldquo;works even without the internet.&rdquo;',
            'edge'       => 'tie',
        ],
        [
            'topic'      => 'Speech model',
            'utterbar'   => 'NVIDIA Parakeet-tdt for English; Whisper for everything else. Local, runs on CPU or DirectML GPU.',
            'competitor' => 'Microsoft&rsquo;s on-device speech recognition. The exact model is not publicly documented.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'File transcription',
            'utterbar'   => 'Yes, included. Drop a recording, get a plain-text transcript next to the source file.',
            'competitor' => 'No. Voice Access is interactive only. Transcribing recorded audio is a different Microsoft tool (Word Transcribe, Microsoft 365).',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'Meeting transcription',
            'utterbar'   => 'Yes. Native WASAPI loopback captures system audio; Meeting Mode chunks long recordings (up to about an hour) and stitches the transcript.',
            'competitor' => 'No. That is Teams&rsquo; live transcription, not Voice Access.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'Custom vocabulary',
            'utterbar'   => 'Yes. Custom words and replacements, applied locally before output. Dictate &ldquo;Utterbar&rdquo;; never see &ldquo;utter bar&rdquo; again.',
            'competitor' => 'Voice Access has spelling and correction commands (&ldquo;spell that&rdquo;, &ldquo;correct that&rdquo;). A persistent personal dictionary in the Dragon-style sense is not documented in the Microsoft docs we reviewed.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'Custom voice commands',
            'utterbar'   => 'Not in the dictation flow. Modes (voice / message / meeting) shape the output, but there is no scriptable command surface.',
            'competitor' => 'Yes. Voice shortcuts let you record your own multi-step commands. Added in Windows 11 23H2.',
            'edge'       => 'competitor',
        ],
        [
            'topic'      => 'Languages',
            'utterbar'   => '30+ out of the box. English uses Parakeet for speed; the rest use Whisper.',
            'competitor' => 'Around 15 locale variants today, including UK, US, Indian, Australian, Canadian, and New Zealand English; Spanish (Spain and Mexico); German; French (France and Canada); Simplified and Traditional Chinese; Japanese; Italian.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'OS-wide voice control',
            'utterbar'   => 'No. We focus on dictation: hotkey, speak, paste.',
            'competitor' => 'Yes. Voice Access can launch apps, click buttons by name, scroll, switch windows, and (on Windows 11 23H2+) sign you in. It replaces Windows Speech Recognition.',
            'edge'       => 'competitor',
        ],
        [
            'topic'      => 'Modes (voice-to-text, message, meeting)',
            'utterbar'   => 'Three modes change post-processing: capitalisation, punctuation, line breaks, custom replacements per mode.',
            'competitor' => 'Single dictation surface. No mode-based output shaping.',
            'edge'       => 'utterbar',
        ],
        [
            'topic'      => 'Telemetry',
            'utterbar'   => 'None. The app does not phone home.',
            'competitor' => 'Standard Windows diagnostic data when telemetry is enabled. Voice Access&rsquo;s on-device claim covers speech processing, not OS-level diagnostics.',
            'edge'       => 'utterbar',
        ],
    ];

    $compareCrumbs = [
        '@context'        => 'https://schema.org',
        '@type'           => 'BreadcrumbList',
        'itemListElement' => [
            ['@type' => 'ListItem', 'position' => 1, 'name' => 'Home',    'item' => url('/')],
            ['@type' => 'ListItem', 'position' => 2, 'name' => 'Compare', 'item' => route('compare.index')],
            ['@type' => 'ListItem', 'position' => 3, 'name' => 'Windows Voice Access', 'item' => url()->current()],
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
    title="Utterbar vs Windows Voice Access"
    description="Voice Access is free and built into Windows 11. Utterbar is free, open source, and dictation-focused. They overlap less than you might think; here is the honest split.">

    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <p class="ub-eyebrow">Comparison</p>
                <h1 class="ub-fg-hi mt-4" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Utterbar vs <x-hl>Windows Voice Access</x-hl>.
                </h1>
                <p class="ub-lead mt-8 max-w-2xl">
                    Voice Access is free, built into Windows 11, and pretty good at what it&rsquo;s built for. Utterbar is also free, but focused on fast local dictation rather than controlling Windows by voice.
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
                            <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--accent);">Local · paid · dictation</span>
                        </div>
                        <p class="mt-4" style="color: var(--fg); line-height: 1.55;">
                            Built for people who dictate a lot, into any app on their PC. Modes for voice-to-text, message, meeting. File and meeting transcription. Custom vocabulary. Free and open source.
                        </p>
                        <ul class="mt-6 space-y-2.5 text-sm">
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">Whisper / Parakeet on-device</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">Custom words and replacements</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">File and meeting transcription included</span></li>
                        </ul>
                    </div>
                    <div class="p-8 md:p-10" style="border-left: 1px solid var(--border);">
                        <div class="flex items-baseline justify-between mb-2">
                            <h2 class="text-2xl" style="color: var(--muted);">Windows Voice Access</h2>
                            <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted);">Local · free · accessibility</span>
                        </div>
                        <p class="mt-4" style="color: var(--muted); line-height: 1.55;">
                            Built into Windows 11 (22H2 and later). Designed for accessibility: voice control of the OS, with dictation as one feature. Free. Replaces Windows Speech Recognition.
                        </p>
                        <ul class="mt-6 space-y-2.5 text-sm">
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">Free, no install, no licence</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">OS-wide voice control of clicks, scrolling, apps</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">Custom voice shortcuts (Windows 11 23H2+)</span></li>
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
                    competitor="Windows Voice Access"
                    sourceUrl="https://support.microsoft.com/en-us/topic/use-voice-access-to-control-your-pc-author-text-with-your-voice-4dcd23ee-f1b9-4fd1-bacc-862ab611f55d"
                    sourceLabel="support.microsoft.com"
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
                        These two products solve overlapping problems but were built for different people. Voice Access is an accessibility feature that happens to dictate. Utterbar is a dictation tool that doesn&rsquo;t pretend to do accessibility. Try Voice Access first; it&rsquo;s on your machine right now.
                    </p>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">Pick Voice Access if&hellip;</h3>
                    <ul class="space-y-3 text-base" style="color: var(--fg);">
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You need to control Windows by voice, not just dictate. Click buttons, navigate menus, switch apps, sign in. That&rsquo;s the core of Voice Access; we don&rsquo;t do it.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You dictate occasionally and your needs are covered by &ldquo;press the mic, speak, the words appear.&rdquo; Voice Access is genuinely good at this and it&rsquo;s already on your machine.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>Cost matters more than feature depth. Free is hard to beat.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You build custom voice shortcuts to automate workflows by speech.</span></li>
                    </ul>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">Pick Utterbar if&hellip;</h3>
                    <ul class="space-y-3 text-base" style="color: var(--fg);">
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You dictate every day and the difference between &ldquo;OK&rdquo; and &ldquo;fast and accurate&rdquo; matters. Whisper Medium and Parakeet on a modern GPU are noticeably ahead of Voice Access on long sentences and uncommon vocabulary.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You need to transcribe meetings (system audio loopback) or files (recordings, podcasts, voice memos). Voice Access doesn&rsquo;t do that.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You need a real custom vocabulary: brand names, technical terms, jargon, with replacements applied automatically before the text reaches the focused app.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You want different output styles for different contexts: voice-to-text for chat, message mode for email, meeting mode for long-form notes. Voice Access produces one stream.</span></li>
                    </ul>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">Honest answer: try Voice Access first</h3>
                    <p class="text-base" style="color: var(--fg);">
                        If you&rsquo;ve never used voice on Windows, open Settings, Accessibility, Speech, and turn Voice Access on. It is genuinely capable, it costs nothing, and it might be all you need. Come back if you find yourself wanting file or meeting transcription, real custom vocabulary, or noticeably better dictation accuracy on technical content. That is the gap Utterbar fills, and it is a real one, but it is not a gap everyone has.
                    </p>
                </div>
            </div>
        </div>
    </section>

    <x-prompt-cta
        :heading="'Free forever for unlimited dictation.'"
        :sub="'No account needed. Free and open source, every feature included.'"
    />
</x-layouts.app>
