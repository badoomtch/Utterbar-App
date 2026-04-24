@props([
    'id',
    'class'   => '',
    'style'   => '',
    'loading' => 'lazy',
    'rounded' => true,
])

@php
    $meta = config("marketing.stills.{$id}");
    if (!$meta) {
        throw new \Exception("Unknown marketing still id: {$id}");
    }
    $base = rtrim((string) config('marketing.base_url'), '/');

    // Cache-bust via file mtime so Cloudflare serves fresh content after a
    // re-render without any manual purge.
    $stillPath = public_path("stills/{$id}.png");
    $v = file_exists($stillPath) ? filemtime($stillPath) : 1;
    $src = "{$base}/stills/{$id}.png?v={$v}";
    $wrapperClass = trim(($rounded ? 'rounded-2xl overflow-hidden ' : '') . $class);
@endphp

<img
    src="{{ $src }}"
    alt="{{ $meta['alt'] }}"
    width="{{ $meta['width'] }}"
    height="{{ $meta['height'] }}"
    loading="{{ $loading }}"
    decoding="async"
    class="{{ $wrapperClass }}"
    style="display: block; width: 100%; height: auto; background: var(--bg-1); border: 1px solid var(--border); {{ $style }}"
/>
