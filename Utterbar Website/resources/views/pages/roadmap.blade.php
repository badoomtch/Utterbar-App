@php
    $columns = [
        'Now' => [
            ['title' => 'Custom model imports',           'desc' => 'Drop a sherpa-onnx tarball, get a new model. Today the catalogue is fixed; opening it up so users can bring their own engines.'],
            ['title' => 'Vocabulary docs + UX polish',    'desc' => 'The vocabulary engine is fully wired but the docs and surfaces still need a proper guide and a few rough-edge fixes before 1.0.'],
        ],
        'Next' => [
            ['title' => 'macOS build',                    'desc' => 'Second-class port once Windows 1.0 ships. The core ASR + post-processing pipeline is portable; system integration (audio, hotkeys, paste) is the rewrite.'],
            ['title' => 'Bilingual dictation modes',      'desc' => 'Switch language mid-sentence without changing mode. Whisper already supports it under the hood; the UX needs to catch up.'],
            ['title' => 'Code-editor extension',          'desc' => 'True in-IDE cursor injection, not keystroke simulation. VS Code first, JetBrains after.'],
        ],
        'Later' => [
            ['title' => 'Linux build',                    'desc' => 'Third-class. No ETA.'],
            ['title' => 'Mobile remote',                  'desc' => 'Press-to-talk from your phone into the active desktop session.'],
        ],
    ];

    $shipped = [
        ['title' => 'Multi-engine ASR',                  'desc' => 'Parakeet-TDT (default English) and Whisper (multilingual; Base, Small, Medium, Large v3). Pick per-mode or globally.'],
        ['title' => 'DirectML GPU acceleration',         'desc' => 'Embedded ONNX Runtime DirectML build runs Whisper Medium, Whisper Large, and Parakeet on any DirectX 12 graphics card. Auto picks the right path per model. CUDA is not bundled.'],
        ['title' => 'Auto-update with signed metadata',  'desc' => 'Updater verifies an Ed25519 signature over release metadata before downloading; SHA-256-checks the binary; refuses anything older than what\'s installed; only pulls from dl.utterbar.com.'],
        ['title' => 'Stable + Beta channels',            'desc' => 'Two tracks: Stable for everyone, Beta for early adopters. Toggle in Settings > Updates.'],
        ['title' => 'File transcription',                'desc' => 'Drop an audio or video file in, get a transcript. Queues, history, retranscribe.'],
        ['title' => 'First-run onboarding + model picker', 'desc' => 'Three-engine picker on first launch with download progress. No more cold-boot README moment.'],
        ['title' => 'Vocabulary capture',                'desc' => '"Add to vocab" from any dictation; literal + regex replacements; hinted words for ASR decoding.'],
        ['title' => 'WASAPI system-audio capture',       'desc' => 'Direct loopback on the default render endpoint. No virtual-cable install. Mic / system / mixed.'],
        ['title' => 'Cycle-mode + 6 other global hotkeys', 'desc' => 'All seven hotkey slots individually rebindable.'],
        ['title' => 'Adaptive noise suppression',        'desc' => 'DC blocker, high-pass, and adaptive gate, in-house, calibrates from the quietest 20% of the frame.'],
        ['title' => 'Keystroke fallback for UIPI-blocked targets', 'desc' => 'If clipboard paste is blocked (RDP, elevated apps), per-character Unicode keystroke injection takes over automatically.'],
        ['title' => 'Shift-to-auto-send',                'desc' => 'Hold Shift while dictating to fire Enter after the paste. For chat-box dictation.'],
        ['title' => 'Retain-audio with day slider',      'desc' => '1&ndash;90 day retention. Off by default. Pruning runs on every new dictation.'],
        ['title' => 'Reduce-motion + high-contrast accessibility', 'desc' => 'Honours Windows reduce-motion + offers in-app overrides for high contrast and larger focus rings.'],
        ['title' => 'Synthesised sound cues + toasts',   'desc' => 'Start / stop / error beeps generated in-app via WebAudio; no sample files. Windows toast notifications for non-blocking feedback.'],
        ['title' => 'Themes + accents, bar visibility, size, outline', 'desc' => 'Three themes &times; six accents (plus custom Pro). Bar hides / appears on record / stays. Compact or Standard, with or without outline.'],
    ];

    $parked = [
        ['title' => 'Code-signed installer',             'desc' => 'Permanently parked. The cert cost outweighs the SmartScreen warning, which fades on its own as install volume builds.'],
    ];

    $badges = ['Now' => 'var(--accent)', 'Next' => 'var(--fg-hi)', 'Later' => 'var(--muted)'];
