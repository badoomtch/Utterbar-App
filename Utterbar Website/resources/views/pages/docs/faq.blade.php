@php
    $items = [
        ['q' => 'Does Utterbar send my voice to the cloud?', 'a' => 'No. Audio is captured, transcribed, and binned on your PC.'],
        ['q' => 'Which Windows versions are supported?', 'a' => 'Windows 10 22H2 and Windows 11, x86_64. WebView2 is required (already on Windows 11; auto-bootstrapped on Windows 10).'],
        ['q' => 'Do I need a GPU?', 'a' => 'No. The default Parakeet v3 build runs comfortably on any modern CPU. GPU acceleration (DirectML on any DX12 card, CUDA on NVIDIA) is supported and speeds dictation up noticeably if you have one, but it isn\'t required.'],
        ['q' => 'What languages are supported?', 'a' => 'Over 30 out of the box, including English, French, German, Spanish, Italian, Portuguese, Dutch, Polish, Turkish, Russian, Arabic, Hindi, Japanese, Korean, and Mandarin. English runs on Parakeet for speed; everything else runs on Whisper. Whisper base is free; Whisper small is Pro for stronger non-English accuracy. Auto-detect is on by default, so you don\'t have to pick.'],
        ['q' => 'Can I use it to talk to Claude / ChatGPT / Gemini / Copilot?', 'a' => 'Yes. Utterbar types into whichever app has the cursor, which includes browser tabs and AI desktop apps. Click the assistant\'s chat input, hold the hotkey, ask your question. The text lands in the box. No browser extension, no API key, no per-assistant setup. Works the same way with the local LLM you run yourself.'],
        ['q' => 'Does it work offline?', 'a' => 'Completely, once the model is downloaded on first run.'],
        ['q' => 'Can I use it in video-call and chat apps?', 'a' => 'Yes, for typing into the chat box. Utterbar never injects audio into a call.'],
        ['q' => 'What happens to my data when I uninstall?', 'a' => '<code>%APPDATA%\\Utterbar\\</code> is left in place so you can re-install without losing history. Delete the folder yourself for a clean wipe.'],
        ['q' => 'Will there be a macOS version?', 'a' => 'After Windows 1.0 ships. Mac already has decent local-only options; I\'d rather make Windows properly good first.'],
        ['q' => 'Will there be a Linux version?', 'a' => 'On the roadmap under "Later". No ETA.'],
        ['q' => 'Can I audit the code?', 'a' => 'Utterbar isn\'t open source (the paid features stay paid that way), but security researchers and enterprise buyers can request a source review under NDA. Email <a href="mailto:' . config('site.email.contact') . '" style="color: var(--accent);">' . config('site.email.contact') . '</a>.'],
        ['q' => 'How do I export my transcription history?', 'a' => 'Settings &rarr; History &rarr; Export. Produces a zip of every transcription (and audio, if you retained it).'],
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
