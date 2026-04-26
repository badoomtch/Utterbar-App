@php
    // Comparison rows. Honest, factual where possible. Sourced from each
    // product's own pricing/features pages on 2026-04-25.
    $rows = [
        [
            'topic'    => 'Where transcription happens',
            'utterbar' => 'On your PC. The audio never leaves your machine.',
            'wispr'    => 'In the cloud. Audio is sent to Wispr&rsquo;s servers for processing.',
            'edge'     => 'utterbar',
        ],
        [
            'topic'    => 'Pricing model',
            'utterbar' => 'Free forever for unlimited dictation. Pro is one-time, &pound;39.99, lifetime.',
            'wispr'    => 'Subscription. Free Basic capped at 2,000 words/week. Pro is $15/mo or $144/year.',
            'edge'     => 'utterbar',
        ],
        [
            'topic'    => 'Works offline',
            'utterbar' => 'Yes. Pull your ethernet cable mid-sentence; nothing changes.',
            'wispr'    => 'No. Cloud processing requires a network connection.',
            'edge'     => 'utterbar',
        ],
        [
            'topic'    => 'Account required',
            'utterbar' => 'No account for the app. The licence key arrives by email and that&rsquo;s it.',
            'wispr'    => 'Sign-up required to use it.',
            'edge'     => 'utterbar',
        ],
        [
            'topic'    => 'Word / time limits on the free tier',
            'utterbar' => 'None. Free is genuinely unlimited.',
            'wispr'    => '2,000 words/week on desktop, 1,000/week on iPhone.',
            'edge'     => 'utterbar',
        ],
        [
            'topic'    => 'AI rewriting / formatting',
            'utterbar' => 'Three modes: voice-to-text, message, meeting. Deterministic post-processing.',
            'wispr'    => 'AI rewriting is the headline feature. Cloud LLM rewrites your speech.',
            'edge'     => 'wispr',
        ],
        [
            'topic'    => 'Custom vocabulary',
            'utterbar' => 'Yes. Custom words and replacements, applied locally before output.',
            'wispr'    => 'Yes.',
            'edge'     => 'tie',
        ],
        [
            'topic'    => 'System-audio capture (transcribe meetings)',
            'utterbar' => 'Yes, native WASAPI loopback. No virtual cable needed.',
            'wispr'    => 'Microphone only.',
            'edge'     => 'utterbar',
        ],
        [
            'topic'    => 'File transcription',
            'utterbar' => 'Yes, Pro tier. Drop a recording, get a plain-text transcript next to the source file.',
            'wispr'    => 'Not advertised on the consumer plans.',
            'edge'     => 'utterbar',
        ],
        [
            'topic'    => 'Platforms',
            'utterbar' => 'Windows 10 / 11.',
            'wispr'    => 'macOS, Windows, iOS, Android.',
            'edge'     => 'wispr',
        ],
        [
            'topic'    => 'Telemetry',
            'utterbar' => 'None. The app does not phone home.',
            'wispr'    => 'Standard cloud-product analytics, plus the audio itself for processing.',
            'edge'     => 'utterbar',
        ],
        [
            'topic'    => '5-year cost',
            'utterbar' => '&pound;39.99, one-time, on three devices.',
            'wispr'    => '~$720 USD on the annual plan, ~$900 on monthly.',
            'edge'     => 'utterbar',
        ],
    ];

    // Page-specific schema. BreadcrumbList for the rich-result eligibility
    // and a Product entry for the compared subject (Utterbar) so the
    // comparison page itself carries a SoftwareApplication / Offers
    // payload — anchors the £39.99 GBP value to the page where it's
    // most likely to be referenced in queries.
    $compareCrumbs = [
        '@context'        => 'https://schema.org',
        '@type'           => 'BreadcrumbList',
        'itemListElement' => [
            ['@type' => 'ListItem', 'position' => 1, 'name' => 'Home',    'item' => url('/')],
            ['@type' => 'ListItem', 'position' => 2, 'name' => 'Compare', 'item' => route('compare.index')],
            ['@type' => 'ListItem', 'position' => 3, 'name' => 'Wispr Flow', 'item' => url()->current()],
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
            'price'         => '39.99',
            'priceCurrency' => 'GBP',
            'category'      => 'OneTimePurchase',
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
    title="Utterbar vs Wispr Flow"
    description="Comparing Utterbar (local-only, Windows-native, £39.99 one-time) with Wispr Flow (cloud-based, cross-platform, $15/mo subscription). Honest table, no spin.">

    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <p class="ub-eyebrow">Comparison</p>
                <h1 class="ub-fg-hi mt-4" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Utterbar vs <x-hl>Wispr Flow</x-hl>.
                </h1>
                <p class="ub-lead mt-8 max-w-2xl">
                    They&rsquo;re both modern dictation apps for knowledge work. They&rsquo;re built on opposite assumptions. Here&rsquo;s the honest difference, line by line.
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
                            <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--accent);">Local · one-time</span>
                        </div>
                        <p class="mt-4" style="color: var(--fg); line-height: 1.55;">
                            Windows-native. Speech recognition runs on-device with NVIDIA Parakeet. Free forever for unlimited dictation; Pro adds the power-user features for a one-time &pound;39.99.
                        </p>
                        <ul class="mt-6 space-y-2.5 text-sm">
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">Audio never leaves your PC</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">Works offline</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.2" /></span><span style="color: var(--fg);">No account, no telemetry, no rate limits</span></li>
                        </ul>
                    </div>
                    <div class="p-8 md:p-10" style="border-left: 1px solid var(--border);">
                        <div class="flex items-baseline justify-between mb-2">
                            <h2 class="text-2xl" style="color: var(--muted);">Wispr Flow</h2>
                            <span class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted);">Cloud · subscription</span>
                        </div>
                        <p class="mt-4" style="color: var(--muted); line-height: 1.55;">
                            Cross-platform (Mac/Windows/iOS/Android). Speech is sent to Wispr&rsquo;s servers and rewritten by their AI. Free Basic capped at 2,000 words/week; Pro is $15/mo.
                        </p>
                        <ul class="mt-6 space-y-2.5 text-sm">
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">Strong AI rewriting / command mode</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">Works on any platform</span></li>
                            <li class="flex items-start gap-2.5"><span class="mt-0.5" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span><span style="color: var(--muted);">Best-in-class for shared team setups</span></li>
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
                <div class="rounded-2xl overflow-hidden" style="background: var(--bg-1); border: 1px solid var(--border);">
                    @foreach ($rows as $i => $row)
                        <div class="grid grid-cols-1 md:grid-cols-12 gap-4 md:gap-6 p-5 md:p-6"
                             style="{{ $i > 0 ? 'border-top: 1px solid var(--border);' : '' }}">
                            <div class="md:col-span-3">
                                <p class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted); letter-spacing: 0.12em;">{{ $row['topic'] }}</p>
                            </div>
                            <div class="md:col-span-{{ 9 / 2 }} md:col-span-4 lg:col-span-4">
                                <div class="flex items-start gap-2.5">
                                    @if ($row['edge'] === 'utterbar')
                                        <span class="mt-0.5 shrink-0" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.4" /></span>
                                    @elseif ($row['edge'] === 'tie')
                                        <span class="mt-0.5 shrink-0" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span>
                                    @else
                                        <span class="mt-0.5 shrink-0" style="color: var(--fg-dim); opacity: 0.6;"><x-icon name="check" size="16" stroke="1.6" /></span>
                                    @endif
                                    <span style="color: {{ $row['edge'] === 'utterbar' ? 'var(--fg-hi)' : 'var(--fg)' }};">{!! $row['utterbar'] !!}</span>
                                </div>
                                <p class="font-mono text-[10px] uppercase tracking-wider mt-2 ml-6" style="color: var(--muted);">Utterbar</p>
                            </div>
                            <div class="md:col-span-5">
                                <div class="flex items-start gap-2.5">
                                    @if ($row['edge'] === 'wispr')
                                        <span class="mt-0.5 shrink-0" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.4" /></span>
                                    @elseif ($row['edge'] === 'tie')
                                        <span class="mt-0.5 shrink-0" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span>
                                    @else
                                        <span class="mt-0.5 shrink-0" style="color: var(--fg-dim); opacity: 0.6;"><x-icon name="check" size="16" stroke="1.6" /></span>
                                    @endif
                                    <span style="color: {{ $row['edge'] === 'wispr' ? 'var(--fg-hi)' : 'var(--muted)' }};">{!! $row['wispr'] !!}</span>
                                </div>
                                <p class="font-mono text-[10px] uppercase tracking-wider mt-2 ml-6" style="color: var(--muted);">Wispr Flow</p>
                            </div>
                        </div>
                    @endforeach
                </div>
                <p class="mt-4 text-xs" style="color: var(--muted);">
                    Wispr Flow data sourced from <a href="https://wisprflow.ai/pricing" target="_blank" rel="noopener" style="color: var(--accent);">wisprflow.ai/pricing</a>, verified 2026-04-25. We&rsquo;ll keep this in sync as their plans change.
                </p>
            </div>
        </div>
    </section>

    {{-- The honest essay: when each one is the right answer --}}
    <section class="py-16 md:py-24">
        <div class="ub-container">
            <div class="max-w-3xl mx-auto">
                <h2 class="text-2xl md:text-3xl mb-8" style="letter-spacing: -0.025em;">Which one should you pick?</h2>

                <div class="space-y-6 text-lg leading-relaxed" style="color: var(--fg);">
                    <p>
                        Most product comparison pages pretend the answer is always &ldquo;ours&rdquo;. It isn&rsquo;t. Wispr Flow is a properly funded, well-built product with a sharp AI-rewriting feature set, and it&rsquo;s the right call for a real subset of buyers. So: the honest version.
                    </p>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">Pick Wispr Flow if&hellip;</h3>
                    <ul class="space-y-3 text-base" style="color: var(--fg);">
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You work on macOS, iOS, or Android, not just Windows. Wispr ships everywhere; Utterbar is Windows-first by design.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You want aggressive AI-driven rewrites of what you said &mdash; not just clean transcription, but the LLM restructuring your sentences into &ldquo;business-appropriate&rdquo; copy. That&rsquo;s Wispr&rsquo;s headline feature; we deliberately don&rsquo;t do it.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You&rsquo;re happy with monthly subscription billing and don&rsquo;t mind your voice going through a third-party server.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span>You need cross-device sync of vocabulary, settings, and history out of the box.</span></li>
                    </ul>
                </div>

                <div class="mt-12">
                    <h3 class="text-xl mb-4" style="color: var(--fg-hi); letter-spacing: -0.015em;">Pick Utterbar if&hellip;</h3>
                    <ul class="space-y-3 text-base" style="color: var(--fg);">
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You&rsquo;re on Windows and you want a Windows-native app, not an Electron shell or a web app pretending to be one.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You don&rsquo;t want your voice leaving your machine. Privacy isn&rsquo;t a feature toggle, it&rsquo;s the architecture &mdash; the app literally has nowhere to send your audio.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You don&rsquo;t want to rent your dictation tool. &pound;39.99 one-time on three devices is what Wispr Pro costs in three months.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You dictate enough that weekly word caps would frustrate you, even on a free tier.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You need to transcribe meetings (system audio) or files (recordings, podcasts), not just speak into a chat box.</span></li>
                        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span>You work in environments where corporate or compliance policy makes a cloud audio service a non-starter &mdash; legal, healthcare, government, security &mdash; and you need a tool that can prove it stays local.</span></li>
                    </ul>
                </div>
            </div>
        </div>
    </section>

    {{-- Other alternatives — quick orientation table --}}
    <section class="py-16 md:py-20" style="background: linear-gradient(180deg, transparent, color-mix(in oklab, var(--bg-1) 60%, transparent), transparent);">
        <div class="ub-container">
            <div class="max-w-4xl mx-auto">
                <h2 class="text-2xl md:text-3xl mb-4" style="letter-spacing: -0.025em;">Other alternatives, briefly.</h2>
                <p class="mb-10 max-w-2xl" style="color: var(--muted);">
                    The dictation market is bigger than just two players. Here&rsquo;s the rest of the realistic shortlist for a Windows buyer in 2026.
                </p>

                <div class="rounded-2xl overflow-hidden" style="background: var(--bg-1); border: 1px solid var(--border);">
                    <div class="grid grid-cols-12 gap-4 p-5 md:p-6 font-mono text-xs uppercase tracking-wider" style="color: var(--muted); background: var(--bg-2); letter-spacing: 0.12em;">
                        <div class="col-span-3">App</div>
                        <div class="col-span-2">Platforms</div>
                        <div class="col-span-2">Pricing</div>
                        <div class="col-span-2">Local?</div>
                        <div class="col-span-3">In one line</div>
                    </div>
                    @foreach ([
                        ['app' => 'Windows Voice Access', 'platforms' => 'Windows 11', 'pricing' => 'Free (built-in)', 'local' => 'Yes', 'note' => 'Accessibility-first; basic dictation, limited customisation.'],
                        ['app' => 'Dragon Professional v16', 'platforms' => 'Windows', 'pricing' => '~$500 one-time', 'local' => 'Yes', 'note' => 'Legacy incumbent; Microsoft no longer markets it to consumers.'],
                        ['app' => 'Superwhisper', 'platforms' => 'Mac, Windows, iOS', 'pricing' => '~$8/mo', 'local' => 'Yes (or cloud)', 'note' => 'Mac-first heritage; Windows is newer.'],
                        ['app' => 'MacWhisper', 'platforms' => 'macOS only', 'pricing' => '~&euro;44–54 one-time', 'local' => 'Yes', 'note' => 'Mac equivalent in spirit; pay-once, local-first.'],
                        ['app' => 'Talon Voice', 'platforms' => 'Mac, Windows, Linux', 'pricing' => 'Free (Patreon for early access)', 'local' => 'Yes', 'note' => 'Power-user voice control framework; steep learning curve.'],
                    ] as $i => $row)
                        <div class="grid grid-cols-12 gap-4 p-5 md:p-6 text-sm"
                             style="{{ $i > 0 ? 'border-top: 1px solid var(--border);' : '' }}">
                            <div class="col-span-3" style="color: var(--fg-hi);">{{ $row['app'] }}</div>
                            <div class="col-span-2" style="color: var(--fg);">{{ $row['platforms'] }}</div>
                            <div class="col-span-2" style="color: var(--fg);">{{ $row['pricing'] }}</div>
                            <div class="col-span-2" style="color: var(--accent);">{{ $row['local'] }}</div>
                            <div class="col-span-3" style="color: var(--muted); line-height: 1.5;">{{ $row['note'] }}</div>
                        </div>
                    @endforeach
                </div>
            </div>
        </div>
    </section>

    <x-prompt-cta
        :heading="'See for yourself.'"
        :sub="'Free forever, no account, downloads in under a minute. If it&rsquo;s not for you, the licence is refundable for 14 days &mdash; no questions.'"
    />
</x-layouts.app>
