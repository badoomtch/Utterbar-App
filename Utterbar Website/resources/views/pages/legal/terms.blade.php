<x-layouts.app title="Terms of service">
    <x-section eyebrow="Legal" title="Terms of service">
        <article class="max-w-3xl space-y-5 leading-relaxed" style="color: var(--fg);">
            <p class="font-mono text-sm" style="color: var(--muted);">Last updated: {{ date('j F Y') }} · Placeholder — full terms published with 1.0.</p>

            <h3>Using this website</h3>
            <p>By using utterbar.com you agree to act in good faith, not attempt to break our waitlist endpoint, and not scrape content for repackaging.</p>

            <h3>The app</h3>
            <p>The desktop app is covered by a separate End-User Licence Agreement (EULA). See <a href="{{ route('legal.eula') }}" style="color: var(--accent);">the EULA</a>.</p>

            <h3>No warranty</h3>
            <p>The website is provided "as is". Utterbar disclaims all implied warranties to the extent permitted by law.</p>

            <h3>Jurisdiction</h3>
            <p>These terms are governed by the laws of England &amp; Wales.</p>

            <h3>Changes</h3>
            <p>If these terms change materially, we'll note it at the top of this page and on the home page for two weeks.</p>
        </article>
    </x-section>
</x-layouts.app>
