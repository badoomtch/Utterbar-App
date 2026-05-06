@php
    $items = [
        ['q' => 'Does Utterbar send my voice to the cloud?', 'a' => 'No. Audio is captured, transcribed, and binned on your PC.'],
        ['q' => 'Which Windows versions are supported?', 'a' => 'Windows 10 22H2 and Windows 11, x86_64. WebView2 is required (already on Windows 11; auto-bootstrapped on Windows 10).'],
        ['q' => 'Do I need a GPU?', 'a' => 'No. The default Parakeet build runs comfortably on any modern CPU. If you have a DirectX 12 graphics card (NVIDIA, AMD, or Intel), Utterbar uses it via DirectML for Whisper Medium, Whisper Large, and Parakeet on Auto. Whisper Small stays on CPU on Auto, because that is actually faster. Explicit DirectML in Settings > Advanced overrides the per-model rule. CUDA is not bundled.'],
        ['q' => 'What languages are supported?', 'a' => 'Over 30 out of the box, including English, French, German, Spanish, Italian, Portuguese, Dutch, Polish, Turkish, Russian, Arabic, Hindi, Japanese, Korean, and Mandarin. English runs on Parakeet for speed; everything else runs on Whisper. Whisper base is free; Whisper small is Pro for stronger non-English accuracy. Auto-detect is on by default, so you don\'t have to pick.'],
        ['q' => 'Can I use it to talk to Claude / ChatGPT / Gemini / Copilot?', 'a' => 'Yes. Utterbar types into whichever app has the cursor, which includes browser tabs and AI desktop apps. Click the assistant\'s chat input, hold the hotkey, ask your question. The text lands in the box. No browser extension, no API key, no per-assistant setup. Works the same way with the local LLM you run yourself.'],
        ['q' => 'Does it work offline?', 'a' => 'Completely, once the model is downloaded on first run.'],
        ['q' => 'Can I use it in video-call and chat apps?', 'a' => 'Yes, for typing into the chat box. Utterbar never injects audio into a call.'],
        ['q' => 'Can it mute Teams / Discord / Zoom while I&rsquo;m dictating?', 'a' => 'Output audio: yes, for all five (Teams, Slack, Discord, Zoom, Webex). Settings &gt; Privacy &gt; Media has a toggle called &ldquo;Mute conferencing app output while recording&rdquo; that silences what you HEAR from those apps for the duration of a dictation, then restores it the moment you let go. The app list under the toggle is editable. <br><br>Microphone (your voice into the call): partially, and experimental. Settings &gt; Advanced &gt; Experimental has a separate, off-by-default toggle for Teams and Discord only. When you turn it on for one of those apps and start a recording, Utterbar sends the mute hotkey you have configured in that app as a global keystroke (Teams uses Ctrl+Shift+M by default; Discord uses whatever you set under User Settings, Keybinds, Toggle Mute). A small banner appears below the floating bar asking you to glance at the call app&rsquo;s own mute indicator to confirm; Utterbar cannot read the real mute state, so the banner is the honest version of &ldquo;we tried, please confirm&rdquo;. Stopping the dictation sends the hotkey again to unmute. Zoom and Webex are not supported for mic mute. If your situation needs a guaranteed mute, use the call app&rsquo;s own mute button.'],
        ['q' => 'Does Utterbar clean up filler words like &ldquo;um&rdquo; and &ldquo;like&rdquo;?', 'a' => 'Optionally, yes. Settings &gt; Text Output &gt; Formatting has a <strong>Smart rewrite</strong> toggle (off by default). When on, Utterbar removes common fillers (&ldquo;um&rdquo;, &ldquo;uh&rdquo;, &ldquo;like&rdquo;, &ldquo;you know&rdquo;), normalises spacing, capitalises the first letter of each sentence, and adds a terminal full stop if missing. It&rsquo;s rule-based and runs locally; no AI model is loaded, nothing is sent anywhere. Best for chat and email-style dictation. Turn it off for code, the capitalise-sentence-starts rule will also capitalise keywords like <code>let</code> or <code>fn</code>. Meeting Mode stays verbatim regardless.'],
        ['q' => 'What happens to my data when I uninstall?', 'a' => '<code>%APPDATA%\\Utterbar\\</code> is left in place so you can re-install without losing history. Delete the folder yourself for a clean wipe.'],
        ['q' => 'Will there be a macOS version?', 'a' => 'After Windows 1.0 ships. Mac already has decent local-only options; I\'d rather make Windows properly good first.'],
        ['q' => 'Will there be a Linux version?', 'a' => 'On the roadmap under "Later". No ETA.'],
        ['q' => 'Can I audit the code?', 'a' => 'Utterbar isn\'t open source (the paid features stay paid that way), but security researchers and enterprise buyers can request a source review under NDA. Email <a href="mailto:' . config('site.email.contact') . '" style="color: var(--accent);">' . config('site.email.contact') . '</a>.'],
        ['q' => 'How do I export my transcription history?', 'a' => 'Settings &gt; History &gt; Export. Produces a zip of every transcription (and audio, if you retained it).'],
        ['q' => 'Why does dictation refuse past 5 minutes?', 'a' => 'Single-shot dictation is capped at 5 minutes. The offline recogniser allocates memory proportional to audio length, so the cap protects you from a long-input crash. For longer recordings, switch to Meeting Mode (Settings &gt; Modes &gt; Meeting): it chunks audio internally and handles up to about an hour. The cap will go away when the streaming-recogniser rebuild lands in a future release.'],
        ['q' => 'Can I use it commercially?', 'a' => 'Yes, once Pro is licensed. Free is for personal use.'],
    ];
@endphp

<x-layouts.docs title="FAQ" description="Frequently asked questions about Utterbar. Windows support, GPU requirements, offline behaviour, languages, exports, and licensing.">
    @php
        // FAQPage schema. Strip the inline HTML from answers so the schema
        // text is plain — Google ignores any `<a>` / `<code>` we leave in.
        $faqSchema = [
            '@context'   => 'https://schema.org',
            '@type'      => 'FAQPage',
            '@id'        => url('/docs/faq#faq'),
            'mainEntity' => collect($items)->map(fn ($f) => [
                '@type'          => 'Question',
                'name'           => trim(strip_tags(str_replace(['&rsquo;', '&mdash;'], ["'", '—'], $f['q']))),
                'acceptedAnswer' => [
                    '@type' => 'Answer',
                    'text'  => trim(strip_tags(str_replace(['&rsquo;', '&mdash;', '&rarr;'], ["'", '—', '→'], $f['a']))),
                ],
            ])->values()->all(),
        ];
    @endphp
    @push('head')
        <script type="application/ld+json">{!! json_encode($faqSchema, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) !!}</script>
    @endpush

    <h1 class="ub-docs-h1">Questions we <x-hl>always</x-hl> get.</h1>
    <p class="ub-lead mt-6">The exhaustive version of the home-page FAQ.</p>

    <div class="mt-8"><x-faq-accordion :items="$items" /></div>
</x-layouts.docs>
