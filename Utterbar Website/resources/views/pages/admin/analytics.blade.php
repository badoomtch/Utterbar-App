<x-layouts.app title="Analytics" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-6xl mx-auto">

                <x-admin-nav active="analytics" />

                <div class="flex flex-col md:flex-row md:items-end md:justify-between gap-4 mb-10">
                    <div>
                        <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.05;">
                            Downloads.
                        </h1>
                        <p class="mt-3 text-sm" style="color: var(--muted);">
                            Aggregate download stats. Deduped per IP+UA per day, hashed with the app key — raw IPs and User-Agents are never stored.
                        </p>
                    </div>
                </div>

                {{-- Top-line counts --}}
                <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-12">
                    <x-admin-stat label="Unique downloaders (all-time)" :value="number_format($uniqueDownloadersAllTime)" accent />
                    <x-admin-stat label="Total download events"        :value="number_format($totalEvents)" />
                    <x-admin-stat label="Releases tracked"             :value="$perRelease->count()" />
                    <x-admin-stat label="Distinct countries"           :value="$byCountry->reject(fn($c) => $c->country === '??')->count()" />
                </div>

                {{-- Per-release table --}}
                <p class="ub-eyebrow mb-4">Downloads by release</p>
                <div class="ub-card mb-12" style="padding: 0; overflow: hidden;">
                    @if ($perRelease->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">No releases yet.</p>
                    @else
                        <table class="w-full text-sm" style="border-collapse: collapse;">
                            <thead>
                                <tr style="background: var(--bg-0); color: var(--muted); text-align: left;">
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Version</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Channel</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Published</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500; text-align: right;">Unique</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500; text-align: right;">Total</th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($perRelease as $r)
                                    <tr style="border-top: 1px solid var(--border);">
                                        <td style="padding: 0.75rem 1rem; color: var(--fg-hi); font-family: var(--font-mono);">v{{ $r->version }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-family: var(--font-mono); font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em;">{{ $r->channel }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-size: 12px;">{{ $r->published_at ? \Illuminate\Support\Carbon::parse($r->published_at)->format('d M Y') : '—' }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg-hi); font-family: var(--font-mono); text-align: right;">{{ number_format($r->unique_downloaders) }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-family: var(--font-mono); text-align: right;">{{ number_format($r->events) }}</td>
                                    </tr>
                                @endforeach
                            </tbody>
                        </table>
                    @endif
                </div>

                {{-- Country split --}}
                <p class="ub-eyebrow mb-4">Top countries (unique downloaders)</p>
                <div class="ub-card mb-12" style="padding: 0; overflow: hidden;">
                    @if ($byCountry->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">No download events yet — country data appears once people start downloading.</p>
                    @else
                        @php
                            $maxUniques = $byCountry->max('uniques') ?: 1;
                        @endphp
                        <ul class="divide-y" style="border-color: var(--border);">
                            @foreach ($byCountry as $row)
                                @php
                                    $pct = max(2, round(($row->uniques / $maxUniques) * 100));
                                @endphp
                                <li class="flex items-center gap-4 px-4 py-3">
                                    <span class="font-mono text-xs" style="color: var(--fg-hi); width: 2.5rem;">{{ $row->country === '??' ? '—' : $row->country }}</span>
                                    <span class="text-xs" style="color: var(--muted); flex: 1;">
                                        {{ $row->country === '??' ? 'Unknown / direct origin' : $row->country }}
                                    </span>
                                    <span class="block h-1.5 rounded-full" style="background: var(--accent); width: {{ $pct }}%; max-width: 12rem;"></span>
                                    <span class="font-mono text-xs tabular-nums" style="color: var(--fg-hi); width: 3rem; text-align: right;">{{ number_format($row->uniques) }}</span>
                                </li>
                            @endforeach
                        </ul>
                    @endif
                </div>

                {{-- Last 30 days --}}
                <p class="ub-eyebrow mb-4">Last 30 days</p>
                <div class="ub-card mb-12" style="padding: 1.25rem 1rem;">
                    @if ($byDay->isEmpty())
                        <p class="text-sm" style="color: var(--muted);">No download events in the last 30 days.</p>
                    @else
                        @php
                            $maxDay = max(1, $byDay->max('events'));
                        @endphp
                        <div class="flex items-end gap-1" style="height: 120px;">
                            @foreach ($byDay as $row)
                                @php
                                    $h = max(2, round(($row->events / $maxDay) * 100));
                                @endphp
                                <div class="flex-1 flex flex-col items-center gap-1" title="{{ $row->event_date }} · {{ $row->events }} events">
                                    <div style="background: var(--accent); width: 100%; height: {{ $h }}%; min-height: 2px; border-radius: 2px;"></div>
                                </div>
                            @endforeach
                        </div>
                        <div class="mt-2 flex justify-between text-xs font-mono" style="color: var(--muted);">
                            <span>{{ $byDay->first()->event_date }}</span>
                            <span>{{ $byDay->last()->event_date }}</span>
                        </div>
                    @endif
                </div>

                <p class="text-xs" style="color: var(--muted); line-height: 1.6;">
                    <strong style="color: var(--fg-hi);">Privacy note.</strong>
                    Download events are deduplicated per (IP&nbsp;hash, User-Agent&nbsp;hash, day) so reinstalls within a day collapse to one row. Hashes are SHA-256 of (value || app_key) — they're not reversible without the app key. Country comes from Cloudflare's CF-IPCountry header on the request and is stored as the two-letter code only.
                </p>
            </div>
        </div>
    </section>
</x-layouts.app>
