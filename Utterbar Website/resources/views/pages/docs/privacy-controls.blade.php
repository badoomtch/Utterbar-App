<x-layouts.docs title="Privacy controls" description="Three global switches (clipboard, selection, active-app) that govern what context Utterbar may read. All off by default, all stay local.">
    <h1 class="ub-docs-h1">Three <x-hl>switches</x-hl>, all off.</h1>
    <p class="ub-lead mt-6">Flip them on in Settings &gt; Privacy when you want Utterbar to know what you were looking at when you spoke. One switch per source, applied globally.</p>

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

    <h2 class="mt-14 ub-docs-h2">Conferencing output mute (Settings &gt; Privacy &gt; Media)</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        A separate switch sits under the same Privacy section, alongside an editable app list. When &ldquo;Mute conferencing app output while recording&rdquo; is on, Utterbar silences what you HEAR from Teams, Slack, Discord, Zoom, and Webex for the duration of a dictation, then restores it the moment you let go of the hotkey. Call audio and notification dings stop bleeding into the room while you&rsquo;re speaking into another window.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        It&rsquo;s output-only: the toggle does <strong>not</strong> mute your microphone inside the call app. True per-app mic mute would require simulating each app&rsquo;s in-call mute shortcut, which is brittle. If you need to mute your mic to a call, use the call app&rsquo;s own mute button. The list below the toggle (&ldquo;Apps to silence&rdquo;) defaults to the five apps above and is editable; add or remove <code class="font-mono" style="color: var(--accent);">.exe</code> names with the input field.
    </p>

    <div class="mt-10 ub-card"><p class="text-sm" style="color: var(--muted);">None of these toggles changes whether audio leaves your machine. It never does. They only affect what local context the post-processor can see, or which call apps get hushed while you dictate. The app does make a handful of network calls (licence checks, update checks, model downloads, pricing lookup) but never sends audio, transcripts, vocabulary, or history. Full breakdown on the <a href="{{ route('legal.privacy') }}" style="color: var(--accent);">privacy policy</a>.</p></div>
</x-layouts.docs>
