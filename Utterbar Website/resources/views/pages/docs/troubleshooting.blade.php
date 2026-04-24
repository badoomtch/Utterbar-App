<x-layouts.docs title="Troubleshooting">
    <h1 class="ub-docs-h1">When something&rsquo;s <x-hl>off</x-hl>.</h1>
    <p class="ub-lead mt-6">When transcription goes quiet, slow, or weird &mdash; here&rsquo;s what to check.</p>

    @php
        $items = [
            ['q' => 'The hotkey does nothing.', 'a' => 'Another app is probably holding the global hotkey. Cycle Utterbar\'s hotkey in Settings → Shortcuts. Windows 11\'s built-in voice access also claims Ctrl+Windows keys — check you don\'t have that on.'],
            ['q' => 'The LED is red.', 'a' => 'The mic input failed. Check Settings → Audio Input. Bluetooth headsets often register twice — pick the one with "Hands-Free" in the name for best results.'],
            ['q' => 'Transcription is slow (>1s after speaking).', 'a' => 'On CPU-only machines under ~4 cores this is expected. Upgrade to the large Parakeet model in Pro, or enable DirectML if you have an AMD/Intel GPU.'],
            ['q' => 'Words are coming out wrong.', 'a' => 'Add them to Vocabulary with a boost weight. For product names or acronyms, add both the full and abbreviated forms.'],
            ['q' => 'The pill disappeared.', 'a' => 'Ctrl+Alt+\\ toggles visibility. If it\'s off-screen, right-click the tray icon and pick Reset Pill Position.'],
        ];
    @endphp

    <div class="mt-8"><x-faq-accordion :items="$items" /></div>

    <div class="mt-12 ub-card">
        <p class="text-sm" style="color: var(--muted);">Still stuck? Email <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a> with the output of Settings → About → Copy diagnostics.</p>
    </div>
</x-layouts.docs>
