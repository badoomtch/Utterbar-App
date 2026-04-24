@php
    $user = $user ?? auth()->user();
    $flash = session('flash');
@endphp

<x-layouts.app title="Your account" :no-index="true">
    <section class="pt-20 md:pt-28 pb-20">
        <div class="ub-container">
            <div class="max-w-4xl mx-auto">

                {{-- Header --}}
                <div class="flex flex-col md:flex-row md:items-end md:justify-between gap-4 mb-10">
                    <div>
                        <h1 class="ub-fg-hi" style="font-size: clamp(2.25rem, 4vw, 3.25rem); letter-spacing: -0.035em; line-height: 1.05;">
                            Your account.
                        </h1>
                        <p class="mt-3 text-base" style="color: var(--muted);">
                            Signed in as <span style="color: var(--fg-hi);">{{ $user->email }}</span>.
                        </p>
                    </div>
                    <form method="POST" action="{{ route('sign-out') }}">
                        @csrf
                        <button type="submit" class="ub-btn ub-btn-ghost">Sign out</button>
                    </form>
                </div>

                {{-- Flash banner --}}
                @if ($flash)
                    <div class="ub-card mb-8" style="padding: 1rem 1.25rem; border-color: {{ $flash['type'] === 'error' ? 'var(--danger)' : 'var(--accent)' }};">
                        <p class="text-sm" style="color: {{ $flash['type'] === 'error' ? 'var(--danger)' : 'var(--accent)' }};">
                            {{ $flash['text'] }}
                        </p>
                    </div>
                @endif

                {{-- Licences --}}
                <div class="mb-6">
                    <p class="ub-eyebrow mb-4">Your licences</p>

                    @if ($licences->isEmpty())
                        <div class="ub-card" style="padding: 2rem; text-align: center;">
                            <p style="color: var(--muted);">No licences on this email yet.</p>
                            <a href="{{ route('pricing') }}" class="ub-btn ub-btn-cta mt-6">Get Utterbar Pro</a>
                        </div>
                    @else
                        <div class="space-y-4">
                            @foreach ($licences as $licence)
                                <div class="ub-card ub-card-rail" style="padding: 1.75rem 2rem;">
                                    <div class="flex flex-col md:flex-row md:items-start md:justify-between gap-4">
                                        <div class="min-w-0 flex-1">
                                            <div class="flex items-center gap-3 mb-2">
                                                <p class="font-semibold" style="color: var(--fg-hi); font-size: 1.125rem; letter-spacing: -0.015em;">
                                                    Utterbar Pro &middot; lifetime
                                                </p>
                                                @if ($licence->status === 'active')
                                                    <span class="font-mono text-[11px]" style="color: var(--accent); letter-spacing: 0.08em; text-transform: uppercase;">
                                                        &#9679; active
                                                    </span>
                                                @else
                                                    <span class="font-mono text-[11px]" style="color: var(--danger); letter-spacing: 0.08em; text-transform: uppercase;">
                                                        &#9679; {{ $licence->status }}
                                                    </span>
                                                @endif
                                            </div>
                                            <p class="text-sm" style="color: var(--muted);">
                                                Issued {{ $licence->issued_at?->format('d M Y') }}
                                                &middot; up to {{ $licence->max_devices }} devices
                                            </p>

                                            <div class="mt-5 rounded-xl" style="background: var(--bg-0); border: 1px solid var(--border-strong); padding: 0.9rem 1.1rem;" x-data="{ copied: false }">
                                                <div class="flex items-center justify-between gap-3 flex-wrap">
                                                    <p class="font-mono" style="font-size: 0.95rem; letter-spacing: 0.06em; color: var(--fg-hi); word-break: break-all;">
                                                        {{ $licence->displayKey() }}
                                                    </p>
                                                    <button type="button"
                                                            @click="navigator.clipboard.writeText('{{ $licence->displayKey() }}'); copied = true; setTimeout(() => copied = false, 1600)"
                                                            class="shrink-0 inline-flex items-center gap-1.5 rounded-lg"
                                                            style="padding: 0.4rem 0.75rem; font-size: 0.75rem; font-weight: 500;
                                                                   background: transparent; border: 1px solid var(--border-strong); color: var(--fg-hi);">
                                                        <span x-show="!copied">Copy</span>
                                                        <span x-show="copied" style="color: var(--accent);">Copied</span>
                                                    </button>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="flex flex-col gap-2 shrink-0">
                                            <form method="POST" action="{{ route('account.licences.resend', ['id' => $licence->id]) }}">
                                                @csrf
                                                <button type="submit"
                                                        class="inline-flex items-center gap-1.5 rounded-lg"
                                                        style="padding: 0.55rem 0.9rem; font-size: 0.85rem; font-weight: 500;
                                                               background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg-hi);">
                                                    Resend by email
                                                </button>
                                            </form>
                                            <a href="utterbar://licence/{{ $licence->key_lookup }}"
                                               class="inline-flex items-center gap-1.5 rounded-lg justify-center"
                                               style="padding: 0.55rem 0.9rem; font-size: 0.85rem; font-weight: 500;
                                                      background: transparent; border: 1px solid color-mix(in oklab, var(--accent) 45%, transparent); color: var(--accent);">
                                                Paste into app
                                            </a>
                                        </div>
                                    </div>

                                    {{-- Devices --}}
                                    <div class="mt-6 pt-5" style="border-top: 1px solid var(--border);">
                                        @php
                                            $devices = $licence->activations;
                                            $used    = $devices->count();
                                            $free    = max(0, $licence->max_devices - $used);
                                        @endphp

                                        <div class="flex items-baseline justify-between mb-3">
                                            <p class="ub-eyebrow" style="font-size: 11px;">Devices</p>
                                            <p class="font-mono text-[11px]" style="color: var(--muted); letter-spacing: 0.06em;">
                                                {{ $used }} / {{ $licence->max_devices }} USED
                                            </p>
                                        </div>

                                        @if ($devices->isEmpty())
                                            <p class="text-sm" style="color: var(--muted);">
                                                No devices activated yet. Paste your key into Utterbar &rarr; Settings &rarr; Licence.
                                            </p>
                                        @else
                                            <ul class="space-y-2">
                                                @foreach ($devices as $dev)
                                                    <li class="flex items-center justify-between gap-3 rounded-lg"
                                                        style="padding: 0.65rem 0.9rem; background: var(--bg-0); border: 1px solid var(--border);">
                                                        <div class="min-w-0">
                                                            <p class="text-sm font-medium" style="color: var(--fg-hi); line-height: 1.25;">
                                                                <span style="color: var(--accent); margin-right: 6px;">&#9679;</span>{{ $dev->device_name }}
                                                            </p>
                                                            <p class="text-[11px] mt-0.5" style="color: var(--muted);">
                                                                {{ strtolower($dev->platform) }}
                                                                @if ($dev->app_version) &middot; v{{ $dev->app_version }} @endif
                                                                &middot; last seen {{ $dev->last_heartbeat?->diffForHumans() }}
                                                            </p>
                                                        </div>
                                                        <x-modal-confirm
                                                            triggerLabel="Deactivate"
                                                            title="Deactivate {{ $dev->device_name }}?"
                                                            description="This device will need to be re-activated from that machine if you want to use Utterbar Pro on it again. The licence slot becomes immediately available."
                                                            confirmLabel="Yes, deactivate"
                                                            variant="danger"
                                                            :action="route('account.licences.devices.revoke', ['id' => $licence->id, 'fingerprint' => $dev->fingerprint_hash])"
                                                        />
                                                    </li>
                                                @endforeach

                                                @for ($i = 0; $i < $free; $i++)
                                                    <li class="flex items-center gap-3 rounded-lg"
                                                        style="padding: 0.65rem 0.9rem; background: transparent; border: 1px dashed var(--border-strong);">
                                                        <span style="color: var(--fg-dim, var(--muted)); margin-right: 2px;">&#9675;</span>
                                                        <span class="text-[11px] font-mono" style="color: var(--muted); letter-spacing: 0.06em; text-transform: uppercase;">
                                                            Slot available
                                                        </span>
                                                    </li>
                                                @endfor
                                            </ul>
                                        @endif
                                    </div>
                                </div>
                            @endforeach
                        </div>
                    @endif
                </div>

                {{-- Downloads --}}
                <div class="mt-14 ub-card" style="padding: 1.75rem 2rem;">
                    <p class="ub-eyebrow mb-3">Downloads</p>
                    <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
                        <div>
                            <p class="font-semibold" style="color: var(--fg-hi);">Utterbar for Windows</p>
                            <p class="text-sm mt-1" style="color: var(--muted);">Windows 10/11 &middot; 64-bit &middot; ~60 MB</p>
                        </div>
                        <a href="{{ route('download') }}" class="ub-btn ub-btn-cta">Get the installer</a>
                    </div>
                </div>

                {{-- Support --}}
                <p class="mt-10 text-center text-sm" style="color: var(--muted);">
                    Something wrong? <a href="{{ route('contact') }}" style="color: var(--accent);">Email Jay</a> &mdash;
                    usually within a day, always within two.
                </p>

            </div>
        </div>
    </section>
</x-layouts.app>
