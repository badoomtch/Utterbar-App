@props([
    'id',               // manifest video id (e.g. 'hero-pill-loop')
    'class'   => '',
    'style'   => '',
    'rounded' => true,  // rounded corners (set false for full-bleed hero usage)
])

@php
    $meta = config("marketing.videos.{$id}");
    if (!$meta) {
        throw new \Exception("Unknown marketing video id: {$id}");
    }
    $base = rtrim((string) config('marketing.base_url'), '/');

    // Append file mtime as a cache-bust version. Means every re-render gets a
    // unique URL, so Cloudflare's edge cache doesn't serve stale versions —
    // old files age out naturally, new files miss the cache and fetch fresh.
    $videoPath  = public_path("videos/{$id}.mp4");
    $posterPath = public_path("posters/{$id}-poster.png");
    $videoV  = file_exists($videoPath)  ? filemtime($videoPath)  : 1;
    $posterV = file_exists($posterPath) ? filemtime($posterPath) : 1;

    $video  = "{$base}/videos/{$id}.mp4?v={$videoV}";
    $poster = "{$base}/posters/{$id}-poster.png?v={$posterV}";
    $w = $meta['width'];
    $h = $meta['height'];
    $alt = $meta['alt'];
    $loops = !empty($meta['loops']);

    // Deliberately minimal wrapper: aspect-ratio to reserve layout space, no
    // bg fill, no border. Compositions render their own bg-0 (#0A0E17) edge
    // to edge, which blends seamlessly into the page section's bg-0.
    $wrapperStyle = trim("aspect-ratio: {$w} / {$h}; position: relative; {$style}");
    $wrapperClass = trim(($rounded ? 'rounded-2xl overflow-hidden ' : '') . $class);
@endphp

<figure
    x-data="ubVideoEmbed()"
    x-init="init()"
    class="ub-video-embed {{ $wrapperClass }}"
    style="{{ $wrapperStyle }}"
>
    {{-- Reduced-motion users: no <video> element, poster only --}}
    <template x-if="reducedMotion">
        <img
            src="{{ $poster }}"
            alt="{{ $alt }}"
            width="{{ $w }}" height="{{ $h }}"
            loading="lazy"
            style="display: block; width: 100%; height: 100%; object-fit: cover;"
        />
    </template>

    <template x-if="!reducedMotion">
        <video
            x-ref="video"
            src="{{ $video }}"
            poster="{{ $poster }}"
            muted
            autoplay
            @if ($loops) loop @endif
            playsinline
            disablepictureinpicture
            disableremoteplayback
            controlslist="nodownload noplaybackrate nofullscreen noremoteplayback"
            preload="metadata"
            aria-label="{{ $alt }}"
            style="display: block; width: 100%; height: 100%; object-fit: cover;"
        ></video>
    </template>
</figure>
