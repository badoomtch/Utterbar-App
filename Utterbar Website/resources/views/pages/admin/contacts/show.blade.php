<x-layouts.app title="Admin · Contact message" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-3xl mx-auto">

                <x-admin-nav active="contacts" />

                <div class="mb-6">
                    <p class="ub-eyebrow">Contact message</p>
                    <h1 class="ub-fg-hi mt-2" style="font-size: clamp(1.5rem, 3vw, 2rem); letter-spacing: -0.025em; line-height: 1.15;">
                        {{ $message->subject ?: '(no subject)' }}
                    </h1>
                    <p class="mt-3 text-sm" style="color: var(--muted);">
                        <a href="{{ route('admin.contacts.index') }}" style="color: var(--accent);">&larr; all messages</a>
                    </p>
                </div>

                <div class="ub-card mb-6" style="padding: 1.5rem 1.75rem;">
                    <dl class="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3 text-sm">
                        <div>
                            <dt class="ub-eyebrow mb-1">From</dt>
                            <dd style="color: var(--fg-hi);">
                                {{ $message->name ?: '—' }}
                                <span style="color: var(--muted);">&lt;{{ $message->email }}&gt;</span>
                            </dd>
                        </div>
                        <div>
                            <dt class="ub-eyebrow mb-1">Received</dt>
                            <dd style="color: var(--muted);">{{ $message->created_at->format('d M Y H:i:s') }}</dd>
                        </div>
                        @if ($message->ip)
                            <div>
                                <dt class="ub-eyebrow mb-1">IP</dt>
                                <dd class="font-mono text-xs" style="color: var(--muted);">{{ $message->ip }}</dd>
                            </div>
                        @endif
                        @if ($message->emailed_at)
                            <div>
                                <dt class="ub-eyebrow mb-1">Mirrored to inbox</dt>
                                <dd style="color: var(--muted);">{{ \Illuminate\Support\Carbon::parse($message->emailed_at)->format('d M Y H:i') }}</dd>
                            </div>
                        @endif
                    </dl>
                </div>

                <div class="ub-card" style="padding: 1.5rem 1.75rem;">
                    <p class="ub-eyebrow mb-3">Message</p>
                    <pre class="text-sm whitespace-pre-wrap" style="color: var(--fg); line-height: 1.6; font-family: inherit;">{{ $message->message }}</pre>
                </div>

                <div class="mt-6">
                    <a href="mailto:{{ $message->email }}?subject=Re: {{ urlencode($message->subject ?: 'your message') }}"
                       class="ub-btn ub-btn-cta">Reply by email</a>
                </div>

            </div>
        </div>
    </section>
</x-layouts.app>
