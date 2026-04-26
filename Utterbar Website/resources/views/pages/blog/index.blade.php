@php
    // Blog index. Posts are file-backed under resources/blog/.
    // No CMS, no DB — edit a Markdown file, redeploy.
    $hasPosts = isset($posts) && count($posts) > 0;
@endphp

<x-layouts.app
    title="Blog"
    description="Notes from building Utterbar — local dictation for Windows. Updates, design choices, and the occasional dive into how local ASR actually works.">

    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <p class="ub-eyebrow" style="color: var(--accent);">Blog</p>
                <h1 class="ub-fg-hi mt-3" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Notes from building Utterbar.
                </h1>
                <p class="ub-lead mt-8 max-w-2xl">
                    Updates, design choices, and the occasional dive into how local speech recognition actually works on a Windows laptop.
                </p>
            </div>
        </div>
    </section>

    <section class="py-12 md:py-16">
        <div class="ub-container">
            <div class="max-w-3xl">
                @if (!$hasPosts)
                    <div class="ub-card" style="padding: 2rem;">
                        <p style="color: var(--muted);">
                            Posts will land here as I write them. If there's a topic you'd want to read, <a href="{{ route('contact') }}" style="color: var(--accent);">drop a line</a>.
                        </p>
                    </div>
                @else
                    <ul class="space-y-8">
                        @foreach ($posts as $post)
                            <li class="ub-card" style="padding: 1.75rem 2rem;">
                                <p class="font-mono text-xs" style="color: var(--muted); letter-spacing: 0.06em; text-transform: uppercase;">
                                    {{ $post->publishedAt()->format('j M Y') }}
                                    @if ($post->draft)
                                        <span style="color: var(--accent);">&middot; draft</span>
                                    @endif
                                </p>
                                <h2 class="mt-2" style="font-size: clamp(1.5rem, 2.5vw, 1.875rem); letter-spacing: -0.02em; line-height: 1.15;">
                                    <a href="{{ $post->url() }}" style="color: var(--fg-hi);" class="hover:underline">{{ $post->title }}</a>
                                </h2>
                                @if ($post->excerpt)
                                    <p class="mt-3 text-base leading-relaxed" style="color: var(--muted);">{{ $post->excerpt }}</p>
                                @endif
                                <a href="{{ $post->url() }}" class="inline-flex items-center mt-5 font-mono text-xs" style="color: var(--accent); letter-spacing: 0.04em;">
                                    Read &rarr;
                                </a>
                            </li>
                        @endforeach
                    </ul>
                @endif
            </div>
        </div>
    </section>
</x-layouts.app>
