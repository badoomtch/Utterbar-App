<x-layouts.docs title="Getting started" description="Install Utterbar on Windows, download the speech model on first run, and dictate into any app within a minute.">
    <h1 class="ub-docs-h1">Up and <x-hl>running</x-hl> in a minute.</h1>
    <p class="ub-lead mt-6">Install Utterbar, let it grab the speech model once, then start dictating into any Windows app. Three steps, no account, no setup wizard wandering.</p>

    {{-- ───────────── Step 01 ───────────── --}}
    <div class="mt-14">
        <span class="ub-eyebrow">Step 01</span>
        <h2 class="mt-3 ub-docs-h2">Install Utterbar</h2>
        <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
            Download the installer from the <a href="{{ route('download') }}" style="color: var(--accent);">download page</a>. It&rsquo;s a single <code class="font-mono px-1.5 py-0.5 rounded text-sm" style="background: var(--bg-2); color: var(--accent);">.exe</code>, around 20&nbsp;MB. Run it. Utterbar installs to your local app data folder, no admin rights required, no system services, no registry pollution beyond the start-menu shortcut.
        </p>
        <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
            Windows 10 22H2 or Windows 11, x86_64. WebView2 is required, it&rsquo;s already on every Windows 11 machine and auto-bootstrapped on Windows 10. If you see a SmartScreen warning the first time, that&rsquo;s because the installer isn&rsquo;t code-signed. Click &ldquo;More info&rdquo; and then &ldquo;Run anyway&rdquo;. SmartScreen learns over time; once enough installs go through, the warning goes away on its own.
        </p>

        <div class="mt-6 ub-card" style="padding: 1rem 1.25rem;">
            <p class="text-sm" style="color: var(--muted); line-height: 1.55;">
                <strong style="color: var(--fg-hi);">Where Utterbar lives.</strong> All app data lives under <code class="font-mono text-xs" style="color: var(--accent);">%APPDATA%\Utterbar\</code>. Logs, transcription history, the speech model, your custom vocabulary, all there. Uninstall doesn&rsquo;t touch this folder by default, so you can re-install without losing anything. Delete it manually for a clean wipe.
            </p>
        </div>
    </div>

    {{-- ───────────── Step 02 ───────────── --}}
    <div class="mt-14">
        <span class="ub-eyebrow">Step 02</span>
        <h2 class="mt-3 ub-docs-h2">First-run model download</h2>
        <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
            The very first time Utterbar launches, it fetches a speech-recognition model from the <a href="https://github.com/k2-fsa/sherpa-onnx/releases" target="_blank" rel="noopener" style="color: var(--accent);">k2-fsa sherpa-onnx releases</a>. The default is the int8 build of NVIDIA Parakeet TDT 0.6B v3, about 460&nbsp;MB. The download takes a minute or two on a normal connection.
        </p>
        <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
            This is the only time Utterbar will ever talk to the network for transcription. After this, you can disconnect from the internet entirely and dictation keeps working, the model lives on your disk, the inference runs on your CPU (or GPU, if you have one), and the audio never leaves your machine.
        </p>
        <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
            Other models live in Settings &gt; Models. Whisper Base (free) and Whisper Small / Medium / Large v3 (Pro) cover 30+ languages between them. If you have a DirectX 12 graphics card, Utterbar uses it via DirectML for Whisper Medium, Whisper Large, and Parakeet. CUDA is not bundled.
        </p>

        <div class="mt-6 ub-card" style="padding: 1rem 1.25rem;">
            <p class="text-sm" style="color: var(--muted); line-height: 1.55;">
                <strong style="color: var(--fg-hi);">Cold-start.</strong> The first dictation after launch loads the model. On CPU that takes a few seconds. On the DirectML GPU path the first Parakeet dictation takes about 20 to 25 seconds while the GPU compiles the model graph for your hardware; the first Whisper Medium or Whisper Large meeting in a session takes longer still, around 30 to 60 seconds before the transcribe itself starts. The pill shows &ldquo;Initialising model&rdquo; or &ldquo;Warming&hellip;&rdquo; during this window. Subsequent dictations and meetings reuse the cached recogniser and start straight away. Quitting the app loses the cache.
            </p>
        </div>
    </div>

    {{-- ───────────── Step 03 ───────────── --}}
    <div class="mt-14">
        <span class="ub-eyebrow">Step 03</span>
        <h2 class="mt-3 ub-docs-h2">Your first dictation</h2>
        <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
            Focus any app that accepts typing, an email draft, a Notion page, a Slack message box, the Windows search bar, anything. Press <x-kbd-chip :keys="['Ctrl','Space']" /> to start recording.
        </p>
        <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
            The floating bar at the top of your screen lights up cyan and the seven-bar waveform starts moving. Speak naturally, you don&rsquo;t need to slow down or pronounce things robotically. When you&rsquo;re done, press <x-kbd-chip :keys="['Ctrl','Space']" /> again to stop. Utterbar transcribes, runs your selected mode&rsquo;s post-processing, and types the result into wherever your cursor is.
        </p>
        <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
            That&rsquo;s the whole interface. There&rsquo;s no toolbar, no &ldquo;send&rdquo; button, no cloud sync prompt. Press, speak, release.
        </p>
    </div>

    {{-- ───────────── What to try next ───────────── --}}
    <h2 class="mt-16 ub-docs-h2">What to try next</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Once your first dictation works, the things most worth doing in roughly this order:
    </p>
    <ul class="mt-4 space-y-3 text-base leading-relaxed" style="color: var(--fg);">
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">Try a different mode.</strong> Press <x-kbd-chip :keys="['Ctrl','Alt','M']" /> to cycle through Voice-to-text, Message, and Meeting. Same speech, three different output shapes. The <a href="{{ route('docs.modes') }}" style="color: var(--accent);">modes doc</a> covers what each one does.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">Add your custom vocabulary.</strong> If your name, your company name, or a domain term keeps coming out wrong, drop it into Settings &gt; Vocabulary. The <a href="{{ route('docs.vocabulary') }}" style="color: var(--accent);">vocabulary doc</a> shows how.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">Pick the audio source you actually want.</strong> Mic by default, but you can also capture system audio (for transcribing meetings) or both at once. Settings &gt; Audio.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--accent);"></span><span><strong style="color: var(--fg-hi);">Rebind the hotkey if Ctrl+Space conflicts.</strong> Some Chinese IMEs and IDE keybindings claim it. <a href="{{ route('docs.shortcuts') }}" style="color: var(--accent);">Shortcuts</a> covers every binding.</span></li>
    </ul>

    {{-- ───────────── If it didn't work ───────────── --}}
    <h2 class="mt-16 ub-docs-h2">If something didn&rsquo;t work</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        The fastest things to check, in order:
    </p>
    <ul class="mt-4 space-y-3 text-base leading-relaxed" style="color: var(--fg);">
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">No floating bar visible?</strong> Check the system tray. Utterbar is running but its pill might be off-screen on a monitor that&rsquo;s no longer connected. Right-click the tray icon, then &ldquo;Reset Utterbar position&rdquo;.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">Hotkey does nothing?</strong> Another app is probably holding it globally. Settings &gt; Shortcuts to rebind, or close whichever app is conflicting.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">&ldquo;No audio detected&rdquo; even when you spoke?</strong> Your mic input may be wrong. Settings &gt; Audio Input. Bluetooth headsets often register twice; pick the one with &ldquo;Hands-Free&rdquo; in the name.</span></li>
        <li class="flex gap-3"><span class="shrink-0 mt-1.5 w-1.5 h-1.5 rounded-full" style="background: var(--muted);"></span><span><strong style="color: var(--fg-hi);">Slow first transcription?</strong> Expected. The model warms up over the first few seconds; subsequent dictations are instant.</span></li>
    </ul>
    <p class="mt-6 text-base leading-relaxed" style="color: var(--fg);">
        For anything else, the <a href="{{ route('docs.troubleshooting') }}" style="color: var(--accent);">troubleshooting page</a> covers the common edge cases. If you&rsquo;re still stuck, email <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a> with the output of Settings &gt; About &gt; Copy diagnostics.
    </p>
</x-layouts.docs>
