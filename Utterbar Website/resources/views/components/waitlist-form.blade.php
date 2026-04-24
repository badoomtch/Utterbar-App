@props(['variant' => 'inline', 'showUseCase' => false, 'id' => 'wl-' . uniqid()])

@php
    $already = request()->cookie('ub_waitlist') === '1' || session('waitlist_status');
    $message = session('waitlist_status');
    $success = session()->has('waitlist_status');
    $isAlready = session('waitlist_already', false);
@endphp

<div x-data="waitlistForm" class="w-full max-w-xl">
    <template x-if="submitted">
        <div class="ub-card text-center" role="status" aria-live="polite" style="padding: 2.25rem 2rem;">
            <div class="flex flex-col items-center">
                {{-- Circular cyan badge with check --}}
                <span class="inline-flex items-center justify-center rounded-full mb-5"
                      style="width: 48px; height: 48px;
                             background: color-mix(in oklab, var(--accent) 14%, transparent);
                             color: var(--accent);
                             border: 1px solid color-mix(in oklab, var(--accent) 45%, transparent);
                             box-shadow: 0 0 24px -4px color-mix(in oklab, var(--accent) 50%, transparent);">
                    <x-icon name="check" size="22" stroke="2.6" />
                </span>
                <p class="font-semibold" style="color: var(--fg-hi); font-size: 1.125rem; letter-spacing: -0.015em;" x-text="message"></p>
                <p class="text-sm mt-2" style="color: var(--muted); max-width: 32ch;">
                    We never spam and there&rsquo;s no unsubscribe dark-pattern.
                </p>
            </div>
        </div>
    </template>

    <template x-if="!submitted">
        <form @submit.prevent="submit" action="{{ route('waitlist.store') }}" method="POST" novalidate>
            @csrf

            {{-- honeypot --}}
            <div aria-hidden="true" style="position: absolute; left: -10000px; top: auto; width: 1px; height: 1px; overflow: hidden;">
                <label for="{{ $id }}-hp">Don't fill this</label>
                <input id="{{ $id }}-hp" name="hp" type="text" tabindex="-1" autocomplete="off">
            </div>

            <div class="flex flex-col sm:flex-row gap-2">
                <label for="{{ $id }}-email" class="sr-only">Email</label>
                <input id="{{ $id }}-email" name="email" type="email" required autocomplete="email" placeholder="you@example.com"
                       x-model="email"
                       class="flex-1 rounded-xl px-4 py-3 font-medium"
                       style="background: var(--bg-1); border: 1px solid var(--border); color: var(--fg); outline-color: var(--accent);">
                <button type="submit" :disabled="busy" class="ub-btn ub-btn-cta shrink-0">
                    <span x-show="!busy">Join</span>
                    <span x-show="busy">…</span>
                </button>
            </div>

            @if ($showUseCase)
                <div class="mt-3">
                    <label for="{{ $id }}-use" class="sr-only">Use case (optional)</label>
                    <input id="{{ $id }}-use" name="use_case" type="text" maxlength="280"
                           x-model="useCase"
                           placeholder="What are you going to use Utterbar for? (optional)"
                           class="w-full rounded-xl px-4 py-3 text-sm"
                           style="background: var(--bg-1); border: 1px solid var(--border); color: var(--fg);">
                </div>
            @endif

            <label class="mt-4 flex items-start gap-2.5 text-xs cursor-pointer" style="color: var(--muted);">
                <input type="checkbox" name="consent" value="1" x-model="consent" required
                       class="mt-0.5 shrink-0 w-4 h-4 rounded cursor-pointer"
                       style="accent-color: var(--accent);">
                <span>I'm happy to receive one confirmation email and the occasional update. See the <a href="{{ route('legal.privacy') }}" style="color: var(--accent);">privacy policy</a>.</span>
            </label>

            <p x-show="error" x-text="error" role="alert" aria-live="polite" class="mt-3 text-sm" style="color: var(--danger);"></p>
        </form>
    </template>
</div>

@once
@push('scripts')
<script>
    document.addEventListener('alpine:init', () => {
        window.Alpine.data('waitlistForm', () => ({
            email: '',
            useCase: '',
            consent: false,
            busy: false,
            submitted: @json($success || $already),
            message: @json($message ?? ($already ? "You're already on the list." : '')),
            error: '',
            async submit(e) {
                this.error = '';
                if (!this.email) { this.error = 'Email is required.'; return; }
                if (!this.consent) { this.error = 'Tick the consent box so we can email you the confirmation.'; return; }
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
                    this.message = data.message || "You're in. We'll email once there's a build worth installing.";
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
@endonce
