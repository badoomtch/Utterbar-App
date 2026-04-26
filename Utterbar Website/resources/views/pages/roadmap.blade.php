@php
    $columns = [
        'Now' => [
            ['title' => 'Signed release pipeline',        'desc' => 'Code-signed MSI + NSIS binaries. Unsigned today.'],
            ['title' => 'Auto-update installer flow',     'desc' => 'The toggle exists in settings; the installer round-trip is what\'s left.'],
            ['title' => 'File-transcription drag-drop',   'desc' => 'Decoder + ASR pipeline already work end-to-end. Polished drop-zone UX is on the punch-list.'],
            ['title' => 'First-run onboarding + model-download UX', 'desc' => 'The first 30 seconds of the app need to feel less like a README.'],
        ],
        'Next' => [
            ['title' => 'macOS build',                    'desc' => 'Second-class port once Windows 1.0 ships. Mac already has decent local-only options meanwhile.'],
            ['title' => 'File transcription batch mode',  'desc' => 'Drop a folder, walk away, come back to transcripts for every file inside.'],
            ['title' => 'Custom model imports',           'desc' => 'Drop a sherpa-onnx tarball, get a new model. Today the catalogue is fixed.'],
            ['title' => 'Multilingual Parakeet',          'desc' => 'The 0.6B v2 models in the catalogue are English-only. Multilingual variants come next.'],
        ],
        'Later' => [
            ['title' => 'Linux build',                    'desc' => 'Third-class. No ETA.'],
            ['title' => 'Bilingual dictation modes',      'desc' => 'Switch language mid-sentence without changing mode.'],
            ['title' => 'Mobile remote',                  'desc' => 'Press-to-talk from your phone into the active desktop session.'],
            ['title' => 'Code-editor extension',          'desc' => 'True in-IDE cursor injection, not keystroke simulation.'],
        ],
    ];

    $shipped = [
        ['title' => 'Local LLM post-processor',          'desc' => 'llama.cpp feature-gated at compile time with a deterministic fallback. Tone / format / summarise.'],
        ['title' => 'Cycle-mode, open-settings, open-history global hotkeys', 'desc' => 'All seven hotkey slots individually rebindable.'],
        ['title' => 'WASAPI system-audio capture',       'desc' => 'Direct loopback on the default render endpoint. No virtual-cable install. Mic / system / mixed.'],
        ['title' => 'Adaptive noise suppression',        'desc' => 'DC blocker &rarr; high-pass &rarr; adaptive gate, in-house, calibrates from the quietest 20% of the frame.'],
        ['title' => 'Keystroke fallback for UIPI-blocked targets', 'desc' => 'If clipboard paste is blocked (RDP, elevated apps), per-character Unicode keystroke injection takes over automatically.'],
        ['title' => 'Shift-to-auto-send',                'desc' => 'Hold Shift while dictating to fire Enter after the paste. For chat-box dictation.'],
        ['title' => 'Retain-audio with day slider',      'desc' => '1&ndash;90 day retention. Off by default. Pruning runs on every new dictation.'],
        ['title' => 'Reduce-motion accessibility',       'desc' => 'Kills transitions across the app for users who ask for it in Windows settings or the app.'],
        ['title' => 'Synthesised sound cues + toasts',   'desc' => 'Start / stop / error beeps generated in-app via WebAudio; no sample files. Windows toast notifications for non-blocking feedback.'],
        ['title' => 'Themes + accents, bar visibility, size, outline', 'desc' => 'Three themes &times; six accents. Bar hides / appears on record / stays. Compact or Standard, with or without outline.'],
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
    <section class="pb-20 pt-4 border-t" style="border-color: var(--border);">
        <div class="ub-container">
            <div class="flex items-baseline gap-3 mb-6 pt-8">
                <span style="color: var(--ok);"><x-icon name="check" size="20" stroke="2.4" /></span>
                <h2 class="text-2xl">Shipped</h2>
                <span class="font-mono text-xs" style="color: var(--muted);">{{ count($shipped) }} &middot; in v{{ config('site.version') }}</span>
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
</x-layouts.app>
