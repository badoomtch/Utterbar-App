@props(['heading' => null, 'sub' => null])

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

            {{-- Slot lets callers supply their own buttons; default is Download + Contact. --}}
            <div class="mt-12 flex flex-wrap items-center justify-center gap-4">
                @isset($slot)
                    @if (trim($slot) !== '')
                        {!! $slot !!}
                    @else
                        <a href="{{ route('download') }}" class="ub-btn ub-btn-cta">Download Utterbar</a>
                        <a href="{{ route('contact') }}" class="ub-btn ub-btn-ghost">Contact us</a>
                    @endif
                @else
                    <a href="{{ route('download') }}" class="ub-btn ub-btn-cta">Download Utterbar</a>
                    <a href="{{ route('contact') }}" class="ub-btn ub-btn-ghost">Contact us</a>
                @endisset
            </div>
        </div>
    </div>
</section>
