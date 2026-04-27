@php
    // Features, each with an optional `video` key pulling from config/marketing.php.
    // The video renders alongside the body copy when present. When absent, the
    // card stays text-only (same layout as before).
    $features = [
        [
            'title' => 'Press. Speak. Done.',
            'body'  => 'One hotkey. Hold it, talk, let go. Your words land in whatever app is focused. That is the whole interface.',
            'wins'  => ['No dialogs', 'Works over any open window', 'No copy-paste dance'],
            'video' => 'feature-press-speak-done',
        ],
        [
            'title' => 'Talk to AI at the speed of speech.',
            'body'  => 'Claude, ChatGPT, Gemini, Copilot, the local model you run yourself &mdash; whatever assistant lives in your browser, Utterbar dictates straight into it. People speak about three times faster than they type, and the assistant doesn\'t care which way the question arrived. Faster prompts, longer context, fewer wrist breaks.',
            'wins'  => ['Works with every assistant in any browser', 'No browser extension, no API key, no integration', 'You speak, the prompt appears, the answer streams back'],
            'video' => 'feature-talk-to-ai',
        ],
        [
            'title' => 'Local means local.',
            'body'  => 'Your voice is transcribed on your own machine. It does not leave. Not for &ldquo;processing&rdquo;, not for &ldquo;analytics&rdquo;, not for anything. Nothing can leak because nothing goes anywhere.',
            'wins'  => ['No cloud', 'No account', 'No telemetry'],
            'video' => 'feature-local-privacy',
        ],
        [
            'title' => 'Works in every Windows app.',
            'body'  => 'Your browser, your editor, your chat app, a legacy tool from 2003 &mdash; if a keyboard works there, Utterbar works there. For locked-down apps that block paste, it types the text out instead. You rarely notice.',
            'wins'  => ['Universal compatibility', 'Automatic fallback', 'Zero integrations required'],
            'video' => 'feature-works-anywhere',
        ],
        [
            'title' => 'Modes that match the job.',
            'body'  => 'Dictate a casual message, get a casual message. Into meeting notes, you get verbatim capture with line breaks. Switch mode in a click or by hotkey &mdash; one per job, one keystroke.',
            'wins'  => ['Three built-in modes', 'Switch in a click or by hotkey', 'Unlimited custom modes on Pro'],
            'video' => 'feature-modes',
        ],
        [
            'title' => 'Speaks 30+ languages.',
            'body'  => 'English uses Parakeet, the fastest engine on the planet for English. Anything else uses Whisper &mdash; French, German, Spanish, Italian, Portuguese, Dutch, Polish, Russian, Arabic, Hindi, Japanese, Korean, Mandarin, and 17 more. All of it on your PC, none of it on someone else\'s server.',
            'wins'  => ['30+ languages out of the box', 'Auto-detect or pin a default', 'Same offline guarantee in every language'],
            'video' => 'feature-multilingual',
        ],
        [
            'title' => 'Catches what your PC is playing, too.',
            'body'  => 'Transcribe the other side of a video call, a podcast, a video playing in your browser &mdash; without installing a single driver or virtual audio cable. Microphone, system audio, or both at once.',
            'wins'  => ['Your microphone', 'Your PC&rsquo;s own audio', 'Both summed together'],
            'video' => 'feature-system-audio',
        ],
        [
            'title' => 'Learns your names.',
            'body'  => 'Add the words it keeps getting wrong. Names, acronyms, product terms. They stop being &ldquo;J parrie&rdquo; and start being whatever you actually said.',
            'wins'  => ['Custom vocabulary', 'Literal + pattern replacements', 'Applied globally'],
            'video' => 'feature-vocabulary',
        ],
        [
            'title' => 'Transcribes files, too.',
            'body'  => 'Drop in recordings &mdash; a meeting, a podcast interview, an old lecture &mdash; and get clean text, subtitles, or captions out the other side. Same local pipeline, same privacy guarantees.',
            'wins'  => ['Common audio and video formats', 'Plain-text transcript next to your file', 'Pro only'],
            'video' => 'feature-file-transcription',
        ],
        [
            'title' => 'Reprocess any past dictation.',
            'body'  => 'Every transcript stays in your history. Open one, pick a different mode, and Utterbar re-runs it &mdash; same recording, a different voice. Turn a rough capture into meeting notes a week later. Rewrite a message in a calmer tone without saying it again.',
            'wins'  => ['Full history', 'Any mode, any recording', 'Audio retention on Pro'],
            'video' => 'feature-history-retranscribe',
        ],
        [
            'title' => 'Make it look like your desk.',
            'body'  => 'Three theme presets &mdash; Obsidian, Graphite, Paper &mdash; with six accent colours each. Compact or standard bar size. Show it always, or only while recording. Your setup, not mine.',
            'wins'  => ['Three themes', 'Six accent colours', 'Compact or standard'],
            'video' => 'feature-customise-bar',
        ],
    ];
@endphp

<x-layouts.app title="Features" description="Eleven things that make Utterbar different: press-to-talk hotkeys, AI-assistant dictation, local-only transcription, 30+ languages, auto-paste into any Windows app, modes, vocabulary, system audio, file transcription, and more.">

    {{-- Hero --}}
    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <h1 class="ub-fg-hi" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    What <x-wordmark /> actually does.
                </h1>
                <p class="ub-lead mt-8 max-w-xl">
                    Eleven things worth paying attention to. Everything else gets out of the way.
                </p>
            </div>
        </div>
    </section>

    @foreach ($features as $i => $f)
        <section class="py-10 md:py-14">
            <div class="ub-container">
                <div class="grid grid-cols-1 md:grid-cols-12 gap-8 items-start">
                    <div class="md:col-span-4 md:sticky md:top-24">
                        <span class="block font-mono leading-none select-none"
                              style="font-size: clamp(3.5rem, 7.5vw, 5.75rem);
                                     font-weight: 800;
                                     letter-spacing: -0.05em;
                                     background: linear-gradient(180deg, color-mix(in oklab, var(--accent) 55%, transparent) 0%, color-mix(in oklab, var(--muted) 20%, transparent) 100%);
                                     -webkit-background-clip: text;
                                     background-clip: text;
                                     -webkit-text-fill-color: transparent;
                                     color: transparent;">
                            {{ str_pad((string)($i + 1), 2, '0', STR_PAD_LEFT) }}
                        </span>
                        <h2 class="mt-4 text-2xl md:text-3xl" style="letter-spacing: -0.03em;">{!! $f['title'] !!}</h2>
                    </div>
                    <div class="md:col-span-8">
                        <p class="text-lg md:text-xl leading-relaxed" style="color: var(--fg); font-weight: 300;">{!! $f['body'] !!}</p>
                        <ul class="mt-6 flex flex-wrap gap-x-6 gap-y-2">
                            @foreach ($f['wins'] as $w)
                                <li class="flex items-center gap-2 text-sm" style="color: var(--muted);">
                                    <span class="shrink-0" style="color: var(--accent);"><x-icon name="check" size="14" stroke="2.2" /></span>
                                    <span>{!! $w !!}</span>
                                </li>
                            @endforeach
                        </ul>

                        @if (!empty($f['video']))
                            <div class="mt-8">
                                <x-video-embed :id="$f['video']" />
                            </div>
                        @endif
                    </div>
                </div>
            </div>
        </section>
        @if (!$loop->last)
            <div class="ub-container"><div class="ub-divider"></div></div>
        @endif
    @endforeach

    <x-prompt-cta
        :heading="'Those are the headlines.'"
        :sub="'The rest is the kind of detail you only notice when it&rsquo;s right.'"
    />
</x-layouts.app>
