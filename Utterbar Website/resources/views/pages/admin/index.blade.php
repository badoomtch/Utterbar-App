@php
    $fmtMoney = fn ($pence, $ccy) => strtoupper($ccy) . ' ' . number_format($pence / 100, 2);
@endphp

<x-layouts.app title="Admin" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-6xl mx-auto">

                <div class="flex flex-col md:flex-row md:items-end md:justify-between gap-4 mb-10">
                    <div>
                        <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.05;">
                            Admin.
                        </h1>
                        <p class="mt-3 text-sm" style="color: var(--muted);">
                            Read-only overview. Destructive actions via artisan:
                            <span class="font-mono" style="color: var(--fg-hi);">licence:revoke</span>,
                            <span class="font-mono" style="color: var(--fg-hi);">licence:unrevoke</span>,
                            <span class="font-mono" style="color: var(--fg-hi);">licence:list</span>,
                            <span class="font-mono" style="color: var(--fg-hi);">licence:resend</span>.
                        </p>
                    </div>
                </div>

                {{-- Stat cards --}}
                <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-12">
                    <x-admin-stat label="Active licences"  :value="$counts['licences_active']"  accent />
                    <x-admin-stat label="Refunded"         :value="$counts['licences_refunded']" />
                    <x-admin-stat label="Revoked"          :value="$counts['licences_revoked']" />
                    <x-admin-stat label="Device activations" :value="$counts['activations_total']" />

                    <x-admin-stat label="Gross"            :value="$fmtMoney($grossPence, $currency)" />
                    <x-admin-stat label="Net of refunds"   :value="$fmtMoney($netPence, $currency)" accent />
                    <x-admin-stat label="Webhook events"   :value="$counts['webhook_total']" />
                    <x-admin-stat label="Webhook errors"   :value="$counts['webhook_errors']"
                                   :class="$counts['webhook_errors'] > 0 ? 'danger' : null" />
                </div>

                {{-- Licences --}}
                <p class="ub-eyebrow mb-4">Recent licences</p>
                <div class="ub-card mb-12" style="padding: 0; overflow: hidden;">
                    @if ($recentLicences->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">No licences yet.</p>
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
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($recentLicences as $l)
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
                                    </tr>
                                @endforeach
                            </tbody>
                        </table>
                    @endif
                </div>

                {{-- Webhook events --}}
                <p class="ub-eyebrow mb-4">Recent Stripe webhook events</p>
                <div class="ub-card mb-12" style="padding: 0; overflow: hidden;">
                    @if ($recentWebhooks->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">No webhook events recorded yet.</p>
                    @else
                        <table class="w-full text-sm" style="border-collapse: collapse;">
                            <thead>
                                <tr style="background: var(--bg-0); color: var(--muted); text-align: left;">
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Event ID</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Type</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Processed</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Error</th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($recentWebhooks as $w)
                                    <tr style="border-top: 1px solid var(--border);">
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-family: var(--font-mono); font-size: 11px;">{{ \Illuminate\Support\Str::limit($w->stripe_event_id, 28) }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg-hi); font-family: var(--font-mono); font-size: 12px;">{{ $w->type }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-size: 12px;">
                                            @if ($w->processed_at)
                                                {{ \Illuminate\Support\Carbon::parse($w->processed_at)->format('d M H:i:s') }}
                                            @else
                                                <span style="color: var(--danger);">&mdash;</span>
                                            @endif
                                        </td>
                                        <td style="padding: 0.75rem 1rem; font-size: 12px;">
                                            @if ($w->error)
                                                <span style="color: var(--danger);">{{ \Illuminate\Support\Str::limit($w->error, 60) }}</span>
                                            @else
                                                <span style="color: var(--muted);">&mdash;</span>
                                            @endif
                                        </td>
                                    </tr>
                                @endforeach
                            </tbody>
                        </table>
                    @endif
                </div>

                {{-- Sign-in tokens --}}
                <p class="ub-eyebrow mb-4">Recent sign-in links</p>
                <div class="ub-card mb-12" style="padding: 0; overflow: hidden;">
                    @if ($recentSignIns->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">No sign-in links issued yet.</p>
                    @else
                        <table class="w-full text-sm" style="border-collapse: collapse;">
                            <thead>
                                <tr style="background: var(--bg-0); color: var(--muted); text-align: left;">
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Email</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">IP</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Issued</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Used</th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($recentSignIns as $t)
                                    <tr style="border-top: 1px solid var(--border);">
                                        <td style="padding: 0.75rem 1rem; color: var(--fg);">{{ $t->email }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-family: var(--font-mono); font-size: 12px;">{{ $t->ip ?? '—' }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-size: 12px;">{{ \Illuminate\Support\Carbon::parse($t->created_at)->format('d M H:i') }}</td>
                                        <td style="padding: 0.75rem 1rem; font-size: 12px;">
                                            @if ($t->used_at)
                                                <span style="color: var(--accent);">&check;</span>
                                            @else
                                                <span style="color: var(--muted);">&mdash;</span>
                                            @endif
                                        </td>
                                    </tr>
                                @endforeach
                            </tbody>
                        </table>
                    @endif
                </div>

            </div>
        </div>
    </section>
</x-layouts.app>
