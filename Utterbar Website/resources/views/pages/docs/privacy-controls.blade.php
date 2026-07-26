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
        It&rsquo;s output-only: this toggle does <strong>not</strong> mute your microphone inside the call app, and it covers all five apps above. The list below the toggle (&ldquo;Apps to silence&rdquo;) defaults to those five names and is editable; add or remove <code class="font-mono" style="color: var(--accent);">.exe</code> names with the input field. If you need to mute the microphone, see the experimental section below.
    </p>

    <h2 class="mt-14 ub-docs-h2">Experimental microphone mute, Teams + Discord (Settings &gt; Advanced &gt; Experimental)</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Different feature, different setting, different boundary. Off by default. Two separate toggles, one for Teams and one for Discord. When you enable a toggle and supply the mute hotkey you have configured inside that app, Utterbar sends the hotkey as a global keystroke when you start a recording, and again when you stop. Teams uses <code class="font-mono" style="color: var(--accent);">Ctrl+Shift+M</code> by default; Discord uses whatever you set under User Settings &gt; Keybinds &gt; Toggle Mute.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        While the toggle is muting, a small banner appears below the floating bar asking you to glance at the call app&rsquo;s own mute indicator to confirm. The banner is there because Utterbar <strong>cannot verify the real mute state</strong>: there is no UI Automation reading the call app&rsquo;s mute button, and there is no operating-system-level capture-endpoint mute. Utterbar sends a keystroke and trusts the app received it. If the keystroke is unbound or being intercepted by another app, the keystroke does nothing and you stay live.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Boundaries to keep in mind:
    </p>
    <ul class="mt-3 space-y-2 text-base leading-relaxed" style="color: var(--fg);">
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">Teams and Discord only.</strong> Zoom and Webex are not supported.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">Off by default.</strong> Hidden under Settings &gt; Advanced &gt; Experimental, with a clear explanation of the trade-off.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">No guarantee.</strong> Confirm in the call app&rsquo;s own mute indicator. If your situation needs a guaranteed mute, use the call app&rsquo;s own mute button.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">Crash safety.</strong> If Utterbar is force-killed mid-dictation, the next launch warns you that an unmute may not have fired so you can re-check the call app yourself.</span></li>
    </ul>

    <div class="mt-10 ub-card"><p class="text-sm" style="color: var(--muted);">None of these toggles changes whether audio leaves your machine. It never does. They only affect what local context the post-processor can see, or which call apps get hushed while you dictate. The app does make a small set of network calls (model downloads and update checks) but never sends audio, transcripts, vocabulary, or history. Full breakdown on the <a href="{{ route('legal.privacy') }}" style="color: var(--accent);">privacy policy</a>.</p></div>
</x-layouts.docs>
