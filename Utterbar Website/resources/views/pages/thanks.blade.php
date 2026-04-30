<x-layouts.app title="Thanks" :no-index="true">
    <section class="pt-20 md:pt-28 pb-24">
        <div class="ub-container">
            <div class="max-w-2xl mx-auto text-center">
                <div class="inline-flex items-center justify-center rounded-full mb-8"
                     style="width: 72px; height: 72px;
                            background: color-mix(in oklab, var(--accent) 14%, transparent);
                            color: var(--accent);
                            border: 1px solid color-mix(in oklab, var(--accent) 45%, transparent);
                            box-shadow: 0 0 40px -6px color-mix(in oklab, var(--accent) 50%, transparent);">
                    <x-icon name="check" size="32" stroke="2.6" />
                </div>

                <h1 class="ub-fg-hi" style="font-size: clamp(2.5rem, 5vw, 4rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Payment received.
                </h1>

                <p class="ub-lead mt-6 max-w-xl mx-auto">
                    Your licence key is on its way by email. Usually within a minute, sometimes two.
                    Check junk if it takes longer.
                </p>

                <div class="mt-10 ub-card text-left" style="padding: 1.75rem 2rem;">
                    <p class="ub-eyebrow mb-3">Next step</p>
                    <ol class="space-y-3 text-sm" style="color: var(--fg);">
                        <li>1. Open the email from <span style="color: var(--accent);">hello@ddraig.dev</span></li>
                        <li>2. Copy the licence key (<span class="font-mono" style="color: var(--fg-hi);">UB-XXXX-XXXX-XXXX-XXXX-XXXX</span>)</li>
                        <li>3. Paste it into Utterbar &gt; Settings &gt; Licence</li>
                    </ol>
                </div>

                <p class="mt-10 text-sm" style="color: var(--muted);">
                    Didn&rsquo;t get the email? <a href="{{ route('contact') }}" style="color: var(--accent);">Email me</a> and I&rsquo;ll resend it by hand.
                </p>
            </div>
        </div>
    </section>
</x-layouts.app>
