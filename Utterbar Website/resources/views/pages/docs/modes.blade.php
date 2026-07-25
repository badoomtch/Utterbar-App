<x-layouts.docs title="Modes" description="How Utterbar's modes shape the same dictated speech into voice-to-text, a clean message, or verbatim meeting notes. Switched in a click or by hotkey.">
    <h1 class="ub-docs-h1">Modes that <x-hl>match</x-hl> the job.</h1>
    <p class="ub-lead mt-6">A mode is a small post-processing recipe: take the raw transcript, apply a formatting rule-set, type the result. Change mode and the same speech comes out shaped differently, no extra dictation, no rewriting.</p>

    {{-- ───────────── The three built-ins ───────────── --}}
    <h2 class="mt-12 ub-docs-h2">The three built-in modes</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Every install ships with three modes. They&rsquo;re the patterns we found ourselves wanting most often during dictation: the raw output, the cleaned-up chat version, and the verbatim-with-line-breaks meeting capture. Cycle through them with <x-kbd-chip :keys="['Ctrl','Alt','M']" /> or pick from the floating bar.
    </p>

    <div class="mt-6 ub-card p-0 overflow-hidden">
        <table class="w-full text-sm">
            <thead>
                <tr style="background: var(--bg-2); color: var(--fg-hi);">
                    <th class="text-left p-3 font-semibold">Mode</th>
                    <th class="text-left p-3 font-semibold">Post-processing</th>
                    <th class="text-left p-3 font-semibold">What it&rsquo;s for</th>
                </tr>
            </thead>
            <tbody>
                @foreach ([
                    ['Voice-to-text', 'None (pass-through)',          'Just types what you said. Default.'],
                    ['Message',       'Deterministic template',       'Casual tone, sentence cap, no filler.'],
                    ['Meeting',       'Deterministic, preserves \\n', 'Verbatim capture; keeps line breaks.'],
                ] as [$name, $proc, $use])
                    <tr class="border-t" style="border-color: var(--border);">
                        <td class="p-3 align-top font-medium" style="color: var(--fg-hi);">{{ $name }}</td>
                        <td class="p-3 align-top" style="color: var(--muted);">{{ $proc }}</td>
                        <td class="p-3 align-top" style="color: var(--fg);">{{ $use }}</td>
                    </tr>
                @endforeach
            </tbody>
        </table>
    </div>

    {{-- ───────────── Voice-to-text ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">Voice-to-text: the default</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Pure pass-through. Whatever the speech model emits is what you get, with no rewriting and no formatting beyond the standard punctuation and capitalisation the model already applies. This is the right mode for one-off short dictations, chat replies, search bars, single sentences.
    </p>
    <div class="mt-5 ub-card" style="padding: 1rem 1.25rem;">
        <p class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted); letter-spacing: 0.12em;">You said</p>
        <p class="mt-2 text-base italic" style="color: var(--fg);">&ldquo;remind me to send the QA report tomorrow morning&rdquo;</p>
        <p class="mt-4 font-mono text-xs uppercase tracking-wider" style="color: var(--accent); letter-spacing: 0.12em;">It types</p>
        <p class="mt-2 text-base" style="color: var(--fg-hi);">Remind me to send the QA report tomorrow morning.</p>
    </div>

    {{-- ───────────── Message ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">Message: cleaned for chat</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        A deterministic template that strips obvious filler (&ldquo;um&rdquo;, &ldquo;like&rdquo;, &ldquo;basically&rdquo;), tightens hedging, and caps the result at a small number of sentences. The shape is conversational rather than verbatim, what you&rsquo;d send to a colleague, not what a court reporter would write down.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        It runs entirely on rules, no LLM, no external service, no &ldquo;your message has been re-imagined as polished business prose&rdquo;. The output stays recognisably yours.
    </p>
    <div class="mt-5 ub-card" style="padding: 1rem 1.25rem;">
        <p class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted); letter-spacing: 0.12em;">You said</p>
        <p class="mt-2 text-base italic" style="color: var(--fg);">&ldquo;so basically we need to push the launch to Tuesday because QA flagged two issues with the auth flow and we want them fixed before marketing pings the list&rdquo;</p>
        <p class="mt-4 font-mono text-xs uppercase tracking-wider" style="color: var(--accent); letter-spacing: 0.12em;">It types</p>
        <p class="mt-2 text-base" style="color: var(--fg-hi);">Quick heads up, pushing launch to Tuesday. QA flagged two auth-flow issues we want fixed before marketing sends the announcement.</p>
    </div>

    {{-- ───────────── Meeting ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">Meeting: verbatim, line-broken</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Verbatim capture, nothing trimmed, nothing rewritten, with line breaks preserved on natural pauses. Use it when you want a literal record of what was said, not a cleaned-up summary. Combined with system-audio capture (Settings &gt; Audio &gt; System audio), this is what most people use for transcribing a meeting they&rsquo;re sitting in.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Meeting Mode runs through Whisper rather than Parakeet, because Whisper handles longer recordings and multi-speaker audio more reliably. The default is Whisper Small, which is comfortably faster than realtime on most laptops. Pick a different one in Settings &gt; Models &gt; Meeting mode model: Fast (Small), Balanced (Medium), or Best accuracy (Large v3). Parakeet and other engines are not selectable for Meeting Mode; the chunked transcribe path is built around Whisper.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        A single Meeting Mode recording is capped at about an hour. Speaker labels are off in this release; transcripts come back timestamped without speaker names. The diarisation rework is planned for a future release.
    </p>
    <div class="mt-5 ub-card" style="padding: 1rem 1.25rem;">
        <p class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted); letter-spacing: 0.12em;">You said</p>
        <p class="mt-2 text-base italic" style="color: var(--fg);">&ldquo;so the launch is moving to Tuesday&hellip; pause &hellip; QA flagged two issues with auth&hellip; pause &hellip; we want them fixed before marketing announces&rdquo;</p>
        <p class="mt-4 font-mono text-xs uppercase tracking-wider" style="color: var(--accent); letter-spacing: 0.12em;">It types</p>
        <pre class="mt-2 text-base whitespace-pre-wrap" style="color: var(--fg-hi); font-family: inherit;">So the launch is moving to Tuesday.
QA flagged two issues with auth.
We want them fixed before marketing announces.</pre>
    </div>

    {{-- ───────────── How to switch ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">How to switch modes</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Three ways:
    </p>
    <ul class="mt-4 space-y-3 text-base leading-relaxed" style="color: var(--fg);">
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">Cycle hotkey.</strong> <x-kbd-chip :keys="['Ctrl','Alt','M']" /> rotates through the active modes. The floating bar shows the current one.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">Per-mode hotkey.</strong> Each mode can have its own dedicated key. Useful if you have one or two modes you use 90% of the time. Set in Settings &gt; Shortcuts.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">Click the mode chip.</strong> The mode name on the floating bar is clickable; pops a small selector.</span></li>
    </ul>

    {{-- ───────────── Custom modes ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">Custom modes</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Utterbar includes unlimited custom modes. Clone a built-in, edit the post-processing rules, and bind it to its own hotkey. The most common reasons people make custom modes:
    </p>
    <ul class="mt-4 space-y-3 text-base leading-relaxed" style="color: var(--fg);">
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">A &ldquo;quick reply&rdquo; mode</strong> for chat apps, tighter than Message, no sentence cap, optional sign-off.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">A &ldquo;code commit&rdquo; mode</strong> that lower-cases the first word and prepends a conventional-commit prefix.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">A &ldquo;markdown notes&rdquo; mode</strong> that turns &ldquo;new bullet&rdquo; / &ldquo;sub-bullet&rdquo; into <code class="font-mono" style="color: var(--accent);">-</code> / <code class="font-mono" style="color: var(--accent);">  -</code>.</span></li>
    </ul>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Whether a mode can read your clipboard, current text selection, or the active app name is governed by the global Privacy settings, one switch per source, applied to every mode. See <a href="{{ route('docs.privacy-controls') }}" style="color: var(--accent);">Privacy controls</a> for the details.
    </p>

    {{-- ───────────── Per-mode vs global Smart rewrite ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">Per-mode post-processing vs the Smart rewrite toggle</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Per-mode post-processing is the recipe baked into a mode: <em>Voice-to-text</em> passes through, <em>Message</em> tightens for chat, <em>Meeting</em> preserves verbatim with line breaks. It runs whenever that mode is active and you can&rsquo;t turn it off without switching modes.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Smart rewrite, on the other hand, is a single global toggle in Settings &gt; Text Output &gt; Formatting that applies <em>across all modes except Meeting</em>. When on, it strips common filler words (&ldquo;um&rdquo;, &ldquo;uh&rdquo;, &ldquo;like&rdquo;, &ldquo;you know&rdquo;), normalises spacing, capitalises sentence starts, and ensures a terminal full stop. It&rsquo;s off by default, deterministic, rule-based, and never loads an AI model. Think of per-mode rules as <em>shape</em> and Smart rewrite as <em>polish</em>. Turn polish off for code dictation; Meeting Mode ignores it either way.
    </p>

    <div class="mt-12 ub-card ub-card-rail">
        <p class="text-sm" style="color: var(--muted);">
            <strong style="color: var(--fg-hi);">Why no LLM-driven modes?</strong> A local LLM big enough to do meaningful free-form rewrites takes seconds per dictation, not milliseconds. The deterministic post-processing approach keeps Utterbar instant. If you want LLM-style rewriting, the <a href="{{ route('compare.wispr-flow') }}" style="color: var(--accent);">comparison page</a> covers when that&rsquo;s the right trade-off.
        </p>
    </div>
</x-layouts.docs>
