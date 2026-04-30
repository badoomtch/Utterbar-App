<x-layouts.docs title="Troubleshooting" description="Quick fixes for the most common Utterbar issues. Silent transcription, slow dictation, mis-recognised words, missing pill, hotkey conflicts.">
    <h1 class="ub-docs-h1">When something&rsquo;s <x-hl>off</x-hl>.</h1>
    <p class="ub-lead mt-6">If dictation is going quiet, slow, or weird, the fix is almost always one of these. Skim the matching section, everything below is in the order you should check it.</p>

    {{-- ───────────── Hotkey ───────────── --}}
    <h2 class="mt-12 ub-docs-h2">The hotkey does nothing</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Almost always a global-hotkey conflict. Another running app is claiming the same combo and taking the keystroke before Utterbar sees it.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Most common culprits in our experience:
    </p>
    <ul class="mt-3 space-y-2 text-base leading-relaxed" style="color: var(--fg);">
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">Windows 11 Voice Access</strong> claims several <code class="font-mono text-sm" style="color: var(--accent);">Win+...</code> combos.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">Chinese / Japanese / Korean IMEs</strong> often hold <code class="font-mono text-sm" style="color: var(--accent);">Ctrl+Space</code> to switch input modes.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">VS Code, IntelliJ and other IDEs</strong> bind <code class="font-mono text-sm" style="color: var(--accent);">Ctrl+Space</code> to autocomplete, that&rsquo;s a window-level binding so it doesn&rsquo;t intercept globally, but it does mean if you press it inside the IDE without focus on a text field, you might see the IDE&rsquo;s autocomplete instead of Utterbar.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">Streaming and screen-recording tools</strong> (OBS, NVIDIA ShadowPlay) bind global hotkeys for clip capture.</span></li>
    </ul>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        The fix: rebind in Settings &gt; Shortcuts. Anything works. <code class="font-mono text-sm" style="color: var(--accent);">F6</code>, <code class="font-mono text-sm" style="color: var(--accent);">Ctrl+`</code>, <code class="font-mono text-sm" style="color: var(--accent);">Caps Lock</code> if you don&rsquo;t use it. Whatever your fingers already know.
    </p>

    {{-- ───────────── No audio ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">The pill says &ldquo;no audio detected&rdquo;</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Either no speech was actually heard, or the audio input is wrong. Check Settings &gt; Audio &gt; Input device.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Two scenarios that catch people:
    </p>
    <ul class="mt-3 space-y-2 text-base leading-relaxed" style="color: var(--fg);">
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">Bluetooth headsets register twice in Windows.</strong> One entry is the high-quality stereo profile (A2DP), the other is the lower-quality but two-way headset profile (HFP/HSP). For dictation, you want the &ldquo;Hands-Free&rdquo; / &ldquo;Headset&rdquo; one. The stereo profile doesn&rsquo;t carry mic input.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">Windows mic privacy settings.</strong> Windows Settings &gt; Privacy &amp; Security &gt; Microphone. Make sure desktop apps are allowed.</span></li>
    </ul>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Settings &gt; Audio also shows a live input meter. If it&rsquo;s flat while you talk, it&rsquo;s an OS-level problem, nothing Utterbar can do until Windows is hearing your mic.
    </p>

    {{-- ───────────── Slow ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">Transcription is slow</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Two distinct things tend to be called &ldquo;slow&rdquo;.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        <strong style="color: var(--fg-hi);">First-dictation cold-load.</strong> The very first dictation after launching Utterbar pays a one-off cost while the engine loads the model. On CPU that takes a few seconds. On the DirectML GPU path the first Parakeet dictation takes about 20 to 25 seconds while the GPU compiles the model graph for your hardware; the first Whisper Medium or Whisper Large meeting in a session takes longer still, around 30 to 60 seconds before the transcribe itself starts. The pill shows &ldquo;Initialising model&rdquo; or &ldquo;Warming&hellip;&rdquo; during this window. Subsequent dictations and meetings reuse the cached recogniser and start straight away. Quitting the app loses the cache.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        <strong style="color: var(--fg-hi);">Every dictation feels slow.</strong> If transcription consistently takes more than ~1 second after you stop speaking, on a normal-length sentence:
    </p>
    <ul class="mt-3 space-y-2 text-base leading-relaxed" style="color: var(--fg);">
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">No GPU, low-core CPU.</strong> Anything under about 4 cores on CPU only will feel sluggish. If you have a DirectX 12 graphics card (NVIDIA, AMD, or Intel), set Settings &gt; Advanced &gt; GPU acceleration to Auto. Auto routes Whisper Medium, Whisper Large, and Parakeet to DirectML and keeps Whisper Small on CPU, because Small is actually faster on CPU than on the GPU. CUDA is not bundled.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">High background CPU load.</strong> Heavy compile, video render, or AV scan competing for cores. The model needs a bit of headroom.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">Battery-saver throttling.</strong> Windows aggressively down-clocks the CPU on battery saver. Plug in or change the power plan.</span></li>
    </ul>

    {{-- ───────────── 5-min cap ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">Single-shot dictation refuses past 5 minutes</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Regular dictation is capped at 5 minutes. Anything longer is refused with a clear message rather than risking a long-input crash. For longer recordings, switch to Meeting Mode (Settings &gt; Modes &gt; Meeting): it chunks audio internally and handles up to about an hour. Single-shot dictation is the hotkey-press-and-speak path; Meeting Mode is the press-record-on-a-call path.
    </p>

    {{-- ───────────── Wrong words ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">Words are coming out wrong</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Speech models are statistical, they pick the most likely word given the audio and the surrounding context. They struggle most with names, acronyms, product terms, and anything that doesn&rsquo;t appear in the data they were trained on.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        The fix is custom vocabulary. Add the word you keep getting wrong to Settings &gt; Vocabulary, the model biases toward it on subsequent transcriptions. For multi-word names, add the full version. For acronyms, add both the full form and the abbreviation. See the <a href="{{ route('docs.vocabulary') }}" style="color: var(--accent);">vocabulary doc</a> for the patterns that work best.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        For wholesale find-and-replace patterns. &ldquo;Always render &lsquo;ty&rsquo; as &lsquo;thank you&rsquo;&rdquo;, or &ldquo;always capitalise QA&rdquo;. Use the Replacements section. These run after transcription, before the typed output, and apply globally to every mode.
    </p>

    {{-- ───────────── Pill missing ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">The floating pill disappeared</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        The pill remembers where you last dragged it, if that location is no longer on a connected monitor (you unplugged a second screen, changed resolution, or docked/undocked a laptop), the pill is on disk in coordinates that don&rsquo;t exist any more.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Right-click the Utterbar tray icon and pick <strong style="color: var(--fg-hi);">&ldquo;Reset Utterbar position&rdquo;</strong>. The pill returns to the default top-centre of your primary monitor.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        If even the tray icon is missing, Utterbar isn&rsquo;t running. Launch from the Start menu or check that it&rsquo;s allowed in Windows Settings &gt; Apps &gt; Startup if you want it on every boot.
    </p>

    {{-- ───────────── Insertion fails ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">Transcription happens, but text doesn&rsquo;t appear</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        The pill shows &ldquo;Inserted&rdquo; but the words don&rsquo;t show up where you expected. Two causes:
    </p>
    <ul class="mt-3 space-y-2 text-base leading-relaxed" style="color: var(--fg);">
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">No focused text field.</strong> If the focused element doesn&rsquo;t accept text (a video player, an image, a non-editable element), the keystrokes go nowhere. Click into a real text field first.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">Locked-down apps that block paste.</strong> Some apps (a few enterprise IT tools, banking apps) block synthetic input via paste. Utterbar falls back to typing the text out character-by-character; you&rsquo;ll see it being typed live rather than appearing in one chunk.</span></li>
    </ul>

    {{-- ───────────── Crashes ───────────── --}}
    <h2 class="mt-14 ub-docs-h2">It crashed</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Crash dumps and the app log live under <code class="font-mono text-sm" style="color: var(--accent);">%APPDATA%\Utterbar\logs\</code>. The newest <code class="font-mono text-sm" style="color: var(--accent);">crash.log</code> usually has the cause printed at the top.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        If you can reproduce it, send the newest crash dump and the log to <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a> with a one-line description of what you were doing. We try to turn around any reproducible crash within the same week.
    </p>

    <div class="mt-14 ub-card ub-card-rail">
        <p class="text-sm leading-relaxed" style="color: var(--muted);">
            <strong style="color: var(--fg-hi);">Still stuck?</strong> Email <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a> with the output of Settings &gt; About &gt; Copy diagnostics. That dumps the relevant config + the last log lines into your clipboard; paste it into the email and we can usually answer same day.
        </p>
    </div>
</x-layouts.docs>