@endphp

<x-layouts.app title="Roadmap">
    <x-section eyebrow="Roadmap" title="What's shipping, what's next, what's already done."
               lead="Public commitments are in Now and Next. Later is ideas, no promises. Shipped is everything you'd find in the app if you installed it today.">
    </x-section>

    <section class="pb-12">
        <div class="ub-container">
            <div class="grid grid-cols-1 md:grid-cols-3 gap-5 items-start">
                @foreach ($columns as $name => $items)
                    <div>
                        <div class="flex items-baseline gap-3 mb-4">
                            <span class="inline-block w-2 h-2 rounded-full" style="background: {{ $badges[$name] }};"></span>
                            <h2 class="text-2xl">{{ $name }}</h2>
                            <span class="font-mono text-xs" style="color: var(--muted);">{{ count($items) }}</span>
                        </div>
                        <div class="space-y-3">
                            @foreach ($items as $item)
                                <div class="ub-card">
                                    <p class="font-semibold" style="color: var(--fg-hi);">{{ $item['title'] }}</p>
                                    <p class="mt-2 text-sm" style="color: var(--muted);">{!! $item['desc'] !!}</p>
                                </div>
                            @endforeach
                        </div>
                    </div>
                @endforeach
            </div>
        </div>
    </section>

    {{-- Shipped --}}
    <section class="pb-12 pt-4 border-t" style="border-color: var(--border);">
        <div class="ub-container">
            <div class="flex items-baseline gap-3 mb-6 pt-8">
                <span style="color: var(--ok);"><x-icon name="check" size="20" stroke="2.4" /></span>
                <h2 class="text-2xl">Shipped</h2>
                <span class="font-mono text-xs" style="color: var(--muted);">{{ count($shipped) }} items &middot; live in the latest release</span>
            </div>
            <p class="max-w-2xl mb-6 text-sm" style="color: var(--muted);">
                Already in the app. Listed here so the pages above stop implying these are still a promise.
            </p>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                @foreach ($shipped as $item)
                    <div class="ub-card" style="padding: 1rem 1.25rem;">
                        <div class="flex items-start gap-3">
                            <span class="mt-0.5 shrink-0" style="color: var(--ok);"><x-icon name="check" size="18" stroke="2.2" /></span>
                            <div>
                                <p class="font-semibold text-sm" style="color: var(--fg-hi);">{{ $item['title'] }}</p>
                                <p class="mt-1 text-sm" style="color: var(--muted);">{!! $item['desc'] !!}</p>
                            </div>
                        </div>
                    </div>
                @endforeach
            </div>
        </div>
    </section>

    {{-- Parked --}}
    <section class="pb-20 pt-4 border-t" style="border-color: var(--border);">
        <div class="ub-container">
            <div class="flex items-baseline gap-3 mb-6 pt-8">
                <span class="inline-block w-2 h-2 rounded-full" style="background: var(--muted);"></span>
                <h2 class="text-2xl">Parked</h2>
                <span class="font-mono text-xs" style="color: var(--muted);">{{ count($parked) }}</span>
            </div>
            <p class="max-w-2xl mb-6 text-sm" style="color: var(--muted);">
                Considered, decided against for now, here so nobody asks twice.
            </p>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                @foreach ($parked as $item)
                    <div class="ub-card" style="padding: 1rem 1.25rem; opacity: 0.85;">
                        <p class="font-semibold text-sm" style="color: var(--fg-hi);">{{ $item['title'] }}</p>
                        <p class="mt-1 text-sm" style="color: var(--muted);">{!! $item['desc'] !!}</p>
                    </div>
                @endforeach
            </div>
        </div>
    </section>
</x-layouts.app>
