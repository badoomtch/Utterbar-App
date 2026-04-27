<x-layouts.app title="Admin · Business enquiries" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto">

                <x-admin-nav active="business" />

                <div class="mb-8">
                    <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.05;">
                        Business enquiries.
                    </h1>
                    <p class="mt-3 text-sm" style="color: var(--muted);">
                        Volume / sales leads from <code class="font-mono text-xs" style="color: var(--accent);">/business/contact</code>. Also mirrored to email at submission time.
                    </p>
                </div>

                <form method="GET" action="{{ route('admin.business.index') }}" class="mb-6 flex flex-wrap gap-3 items-center">
                    <input type="text" name="q" value="{{ $q }}" placeholder="Search by email, name, company, use case..."
                           class="flex-1 min-w-[220px] px-3 py-2 rounded-lg text-sm"
                           style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                    <select name="status" class="px-3 py-2 rounded-lg text-sm"
                            style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                        <option value="" {{ $status === '' ? 'selected' : '' }}>Any status</option>
                        <option value="new" {{ $status === 'new' ? 'selected' : '' }}>New</option>
                        <option value="contacted" {{ $status === 'contacted' ? 'selected' : '' }}>Contacted</option>
                        <option value="closed_won" {{ $status === 'closed_won' ? 'selected' : '' }}>Closed: won</option>
                        <option value="closed_lost" {{ $status === 'closed_lost' ? 'selected' : '' }}>Closed: lost</option>
                    </select>
                    <button type="submit" class="px-3 py-2 rounded-lg text-sm" style="background: var(--bg-2); border: 1px solid var(--border); color: var(--fg-hi);">Filter</button>
                </form>

                <div class="ub-card" style="padding: 0; overflow: hidden;">
                    @if ($enquiries->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">No enquiries match.</p>
                    @else
                        <table class="w-full text-sm" style="border-collapse: collapse;">
                            <thead>
                                <tr style="background: var(--bg-0); color: var(--muted); text-align: left;">
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Company</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Tier</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Seats</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Status</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Received</th>
                                    <th style="padding: 0.75rem 1rem;"></th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($enquiries as $e)
                                    <tr style="border-top: 1px solid var(--border);">
                                        <td style="padding: 0.75rem 1rem;">
                                            <div style="color: var(--fg-hi);">{{ $e->company }}</div>
                                            <div class="text-xs" style="color: var(--muted);">{{ $e->name }} · {{ $e->email }}</div>
                                        </td>
                                        <td style="padding: 0.75rem 1rem; color: var(--accent); font-weight: 500;">{{ $e->tierLabel() }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg); font-family: var(--font-mono); font-size: 0.85rem;">{{ number_format($e->seat_count) }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg); font-size: 0.85rem;">
                                            @php $statusColors = ['new' => 'var(--accent)', 'contacted' => 'var(--fg-hi)', 'closed_won' => '#34D399', 'closed_lost' => 'var(--muted)']; @endphp
                                            <span style="color: {{ $statusColors[$e->status] ?? 'var(--muted)' }};">{{ str_replace('_', ' ', $e->status) }}</span>
                                        </td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-size: 12px;">{{ $e->created_at->format('d M Y H:i') }}</td>
                                        <td style="padding: 0.75rem 1rem; text-align: right;">
                                            <a href="{{ route('admin.business.show', $e->id) }}"
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
                    {{ $enquiries->links() }}
                </div>

            </div>
        </div>
    </section>
</x-layouts.app>
