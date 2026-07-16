@php
    $fmtMoney = function ($pence, $ccy) {
        $n = number_format($pence / 100, 2);
        return strtoupper($ccy) . ' ' . $n;
    };
@endphp

<x-layouts.app title="Tips" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-6xl mx-auto">

                <x-admin-nav active="tips" />

                <div class="flex flex-col md:flex-row md:items-end md:justify-between gap-4 mb-10">
                    <div>
                        <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.05;">
                            Tips.
                        </h1>
                        <p class="mt-3 text-sm" style="color: var(--muted);">
                            Voluntary "Tip the developer" payments via Stripe. Recorded by webhook the moment Stripe says paid; refunds are not tracked here yet (rare enough to investigate in Stripe directly).
                        </p>
                    </div>
                </div>

                {{-- Top-line stats --}}
                <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-12">
                    @if ($totalsByCurrency->isEmpty())
                        <x-admin-stat label="Total raised"  value="-" />
                        <x-admin-stat label="Tips received" value="0" />
                        <x-admin-stat label="Average tip"   value="-" />
                        <x-admin-stat label="Currencies"    value="0" />
                    @else
                        @php $primary = $totalsByCurrency->first(); @endphp
                        <x-admin-stat label="Total raised"  :value="$fmtMoney($primary->total, $primary->currency)" accent />
                        <x-admin-stat label="Tips received" :value="number_format($totalCount)" />
                        <x-admin-stat label="Average tip"   :value="$fmtMoney($averagePence, $primaryCurrency)" />
                        <x-admin-stat label="Currencies"    :value="$totalsByCurrency->count()" />
                    @endif
                </div>

                @if ($totalsByCurrency->count() > 1)
                    <p class="ub-eyebrow mb-3">All currencies</p>
                    <div class="ub-card mb-12 flex flex-wrap gap-x-8 gap-y-3" style="padding: 1.25rem 1.5rem;">
                        @foreach ($totalsByCurrency as $row)
                            <div>
                                <div class="text-xs font-mono uppercase tracking-wider" style="color: var(--muted);">{{ strtoupper($row->currency) }}</div>
                                <div class="font-mono text-base" style="color: var(--fg-hi);">{{ $fmtMoney($row->total, $row->currency) }}</div>
                                <div class="text-[11px]" style="color: var(--muted);">{{ $row->count }} {{ $row->count === 1 ? 'tip' : 'tips' }}</div>
                            </div>
                        @endforeach
                    </div>
                @endif

                {{-- Last 30 days --}}
                <p class="ub-eyebrow mb-4">Last 30 days</p>
                <div class="ub-card mb-12" style="padding: 1.25rem 1rem;">
                    @if ($byDay->isEmpty())
                        <p class="text-sm" style="color: var(--muted);">No tips in the last 30 days.</p>
                    @else
                        @php
                            $maxDay = max(1, $byDay->max('count'));
                        @endphp
                        <div class="flex items-end gap-1" style="height: 120px;">
                            @foreach ($byDay as $row)
                                @php
                                    $h = max(2, round(($row->count / $maxDay) * 100));
                                @endphp
                                <div class="flex-1 flex flex-col items-center gap-1" title="{{ $row->day }} · {{ $row->count }} tips · {{ $fmtMoney($row->total, $primaryCurrency) }}">
                                    <div style="background: var(--accent); width: 100%; height: {{ $h }}%; min-height: 2px; border-radius: 2px;"></div>
                                </div>
                            @endforeach
                        </div>
                        <div class="mt-2 flex justify-between text-xs font-mono" style="color: var(--muted);">
                            <span>{{ $byDay->first()->day }}</span>
                            <span>{{ $byDay->last()->day }}</span>
                        </div>
                    @endif
                </div>

                {{-- Recent tips --}}
                <p class="ub-eyebrow mb-4">Recent tips</p>
                <div class="ub-card mb-12" style="padding: 0; overflow: hidden;">
                    @if ($recent->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">No tips yet.</p>
                    @else
                        <table class="w-full text-sm" style="border-collapse: collapse;">
                            <thead>
                                <tr style="background: var(--bg-0); color: var(--muted); text-align: left;">
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">When</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Email</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Country</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500; text-align: right;">Amount</th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($recent as $tip)
                                    <tr style="border-top: 1px solid var(--border);">
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-size: 12px;">{{ $tip->paid_at?->format('d M Y H:i') }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg);">{{ $tip->email ?? '-' }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-family: var(--font-mono); font-size: 12px;">{{ $tip->country ?? '-' }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg-hi); font-family: var(--font-mono); text-align: right;">{{ $fmtMoney($tip->amount_pence, $tip->currency) }}</td>
                                    </tr>
                                @endforeach
                            </tbody>
                        </table>
                    @endif
                </div>

                <p class="text-xs" style="color: var(--muted); line-height: 1.6;">
                    Tips are voluntary one-time payments through Stripe. The webhook (StripeWebhookController) inserts a row when Stripe confirms payment with sku=utterbar-tip.
                </p>
            </div>
        </div>
    </section>
</x-layouts.app>
