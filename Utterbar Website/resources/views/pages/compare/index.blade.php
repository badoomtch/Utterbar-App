@php
    // Per-competitor teasers. Order chosen for the way buyers
    // typically arrive: Wispr Flow (most-searched modern competitor),
    // Voice Access (the "why pay anything" question), Dragon (the
    // legacy enterprise incumbent), Superwhisper (the cross-platform
    // sibling product). Each teaser leads with the angle of
    // difference, not "we win".
    $teasers = [
        [
            'href'     => route('compare.wispr-flow'),
            'eyebrow'  => 'Cloud subscription vs local and free',
            'title'    => 'Utterbar vs Wispr Flow',
            'body'     => 'Wispr Flow is the most-marketed modern dictation app. They send your voice to their servers and bill you monthly; we run it on your PC and charge once. Full table, honest essay.',
        ],
        [
            'href'     => route('compare.voice-access'),
            'eyebrow'  => 'Free Windows feature vs paid app',
            'title'    => 'Utterbar vs Windows Voice Access',
            'body'     => 'Voice Access is free and built into Windows 11. It is genuinely good at OS voice control and basic dictation. So why pay anything? Honest answer: only if Voice Access does not do what you actually need.',
        ],
        [
            'href'     => route('compare.dragon'),
            'eyebrow'  => 'Legacy enterprise vs modern Windows',
            'title'    => 'Utterbar vs Dragon',
            'body'     => 'Dragon (Microsoft-owned since 2022) has run Windows dictation for thirty years. It still wins on legal/medical vocabularies and OS-wide voice control. We win on direct GBP pricing, modern installer, and active updates.',
        ],
        [
            'href'     => route('compare.superwhisper'),
            'eyebrow'  => 'Cross-platform sibling',
            'title'    => 'Utterbar vs Superwhisper',
            'body'     => 'Superwhisper ships on Mac, Windows, and iOS with strong privacy defaults and optional LLM rewrites. We are Windows-only with strict on-device processing, free and open source. Both are honest products with different shapes.',
        ],
    ];
@endphp

<x-layouts.app
    title="How Utterbar compares"
    description="Honest, line-by-line comparisons between Utterbar and the dictation tools you might already be considering: Wispr Flow, Windows Voice Access, Dragon, and Superwhisper. Free and open source, local-only, Windows-native.">

    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <p class="ub-eyebrow">Comparisons</p>
                <h1 class="ub-fg-hi mt-4" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    How Utterbar <x-hl>compares</x-hl>.
                </h1>
                <p class="ub-lead mt-8 max-w-2xl">
                    Honest, line-by-line comparisons against the dictation tools you might already be considering. No spin, no fake checkmarks, no pretending the answer is always &ldquo;ours&rdquo;.
                </p>
            </div>
        </div>
    </section>

    {{-- Per-competitor teaser cards. Two-up on desktop so each card has
         room for a real paragraph; one-up on mobile. The accent rail
         on hover matches the existing card pattern used on /docs and
         /features. --}}
    <section class="py-8 md:py-12">
        <div class="ub-container">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-5 max-w-5xl">
                @foreach ($teasers as $t)
                    <a href="{{ $t['href'] }}" class="ub-card ub-card-rail block">
                        <p class="ub-eyebrow mb-3">{{ $t['eyebrow'] }}</p>
                        <h2 class="text-2xl md:text-3xl" style="letter-spacing: -0.025em; color: var(--fg-hi);">
                            {{ $t['title'] }}
                        </h2>
                        <p class="mt-4" style="color: var(--muted); line-height: 1.55;">
                            {{ $t['body'] }}
                        </p>
                        <p class="mt-6 font-mono text-xs uppercase tracking-wider" style="color: var(--accent); letter-spacing: 0.14em;">
                            Read the comparison
                        </p>
                    </a>
                @endforeach
            </div>
        </div>
    </section>

    {{-- General "cloud vs local" orientation block. Sits below the
         per-competitor cards so a visitor who isn't sure which
         competitor to compare against still gets something concrete
         to read. Reuses the existing <x-compare-grid> component. --}}
    <section class="py-12 md:py-20" style="background: linear-gradient(180deg, transparent, color-mix(in oklab, var(--bg-1) 60%, transparent), transparent);">
        <div class="ub-container">
            <div class="max-w-3xl mx-auto">
                <h2 class="text-2xl md:text-3xl mb-4" style="letter-spacing: -0.025em;">Cloud or local?</h2>
                <p class="mb-8" style="color: var(--muted);">
                    Most modern dictation apps fall into one of two camps. The headline trade-offs are the same regardless of which specific product you compare against, so it is worth knowing them before you read any individual page.
                </p>
            </div>
            <div class="max-w-4xl mx-auto">
                <x-compare-grid />
            </div>
        </div>
    </section>

    {{-- Other apps mentioned but not yet given a full page. Honest
         about scope: we list them so a visitor doesn't bounce
         looking for "the MacWhisper page", and we're up front
         that they are not full deep dives yet. --}}
    <section class="py-12 md:py-16">
        <div class="ub-container">
            <div class="max-w-4xl mx-auto">
                <h2 class="text-2xl md:text-3xl mb-4" style="letter-spacing: -0.025em;">Other apps in the same conversation.</h2>
                <p class="mb-6" style="color: var(--muted);">
                    These come up often in &ldquo;what should I use to dictate?&rdquo; threads. We don&rsquo;t have full pages for them yet; here is the honest one-liner so you know what to look at next.
                </p>
                <ul class="space-y-3 text-base" style="color: var(--fg);">
                    <li><strong style="color: var(--fg-hi);">MacWhisper</strong>: macOS-only Whisper wrapper, pay-once. The closest spiritual sibling to Utterbar on the Mac side.</li>
                    <li><strong style="color: var(--fg-hi);">Talon Voice</strong>: cross-platform voice control framework, free, steep learning curve. For power users who treat voice as a programming surface.</li>
                    <li><strong style="color: var(--fg-hi);">Aqua Voice</strong>: cloud, AI-rewriting focused. Adjacent to Wispr Flow rather than to us.</li>
                </ul>
            </div>
        </div>
    </section>

    <x-prompt-cta
        :heading="'Or just try it.'"
        :sub="'Free forever for unlimited dictation, no account, no rate limits. Two-minute install.'"
    />
</x-layouts.app>
