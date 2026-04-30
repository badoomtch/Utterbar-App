<x-layouts.app title="Confirm subscription" :no-index="true">
    <section class="pt-20 md:pt-28 pb-24">
        <div class="ub-container">
            <div class="max-w-xl mx-auto text-center">

                <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.1;">
                    Confirm release-update emails
                </h1>

                <p class="ub-lead mt-6" style="font-size: 1rem;">
                    Click the button to finish subscribing. We&rsquo;ll only email when there&rsquo;s a new release worth installing. Never marketing, never digests, never anyone else&rsquo;s product.
                </p>

                <form method="POST" action="{{ route('updates.confirm.post') }}" class="mt-10">
                    @csrf
                    <input type="hidden" name="t" value="{{ $token }}">
                    <button type="submit" class="ub-btn ub-btn-cta">Confirm subscription</button>
                </form>

                <p class="mt-8" style="color: var(--muted); font-size: 0.875rem;">
                    Didn&rsquo;t mean to subscribe? Just close this tab. Your subscription stays unconfirmed and the link expires by itself.
                </p>

            </div>
        </div>
    </section>
</x-layouts.app>
