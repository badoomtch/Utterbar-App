<x-layouts.app title="Terms of service" description="The terms covering your use of utterbar.com and the free, open-source Utterbar desktop app.">
    <x-section eyebrow="Legal" title="Terms of service">
        <article class="max-w-3xl space-y-5 leading-relaxed" style="color: var(--fg);">
            <p class="font-mono text-sm" style="color: var(--muted);">Last updated: 24 July 2026</p>

            <p style="color: var(--muted);">
                These terms apply to your use of <strong>utterbar.com</strong> and the Utterbar desktop app. The app is free and open source. It is additionally covered by the <a href="{{ route('legal.eula') }}" style="color: var(--accent);">End-User Licence Agreement</a> bundled with the installer and by the open-source licence included with its source code.
            </p>

            <h3>1. Who you&rsquo;re contracting with</h3>
            <p>Utterbar is operated by Jay Parry, trading as Utterbar, based in the United Kingdom. References to &ldquo;we&rdquo;, &ldquo;us&rdquo;, &ldquo;our&rdquo; mean Utterbar; references to &ldquo;you&rdquo; mean the person using the site or the app.</p>

            <h3>2. Using the website</h3>
            <p>By using utterbar.com you agree to act in good faith, not attempt to disrupt or probe our infrastructure, and not scrape or republish our content. The site is provided as-is for personal and business reading; the app is the product.</p>

            <h3>3. The software is free</h3>
            <p>Utterbar is free of charge and open source. There is no purchase, no subscription, no account, and no licence key. You may download, install, and run it on as many devices as you like, for personal or commercial use. Your use of the source code is governed by the open-source licence included with it.</p>

            <h3>4. Updates</h3>
            <p>We may issue updates to Utterbar at our discretion. Updates may add, change, or remove features. Updates are delivered through the in-app updater and are optional; the app continues to work without them.</p>

            <h3>5. Acceptable use</h3>
            <p>You will not use Utterbar to dictate or transcribe content you do not have the legal right to record (for example, calls or meetings where applicable consent has not been obtained). You are responsible for ensuring your use complies with the laws of your jurisdiction.</p>

            <h3>6. Availability and support</h3>
            <p>We aim to keep utterbar.com and the update feed reachable, but we do not guarantee uptime. The desktop app does not require our servers to operate day-to-day: everything runs on your machine.</p>
            <p>Support is best-effort via the <a href="{{ route('contact') }}" style="color: var(--accent);">contact form</a> or email, and via the public issue tracker for the open-source project. We aim to respond within five business days.</p>

            <h3>7. Liability</h3>
            <p>Nothing in these terms limits or excludes our liability for death or personal injury caused by negligence, fraud, or any other liability that cannot lawfully be excluded under English law.</p>
            <p>Subject to that, and because Utterbar is provided free of charge, to the fullest extent permitted by law our total liability to you for any claim arising out of or in connection with these terms is limited to &pound;50. We are not liable for indirect, incidental, or consequential losses (including lost profits, lost data, or business interruption).</p>

            <h3>8. Intellectual property</h3>
            <p>Utterbar, the wordmark, and the logo are the intellectual property of Jay Parry. The Utterbar source code is made available under the open-source licence included with it. The Parakeet ASR model is licensed from NVIDIA under its own model licence and is not Utterbar IP.</p>

            <h3>9. Termination</h3>
            <p>You may stop using Utterbar at any time. Because the app is free and runs entirely on your machine, there is nothing for us to terminate; these website terms simply cease to apply to you when you stop using the site.</p>

            <h3>10. Governing law and jurisdiction</h3>
            <p>These terms are governed by the laws of England and Wales. Any dispute is subject to the exclusive jurisdiction of the courts of England and Wales, except that consumers domiciled elsewhere in the UK may bring proceedings in their home courts where applicable mandatory law allows.</p>

            <h3>11. Changes</h3>
            <p>If we change these terms materially, we&rsquo;ll update the &ldquo;Last updated&rdquo; date at the top and post a notice on the site for at least 30 days. Continued use of the site or the app after a material change constitutes acceptance.</p>

            <h3>12. Contact</h3>
            <p>Questions about these terms: <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a>.</p>
        </article>
    </x-section>
</x-layouts.app>
