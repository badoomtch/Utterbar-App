<x-layouts.app title="Waitlist">
    <section class="relative overflow-hidden">
        <div class="ub-container py-16 md:py-24">
            <div class="max-w-2xl mx-auto text-center">
                <h1><span class="ub-gradient-text">Join the list.</span></h1>
                <p class="ub-lead mt-6">
                    We'll email once there's a build worth installing. Waitlist members get early access and 50% off Pro for life once 1.0 ships.
                </p>

                <div class="mt-10 flex justify-center">
                    <x-waitlist-form :showUseCase="true" />
                </div>

                <p class="mt-10 text-sm" style="color: var(--muted);">
                    No unsubscribe dark-pattern. One-click out of any email we send you.
                </p>
            </div>

        </div>
    </section>
</x-layouts.app>
