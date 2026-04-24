<x-layouts.app title="Check your email" :no-index="true">
    <section class="pt-20 md:pt-28 pb-24">
        <div class="ub-container">
            <div class="max-w-md mx-auto text-center">
                <div class="inline-flex items-center justify-center rounded-full mb-8"
                     style="width: 64px; height: 64px;
                            background: color-mix(in oklab, var(--accent) 14%, transparent);
                            color: var(--accent);
                            border: 1px solid color-mix(in oklab, var(--accent) 45%, transparent);
                            box-shadow: 0 0 40px -8px color-mix(in oklab, var(--accent) 50%, transparent);">
                    <x-icon name="check" size="28" stroke="2.6" />
                </div>

                <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.1;">
                    Check your inbox.
                </h1>

                <p class="ub-lead mt-5" style="font-size: 1rem;">
                    If <span style="color: var(--fg-hi);">{{ $email }}</span> has an Utterbar account or licence,
                    we just sent a sign-in link. Click it within 20 minutes.
                </p>

                <p class="mt-8 text-sm" style="color: var(--muted);">
                    Nothing arrived? <a href="{{ route('sign-in.show') }}" style="color: var(--accent);">Try again</a>
                    or <a href="{{ route('contact') }}" style="color: var(--accent);">email me directly</a>.
                </p>
            </div>
        </div>
    </section>
</x-layouts.app>
