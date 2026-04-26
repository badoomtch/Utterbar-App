@php
    $alreadySent = session('contact_status');
    $statusMessage = session('contact_status');
    $subjectPrefill = request()->query('subject', '');
@endphp

<x-layouts.app title="Contact" description="Get in touch with Jay at hello@utterbar.com. Questions, feature ideas, bug reports, partnership pitches — one inbox, one person, usually a reply within a day.">
    <section class="relative overflow-hidden">
        <div class="ub-hero-glow" aria-hidden="true"></div>
        <div class="relative ub-container pt-20 md:pt-28 pb-12">
            <div class="max-w-4xl">
                <h1 class="ub-fg-hi" style="font-size: clamp(2.75rem, 6vw, 5rem); letter-spacing: -0.04em; line-height: 1.05;">
                    Drop me <x-hl>a line</x-hl>.
                </h1>
                <p class="ub-lead mt-8 max-w-xl">
                    Questions, feature ideas, bug reports, partnership pitches. One inbox, one person, usually a reply within a day or two.
                </p>
            </div>
        </div>
    </section>

    <section class="pb-20 md:pb-28">
        <div class="ub-container">
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-10 lg:gap-16 max-w-5xl">

                {{-- Form --}}
                <div class="lg:col-span-7">
                    <div x-data="contactForm()">
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
                                    <p class="text-sm mt-2" style="color: var(--muted); max-width: 36ch;">
                                        I&rsquo;ll reply from <span style="color: var(--accent);">hello@utterbar.com</span>. Check junk if it takes more than a day.
                                    </p>
                                </div>
                            </div>
                        </template>

                        <template x-if="!submitted">
                            <form @submit.prevent="submit" action="{{ route('contact.store') }}" method="POST" novalidate>
                                @csrf

                                {{-- Honeypot --}}
                                <div aria-hidden="true" style="position: absolute; left: -10000px; top: auto; width: 1px; height: 1px; overflow: hidden;">
                                    <label for="contact-hp">Don't fill this</label>
                                    <input id="contact-hp" name="hp" type="text" tabindex="-1" autocomplete="off">
                                </div>

                                <div class="space-y-5">
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <div>
                                            <label for="c-name" class="ub-eyebrow mb-2 block">Name <span style="color: var(--fg-dim); text-transform: none; letter-spacing: 0; font-weight: 400;">(optional)</span></label>
                                            <input id="c-name" name="name" type="text" maxlength="120" autocomplete="name"
                                                   x-model="name"
                                                   class="w-full rounded-xl px-4 py-3"
                                                   style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                                        </div>
                                        <div>
                                            <label for="c-email" class="ub-eyebrow mb-2 block">Email</label>
                                            <input id="c-email" name="email" type="email" required maxlength="255" autocomplete="email"
                                                   x-model="email"
                                                   placeholder="you@example.com"
                                                   class="w-full rounded-xl px-4 py-3"
                                                   style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                                        </div>
                                    </div>

                                    <div>
                                        <label for="c-subject" class="ub-eyebrow mb-2 block">Subject <span style="color: var(--fg-dim); text-transform: none; letter-spacing: 0; font-weight: 400;">(optional)</span></label>
                                        <input id="c-subject" name="subject" type="text" maxlength="200"
                                               x-model="subject"
                                               placeholder="What&rsquo;s this about?"
                                               class="w-full rounded-xl px-4 py-3"
                                               style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                                    </div>

                                    <div>
                                        <label for="c-message" class="ub-eyebrow mb-2 block">Message</label>
                                        <textarea id="c-message" name="message" required minlength="5" maxlength="5000" rows="7"
                                                  x-model="body"
                                                  placeholder="Write whatever you need to&hellip;"
                                                  class="w-full rounded-xl px-4 py-3 resize-y"
                                                  style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; line-height: 1.6; outline-color: var(--accent); min-height: 180px;"></textarea>
                                    </div>

                                    <label class="flex items-start gap-2.5 text-xs cursor-pointer" style="color: var(--muted);">
                                        <input type="checkbox" name="consent" value="1" x-model="consent" required
                                               class="mt-0.5 shrink-0 w-4 h-4 rounded cursor-pointer"
                                               style="accent-color: var(--accent);">
                                        <span>I&rsquo;m happy for Jay to reply by email and store this message. See the <a href="{{ route('legal.privacy') }}" style="color: var(--accent);">privacy policy</a>.</span>
                                    </label>

                                    <p x-show="error" x-text="error" role="alert" aria-live="polite" class="text-sm" style="color: var(--danger);"></p>

                                    <div class="pt-2">
                                        <button type="submit" :disabled="busy" class="ub-btn ub-btn-cta">
                                            <span x-show="!busy">Send message</span>
                                            <span x-show="busy">Sending&hellip;</span>
                                        </button>
                                    </div>
                                </div>
                            </form>
                        </template>
                    </div>
                </div>

                {{-- Aside: direct email + quick links --}}
                <aside class="lg:col-span-5">
                    <div class="ub-card ub-card-rail" style="padding: 2rem;">
                        <p class="ub-eyebrow">Prefer email?</p>
                        <p class="mt-4 text-base" style="color: var(--fg); line-height: 1.6;">
                            Write directly to
                            <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent); text-decoration: none; border-bottom: 1px solid color-mix(in oklab, var(--accent) 40%, transparent);">
                                {{ config('site.email.contact') }}
                            </a>.
                            It&rsquo;s the same inbox either way.
                        </p>
                    </div>

                    <div class="mt-5 ub-card" style="padding: 1.75rem;">
                        <p class="ub-eyebrow">Before you write</p>
                        <ul class="mt-4 space-y-3">
                            @foreach ([
                                ['Looking for the pricing?',  route('pricing')],
                                ['Want to download?',          route('download')],
                                ['Privacy questions?',         route('privacy')],
                                ['Press enquiry?',             route('press')],
                            ] as [$q, $href])
                                <li>
                                    <a href="{{ $href }}" class="ub-footer-link" style="color: var(--fg);">{{ $q }}</a>
                                </li>
                            @endforeach
                        </ul>
                    </div>
                </aside>

            </div>
        </div>
    </section>

    @push('scripts')
    <script>
        document.addEventListener('alpine:init', () => {
            window.Alpine.data('contactForm', () => ({
                name: '',
                email: '',
                subject: @json($subjectPrefill),
                body: '',
                consent: false,
                busy: false,
                submitted: @json((bool) $alreadySent),
                message: @json($statusMessage ?? "Got it. I'll reply from hello@utterbar.com within a day or two."),
                error: '',
                async submit(e) {
                    this.error = '';
                    if (!this.email) { this.error = 'Email is required.'; return; }
                    if (!this.body || this.body.trim().length < 5) { this.error = 'Write a bit more than that.'; return; }
                    if (!this.consent) { this.error = 'Tick the consent box so I know a reply is OK.'; return; }
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
