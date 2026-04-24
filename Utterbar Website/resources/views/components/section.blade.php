@props(['eyebrow' => null, 'title' => null, 'lead' => null, 'align' => 'left', 'id' => null])

<section @if($id) id="{{ $id }}" @endif {{ $attributes->merge(['class' => 'py-20 md:py-28']) }}>
    <div class="ub-container">
        @if ($eyebrow || $title || $lead)
            <div class="max-w-3xl {{ $align === 'center' ? 'mx-auto text-center' : '' }} mb-12 md:mb-16">
                @if ($eyebrow)
                    <p class="ub-eyebrow">{!! $eyebrow !!}</p>
                @endif
                @if ($title)
                    <h2 class="mt-4">{!! $title !!}</h2>
                @endif
                @if ($lead)
                    <p class="ub-lead mt-5">{!! $lead !!}</p>
                @endif
            </div>
        @endif
        {{ $slot }}
    </div>
</section>
