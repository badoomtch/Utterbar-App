@props(['label' => null, 'target' => null, 'heading' => null, 'sub' => null])

{{-- Quiet closing CTA. No terminal chrome, no window mockup. Single confident section. --}}
<section class="py-32 md:py-40">
    <div class="ub-container">
        <div class="max-w-2xl mx-auto text-center">
            @if ($heading)
                <h2 class="tracking-tight">{!! $heading !!}</h2>
            @endif
            @if ($sub)
                <p class="mt-5 text-lg" style="color: var(--muted); line-height: 1.6;">{!! $sub !!}</p>
            @endif

            <div class="mt-12 flex justify-center">
                <x-waitlist-form />
            </div>
        </div>
    </div>
</section>
