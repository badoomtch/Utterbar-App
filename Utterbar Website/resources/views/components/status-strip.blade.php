@props(['tokens' => []])

@php
    if (empty($tokens)) {
        $tokens = [
            ['kind' => 'led'],
            ['kind' => 'text', 'label' => 'LOCAL'],
            ['kind' => 'sep'],
            ['kind' => 'text', 'label' => 'parakeet-tdt-0.6b-v3-int8'],
            ['kind' => 'sep'],
            ['kind' => 'text', 'label' => '<250ms latency'],
            ['kind' => 'sep'],
            ['kind' => 'text', 'label' => 'v' . config('site.version')],
        ];
    }
@endphp

<div class="inline-flex items-center gap-3 font-mono text-[11px] tracking-wider whitespace-nowrap px-3 py-1.5 rounded-full"
     style="background: color-mix(in oklab, var(--bg-1) 85%, transparent); border: 1px solid var(--border); color: var(--muted);">
    @foreach ($tokens as $t)
        @if ($t['kind'] === 'led')
            <span class="inline-flex items-center gap-2">
                <span class="inline-block w-1.5 h-1.5 rounded-full" style="background: var(--accent); box-shadow: 0 0 8px var(--accent), 0 0 2px var(--accent-2); animation: ub-pulse 1.8s ease-in-out infinite;"></span>
            </span>
        @elseif ($t['kind'] === 'sep')
            <span style="color: color-mix(in oklab, var(--muted) 50%, transparent);">·</span>
        @else
            <span class="uppercase" style="color: var(--muted);">{{ $t['label'] }}</span>
        @endif
    @endforeach
</div>
