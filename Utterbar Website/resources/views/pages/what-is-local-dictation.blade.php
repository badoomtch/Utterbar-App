<x-layouts.app
    title="What is local dictation?"
    description="A plain-English explainer of on-device speech recognition: what it means, why it's possible now, what's actually running on your machine, and how it compares to cloud dictation services.">

    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <p class="ub-eyebrow">Explainer</p>
                <h1 class="ub-fg-hi mt-4" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    What is <x-hl>local dictation</x-hl>?
                </h1>
                <p class="ub-lead mt-8 max-w-2xl">
                    A plain-English answer to &ldquo;what does it actually mean when an app says it&rsquo;s local-only?&rdquo; Plus why it suddenly works, and where the trade-offs are.
                </p>
            </div>
        </div>
    </section>

    {{-- Body --}}
    <section class="py-8 md:py-12">
        <div class="ub-container">
            <article class="max-w-3xl space-y-8 text-lg leading-relaxed" style="color: var(--fg);">

                <p>
                    Dictation has been around for decades, but until very recently, anything good required either a thousand-pound box of accessibility software (Dragon NaturallySpeaking) or a cloud service quietly streaming your microphone to a data centre in another country. Local dictation, running the speech-to-text engine entirely on your own machine, with no network involved, only became genuinely viable for normal laptops in the last two or three years. This page explains what changed, what&rsquo;s actually happening when you press the hotkey, and why the trade-offs aren&rsquo;t what most people assume.
                </p>

                <h2 class="text-2xl md:text-3xl mt-12 mb-2" style="color: var(--fg-hi); letter-spacing: -0.025em;">The two architectures</h2>
                <p>
                    There are two ways to turn the noise from your microphone into text. They look identical when you&rsquo;re using them, and the difference is invisible until something goes wrong.
                </p>
                <p>
                    <strong style="color: var(--fg-hi);">Cloud dictation</strong> records your voice on your device, packages the audio, and sends it over the internet to a server, somebody else&rsquo;s computer. That server runs the speech model, generates the text, and sends the result back. Most modern dictation tools work this way. It&rsquo;s why they need an account, why they have rate limits (servers cost money), and why they sometimes feel sluggish over a flaky Wi-Fi connection. It&rsquo;s also why their privacy policies have to say things like &ldquo;your voice data may be retained for service improvement&rdquo;, once it&rsquo;s on their server, it&rsquo;s in their territory.
                </p>
                <p>
                    <strong style="color: var(--fg-hi);">Local dictation</strong> never moves the audio off your computer. The speech model lives on your hard drive, the audio goes from microphone to model to text without ever crossing a network boundary, and the result types itself into whatever app has focus. The whole loop happens between your CPU (or GPU), your RAM, and your input device. The internet might as well be unplugged.
                </p>
                <p>
                    The functional difference, when it&rsquo;s working: nothing. The architectural difference: enormous.
                </p>

                <h2 class="text-2xl md:text-3xl mt-12 mb-2" style="color: var(--fg-hi); letter-spacing: -0.025em;">Why it works now</h2>
                <p>
                    Local dictation has been technically possible for years. The reason it&rsquo;s only just becoming viable for normal users is a stack of three things lining up:
                </p>
                <ul class="space-y-3 list-disc pl-6" style="color: var(--fg);">
                    <li><strong style="color: var(--fg-hi);">The models got smaller.</strong> Five years ago, accurate speech recognition needed a model the size of a small video game. Today, a state-of-the-art model fits in roughly half a gigabyte of disk space and runs faster than real time on a laptop CPU.</li>
                    <li><strong style="color: var(--fg-hi);">The models got better.</strong> Researchers (especially at NVIDIA, with the Parakeet family, and OpenAI, with Whisper) figured out how to train smaller models on much more data, in ways that match or exceed the cloud-only models from a few years back. NVIDIA Parakeet TDT 0.6B, the model Utterbar uses by default, reads English about as accurately as humans do.</li>
                    <li><strong style="color: var(--fg-hi);">Consumer hardware got fast enough.</strong> Modern Windows laptops run inference workloads through DirectML on any DirectX 12 graphics card, including integrated chips, fast enough that the model can finish transcribing your sentence before you&rsquo;ve let go of the hotkey. Even a machine without a GPU handles the smaller int8 model comfortably on CPU.</li>
                </ul>
                <p>
                    Each of those happened independently. The combination is what makes &ldquo;press a hotkey, speak, see text&rdquo; finally feel as snappy as cloud services, without the cloud.
                </p>

                <h2 class="text-2xl md:text-3xl mt-12 mb-2" style="color: var(--fg-hi); letter-spacing: -0.025em;">What&rsquo;s actually on your machine</h2>
                <p>
                    A local dictation app like Utterbar ships three things:
                </p>
                <ul class="space-y-3 list-disc pl-6" style="color: var(--fg);">
                    <li><strong style="color: var(--fg-hi);">A speech model file.</strong> A few hundred megabytes of weights, the numerical parameters the neural network uses to convert audio waveforms into letters. Utterbar downloads it from Hugging Face on first run, then never touches the network again for transcription.</li>
                    <li><strong style="color: var(--fg-hi);">An inference runtime.</strong> The code that loads the model into memory and runs it efficiently. Utterbar uses sherpa-onnx, an open-source C++ runtime, behind a thin Rust wrapper. The same runtime backs many of the other local-Whisper apps you might have heard of.</li>
                    <li><strong style="color: var(--fg-hi);">An audio capture path.</strong> The bit that listens to your microphone (or your system audio, in Utterbar&rsquo;s case), feeds it to the runtime, and types the result wherever your cursor is.</li>
                </ul>
                <p>
                    There&rsquo;s no part of that chain that calls home. The only network activity Utterbar ever does is the model download on first run, the once-a-day check for an app update, and the once-on-purchase licence verification. You can verify it yourself with any network sniffer.
                </p>

                <h2 class="text-2xl md:text-3xl mt-12 mb-2" style="color: var(--fg-hi); letter-spacing: -0.025em;">Why anyone would care</h2>
                <p>
                    Local dictation matters in three concrete ways most people only notice once they&rsquo;ve started using it.
                </p>
                <p>
                    <strong style="color: var(--fg-hi);">It works offline.</strong> On a train, on a plane, in a coffee shop with bad Wi-Fi, or just five minutes after your ISP has decided to renegotiate its routes. Cloud dictation tools degrade gracefully, they queue your speech and try again, but they don&rsquo;t work without a network. Local dictation does, every time, because the network was never part of the loop.
                </p>
                <p>
                    <strong style="color: var(--fg-hi);">It&rsquo;s genuinely private.</strong> Not in a marketing sense, in an architectural sense. There is no server logging your voice, because there is no server. There is no &ldquo;we may use your data to improve the service&rdquo; clause, because there&rsquo;s no place for them to use it. If your job involves dictating things you wouldn&rsquo;t paste into a free web tool, legal correspondence, medical notes, security work, anything compliance-bounded, this matters quite a lot.
                </p>
                <p>
                    <strong style="color: var(--fg-hi);">It doesn&rsquo;t need a subscription.</strong> Cloud dictation is fundamentally a service: the company pays for compute every time you use it, so you pay them every month. Local dictation is fundamentally software: the company writes it once, ships it once, and the running cost is borne by your laptop&rsquo;s power budget. That&rsquo;s why local apps tend to be one-time purchases, and cloud apps almost never are.
                </p>

                <h2 class="text-2xl md:text-3xl mt-12 mb-2" style="color: var(--fg-hi); letter-spacing: -0.025em;">The trade-offs</h2>
                <p>
                    A fair version of this article has to cover where local dictation is genuinely worse, not just where it&rsquo;s better.
                </p>
                <p>
                    <strong style="color: var(--fg-hi);">It uses your CPU.</strong> Or your GPU, if you have one. Speech inference is a real workload, not enormous, but not free. On a laptop running on battery, a few minutes of dictation will use a few percent of your charge. Cloud services don&rsquo;t do that.
                </p>
                <p>
                    <strong style="color: var(--fg-hi);">It needs a one-time download.</strong> The first run pulls a few hundred megabytes of speech model from the internet. After that, never again. But the first launch isn&rsquo;t instant.
                </p>
                <p>
                    <strong style="color: var(--fg-hi);">It can&rsquo;t lean on cloud LLMs for fancy rewriting.</strong> Some cloud dictation tools take your transcript and run it through a large language model to clean it up, restructuring sentences, fixing tone, formatting bullet points. A local app can do basic deterministic cleanup (capitalisation, punctuation, common typo fixes), but won&rsquo;t do the same kind of free-form rewrites unless you&rsquo;re willing to wait several seconds for a much larger local LLM to run. Utterbar&rsquo;s approach is to lean into the cleanup that&rsquo;s fast and reliable on-device, and skip the fancy stuff that needs a big model. Different products will draw that line in different places.
                </p>
                <p>
                    <strong style="color: var(--fg-hi);">Sync is your problem.</strong> If you have a custom vocabulary on your work laptop and you want it on your home laptop, that&rsquo;s a manual export. Cloud services do this for free.
                </p>

                <h2 class="text-2xl md:text-3xl mt-12 mb-2" style="color: var(--fg-hi); letter-spacing: -0.025em;">How to tell if an app actually runs locally</h2>
                <p>
                    &ldquo;Local&rdquo; has become a marketing word, which means some apps that claim it aren&rsquo;t. The honest test, in two minutes:
                </p>
                <ol class="space-y-3 list-decimal pl-6" style="color: var(--fg);">
                    <li>Open the app and finish its first-run setup, including any model downloads it does.</li>
                    <li>Disconnect from the internet. Pull the ethernet cable; turn off Wi-Fi.</li>
                    <li>Try to dictate something.</li>
                </ol>
                <p>
                    If it works, the app runs locally. If it errors out or queues your speech for later, it&rsquo;s a cloud service. There&rsquo;s no marketing copy that beats this test.
                </p>

                <h2 class="text-2xl md:text-3xl mt-12 mb-2" style="color: var(--fg-hi); letter-spacing: -0.025em;">Where Utterbar fits</h2>
                <p>
                    Utterbar is one of a small number of fully-local dictation apps for Windows. It uses NVIDIA Parakeet by default, runs on CPU (or GPU if you have one), and the only network calls it ever makes are the initial model download and a once-a-day update check. The audio is processed in memory and immediately discarded; the transcript is yours to do whatever you want with.
                </p>
                <p>
                    If that sounds useful, the <a href="{{ route('download') }}" style="color: var(--accent);">free version</a> is a one-minute install. If you want the longer comparison against the cloud-based alternatives, the <a href="{{ route('compare.wispr-flow') }}" style="color: var(--accent);">Utterbar vs Wispr Flow</a> page is the next read.
                </p>

            </article>
        </div>
    </section>

    <x-prompt-cta
        :heading="'Try the local version.'"
        :sub="'Free forever, no account, downloads in under a minute. Disconnect from the internet right after install, it&rsquo;ll keep working.'"
    />
</x-layouts.app>
