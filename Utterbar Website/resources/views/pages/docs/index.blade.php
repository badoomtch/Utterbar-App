<x-layouts.docs title="Docs" description="The Utterbar docs. Getting started, modes, vocabulary, shortcuts, privacy controls, troubleshooting, FAQ.">
    <h1 class="ub-docs-h1"><x-wordmark /> <x-hl>docs</x-hl>.</h1>
    <p class="ub-lead mt-6">
        How to dictate well on Windows. Updated with every release.
    </p>

    <div class="mt-12 grid grid-cols-1 md:grid-cols-2 gap-5">
        @foreach ([
            ['Getting started', 'Install, first-run model download, your first dictation.', route('docs.getting-started'), 'rocket'],
            ['Modes', 'How modes bundle a model, prompt, and formatting rules.', route('docs.modes'), 'layers'],
            ['Vocabulary', 'Teach Utterbar your names, acronyms, and product terms.', route('docs.vocabulary'), 'book-open'],
            ['Shortcuts', 'Every hotkey and how to remap them.', route('docs.shortcuts'), 'keyboard'],
            ['Privacy controls', 'Global toggles for clipboard, selection, and active-app context.', route('docs.privacy-controls'), 'lock'],
            ['Troubleshooting', 'When transcription goes quiet, slow, or weird.', route('docs.troubleshooting'), 'alert-circle'],
            ['FAQ', 'The questions we always get.', route('docs.faq'), 'info'],
            ['IT deployment', 'For the IT team rolling Utterbar out across a fleet.', route('docs.it-deployment'), 'shield'],
        ] as [$title, $body, $href, $icon])
            <a href="{{ $href }}" class="ub-card ub-card-rail flex items-start gap-4 no-underline">
                <span class="inline-flex items-center justify-center w-10 h-10 rounded-lg shrink-0" style="background: var(--bg-2); color: var(--accent);">
                    <x-icon :name="$icon" size="20" />
                </span>
                <div>
                    <p class="font-semibold" style="color: var(--fg-hi);">{{ $title }}</p>
                    <p class="mt-1 text-sm" style="color: var(--muted);">{{ $body }}</p>
                </div>
            </a>
        @endforeach
    </div>
</x-layouts.docs>
