@php
    $alreadySent  = session('business_status');
    $statusMessage = session('business_status');
@endphp

<x-layouts.app
    title="Get a business quote"
    description="Tell us seat count, use case, and any deployment requirements. You get a quote and the IT deployment notes back within one working day.">

    <section class="relative overflow-hidden">
        <div class="ub-hero-glow" aria-hidden="true"></div>
        <div class="relative ub-container pt-20 md:pt-28 pb-12">
            <div class="max-w-4xl">
                <p class="ub-eyebrow">For business</p>
                <h1 class="ub-fg-hi mt-4" style="font-size: clamp(2.5rem, 5.5vw, 4.25rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Get a <x-hl>quote</x-hl>.
                </h1>
                <p class="ub-lead mt-8 max-w-xl">
                    The form is short on purpose. Seat count, use case, anything specific your IT team needs. Reply within one working day.
                </p>
            </div>
        </div>
    </section>

    <section class="pb-20 md:pb-28">
        <div class="ub-container">
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-10 lg:gap-16 max-w-5xl">

                <div class="lg:col-span-7">
                    <div x-data="businessForm()">
                        <template x-if="submitted">
                            <div class="ub-card text-center" role="status" aria-live="polite" style="padding: 2.25rem 2rem;">
                                <div class="flex flex-col items-center">
                                    <span class="inline-flex items-center justify-center rounded-full mb-5"
                                          style="width: 48px; height: 48px;
                                                 background: color-mix(in oklab, var(--accent) 14%, transparent);
                                                 color: var(--accent);
                                                 border: 1px solid color-mix(in oklab, var(--accent) 45%, transparent);
                                                 box-shadow: 0 0 24px -4px color-mix(in oklab, var(--accent) 50%, transparent);">
                                        <x-icon name="check" size="22" stroke="2.6" />
                                    </span>
                                    <p class="font-semibold" style="color: var(--fg-hi); font-size: 1.125rem; letter-spacing: -0.015em;" x-text="message"></p>
                                    <p class="text-sm mt-2" style="color: var(--muted); max-width: 42ch;">
                                        Reply will come from <span style="color: var(--accent);">{{ config('site.email.contact') }}</span>. Check junk if it takes more than a working day.
                                    </p>
                                </div>
                            </div>
                        </template>

                        <template x-if="!submitted">
                            <form @submit.prevent="submit" action="{{ route('business.store') }}" method="POST" novalidate>
                                @csrf

                                {{-- Honeypot --}}
                                <div aria-hidden="true" style="position: absolute; left: -10000px; top: auto; width: 1px; height: 1px; overflow: hidden;">
                                    <label for="b-hp">Don't fill this</label>
                                    <input id="b-hp" name="hp" type="text" tabindex="-1" autocomplete="off">
                                </div>

                                <div class="space-y-5">
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <div>
                                            <label for="b-name" class="ub-eyebrow mb-2 block">Your name</label>
                                            <input id="b-name" name="name" type="text" required maxlength="120" autocomplete="name"
                                                   x-model="name"
                                                   class="w-full rounded-xl px-4 py-3"
                                                   style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                                        </div>
                                        <div>
                                            <label for="b-email" class="ub-eyebrow mb-2 block">Work email</label>
                                            <input id="b-email" name="email" type="email" required maxlength="191" autocomplete="email"
                                                   x-model="email"
                                                   placeholder="you@company.com"
                                                   class="w-full rounded-xl px-4 py-3"
                                                   style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                                        </div>
                                    </div>

                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <div>
                                            <label for="b-company" class="ub-eyebrow mb-2 block">Company</label>
                                            <input id="b-company" name="company" type="text" required maxlength="160" autocomplete="organization"
                                                   x-model="company"
                                                   class="w-full rounded-xl px-4 py-3"
                                                   style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                                        </div>
                                        <div>
                                            <label for="b-role" class="ub-eyebrow mb-2 block">Your role <span style="color: var(--fg-dim); text-transform: none; letter-spacing: 0; font-weight: 400;">(optional)</span></label>
                                            <input id="b-role" name="role" type="text" maxlength="120" autocomplete="organization-title"
                                                   x-model="role"
                                                   placeholder="e.g. IT Manager, Head of Operations"
                                                   class="w-full rounded-xl px-4 py-3"
                                                   style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                                        </div>
                                    </div>

                                    <div>
                                        <label for="b-seats" class="ub-eyebrow mb-2 block">Approximate seat count</label>
                                        <input id="b-seats" name="seat_count" type="number" min="1" max="100000" required
                                               x-model="seatCount"
                                               placeholder="e.g. 50"
                                               class="w-full rounded-xl px-4 py-3"
                                               style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                                        <p class="text-xs mt-2" style="color: var(--muted);">Rough number is fine. We'll firm it up in the quote.</p>
                                    </div>

                                    <div>
                                        <label for="b-use-case" class="ub-eyebrow mb-2 block">Use case</label>
                                        <textarea id="b-use-case" name="use_case" required minlength="5" maxlength="2000" rows="4"
                                                  x-model="useCase"
                                                  placeholder="What does your team dictate? Case notes, tickets, meetings, something else?"
                                                  class="w-full rounded-xl px-4 py-3"
                                                  style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent); resize: vertical;"></textarea>
                                    </div>

                                    <div>
                                        <label class="ub-eyebrow mb-3 block">Anything specific?</label>
                                        <p class="text-sm mb-3" style="color: var(--muted);">Tick what applies. Leave the rest. We'll cover anything you tick in the reply.</p>
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                                            @foreach ($requirementOptions as $key => $label)
                                                <label class="flex items-start gap-2.5 cursor-pointer rounded-lg p-2.5"
                                                       style="border: 1px solid var(--border); background: var(--bg-0);">
                                                    <input type="checkbox" name="requirements[]" value="{{ $key }}"
                                                           x-model="requirements"
                                                           class="mt-0.5 shrink-0"
                                                           style="accent-color: var(--accent);">
                                                    <span class="text-sm" style="color: var(--fg);">{{ $label }}</span>
                                                </label>
                                            @endforeach
                                        </div>
                                    </div>

                                    <div>
                                        <label for="b-message" class="ub-eyebrow mb-2 block">Anything else <span style="color: var(--fg-dim); text-transform: none; letter-spacing: 0; font-weight: 400;">(optional)</span></label>
                                        <textarea id="b-message" name="message" maxlength="5000" rows="3"
                                                  x-model="message"
                                                  placeholder="Compliance constraints, deployment timeline, specific questions."
                                                  class="w-full rounded-xl px-4 py-3"
                                                  style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent); resize: vertical;"></textarea>
                                    </div>

                                    <label class="flex items-start gap-3 cursor-pointer">
                                        <input type="checkbox" name="consent" required
                                               x-model="consent"
                                               class="mt-0.5 shrink-0"
                                               style="accent-color: var(--accent);">
                                        <span class="text-sm" style="color: var(--muted); line-height: 1.55;">
                                            OK to reply to this email about Utterbar. We don't add you to any list and we don't share the address.
                                        </span>
                                    </label>

                                    <p x-show="error" x-text="error" role="alert" aria-live="polite" class="text-sm" style="color: var(--danger);"></p>

                                    <div class="pt-2">
                                        <button type="submit" :disabled="busy" class="ub-btn ub-btn-cta">
                                            <span x-show="!busy">Send enquiry</span>
                                            <span x-show="busy">Sending&hellip;</span>
                                        </button>
                                    </div>
                                </div>
                            </form>
                        </template>
                    </div>
                </div>

                {{-- Aside: pricing summary + IT deployment link --}}
                <aside class="lg:col-span-5">
                    <div class="ub-card ub-card-rail" style="padding: 2rem;">
                        <p class="ub-eyebrow">Pricing summary</p>
                        <ul class="mt-4 space-y-3 text-sm" style="color: var(--fg);">
                            <li><strong style="color: var(--fg-hi);">Team (3&ndash;24 seats)</strong>: &pound;29 per seat, lifetime.</li>
                            <li><strong style="color: var(--fg-hi);">Business (25&ndash;249 seats)</strong>: &pound;19 per seat, lifetime.</li>
                            <li><strong style="color: var(--fg-hi);">Enterprise (250+ seats)</strong>: custom quote.</li>
                        </ul>
                        <p class="mt-5 text-sm" style="color: var(--muted); line-height: 1.6;">
                            All tiers are one-time licences. No subscription, no annual renewal. Pay by card or invoice.
                        </p>
                    </div>

                    <div class="mt-5 ub-card" style="padding: 1.75rem;">
                        <p class="ub-eyebrow">For your IT team</p>
                        <p class="mt-4 text-sm" style="color: var(--fg); line-height: 1.6;">
                            The full deployment notes (silent install, network egress, group policy, air-gapped activation) are at
                            <a href="{{ route('docs.it-deployment') }}" style="color: var(--accent);">docs / IT deployment</a>.
                        </p>
                    </div>

                    <div class="mt-5 ub-card" style="padding: 1.75rem;">
                        <p class="ub-eyebrow">Prefer email?</p>
                        <p class="mt-4 text-sm" style="color: var(--fg); line-height: 1.6;">
                            <a href="mailto:{{ config('site.email.contact') }}?subject=Business%20enquiry"
                               style="color: var(--accent); text-decoration: none; border-bottom: 1px solid color-mix(in oklab, var(--accent) 40%, transparent);">
                                {{ config('site.email.contact') }}
                            </a>
                        </p>
                    </div>
                </aside>
            </div>
        </div>
    </section>

    @push('scripts')
    <script>
        document.addEventListener('alpine:init', () => {
            window.Alpine.data('businessForm', () => ({
                name: '',
                email: '',
                company: '',
                role: '',
                seatCount: '',
                useCase: '',
                requirements: [],
                message: '',
                consent: false,
                busy: false,
                submitted: @json((bool) $alreadySent),
                message: @json($statusMessage ?? "Got it. I'll get back to you within one working day with a quote and the deployment notes for your environment."),
                error: '',
                async submit(e) {
                    this.error = '';
                    if (!this.name) { this.error = 'Your name, please.'; return; }
                    if (!this.email) { this.error = 'Work email is required.'; return; }
                    if (!this.company) { this.error = 'Company name, please.'; return; }
                    if (!this.seatCount || Number(this.seatCount) < 1) { this.error = 'Roughly how many seats?'; return; }
                    if (!this.useCase || this.useCase.trim().length < 5) { this.error = 'A sentence or two on the use case.'; return; }
                    if (!this.consent) { this.error = 'Tick the consent box so a reply is OK.'; return; }
                    this.busy = true;
                    try {
                        const form = e.target;
                        const fd = new FormData(form);
                        const res = await fetch(form.action, {
                            method: 'POST',
                            headers: { 'Accept': 'application/json', 'X-Requested-With': 'XMLHttpRequest' },
                            body: fd,
                            credentials: 'same-origin',
                        });
                        const data = await res.json().catch(() => ({}));
                        if (!res.ok) {
                            this.error = data.message || (data.errors ? Object.values(data.errors).flat()[0] : 'Something went wrong. Try again.');
                            return;
                        }
                        this.message = data.message || this.message;
                        this.submitted = true;
                    } catch (err) {
                        this.error = 'Network error. Try again.';
                    } finally {
                        this.busy = false;
                    }
                }
            }));
        });
    </script>
    @endpush
</x-layouts.app>
