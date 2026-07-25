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
            'body'  => 'Claude, ChatGPT, Gemini, Copilot, the local model you run yourself. Whichever assistant you already use, Utterbar dictates straight into the chat box. People speak about three times faster than they type. Longer prompts, fuller context, fewer sore wrists.',
            'wins'  => ['Works with every assistant in any browser', 'No browser extension, no API key, no integration', 'You speak, the prompt appears, the reply streams back'],
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
            'body'  => 'Your browser, your editor, your chat app, a legacy tool from 2003. If a keyboard works there, Utterbar works there. For locked-down apps that block paste, it types the text out character by character instead. You rarely notice the difference.',
            'wins'  => ['Any app that takes a keyboard', 'Automatic paste-blocked fallback', 'Nothing to install per app'],
            'video' => 'feature-works-anywhere',
        ],
        [
            'title' => 'Modes that match the job.',
            'body'  => 'Dictate a casual message, get a casual message. Dictate meeting notes, get verbatim capture with line breaks. Switch mode in a click or by hotkey. One per job, one keystroke.',
            'wins'  => ['Three built-in modes', 'Switch in a click or by hotkey', 'Unlimited custom modes'],
            'video' => 'feature-modes',
        ],
        [
            'title' => 'Speaks 30+ languages.',
            'body'  => 'English uses NVIDIA Parakeet, the fastest English ASR model going. Anything else uses Whisper: French, German, Spanish, Italian, Portuguese, Dutch, Polish, Russian, Arabic, Hindi, Japanese, Korean, Mandarin, and 17 more. All of it on your PC. None of it on anyone else&rsquo;s server.',
            'wins'  => ['30+ languages out of the box', 'Auto-detect or pin a default', 'Same offline guarantee in every language'],
            'video' => 'feature-multilingual',
        ],
        [
            'title' => 'Catches what your PC is playing, too.',
            'body'  => 'Transcribe the other side of a video call, a podcast, a video playing in your browser. No driver to install, no virtual audio cable to set up. Microphone, system audio, or both at once.',
            'wins'  => ['Your microphone', 'Your PC&rsquo;s own audio', 'Both summed together'],
            'video' => 'feature-system-audio',
        ],
        [
            'title' => 'Learns your names.',
            'body'  => 'Add the words it keeps getting wrong. Names, acronyms, product terms. They stop being &ldquo;J parrie&rdquo; and start being whatever you actually said.',
            'wins'  => ['Custom vocabulary', 'Literal and pattern replacements', 'Applied globally'],
            'video' => 'feature-vocabulary',
        ],
        [
            'title' => 'Transcribes files, too.',
            'body'  => 'Drop in a recording: a meeting, a podcast, an old lecture. Get a plain-text transcript next to it, with subtitle files alongside. Same engine as the floating bar, longer audio.',
            'wins'  => ['Common audio and video formats', 'Plain-text plus .srt and .vtt', 'Batch a whole folder'],
            'video' => 'feature-file-transcription',
        ],
        [
            'title' => 'Reprocess any past dictation.',
            'body'  => 'Every transcript stays in your history. Open one, pick a different mode, run it again. Turn a rough capture into meeting notes a week later. Rewrite a message in a calmer tone without saying it again.',
            'wins'  => ['Full history', 'Any mode, any recording', 'Optional audio retention'],
            'video' => 'feature-history-retranscribe',
        ],
        [
            'title' => 'Looks like your desk.',
            'body'  => 'Three themes: Obsidian, Graphite, Paper. Six accent colours each. Compact or standard bar size. Show it always or only while recording. Your setup, not mine.',
            'wins'  => ['Three themes', 'Six accent colours', 'Compact or standard'],
            'video' => 'feature-customise-bar',
        ],
        [
            'title' => 'Tidies up filler words, optionally.',
            'body'  => 'Turn on Smart rewrite in Settings &gt; Text Output &gt; Formatting and Utterbar strips common fillers (&ldquo;um&rdquo;, &ldquo;uh&rdquo;, &ldquo;like&rdquo;, &ldquo;you know&rdquo;), tightens spacing, capitalises the first letter of each sentence, and adds a final full stop if you forgot one. Off by default. Fully local, rule-based, no model loaded, nothing sent anywhere. Leave it off for code dictation; the same rule that capitalises sentence starts will also capitalise keywords like <code class="font-mono" style="color: var(--accent);">let</code> or <code class="font-mono" style="color: var(--accent);">fn</code>. Meeting Mode stays verbatim regardless of the toggle.',
            'wins'  => ['Off by default', 'Deterministic, no LLM', 'Meeting Mode unaffected'],
        ],
        [
            'title' => 'Mutes call-app audio while you dictate.',
            'body'  => 'When you are on a call and need to dictate something into another window, Utterbar can silence what you HEAR from Teams, Slack, Discord, Zoom, and Webex for the duration of the recording, then restore it the moment you let go of the hotkey. Call audio and notification dings stop bleeding into your dictation. Toggle and editable app list live in Settings &gt; Privacy &gt; Media. This is output-only and covers all five apps.',
            'wins'  => ['Output audio for all five apps', 'Editable app list', 'Auto-restored after dictation'],
        ],
        [
            'title' => 'Optional experimental mic mute for Teams and Discord.',
            'body'  => 'Different setting, different boundary. Settings &gt; Advanced &gt; Experimental has two off-by-default toggles, one for Teams and one for Discord. Enable a toggle, supply the mute hotkey you have configured in that app (Teams uses Ctrl+Shift+M by default; Discord uses whatever you set under User Settings, Keybinds, Toggle Mute), and Utterbar sends the hotkey as a global keystroke when you start and stop recording. A banner asks you to glance at the call app&rsquo;s own mute indicator to confirm: Utterbar cannot read the real mute state, so the banner is the honest version of &ldquo;we tried, please confirm&rdquo;. Teams and Discord only. Zoom and Webex are not supported. If your situation needs a guaranteed mute, use the call app&rsquo;s own mute button.',
            'wins'  => ['Off by default', 'Teams + Discord only', 'Banner asks you to confirm'],
        ],
    ];
@endphp

<x-layouts.app title="Features" description="Fourteen things that make Utterbar different: press-to-talk hotkeys, AI-assistant dictation, local-only transcription, 30+ languages, auto-paste into any Windows app, modes, vocabulary, system audio, file transcription, optional Smart rewrite, conferencing output mute, and an experimental Teams + Discord mic mute.">

    {{-- Hero --}}
    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <h1 class="ub-fg-hi" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    What <x-wordmark /> actually does.
                </h1>
                <p class="ub-lead mt-8 max-w-xl">
                    Fourteen things worth paying attention to. Everything else gets out of the way.
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
