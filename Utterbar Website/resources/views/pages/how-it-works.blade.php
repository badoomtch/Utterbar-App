@php
    $principles = [
        [
            'label' => 'Local',
            'title' => 'Your voice never leaves.',
            'body'  => 'The speech model runs on your CPU or GPU. Audio is transcribed and forgotten the same second. Nothing gets uploaded, ever.',
        ],
        [
            'label' => 'Fast',
            'title' => 'Sub-second from stop to text.',
            'body'  => 'No round-trip to a server, no queue, no rate limit. You stop talking and the words are already on the screen.',
        ],
        [
            'label' => 'Universal',
            'title' => 'Works everywhere a keyboard works.',
            'body'  => 'If your cursor is blinking somewhere, Utterbar can put text there. Includes the awkward ones that block paste.',
        ],
        [
            'label' => 'Private',
            'title' => 'No account. No telemetry. No phone-home.',
            'body'  => 'The only network activity is the one-off speech-model download on first install. After that, pull the ethernet cable and nothing changes.',
        ],
    ];
@endphp

<x-layouts.app title="How it works" description="How Utterbar works: press a hotkey, speak, your words land where the cursor is. Three steps, fully local to your Windows PC, no cloud, no account.">
    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <h1 class="ub-fg-hi" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    How it <x-hl>actually</x-hl> works.
                </h1>
                <p class="ub-lead mt-8 max-w-xl">
                    One hotkey. One model. One machine. That&rsquo;s the entire pitch.
                </p>
            </div>
            <div class="mt-20">
                <x-how-it-works-stepper />
            </div>
        </div>
    </section>

    {{-- Full walkthrough --}}
    <section class="py-12 md:py-16">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto">
                <x-video-embed id="how-it-works-walkthrough" />
            </div>
        </div>
    </section>

    {{-- Four principles as a tight grid, no long prose --}}
    <section class="py-16 md:py-24">
        <div class="ub-container">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                @foreach ($principles as $p)
                    <article class="ub-card ub-card-rail">
                        <p class="ub-eyebrow mb-4">{{ $p['label'] }}</p>
                        <h3 class="text-xl md:text-2xl" style="letter-spacing: -0.025em; line-height: 1.2;">{{ $p['title'] }}</h3>
                        <p class="mt-4 text-base leading-relaxed" style="color: var(--muted);">{{ $p['body'] }}</p>
                    </article>
                @endforeach
            </div>
        </div>
    </section>

    <x-prompt-cta
        :heading="'That&rsquo;s all of it.'"
        :sub="'If that&rsquo;s the shape of app you&rsquo;ve been waiting for, drop your email.'"
    />
</x-layouts.app>
