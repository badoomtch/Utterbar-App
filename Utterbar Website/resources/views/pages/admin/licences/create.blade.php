<x-layouts.app title="Admin · Mint licence" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-4xl mx-auto">

                <x-admin-nav active="licences" />

                <div class="mb-8">
                    <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.05;">
                        Mint licence.
                    </h1>
                    <p class="mt-3 text-sm" style="color: var(--muted);">
                        Issues a licence without a Stripe session. Use for comp copies, press keys, support re-issues.
                    </p>
                </div>

                @if ($errors->any())
                    <div class="ub-card mb-6" style="padding: 1rem 1.25rem; border-color: var(--danger);">
                        <ul class="text-sm" style="color: var(--danger);">
                            @foreach ($errors->all() as $msg)
                                <li>{{ $msg }}</li>
                            @endforeach
                        </ul>
                    </div>
                @endif

                <form method="POST" action="{{ route('admin.licences.store') }}" class="ub-card space-y-5" style="padding: 2rem;">
                    @csrf

                    <div>
                        <label for="l-email" class="ub-eyebrow mb-2 block">Email</label>
                        <input id="l-email" name="email" type="email" required autofocus
                               value="{{ old('email', $default['email']) }}"
                               placeholder="customer@example.com"
                               class="w-full px-3 py-2 rounded-lg text-sm"
                               style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label for="l-sku" class="ub-eyebrow mb-2 block">SKU</label>
                            <input id="l-sku" name="sku" type="text" required
                                   value="{{ old('sku', $default['sku']) }}"
                                   class="w-full px-3 py-2 rounded-lg text-sm font-mono"
                                   style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                        </div>
                        <div>
                            <label for="l-max" class="ub-eyebrow mb-2 block">Max devices</label>
                            <input id="l-max" name="max_devices" type="number" min="1" max="50" required
                                   value="{{ old('max_devices', $default['max_devices']) }}"
                                   class="w-full px-3 py-2 rounded-lg text-sm"
                                   style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                        </div>
                    </div>

                    <div>
                        <label for="l-note" class="ub-eyebrow mb-2 block">Internal note (optional)</label>
                        <input id="l-note" name="note" type="text" maxlength="500"
                               value="{{ old('note', $default['note']) }}"
                               placeholder="e.g. 'comp for journalist X' or 'replacement for lost email'"
                               class="w-full px-3 py-2 rounded-lg text-sm"
                               style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                        <p class="mt-2 text-xs" style="color: var(--muted);">Logged to the application log only; not sent to the customer.</p>
                    </div>

                    <label class="flex items-start gap-3 cursor-pointer">
                        <input type="checkbox" name="send_email" value="1" {{ old('send_email', $default['send_email']) ? 'checked' : '' }}
                               style="accent-color: var(--accent);" class="mt-1 shrink-0 w-4 h-4 rounded cursor-pointer">
                        <span class="text-sm" style="color: var(--fg);">
                            Send the licence email to the customer.
                            <span style="color: var(--muted);">Uses the same template as a normal purchase.</span>
                        </span>
                    </label>

                    <label class="flex items-start gap-3 cursor-pointer">
                        <input type="checkbox" name="confirm_duplicate" value="1"
                               style="accent-color: var(--accent);" class="mt-1 shrink-0 w-4 h-4 rounded cursor-pointer">
                        <span class="text-sm" style="color: var(--fg);">
                            Confirm duplicate.
                            <span style="color: var(--muted);">Tick if the email already has an active licence and you definitely want a second one.</span>
                        </span>
                    </label>

                    <div class="flex items-center justify-end gap-3 pt-2">
                        <a href="{{ route('admin.licences.index') }}" class="ub-btn ub-btn-ghost">Cancel</a>
                        <button type="submit" class="ub-btn ub-btn-cta">Mint licence</button>
                    </div>
                </form>

            </div>
        </div>
    </section>
</x-layouts.app>
