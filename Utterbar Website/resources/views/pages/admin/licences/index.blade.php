@php $flash = session('flash'); @endphp

<x-layouts.app title="Admin · Licences" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-6xl mx-auto">

                <x-admin-nav active="licences" />

                <div class="flex flex-col md:flex-row md:items-end md:justify-between gap-4 mb-8">
                    <div>
                        <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.05;">
                            Licences.
                        </h1>
                        <p class="mt-3 text-sm" style="color: var(--muted);">
                            All minted licences. Search by email, key, or Stripe ID.
                        </p>
                    </div>
                    <a href="{{ route('admin.licences.create') }}" class="ub-btn ub-btn-cta">
                        Mint licence
                    </a>
                </div>

                @if ($flash)
                    <div class="ub-card mb-6" style="padding: 1rem 1.25rem; border-color: {{ $flash['type'] === 'error' ? 'var(--danger)' : 'var(--accent)' }};">
                        <p class="text-sm" style="color: {{ $flash['type'] === 'error' ? 'var(--danger)' : 'var(--accent)' }};">{{ $flash['text'] }}</p>
                    </div>
                @endif

                {{-- Search + filter --}}
                <form method="GET" action="{{ route('admin.licences.index') }}" class="mb-6 flex flex-col md:flex-row gap-3">
                    <input type="text" name="q" value="{{ $q }}"
                           placeholder="Search by email, UB-XXXX key, Stripe session/customer..."
                           class="flex-1 px-3 py-2 rounded-lg text-sm"
                           style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                    <select name="status" class="px-3 py-2 rounded-lg text-sm"
                            style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                        <option value="" @selected($status === '')>All statuses</option>
                        <option value="active"   @selected($status === 'active')>Active</option>
                        <option value="revoked"  @selected($status === 'revoked')>Revoked</option>
                        <option value="refunded" @selected($status === 'refunded')>Refunded</option>
                    </select>
                    <button type="submit" class="ub-btn ub-btn-ghost">Filter</button>
                </form>

                <div class="ub-card" style="padding: 0; overflow: hidden;">
                    @if ($licences->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">
                            No licences match. <a href="{{ route('admin.licences.create') }}" style="color: var(--accent);">Mint one</a>.
                        </p>
                    @else
                        <table class="w-full text-sm" style="border-collapse: collapse;">
                            <thead>
                                <tr style="background: var(--bg-0); color: var(--muted); text-align: left;">
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">ID</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Key</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Email</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Status</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Devices</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Issued</th>
                                    <th style="padding: 0.75rem 1rem;"></th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($licences as $l)
                                    <tr style="border-top: 1px solid var(--border);">
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-family: var(--font-mono);">{{ $l->id }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg-hi); font-family: var(--font-mono); font-size: 12px; letter-spacing: 0.04em;">{{ $l->displayKey() }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg);">{{ $l->email }}</td>
                                        <td style="padding: 0.75rem 1rem;">
                                            @if ($l->status === 'active')
                                                <span class="font-mono" style="color: var(--accent); font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase;">&#9679; active</span>
                                            @else
                                                <span class="font-mono" style="color: var(--danger); font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase;">&#9679; {{ $l->status }}</span>
                                            @endif
                                        </td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg);">{{ $l->activations_count }} / {{ $l->max_devices }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-size: 12px;">{{ $l->issued_at?->format('d M Y H:i') }}</td>
                                        <td style="padding: 0.75rem 1rem; text-align: right;">
                                            <a href="{{ route('admin.licences.show', $l->id) }}"
                                               class="inline-flex items-center rounded-lg"
                                               style="padding: 0.35rem 0.7rem; font-size: 0.72rem; font-weight: 500; background: transparent; border: 1px solid var(--border-strong); color: var(--fg);">
                                                View
                                            </a>
                                        </td>
                                    </tr>
                                @endforeach
                            </tbody>
                        </table>
                    @endif
                </div>

                <div class="mt-6">
                    {{ $licences->links() }}
                </div>

            </div>
        </div>
    </section>
</x-layouts.app>
