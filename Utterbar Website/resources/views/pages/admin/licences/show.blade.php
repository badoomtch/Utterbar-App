@php $flash = session('flash'); @endphp

<x-layouts.app title="Admin · Licence" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto">

                <x-admin-nav active="licences" />

                <div class="mb-8">
                    <p class="ub-eyebrow">Licence</p>
                    <h1 class="ub-fg-hi mt-2 font-mono" style="font-size: clamp(1.5rem, 3vw, 2.25rem); letter-spacing: -0.025em; line-height: 1.1;">
                        {{ $licence->displayKey() }}
                    </h1>
                    <p class="mt-3 text-sm" style="color: var(--muted);">
                        <a href="{{ route('admin.licences.index') }}" style="color: var(--accent);">&larr; all licences</a>
                    </p>
                </div>

                @if ($flash)
                    <div class="ub-card mb-6" style="padding: 1rem 1.25rem; border-color: {{ $flash['type'] === 'error' ? 'var(--danger)' : 'var(--accent)' }};">
                        <p class="text-sm" style="color: {{ $flash['type'] === 'error' ? 'var(--danger)' : 'var(--accent)' }};">{{ $flash['text'] }}</p>
                    </div>
                @endif

                {{-- Top: facts about the licence --}}
                <div class="ub-card mb-6" style="padding: 1.5rem 1.75rem;">
                    <dl class="grid grid-cols-2 md:grid-cols-3 gap-x-6 gap-y-4 text-sm">
                        <div>
                            <dt class="ub-eyebrow mb-1">Status</dt>
                            <dd>
                                @if ($licence->status === 'active')
                                    <span class="font-mono" style="color: var(--accent); font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase;">&#9679; active</span>
                                @else
                                    <span class="font-mono" style="color: var(--danger); font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase;">&#9679; {{ $licence->status }}</span>
                                @endif
                            </dd>
                        </div>
                        <div>
                            <dt class="ub-eyebrow mb-1">Email</dt>
                            <dd style="color: var(--fg-hi);">{{ $licence->email }}</dd>
                        </div>
                        <div>
                            <dt class="ub-eyebrow mb-1">SKU</dt>
                            <dd class="font-mono" style="color: var(--fg);">{{ $licence->sku }}</dd>
                        </div>
                        <div>
                            <dt class="ub-eyebrow mb-1">Devices</dt>
                            <dd style="color: var(--fg);">{{ $licence->activations->count() }} / {{ $licence->max_devices }}</dd>
                        </div>
                        <div>
                            <dt class="ub-eyebrow mb-1">Issued</dt>
                            <dd style="color: var(--muted);">{{ $licence->issued_at?->format('d M Y H:i') }}</dd>
                        </div>
                        @if ($licence->status === 'revoked')
                            <div>
                                <dt class="ub-eyebrow mb-1">Revoked</dt>
                                <dd style="color: var(--danger);">
                                    {{ $licence->revoked_at?->format('d M Y H:i') }}
                                    @if ($licence->revoked_reason)
                                        <span style="color: var(--muted);">({{ $licence->revoked_reason }})</span>
                                    @endif
                                </dd>
                            </div>
                        @endif
                    </dl>

                    @if ($licence->stripe_checkout_session_id || $licence->stripe_customer_id)
                        <dl class="mt-5 pt-5 border-t grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3 text-xs" style="border-color: var(--border);">
                            @if ($licence->stripe_customer_id)
                                <div>
                                    <dt class="ub-eyebrow mb-1">Stripe customer</dt>
                                    <dd class="font-mono" style="color: var(--muted); word-break: break-all;">{{ $licence->stripe_customer_id }}</dd>
                                </div>
                            @endif
                            @if ($licence->stripe_checkout_session_id)
                                <div>
                                    <dt class="ub-eyebrow mb-1">Stripe session</dt>
                                    <dd class="font-mono" style="color: var(--muted); word-break: break-all;">{{ $licence->stripe_checkout_session_id }}</dd>
                                </div>
                            @endif
                        </dl>
                    @endif
                </div>

                {{-- Actions --}}
                <div class="ub-card mb-6" style="padding: 1.5rem 1.75rem;">
                    <p class="ub-eyebrow mb-4">Actions</p>
                    <div class="flex flex-wrap items-center gap-3">

                        {{-- Resend --}}
                        <form method="POST" action="{{ route('admin.licences.resend', $licence->id) }}" class="inline">
                            @csrf
                            <button type="submit" class="ub-btn ub-btn-ghost">Resend licence email</button>
                        </form>

                        {{-- Remint --}}
                        <form method="POST" action="{{ route('admin.licences.remint', $licence->id) }}" class="inline"
                              onsubmit="return confirm('Re-sign this licence with the current private key?');">
                            @csrf
                            <button type="submit" class="ub-btn ub-btn-ghost">Re-sign blob</button>
                        </form>

                        {{-- Download licence.bin for managed fleet deployment.
                             Serves the existing signed_blob column as
                             application/octet-stream with a stable filename so
                             an IT admin can drop it at %APPDATA%\Utterbar\
                             licence.bin on managed machines. The desktop app
                             treats a verified file as the active Pro licence
                             and skips paste / activation entirely. --}}
                        <a href="{{ route('admin.licences.download', $licence->id) }}"
                           class="ub-btn ub-btn-ghost"
                           download>
                            Download licence.bin
                        </a>

                        {{-- Revoke / unrevoke --}}
                        @if ($licence->status === 'active')
                            <form method="POST" action="{{ route('admin.licences.revoke', $licence->id) }}" class="inline"
                                  onsubmit="return confirm('Revoke this licence? Devices disable on next heartbeat.');">
                                @csrf
                                <input type="hidden" name="reason" value="admin revoke">
                                <button type="submit" class="ub-btn ub-btn-ghost" style="color: var(--danger); border-color: var(--danger);">Revoke</button>
                            </form>
                        @elseif ($licence->status === 'revoked')
                            <form method="POST" action="{{ route('admin.licences.unrevoke', $licence->id) }}" class="inline">
                                @csrf
                                <button type="submit" class="ub-btn ub-btn-ghost">Un-revoke</button>
                            </form>
                        @endif

                    </div>
                </div>

                {{-- Activations --}}
                <p class="ub-eyebrow mb-3">Device activations</p>
                <div class="ub-card" style="padding: 0; overflow: hidden;">
                    @if ($licence->activations->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">No devices have activated this licence yet.</p>
                    @else
                        <table class="w-full text-sm" style="border-collapse: collapse;">
                            <thead>
                                <tr style="background: var(--bg-0); color: var(--muted); text-align: left;">
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Device</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Platform</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">App</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Last heartbeat</th>
                                    <th style="padding: 0.75rem 1rem;"></th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($licence->activations as $a)
                                    <tr style="border-top: 1px solid var(--border);">
                                        <td style="padding: 0.75rem 1rem;">
                                            <div style="color: var(--fg-hi);">{{ $a->device_name ?: '(unnamed)' }}</div>
                                            <div class="font-mono text-[11px] mt-1" style="color: var(--muted);">{{ \Illuminate\Support\Str::limit($a->fingerprint_hash, 16, '') }}...</div>
                                        </td>
                                        <td style="padding: 0.75rem 1rem; color: var(--fg);">{{ $a->platform }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-family: var(--font-mono); font-size: 12px;">{{ $a->app_version }}</td>
                                        <td style="padding: 0.75rem 1rem; color: var(--muted); font-size: 12px;">{{ $a->last_heartbeat?->diffForHumans() ?? '—' }}</td>
                                        <td style="padding: 0.75rem 1rem; text-align: right;">
                                            <form method="POST" action="{{ route('admin.licences.devices.deactivate', ['id' => $licence->id, 'fingerprint' => $a->fingerprint_hash]) }}"
                                                  onsubmit="return confirm('Deactivate this device? The slot becomes free.');" class="inline">
                                                @csrf
                                                <button type="submit"
                                                        class="inline-flex items-center rounded-lg"
                                                        style="padding: 0.35rem 0.7rem; font-size: 0.72rem; font-weight: 500; background: transparent; border: 1px solid var(--border-strong); color: var(--danger);">
                                                    Deactivate
                                                </button>
                                            </form>
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
