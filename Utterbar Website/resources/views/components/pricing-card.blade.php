@props(['name' => '', 'price' => '', 'period' => '', 'blurb' => '', 'features' => [], 'highlight' => false, 'cta' => 'See pricing', 'ctaHref' => null])

@php
    $ctaHref = $ctaHref ?? route('pricing');
    $ring = $highlight ? 'ring-2' : '';
@endphp

<article class="ub-card h-full flex flex-col {{ $ring }}" @if($highlight) style="border-color: var(--accent); box-shadow: 0 0 0 2px color-mix(in oklab, var(--accent) 30%, transparent);" @endif>
    @if ($highlight)
        <span class="self-start -mt-9 mb-3 inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full font-mono text-[0.7rem]" style="background: var(--gradient-cyan); color: var(--bg-0);">
            <span class="w-1.5 h-1.5 rounded-full" style="background: var(--bg-0);"></span>
            MOST POPULAR
        </span>
    @endif

    <h3 class="text-xl">{{ $name }}</h3>
    <p class="mt-2 text-sm" style="color: var(--muted);">{{ $blurb }}</p>

    <div class="mt-6 flex items-baseline gap-1.5">
        <span class="text-4xl font-extrabold tracking-tight" style="color: var(--fg-hi);">{{ $price }}</span>
        @if ($period)
            <span class="text-sm" style="color: var(--muted);">{{ $period }}</span>
        @endif
    </div>

    <ul class="mt-6 space-y-3 flex-1">
        @foreach ($features as $f)
            <li class="flex items-start gap-2.5 text-sm" style="color: var(--fg);">
                <span class="mt-0.5 shrink-0" style="color: var(--accent);">
                    <x-icon name="check" size="16" stroke="2" />
                </span>
                <span>{!! $f !!}</span>
            </li>
        @endforeach
    </ul>

    <div class="mt-8">
        <a href="{{ $ctaHref }}" class="ub-btn {{ $highlight ? 'ub-btn-cta' : 'ub-btn-ghost' }} w-full">{{ $cta }}</a>
    </div>
</article>
