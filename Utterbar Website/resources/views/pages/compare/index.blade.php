<x-layouts.app
    title="How Utterbar compares"
    description="Honest comparisons between Utterbar and the other dictation apps you might be considering: Wispr Flow, Dragon, Voice Access, Superwhisper, and others.">

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

    <section class="py-12 md:py-16">
        <div class="ub-container">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-5 max-w-5xl">
                <a href="{{ route('compare.wispr-flow') }}" class="ub-card ub-card-rail block">
                    <p class="ub-eyebrow mb-3">Cloud subscription vs local one-time</p>
                    <h2 class="text-2xl md:text-3xl" style="letter-spacing: -0.025em; color: var(--fg-hi);">
                        Utterbar vs Wispr Flow
                    </h2>
                    <p class="mt-4" style="color: var(--muted); line-height: 1.55;">
                        Wispr Flow is the most-marketed modern dictation app and the closest direct comparison. They send your voice to their servers and bill you monthly; we run it on your PC and charge once. Full table, honest essay.
                    </p>
                    <p class="mt-6 font-mono text-xs uppercase tracking-wider" style="color: var(--accent); letter-spacing: 0.14em;">
                        Read the comparison
                    </p>
                </a>
                <div class="ub-card" style="opacity: 0.55;">
                    <p class="ub-eyebrow mb-3">More comparisons</p>
                    <h2 class="text-2xl md:text-3xl" style="letter-spacing: -0.025em; color: var(--fg);">
                        Coming soon
                    </h2>
                    <p class="mt-4" style="color: var(--muted); line-height: 1.55;">
                        We&rsquo;re drafting comparisons against Dragon, Windows Voice Access, Superwhisper, and Talon. Email <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a> if there&rsquo;s a specific one you want next.
                    </p>
                </div>
            </div>
        </div>
    </section>

    <x-prompt-cta
        :heading="'Or just try it.'"
        :sub="'Free forever, no account, no rate limits. Two-minute install.'"
    />
</x-layouts.app>
