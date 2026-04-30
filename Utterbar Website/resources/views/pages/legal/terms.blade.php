<x-layouts.app title="Terms of service" description="The terms covering your use of utterbar.com and your purchase of an Utterbar Pro licence.">
    <x-section eyebrow="Legal" title="Terms of service">
        <article class="max-w-3xl space-y-5 leading-relaxed" style="color: var(--fg);">
            <p class="font-mono text-sm" style="color: var(--muted);">Last updated: 26 April 2026</p>

            <p style="color: var(--muted);">
                These terms apply to your use of <strong>utterbar.com</strong> and to any purchase of an Utterbar Pro licence. The desktop app is additionally covered by the <a href="{{ route('legal.eula') }}" style="color: var(--accent);">End-User Licence Agreement</a> bundled with the installer.
            </p>

            <h3>1. Who you&rsquo;re contracting with</h3>
            <p>Utterbar is operated by Jay Parry, trading as Utterbar, based in the United Kingdom. References to &ldquo;we&rdquo;, &ldquo;us&rdquo;, &ldquo;our&rdquo; mean Utterbar; references to &ldquo;you&rdquo; mean the person using the site or buying a licence.</p>

            <h3>2. Using the website</h3>
            <p>By using utterbar.com you agree to act in good faith, not attempt to disrupt or probe our infrastructure, and not scrape or republish our content. The site is provided as-is for personal and business reading; the app is the product.</p>

            <h3>3. Buying Utterbar Pro</h3>
            <p>Pro is a one-time purchase of <strong>&pound;39.99 GBP</strong>, inclusive of any UK VAT we&rsquo;re required to charge. There is no subscription. Payment is processed by <a href="https://stripe.com/gb/legal/consumer" style="color: var(--accent);">Stripe</a>; the contract for the licence is between you and Utterbar.</p>
            <p>Once we&rsquo;ve received your payment, we email you a licence key. The key activates Utterbar Pro on up to three devices simultaneously; you can transfer it between machines from your <a href="{{ route('account') }}" style="color: var(--accent);">account page</a>.</p>

            <h3>4. Refunds (UK Consumer Contracts Regulations 2013)</h3>
            <p>Because Utterbar Pro is digital content delivered immediately on purchase, you have <strong>14 calendar days</strong> from the date of purchase to request a refund, provided you have not started a meaningful amount of use. Email <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a> with your licence key or order email and we&rsquo;ll process the refund. We honour refund requests inside the 14-day window without question; outside the window we&rsquo;ll consider individual cases on the merits.</p>
            <p>If your refund is granted, we revoke the licence key and any active device activations.</p>

            <h3>5. Licence grant</h3>
            <p>Subject to payment in full and your continued compliance with these terms and the EULA, we grant you a non-exclusive, non-transferable (except by you between your own devices via the account page), non-sub-licensable, perpetual licence to install and run Utterbar on up to three devices for personal or commercial use.</p>
            <p>You may not redistribute the installer, share your licence key, or attempt to remove or bypass licence verification. Doing so will result in revocation of the licence without refund.</p>

            <h3>6. Updates</h3>
            <p>We may issue updates to Utterbar at our discretion. Updates may add, change, or remove features. Pro features that ship in v1.x will continue to work in any v1.x update; if we ever release a v2.0 with materially different licensing, your existing v1.x licence will continue to work for the v1.x line.</p>

            <h3>7. Acceptable use</h3>
            <p>You will not use Utterbar to dictate or transcribe content you do not have the legal right to record (for example, calls or meetings where applicable consent has not been obtained). You are responsible for ensuring your use complies with the laws of your jurisdiction.</p>

            <h3>8. Availability and support</h3>
            <p>We aim to keep utterbar.com and the licence APIs reachable, but we do not guarantee uptime. The desktop app does not require our servers to operate day-to-day. Once a licence is activated, it verifies locally on every launch.</p>
            <p>Support is best-effort via the <a href="{{ route('contact') }}" style="color: var(--accent);">contact form</a> or email. We aim to respond within five business days.</p>

            <h3>9. Liability</h3>
            <p>Nothing in these terms limits or excludes our liability for death or personal injury caused by negligence, fraud, or any other liability that cannot lawfully be excluded under English law.</p>
            <p>Subject to that, our total liability to you for any claim arising out of or in connection with these terms is limited to the amount you paid for the licence in the 12 months preceding the event giving rise to the claim. We are not liable for indirect, incidental, or consequential losses (including lost profits, lost data, or business interruption).</p>

            <h3>10. Intellectual property</h3>
            <p>Utterbar, the wordmark, and the logo are the intellectual property of Jay Parry. The Parakeet ASR model is licensed from NVIDIA under its own model licence and is not Utterbar IP.</p>

            <h3>11. Termination</h3>
            <p>You may stop using Utterbar at any time. We may suspend or terminate your licence if you breach these terms, the EULA, or use Utterbar to facilitate clearly unlawful activity. Termination for cause does not entitle you to a refund.</p>

            <h3>12. Governing law and jurisdiction</h3>
            <p>These terms are governed by the laws of England and Wales. Any dispute is subject to the exclusive jurisdiction of the courts of England and Wales, except that consumers domiciled elsewhere in the UK may bring proceedings in their home courts where applicable mandatory law allows.</p>

            <h3>13. Changes</h3>
            <p>If we change these terms materially, we&rsquo;ll update the &ldquo;Last updated&rdquo; date at the top and post a notice on the site for at least 30 days. Continued use of the site or the app after a material change constitutes acceptance.</p>

            <h3>14. Contact</h3>
            <p>Questions about these terms: <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a>.</p>
        </article>
    </x-section>
</x-layouts.app>
