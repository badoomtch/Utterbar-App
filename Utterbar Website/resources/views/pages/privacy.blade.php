@php
    $comparison = [
        ['utterbar' => 'Runs on your PC',             'others' => 'Runs on someone else\'s server'],
        ['utterbar' => 'No account, ever',            'others' => 'Sign up to dictate'],
        ['utterbar' => 'Offline forever',             'others' => 'Needs internet every time'],
        ['utterbar' => 'Pay once, own it',            'others' => 'Rent it every month'],
        ['utterbar' => 'Captures your PC\'s audio',    'others' => 'Microphone only'],
        ['utterbar' => 'No length limits',            'others' => 'Word caps and rate limits'],
        ['utterbar' => 'Audio discarded on the spot', 'others' => 'Audio uploaded and stored'],
        ['utterbar' => 'No telemetry, not even opt-in','others' => 'Telemetry on by default'],
        ['utterbar' => 'Visible network log',         'others' => 'Opaque background calls'],
        ['utterbar' => 'Works with the ethernet unplugged', 'others' => 'Works when their servers feel like it'],
    ];
@endphp

<x-layouts.app title="Privacy" description="Utterbar's privacy promise: zero cloud processing, no account, no telemetry. Your voice is transcribed on your PC and never leaves it. Verifiable.">
    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <h1 class="ub-fg-hi" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Your voice <x-hl>never</x-hl> leaves.
                </h1>
                <p class="ub-lead mt-8 max-w-xl">
                    Not &lsquo;encrypted in transit&rsquo;. Not &lsquo;processed on secure servers&rsquo;. It just doesn&rsquo;t go anywhere. This is the essay version; the legal page lives under /legal/privacy.
                </p>
            </div>
        </div>
    </section>

    {{-- Primary visual: mic → local Parakeet → app, with cloud path crossed out --}}
    <section class="pt-4 pb-12 md:pb-16">
        <div class="ub-container">
            <div class="max-w-4xl mx-auto">
                <x-video-embed id="feature-local-privacy" />
            </div>
        </div>
    </section>

    {{-- Comparison: Utterbar vs everything else --}}
    <section class="py-6">
        <div class="ub-container">
            <div class="max-w-4xl mx-auto">
                <h2 class="text-2xl md:text-3xl"><x-wordmark /> vs everything else.</h2>
                <p class="ub-lead mt-4">Ten lines. The whole pitch.</p>
            </div>

            <div class="mt-10 max-w-5xl mx-auto rounded-2xl overflow-hidden" style="background: var(--bg-1);">
                <div class="grid grid-cols-1 md:grid-cols-2">

                    {{-- Utterbar column --}}
                    <div class="p-8 md:p-10 relative"
                         style="background:
                            radial-gradient(120% 80% at 0% 0%, color-mix(in oklab, var(--accent) 12%, transparent), transparent 60%),
                            color-mix(in oklab, var(--bg-2) 60%, var(--bg-1));
                            border-left: 2px solid var(--accent);">
                        <div class="flex items-center gap-3 mb-8">
                            <span style="color: var(--accent);"><x-icon name="check" size="18" stroke="2.4" /></span>
                            <h3 class="text-xl" style="letter-spacing: -0.02em;"><x-wordmark /></h3>
                        </div>
                        <ul class="space-y-4">
                            @foreach ($comparison as $row)
                                <li class="flex items-start gap-3 text-base" style="color: var(--fg);">
                                    <span class="mt-0.5 shrink-0" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.4" /></span>
                                    <span>{{ $row['utterbar'] }}</span>
                                </li>
                            @endforeach
                        </ul>
                    </div>

                    {{-- Everything else column --}}
                    <div class="p-8 md:p-10" style="border-left: 1px solid var(--border);">
                        <div class="flex items-center gap-3 mb-8">
                            <span style="color: var(--fg-dim); opacity: 0.6;"><x-icon name="x" size="18" stroke="2" /></span>
                            <h3 class="text-xl" style="letter-spacing: -0.02em; color: var(--muted);">Everything else</h3>
                        </div>
                        <ul class="space-y-4">
                            @foreach ($comparison as $row)
                                <li class="flex items-start gap-3 text-base" style="color: var(--muted);">
                                    <span class="mt-0.5 shrink-0" style="color: var(--muted); opacity: 0.55;"><x-icon name="x" size="16" stroke="2" /></span>
                                    <span style="text-decoration: line-through; text-decoration-color: color-mix(in oklab, var(--muted) 40%, transparent); text-decoration-thickness: 1px;">{{ $row['others'] }}</span>
                                </li>
                            @endforeach
                        </ul>
                    </div>

                </div>
            </div>
        </div>
    </section>

    <x-section title='What <span class="ub-wm"><span>Utter</span><span class="ub-wm-bar">bar</span></span> stores on disk.'>
        <div class="ub-card max-w-3xl">
            <ul class="space-y-3">
                <li class="flex gap-3"><span style="color: var(--accent);"><x-icon name="hard-drive" size="20" /></span><span>Everything lives under <code class="font-mono text-sm px-1 rounded" style="background: var(--bg-2); color: var(--accent);">%APPDATA%\\Utterbar\\</code>.</span></li>
                <li class="flex gap-3"><span style="color: var(--accent);"><x-icon name="file-text" size="20" /></span><span>Transcripts are saved in a local database.</span></li>
                <li class="flex gap-3"><span style="color: var(--accent);"><x-icon name="mic" size="20" /></span><span>Audio is only retained if you turn on history-with-audio (it's off by default).</span></li>
                <li class="flex gap-3"><span style="color: var(--accent);"><x-icon name="x" size="20" /></span><span>Delete the folder to wipe everything. No residue.</span></li>
            </ul>
        </div>
    </x-section>

    <x-section title='What <span class="ub-wm"><span>Utter</span><span class="ub-wm-bar">bar</span></span> sends over the network.'
               lead="The speech model downloads once, from a short allowlist of hosts. Nothing else.">
        <div class="ub-card max-w-3xl font-mono text-sm" style="background: var(--bg-0);">
            <p style="color: var(--muted);"># network log &mdash; first run</p>
            <p class="mt-2"><span style="color: var(--accent);">&rarr;</span> GET model-host/&hellip;/speech-model</p>
            <p><span style="color: var(--accent);">&rarr;</span> GET model-host/&hellip;/tokenizer</p>
            <p class="mt-2" style="color: var(--muted);"># every subsequent run</p>
            <p><span style="color: var(--muted);">&mdash;</span> nothing</p>
        </div>
    </x-section>

    <x-section title="Why there&rsquo;s no telemetry.">
        <div class="max-w-3xl">
            <p class="text-lg" style="color: var(--fg);">Every telemetry pipeline starts as an honest wish to improve the product. A year later it&rsquo;s forty fields, three vendors, and a banner nobody reads. I&rsquo;ve built that pipeline at other jobs; I&rsquo;m not building it here.</p>
            <p class="mt-4 text-lg" style="color: var(--muted);">If I ever add diagnostics it will be opt-in, off by default, and you&rsquo;ll be able to read every payload in plain text before it&rsquo;s sent. Until then: zero.</p>
        </div>
    </x-section>

    <x-section>
        <div class="flex flex-wrap gap-3">
            <a href="{{ route('legal.privacy') }}" class="ub-btn ub-btn-ghost">Read the legal privacy policy</a>
            <a href="mailto:{{ config('site.email.contact') }}?subject=Utterbar%20source%20review" class="ub-btn ub-btn-ghost">Request a source review</a>
        </div>
        <p class="mt-4 text-sm max-w-2xl" style="color: var(--muted);">
            Security researchers and enterprise buyers can request access to the Rust core under NDA.
            The app itself is distributed as a signed binary, not open source &mdash; that way
            the paid features stay paid.
        </p>
    </x-section>
</x-layouts.app>
