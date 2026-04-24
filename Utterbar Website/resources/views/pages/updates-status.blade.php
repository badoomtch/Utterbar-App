<x-layouts.app :title="$title" :no-index="true">
    <section class="pt-20 md:pt-28 pb-24">
        <div class="ub-container">
            <div class="max-w-xl mx-auto text-center">
                @if ($status === 'ok')
                    <div class="inline-flex items-center justify-center rounded-full mb-8"
                         style="width: 64px; height: 64px;
                                background: color-mix(in oklab, var(--accent) 14%, transparent);
                                color: var(--accent);
                                border: 1px solid color-mix(in oklab, var(--accent) 45%, transparent);
                                box-shadow: 0 0 40px -8px color-mix(in oklab, var(--accent) 50%, transparent);">
                        <x-icon name="check" size="28" stroke="2.6" />
                    </div>
                @endif

                <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.1;">
                    {{ $title }}
                </h1>

                <p class="ub-lead mt-6" style="font-size: 1rem;">
                    {{ $body }}
                </p>

                <a href="{{ route('home') }}" class="ub-btn ub-btn-ghost mt-10">Back to utterbar.com</a>
            </div>
        </div>
    </section>
</x-layouts.app>
