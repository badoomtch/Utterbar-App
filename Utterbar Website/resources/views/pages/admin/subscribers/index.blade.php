@php $flash = session('flash'); @endphp

<x-layouts.app title="Admin · Subscribers" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto">

                <x-admin-nav active="subscribers" />

                <div class="mb-8">
                    <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.05;">
                        Subscribers.
                    </h1>
                    <p class="mt-3 text-sm" style="color: var(--muted);">
                        Emails on the release-announcement list. Gathered from the in-app "Email me when there's a new release" setting.
                    </p>
                </div>

                @if ($flash)
                    <div class="ub-card mb-6" style="padding: 1rem 1.25rem; border-color: {{ $flash['type'] === 'error' ? 'var(--danger)' : 'var(--accent)' }};">
                        <p class="text-sm" style="color: {{ $flash['type'] === 'error' ? 'var(--danger)' : 'var(--accent)' }};">{{ $flash['text'] }}</p>
                    </div>
                @endif

                <form method="GET" action="{{ route('admin.subscribers.index') }}" class="mb-6 flex flex-col md:flex-row gap-3">
                    <input type="text" name="q" value="{{ $q }}" placeholder="Search by email..."
                           class="flex-1 px-3 py-2 rounded-lg text-sm"
                           style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                    <select name="status" class="px-3 py-2 rounded-lg text-sm"
                            style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                        <option value="" @selected($status === '')>All statuses</option>
                        <option value="confirmed"    @selected($status === 'confirmed')>Confirmed</option>
                        <option value="unconfirmed"  @selected($status === 'unconfirmed')>Unconfirmed</option>
                        <option value="unsubscribed" @selected($status === 'unsubscribed')>Unsubscribed</option>
                    </select>
                    <button type="submit" class="ub-btn ub-btn-ghost">Filter</button>
                </form>

                <div class="ub-card" style="padding: 0; overflow: hidden;">
                    @if ($subs->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">No subscribers match.</p>
                    @else
                        <table class="w-full text-sm" style="border-collapse: collapse;">
                            <thead>
                                <tr style="background: var(--bg-0); color: var(--muted); text-align: left;">
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Email</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Status</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Subscribed</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Source</th>
                                    <th style="padding: 0.75rem 1rem;"></th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($subs as $sub)
                                    <tr style="border-top: 1px solid var(--border);">
                                        <td style="padding: 0.75rem 1rem; color: var(--fg);">{{ $sub->email }}</td>
                                        <td style="padding: 0.75rem 1rem;">
                                            @if ($sub->unsubscribed_at)
                                                <span class="font-mono" style="color: var(--danger); font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase;">&#9679; unsubscribed</span>
                                            @elseif ($sub->confirmed_at)
                                                <span class="font-mono" style="color: var(--accent); font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase;">&#9679; confirmed</span>
                                            @else
                                                <span class="font-mono" style="color: var(--muted); font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase;">&#9679; pending</span>
                                            @endif
                                        </td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-size: 12px;">{{ $sub->subscribed_at?->format('d M Y H:i') }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-family: var(--font-mono); font-size: 12px;">{{ $sub->source ?? '-' }}</td>
                                        <td style="padding: 0.75rem 1rem; text-align: right;">
                                            @if (!$sub->unsubscribed_at)
                                                <form method="POST" action="{{ route('admin.subscribers.unsubscribe', $sub->id) }}"
                                                      onsubmit="return confirm('Unsubscribe {{ $sub->email }}? They will not receive future release emails.');" class="inline">
                                                    @csrf
                                                    <button type="submit"
                                                            class="inline-flex items-center rounded-lg"
                                                            style="padding: 0.35rem 0.7rem; font-size: 0.72rem; font-weight: 500; background: transparent; border: 1px solid var(--border-strong); color: var(--danger);">
                                                        Unsubscribe
                                                    </button>
                                                </form>
                                            @endif
                                        </td>
                                    </tr>
                                @endforeach
                            </tbody>
                        </table>
                    @endif
                </div>

                <div class="mt-6">
                    {{ $subs->links() }}
                </div>

            </div>
        </div>
    </section>
</x-layouts.app>
