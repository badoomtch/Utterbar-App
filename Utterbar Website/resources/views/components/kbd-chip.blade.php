@props(['keys' => []])

<span class="inline-flex items-center gap-1">
    @foreach ($keys as $i => $k)
        <kbd class="ub-kbd">{{ $k }}</kbd>
        @if ($i < count($keys) - 1)
            <span class="ub-muted text-xs">+</span>
        @endif
    @endforeach
</span>
