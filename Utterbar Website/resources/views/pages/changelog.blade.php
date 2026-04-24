<x-layouts.app title="Changelog">
    <x-section eyebrow="Changelog" title="Release notes.">
        <div class="ub-card max-w-3xl">
            <p class="font-mono text-sm" style="color: var(--accent);">v{{ config('site.version') }} — preview</p>
            <p class="mt-3" style="color: var(--muted);">Utterbar is pre-1.0. Full release notes start landing once the public build ships. Until then, <a href="{{ route('waitlist') }}" style="color: var(--accent);">join the waitlist</a> to get them by email as they happen.</p>
        </div>
    </x-section>
</x-layouts.app>
