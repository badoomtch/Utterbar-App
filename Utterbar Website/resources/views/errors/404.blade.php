<x-layouts.app title="404">
    <section class="py-24 md:py-32">
        <div class="ub-container text-center max-w-2xl mx-auto">
            <div class="flex justify-center mb-10">
                <div class="relative inline-flex items-center gap-4 rounded-full px-8 py-5"
                     style="background: color-mix(in oklab, var(--bg-1) 92%, transparent); border: 1px solid var(--border); box-shadow: 0 30px 80px -30px rgba(34,211,238,0.35);">
                    <span class="w-2.5 h-2.5 rounded-full" style="background: var(--muted);"></span>
                    <span class="inline-block h-1 w-32 rounded-full" style="background: var(--muted);"></span>
                    <span class="font-mono text-sm" style="color: var(--muted);">404</span>
                </div>
            </div>
            <h1 class="text-4xl md:text-5xl">Silence detected.</h1>
            <p class="mt-4 text-lg" style="color: var(--muted);">That page isn't here.</p>
            <div class="mt-8">
                <a href="{{ route('home') }}" class="ub-btn ub-btn-primary">Back home</a>
            </div>
        </div>
    </section>
</x-layouts.app>
