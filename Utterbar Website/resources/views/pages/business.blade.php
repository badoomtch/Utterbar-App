@php
    $businessDesc = 'Utterbar for teams. Local-only dictation that stays on your PCs. Volume pricing from £29 per seat for 5 to 24 seats, £19 per seat for 25 to 100. One-time licences, no subscription, IT-deployable.';
@endphp

<x-layouts.app title="For business" :description="$businessDesc">

    {{-- Hero --}}
    <section class="relative overflow-hidden">
        <div class="ub-hero-glow" aria-hidden="true"></div>
        <div class="relative ub-container pt-20 md:pt-28 pb-12">
            <div class="max-w-4xl">
                <p class="ub-eyebrow">For business</p>
                <h1 class="ub-fg-hi mt-4" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Dictation that stays on <x-hl>your</x-hl> PCs.
                </h1>
                <p class="ub-lead mt-8 max-w-2xl">
                    Utterbar transcribes on the device. There is no cloud audio service to leak from, no per-user account to provision, no subscription that renews behind your back. One-time licences, deployed by your team to your fleet.
                </p>
                <div class="mt-10 flex flex-wrap gap-3">
                    <a href="{{ route('business.contact') }}" class="ub-btn ub-btn-cta">
                        Get a quote
                        <x-icon name="arrow-right" size="16" stroke="2" />
                    </a>
                    <a href="#pricing" class="ub-btn ub-btn-ghost">See volume pricing</a>
                    <a href="{{ route('docs.it-deployment') }}" class="ub-btn ub-btn-ghost">IT deployment notes</a>
                </div>
            </div>
        </div>
    </section>

    {{-- Why Utterbar for a team. Three concrete reasons, no enterprise
         theatre. Privacy is the wedge; one-time licensing is the
         commercial wedge; deployment friendliness is the practical wedge. --}}
    <section class="py-20 md:py-24">
        <div class="ub-container">
            <div class="max-w-3xl mb-14">
                <h2 style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    What changes at <x-hl>team scale</x-hl>.
                </h2>
                <p class="ub-lead mt-6 max-w-xl">
                    The product is the same product. The buying conversation is different.
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-5">
                <div class="ub-card">
                    <p class="ub-eyebrow mb-3">01</p>
                    <h3 class="text-2xl mb-3" style="letter-spacing: -0.02em;">Audio never leaves the machine</h3>
                    <p style="color: var(--muted); line-height: 1.55;">
                        Every transcript is generated on the user&rsquo;s PC. No third-party speech API, no shared tenant, no cloud bucket of recordings to subpoena. For legal, healthcare, finance, government, and any compliance-bounded environment, &ldquo;the audio doesn&rsquo;t go anywhere&rdquo; isn&rsquo;t a feature toggle. It&rsquo;s the architecture.
                    </p>
                </div>
                <div class="ub-card">
                    <p class="ub-eyebrow mb-3">02</p>
                    <h3 class="text-2xl mb-3" style="letter-spacing: -0.02em;">Pay once per seat, not every month</h3>
                    <p style="color: var(--muted); line-height: 1.55;">
                        Lifetime licences. One purchase order. No annual renewal cliff, no per-user inflation when your team grows, no end-of-life surprise when the vendor pivots. The maths is easier and procurement gets a single line item.
                    </p>
                </div>
                <div class="ub-card">
                    <p class="ub-eyebrow mb-3">03</p>
                    <h3 class="text-2xl mb-3" style="letter-spacing: -0.02em;">Built for IT to deploy</h3>
                    <p style="color: var(--muted); line-height: 1.55;">
                        Per-user MSI and NSIS installers, silent install via SCCM or Intune, settings stored in <code class="font-mono text-sm" style="color: var(--accent);">%APPDATA%\Utterbar\</code>, no admin rights to install. The full network-egress list is short and documented. Your IT team gets honest answers, not a pre-sales call.
                    </p>
                </div>
            </div>
        </div>
    </section>

    {{-- Who's actually buying. Concrete personas, not generic
         "knowledge workers". --}}
    <section class="py-20 md:py-24" style="background: var(--bg-1);">
        <div class="ub-container">
            <div class="max-w-3xl mb-12">
                <p class="ub-eyebrow">Built for</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Teams that already pay <x-hl>someone</x-hl> for this.
                </h2>
                <p class="ub-lead mt-6 max-w-xl">
                    If your team has a dictation budget, you have one of these problems.
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-5 max-w-4xl">
                <div class="ub-card">
                    <h3 class="text-xl mb-2" style="color: var(--fg-hi); letter-spacing: -0.015em;">Call centres and customer support</h3>
                    <p style="color: var(--muted); line-height: 1.55; font-size: 0.95rem;">
                        Agents handle hundreds of cases a day. Voice is faster than typing for case notes, ticket replies, and CRM updates. Cloud dictation tools mean every customer conversation passes through a third party. Utterbar doesn&rsquo;t.
                    </p>
                </div>
                <div class="ub-card">
                    <h3 class="text-xl mb-2" style="color: var(--fg-hi); letter-spacing: -0.015em;">Legal and paralegal teams</h3>
                    <p style="color: var(--muted); line-height: 1.55; font-size: 0.95rem;">
                        Privileged communication, witness statements, contract drafting. The compliance bar is &ldquo;this audio cannot leave the firm&rdquo;. Local-only is the only honest answer to that.
                    </p>
                </div>
                <div class="ub-card">
                    <h3 class="text-xl mb-2" style="color: var(--fg-hi); letter-spacing: -0.015em;">Healthcare admin</h3>
                    <p style="color: var(--muted); line-height: 1.55; font-size: 0.95rem;">
                        Patient notes, referral letters, intake forms. Cloud dictation is technically GDPR-compatible if you trust the vendor&rsquo;s data-processing agreement. Local-only sidesteps the question.
                    </p>
                </div>
                <div class="ub-card">
                    <h3 class="text-xl mb-2" style="color: var(--fg-hi); letter-spacing: -0.015em;">Internal IT and security teams</h3>
                    <p style="color: var(--muted); line-height: 1.55; font-size: 0.95rem;">
                        You&rsquo;re writing tickets, runbooks, and incident notes. Some of those are about your own infrastructure. Sending that audio to a third-party transcription service is a bad shape, even when it&rsquo;s technically allowed.
                    </p>
                </div>
            </div>
        </div>
    </section>

    {{-- The IT-buyer FAQ. Answers the questions an IT desk asks BEFORE
         scheduling a call, so the call is shorter and starts further
         along. Honest about what we have and don't have. --}}
    <section class="py-20 md:py-24">
        <div class="ub-container">
            <div class="max-w-3xl mb-12">
                <p class="ub-eyebrow">For IT</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Honest answers to the obvious questions.
                </h2>
                <p class="ub-lead mt-6 max-w-xl">
                    The full deployment notes are at <a href="{{ route('docs.it-deployment') }}" style="color: var(--accent);">docs / IT deployment</a>. The ten-second version is below.
                </p>
            </div>

            <div class="max-w-3xl space-y-6">
                @foreach ([
                    [
                        'q' => 'Where does the audio go?',
                        'a' => 'Nowhere. Audio is captured, transcribed, and discarded on the user\'s machine. There is no audio stream sent to any server, ours or anyone else\'s. The desktop app makes a small number of network calls (licence activation, update checks, model downloads, pricing lookup). Audio, transcripts, vocabulary, and history all stay local.',
                    ],
                    [
                        'q' => 'How do we deploy it across a fleet?',
                        'a' => 'Per-user MSI or NSIS, silent install with the standard /S flag, no admin rights required. Bundle the installer in your standard deployment tool (SCCM, Intune, PDQ Deploy, whatever you use). Settings live in %APPDATA% per user. Group-policy registry overrides are supported for a few admin-pinnable settings.',
                    ],
                    [
                        'q' => 'Can it run on an air-gapped network?',
                        'a' => 'Yes, on request. The model files have to reach the machines once. After that the app runs offline indefinitely. Air-gapped activation (sign a request file offline, ingest a signed activation file) is supported on request for Business and Enterprise customers. Mention it in your enquiry.',
                    ],
                    [
                        'q' => 'Is the installer code-signed?',
                        'a' => 'Not yet. Today users see a Windows SmartScreen warning on first install. We will sign for Business and Enterprise customers as part of the rollout. For pilot deployments we can supply a hash and the binary.',
                    ],
                    [
                        'q' => 'What happens if you go out of business?',
                        'a' => 'Licence keys are verified locally with Ed25519 signatures. The app does not phone home for activation status. If we shut down, your existing installs keep working. Business customers get a written commitment to release the source under a non-commercial licence in that scenario.',
                    ],
                    [
                        'q' => 'Do we need to maintain user accounts?',
                        'a' => 'No. There is no per-user account on our side. A licence is a key. Your IT team manages who has the key. The user installs the app, pastes the key, dictates. No SSO to integrate, no SAML to configure.',
                    ],
                ] as $item)
                    <div class="ub-card">
                        <h3 class="text-lg mb-2" style="color: var(--fg-hi); letter-spacing: -0.01em;">{{ $item['q'] }}</h3>
                        <p style="color: var(--muted); line-height: 1.6;">{{ $item['a'] }}</p>
                    </div>
                @endforeach
            </div>
        </div>
    </section>

    {{-- Pricing block. Three visible tiers, fourth is contact-us.
         Mirrors /pricing but lives here too so a buyer who lands on
         /business doesn't have to bounce around. --}}
    <section id="pricing" class="py-20 md:py-24" style="background: var(--bg-1);">
        <div class="ub-container">
            <div class="max-w-3xl mb-12">
                <p class="ub-eyebrow">Volume pricing</p>
                <h2 class="mt-4" style="font-size: clamp(2rem, 4.5vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Pay <x-hl>once</x-hl>. Per seat. For life.
                </h2>
                <p class="ub-lead mt-6 max-w-xl">
                    Lifetime licences in plain GBP. The per-seat price drops at volume. Pay by card under £500, by invoice and PO above it.
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-5 max-w-5xl">
                {{-- Team --}}
                <div class="ub-card">
                    <p class="ub-eyebrow mb-3">Team</p>
                    <div class="flex items-baseline gap-1">
                        <span class="text-4xl font-extrabold tracking-tight" style="color: var(--fg-hi);">&pound;29</span>
                        <span class="text-sm" style="color: var(--muted);">per seat, lifetime</span>
                    </div>
                    <p class="mt-3 text-sm" style="color: var(--muted);">5 to 24 seats.</p>
                    <ul class="mt-6 space-y-2.5 text-sm" style="color: var(--fg);">
                        <li>Full Pro feature set per seat</li>
                        <li>One licence, your seat count, your devices</li>
                        <li>Pay by card or invoice</li>
                        <li>Email support, one working day</li>
                    </ul>
                    <a href="{{ route('business.contact') }}" class="ub-btn ub-btn-ghost mt-8 w-full">Get a quote</a>
                </div>

                {{-- Business --}}
                <div class="ub-card relative" style="background:
                        radial-gradient(120% 80% at 100% 0%, color-mix(in oklab, var(--accent) 12%, transparent), transparent 60%),
                        var(--bg-2);">
                    <p class="ub-eyebrow mb-3" style="color: var(--accent);">Business</p>
                    <div class="flex items-baseline gap-1">
                        <span class="text-4xl font-extrabold tracking-tight ub-gradient-text">&pound;19</span>
                        <span class="text-sm" style="color: var(--muted);">per seat, lifetime</span>
                    </div>
                    <p class="mt-3 text-sm" style="color: var(--muted);">25 to 100 seats.</p>
                    <ul class="mt-6 space-y-2.5 text-sm" style="color: var(--fg);">
                        <li>Everything in Team</li>
                        <li>Invoice / PO billing standard</li>
                        <li>Air-gapped activation on request</li>
                        <li>Group-policy settings overrides supported</li>
                        <li>Optional priority support add-on</li>
                    </ul>
                    <a href="{{ route('business.contact') }}" class="ub-btn ub-btn-cta mt-8 w-full">Get a quote</a>
                </div>

                {{-- Enterprise --}}
                <div class="ub-card">
                    <p class="ub-eyebrow mb-3">Enterprise</p>
                    <div class="flex items-baseline gap-1">
                        <span class="text-4xl font-extrabold tracking-tight" style="color: var(--fg-hi);">Talk to us</span>
                    </div>
                    <p class="mt-3 text-sm" style="color: var(--muted);">250+ seats.</p>
                    <ul class="mt-6 space-y-2.5 text-sm" style="color: var(--fg);">
                        <li>Custom per-seat pricing</li>
                        <li>Custom contract terms</li>
                        <li>Code-signed installer included</li>
                        <li>Source-escrow clause available</li>
                        <li>Named contact for the duration</li>
                    </ul>
                    <a href="{{ route('business.contact') }}" class="ub-btn ub-btn-ghost mt-8 w-full">Get in touch</a>
                </div>
            </div>

            <p class="mt-8 max-w-3xl text-sm" style="color: var(--muted); line-height: 1.6;">
                <strong style="color: var(--fg);">Optional: priority support.</strong>
                &pound;15 per seat per year on top of the lifetime licence. 24-hour email response, named contact, four-week beta soak before auto-update touches your fleet, source-escrow clause if Utterbar ever shuts down. Most teams don&rsquo;t need it. The ones that do are the ones running compliance-bounded deployments.
            </p>
        </div>
    </section>

    {{-- Final CTA --}}
    <x-prompt-cta
        :heading="'Ready to talk?'"
        :sub="'Tell us seat count and use case. You get a quote and the IT deployment notes back within one working day.'"
    >
        <a href="{{ route('business.contact') }}" class="ub-btn ub-btn-cta">Get a quote</a>
        <a href="{{ route('docs.it-deployment') }}" class="ub-btn ub-btn-ghost">IT deployment notes</a>
    </x-prompt-cta>
</x-layouts.app>
