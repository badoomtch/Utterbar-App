<x-layouts.app title="Admin · Contacts" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto">

                <x-admin-nav active="contacts" />

                <div class="mb-8">
                    <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.05;">
                        Contact messages.
                    </h1>
                    <p class="mt-3 text-sm" style="color: var(--muted);">
                        Everything submitted via the contact form. Also mirrored to email at submission time.
                    </p>
                </div>

                <form method="GET" action="{{ route('admin.contacts.index') }}" class="mb-6">
                    <input type="text" name="q" value="{{ $q }}" placeholder="Search by email, name, subject, or body..."
                           class="w-full px-3 py-2 rounded-lg text-sm"
                           style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                </form>

                <div class="ub-card" style="padding: 0; overflow: hidden;">
                    @if ($messages->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">No messages match.</p>
                    @else
                        <table class="w-full text-sm" style="border-collapse: collapse;">
                            <thead>
                                <tr style="background: var(--bg-0); color: var(--muted); text-align: left;">
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">From</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Subject</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Received</th>
                                    <th style="padding: 0.75rem 1rem;"></th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($messages as $m)
                                    <tr style="border-top: 1px solid var(--border);">
                                        <td style="padding: 0.75rem 1rem;">
                                            <div style="color: var(--fg-hi);">{{ $m->name ?: '—' }}</div>
                                            <div class="text-xs" style="color: var(--muted);">{{ $m->email }}</div>
                                        </td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg);">{{ $m->subject ?: '(no subject)' }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-size: 12px;">{{ $m->created_at->format('d M Y H:i') }}</td>
                                        <td style="padding: 0.75rem 1rem; text-align: right;">
                                            <a href="{{ route('admin.contacts.show', $m->id) }}"
                                               class="inline-flex items-center rounded-lg"
                                               style="padding: 0.35rem 0.7rem; font-size: 0.72rem; font-weight: 500; background: transparent; border: 1px solid var(--border-strong); color: var(--fg);">
                                                Open
                                            </a>
                                        </td>
                                    </tr>
                                @endforeach
                            </tbody>
                        </table>
                    @endif
                </div>

                <div class="mt-6">
                    {{ $messages->links() }}
                </div>

            </div>
        </div>
    </section>
</x-layouts.app>
