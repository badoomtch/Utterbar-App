<x-layouts.app title="Privacy policy">
    <x-section eyebrow="Legal" title="Privacy policy">
        <article class="max-w-3xl space-y-5 leading-relaxed" style="color: var(--fg);">
            <p class="font-mono text-sm" style="color: var(--muted);">Last updated: {{ date('j F Y') }} · Placeholder — full policy published with 1.0.</p>

            <h3>Summary</h3>
            <p>Utterbar is local-only. Your voice and transcripts stay on your machine. The website runs cookieless analytics (Plausible) and stores a single waitlist email on our MySQL database if you give us one.</p>

            <h3>What we collect via the website</h3>
            <ul class="list-disc pl-5 space-y-1.5" style="color: var(--muted);">
                <li>Your email address, if you join the waitlist.</li>
                <li>An optional single-line "what will you use Utterbar for" if you give us one.</li>
                <li>Your IP and user-agent string at the moment of signup, for abuse prevention. Retained for 90 days, then truncated.</li>
                <li>Cookieless, aggregated page-view counts via Plausible.</li>
            </ul>

            <h3>What we do not collect</h3>
            <p>Audio. Transcripts. Anything the desktop app generates. The app never phones home.</p>

            <h3>Your rights (GDPR / UK GDPR)</h3>
            <p>You can request access, export, rectification, or deletion of your waitlist record any time by emailing <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a>.</p>

            <h3>Contact</h3>
            <p>Data controller: Jay Parry, trading as Utterbar. Email <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a>.</p>
        </article>
    </x-section>
</x-layouts.app>
