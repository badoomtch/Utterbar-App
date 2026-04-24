@props(['num' => null, 'title' => '', 'body' => ''])

<article class="ub-card ub-card-rail h-full">
    @if ($num)
        <p class="ub-eyebrow mb-5">{{ $num }}</p>
    @endif
    <h3 style="font-size: clamp(1.5rem, 2vw, 1.9rem); line-height: 1.1; letter-spacing: -0.03em; font-weight: 700; color: var(--fg-hi);">
        {{ $title }}
    </h3>
    <p class="mt-4 text-base md:text-lg" style="color: var(--fg); line-height: 1.55; font-weight: 300;">
        {!! $body !!}
    </p>
</article>
