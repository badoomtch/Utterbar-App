<x-layouts.app title="Thanks for the tip" :no-index="true">
    <section class="pt-20 pb-32">
        <div class="ub-container">
            <div class="max-w-2xl mx-auto text-center">
                <div class="inline-flex items-center justify-center mb-6 w-16 h-16 rounded-full"
                     style="background: color-mix(in oklab, var(--accent) 14%, var(--bg-2));">
                    <x-icon name="sparkles" size="32" stroke="2" style="color: var(--accent);" />
                </div>
                <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em;">
                    Thank you.
                </h1>
                <p class="mt-5 text-lg leading-relaxed" style="color: var(--muted);">
                    Your tip is in. Stripe is sending you a receipt now. Check your inbox if you want one for the records. Tips go straight into the next release.
                </p>
                <div class="mt-10 flex flex-wrap justify-center gap-3">
                    <a href="{{ route('home') }}" class="ub-btn ub-btn-ghost">Back to Utterbar</a>
                    <a href="{{ route('changelog') }}" class="ub-btn ub-btn-cta">See what's shipping</a>
                </div>
            </div>
        </div>
    </section>
</x-layouts.app>
