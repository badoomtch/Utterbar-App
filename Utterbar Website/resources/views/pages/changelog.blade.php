@php
    $latest = $releases->first();
    $changelogDescription = $latest
        ? 'What changed in every Utterbar release. Latest: v' . $latest->version . ' on ' . $latest->published_at?->format('j F Y') . '.'
        : 'What changed in every Utterbar release. Detailed notes for each shipped version.';
@endphp

<x-layouts.app title="Changelog" :description="$changelogDescription">
    <section class="pt-20 md:pt-28 pb-12">
        <div class="ub-container">
            <div class="max-w-4xl">
                <p class="ub-eyebrow">Changelog</p>
                <h1 class="ub-fg-hi mt-4" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Every release, <x-hl>in detail</x-hl>.
                </h1>
                <p class="ub-lead mt-8 max-w-2xl">
                    What shipped, when, and why. Each release note is the same one that goes out to the in-app updater &mdash; no marketing rewrite, no &ldquo;various improvements and bug fixes&rdquo;.
                </p>
                @if ($latest)
                    <p class="mt-6 font-mono text-sm" style="color: var(--muted);">
                        Latest: <span style="color: var(--accent);">v{{ $latest->version }}</span> &middot; {{ $latest->published_at?->format('j F Y') }}
                        &middot; <a href="{{ route('download') }}" style="color: var(--accent);">download</a>
                    </p>
                @endif
            </div>
        </div>
    </section>

    <section class="py-8 md:py-12">
        <div class="ub-container">
            @if ($releases->isEmpty())
                <div class="ub-card max-w-3xl">
                    <p style="color: var(--muted);">Release notes will appear here as new builds ship. <a href="{{ route('download') }}" style="color: var(--accent);">Grab the latest build</a> from the download page.</p>
                </div>
            @else
                <div class="max-w-3xl space-y-10">
                    @foreach ($releases as $r)
                        <article id="v{{ $r->version }}" class="ub-card" style="scroll-margin-top: 6rem;">
                            <header class="flex items-baseline gap-3 flex-wrap">
                                <a href="#v{{ $r->version }}" class="font-mono" style="font-size: 1.5rem; font-weight: 600; color: var(--fg-hi); letter-spacing: -0.02em; text-decoration: none;">v{{ $r->version }}</a>
                                @if ($r->tag)
                                    <span class="font-mono text-[10px] uppercase tracking-wider px-2 py-0.5 rounded-full" style="background: color-mix(in oklab, var(--accent) 15%, transparent); color: var(--accent); border: 1px solid color-mix(in oklab, var(--accent) 30%, transparent);">
                                        {{ $r->tag }}
                                    </span>
                                @endif
                                <span class="font-mono text-xs" style="color: var(--muted); letter-spacing: 0.04em;">{{ $r->published_at?->format('j F Y') }}</span>
                                @if ($r->file_size_bytes)
                                    <span class="font-mono text-xs" style="color: var(--muted); letter-spacing: 0.04em;">&middot; {{ $r->sizeLabel() }}</span>
                                @endif
                            </header>
                            <div class="ub-markdown mt-5" style="color: var(--fg); line-height: 1.7; font-size: 1rem;">
                                {!! \App\Support\Markdown::renderEmbedded($r->notes_markdown) !!}
                            </div>
                            @if ($r->sha256)
                                <details class="mt-6 text-xs" style="color: var(--muted);">
                                    <summary class="cursor-pointer font-mono uppercase tracking-wider" style="letter-spacing: 0.12em;">SHA-256</summary>
                                    <p class="mt-2 font-mono text-[11px]" style="word-break: break-all; color: var(--fg);">{{ $r->sha256 }}</p>
                                </details>
                            @endif
                        </article>
                    @endforeach
                </div>
            @endif
        </div>
    </section>

    <x-prompt-cta
        :heading="'Get the latest build.'"
        :sub="'Auto-update is built in &mdash; if you&rsquo;re already running Utterbar, the app will tell you when there&rsquo;s a new version.'"
    />
</x-layouts.app>
