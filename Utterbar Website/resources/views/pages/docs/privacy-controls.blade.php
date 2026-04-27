<x-layouts.docs title="Privacy controls" description="Three global switches (clipboard, selection, active-app) that govern what context Utterbar may read. All off by default, all stay local.">
    <h1 class="ub-docs-h1">Three <x-hl>switches</x-hl>, all off.</h1>
    <p class="ub-lead mt-6">Flip them on in Settings &rarr; Privacy when you want Utterbar to know what you were looking at when you spoke. One switch per source, applied globally.</p>

    <div class="mt-10 space-y-6">
        @foreach ([
            ['Clipboard access', 'When on: the current clipboard contents are available to the post-processing prompt. Useful for "rewrite this in a formal tone" where the clipboard holds the source text.'],
            ['Selection access', 'When on: the currently-selected text in the focused app is passed to post-processing as context.'],
            ['Active-app context', 'When on: the name of the focused app is passed into post-processing so modes can adapt their output to it. Nothing else, no window title, no URL.'],
        ] as [$title, $body])
            <div class="ub-card">
                <div class="flex items-start gap-4">
                    <span style="color: var(--accent);"><x-icon name="lock" size="22" /></span>
                    <div>
                        <h3 class="text-xl">{{ $title }}</h3>
                        <p class="mt-2 text-sm leading-relaxed" style="color: var(--muted);">{{ $body }}</p>
                    </div>
                </div>
            </div>
        @endforeach
    </div>

    <div class="mt-10 ub-card"><p class="text-sm" style="color: var(--muted);">None of these toggles changes whether audio leaves your machine. It never does. They only affect what local context the post-processor can see. The app does make a handful of network calls (licence checks, update checks, model downloads, pricing lookup) but never sends audio, transcripts, vocabulary, or history. Full breakdown on the <a href="{{ route('legal.privacy') }}" style="color: var(--accent);">privacy policy</a>.</p></div>
</x-layouts.docs>
