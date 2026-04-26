<x-layouts.app title="Privacy policy" description="Utterbar's formal privacy policy: what the website stores, what the desktop app does not collect, and your UK GDPR rights.">
    <x-section eyebrow="Legal" title="Privacy policy">
        <article class="max-w-3xl space-y-5 leading-relaxed" style="color: var(--fg);">
            <p class="font-mono text-sm" style="color: var(--muted);">Last updated: 26 April 2026</p>

            <p style="color: var(--muted);">
                For the short, plain-English summary of how the desktop app handles your voice, see <a href="{{ route('privacy') }}" style="color: var(--accent);">the privacy overview</a>. This page is the formal policy.
            </p>

            <h3>1. The short version</h3>
            <p>Utterbar is local-only. Your voice, transcripts, vocabulary and history stay on your PC and never reach our servers. The website itself collects only what it needs to sell you a licence, send you the things you asked for, and run the site.</p>

            <h3>2. Who we are</h3>
            <p>The data controller is Jay Parry, trading as Utterbar, based in the United Kingdom. You can reach us at <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a>.</p>

            <h3>3. What the desktop app collects</h3>
            <p>Nothing about your voice, transcripts, or use is uploaded to us. The app makes a small number of outbound HTTP calls, all listed here:</p>
            <ul class="list-disc pl-5 space-y-1.5" style="color: var(--muted);">
                <li><strong>Speech-model download.</strong> On first run, fetches the Parakeet ASR model file from a content delivery network. No identifiers are sent.</li>
                <li><strong>Licence verification.</strong> If you buy Pro, the app contacts our API to activate your licence on the device. We store the activation (see &sect;4).</li>
                <li><strong>Update check.</strong> If the &ldquo;Check for updates automatically&rdquo; setting is on (you choose during onboarding), the app sends one HTTP request per launch containing only your current version number, asking whether a newer build is available. You can turn it off any time in Settings &rarr; General.</li>
                <li><strong>Update download.</strong> If you accept an update, the new installer is fetched from <code class="font-mono px-1.5 py-0.5 rounded text-sm" style="background: var(--bg-2); color: var(--accent);">dl.utterbar.com</code>; this download is logged on our side as described in &sect;5.</li>
            </ul>

            <h3>4. What we collect via the licence system</h3>
            <ul class="list-disc pl-5 space-y-1.5" style="color: var(--muted);">
                <li><strong>Purchase records.</strong> When you buy Utterbar Pro, we store the email address used at checkout, the Stripe customer and checkout-session IDs, and the licence key we mint for you. Card details are handled entirely by <a href="https://stripe.com/gb/privacy" style="color: var(--accent);">Stripe</a>; we never see them.</li>
                <li><strong>Device activations.</strong> When you activate Utterbar on a machine, we store a SHA-256 hash of the machine&rsquo;s device identifier, the device name you provided, and the date of activation, so we can enforce the three-device cap and let you transfer the licence between machines.</li>
                <li><strong>Update subscribers.</strong> If you opt in to release-announcement emails from inside the app, we store your email address, the date you opted in, and a confirmation token. One click unsubscribes you.</li>
            </ul>

            <h3>5. What we collect via the website</h3>
            <ul class="list-disc pl-5 space-y-1.5" style="color: var(--muted);">
                <li><strong>Contact form.</strong> If you message us via the contact form, we store the name, email and message you sent. We may also retain the IP address and User-Agent for abuse prevention.</li>
                <li><strong>Download analytics.</strong> When you click a download link, we log a single event recording: the release version, a SHA-256 hash of <em>(your IP address || a server-side secret)</em>, a SHA-256 hash of <em>(your User-Agent || a server-side secret)</em>, the two-letter country code provided by Cloudflare (where available), and the date. Reinstalls within the same day collapse to one event. The hashes cannot be reversed without the server-side secret, which never leaves the server. We use this only to count unique downloads per release and per country in aggregate.</li>
                <li><strong>Cookies.</strong> See &sect;7.</li>
                <li><strong>Server logs.</strong> Our hosting provider keeps standard request logs (timestamp, URL, IP, User-Agent) for a short period as a security and abuse measure. We don&rsquo;t use these for marketing or analytics.</li>
            </ul>

            <h3>6. Analytics</h3>
            <p>If you tap &ldquo;Accept&rdquo; on the cookie banner, we load Google Analytics 4 to measure aggregate traffic patterns (pages viewed, broad referrer category, country). We have not enabled Google Signals, advertising features, or cross-device tracking. If you choose &ldquo;Essential only&rdquo; or close the banner without choosing, no Analytics script is loaded at all and no Analytics data is collected from you.</p>

            <h3>7. Cookies</h3>
            <ul class="list-disc pl-5 space-y-1.5" style="color: var(--muted);">
                <li><strong>Essential.</strong> Laravel session cookie for sign-in and checkout, an XSRF token for form security, and the consent cookie itself (<code class="font-mono px-1.5 py-0.5 rounded text-sm" style="background: var(--bg-2); color: var(--accent);">utterbar_consent</code>) so we don&rsquo;t re-prompt you on every page. These are set without consent because the site cannot work without them.</li>
                <li><strong>Optional &mdash; Analytics.</strong> Google Analytics sets <code class="font-mono px-1.5 py-0.5 rounded text-sm" style="background: var(--bg-2); color: var(--accent);">_ga</code> and <code class="font-mono px-1.5 py-0.5 rounded text-sm" style="background: var(--bg-2); color: var(--accent);">_ga_*</code> cookies (default lifetime ~13 months) only if you accept.</li>
            </ul>
            <p>You can revisit the choice any time via the <button type="button" onclick="utterbarRevisitCookieConsent(); return false;" style="color: var(--accent); text-decoration: underline; background: none; border: none; padding: 0; cursor: pointer; font: inherit;">Cookies</button> link in the footer.</p>

            <h3>8. Lawful basis for processing</h3>
            <ul class="list-disc pl-5 space-y-1.5" style="color: var(--muted);">
                <li><strong>Contract.</strong> Purchase records, licence keys, device activations &mdash; necessary to provide the product you bought.</li>
                <li><strong>Legitimate interests.</strong> Aggregate, hashed download counters; abuse prevention on the contact form; security logs.</li>
                <li><strong>Consent.</strong> Google Analytics and any future optional cookies. You can withdraw consent any time using the Cookies link in the footer.</li>
            </ul>

            <h3>9. Retention</h3>
            <ul class="list-disc pl-5 space-y-1.5" style="color: var(--muted);">
                <li><strong>Purchase + licence records.</strong> Kept for the lifetime of the product (so we can re-issue your key if you ask) and for any period required by UK tax law.</li>
                <li><strong>Activations.</strong> Until you deactivate the device or delete the licence.</li>
                <li><strong>Contact form messages.</strong> Up to 24 months, after which they&rsquo;re deleted unless they form part of an open support thread.</li>
                <li><strong>Download events.</strong> Indefinitely &mdash; they contain only hashes and country codes, no personal data.</li>
                <li><strong>Update subscribers.</strong> Until you unsubscribe, then deleted within 30 days.</li>
            </ul>

            <h3>10. Sharing</h3>
            <p>We don&rsquo;t sell or rent your data. The third parties involved in delivering the service are: <strong>Stripe</strong> (payment processing), <strong>Cloudflare</strong> (CDN, DDoS protection), <strong>Cloudways / DigitalOcean</strong> (hosting), <strong>Google</strong> (Analytics, only with your consent). Each is a data processor under UK GDPR and operates under their own published privacy notice.</p>

            <h3>11. Your rights (UK GDPR)</h3>
            <p>You have the right to access, correct, export, restrict the processing of, or delete any personal data we hold about you, and to object to processing based on legitimate interests. Email <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a> and we&rsquo;ll respond within 30 days. You can also complain to the UK Information Commissioner&rsquo;s Office at <a href="https://ico.org.uk/make-a-complaint/" style="color: var(--accent);">ico.org.uk/make-a-complaint</a>.</p>

            <h3>12. International transfers</h3>
            <p>Stripe and Google may process some data outside the UK. Both rely on the UK extension to the EU&ndash;US Data Privacy Framework or Standard Contractual Clauses to do so lawfully.</p>

            <h3>13. Changes</h3>
            <p>If we materially change this policy we&rsquo;ll update the &ldquo;Last updated&rdquo; date at the top and, where the change is significant, post a notice on the site for at least 30 days.</p>
        </article>
    </x-section>
</x-layouts.app>
