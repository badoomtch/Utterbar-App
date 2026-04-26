@php $flash = session('flash'); @endphp

<x-layouts.app title="Admin · Releases" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-6xl mx-auto">

                <x-admin-nav active="releases" />

                <div class="flex flex-col md:flex-row md:items-end md:justify-between gap-4 mb-8">
                    <div>
                        <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.05;">
                            Releases.
                        </h1>
                        <p class="mt-3 text-sm" style="color: var(--muted);">
                            Published builds. Listed newest first.
                        </p>
                    </div>
                    <a href="{{ route('admin.releases.create') }}" class="ub-btn ub-btn-cta">
                        New release
                    </a>
                </div>

                @if ($flash)
                    <div class="ub-card mb-6" style="padding: 1rem 1.25rem; border-color: {{ $flash['type'] === 'error' ? 'var(--danger)' : 'var(--accent)' }};">
                        <p class="text-sm" style="color: {{ $flash['type'] === 'error' ? 'var(--danger)' : 'var(--accent)' }};">{{ $flash['text'] }}</p>
                    </div>
                @endif

                <div class="ub-card" style="padding: 0; overflow: hidden;">
                    @if ($releases->isEmpty())
                        <p class="text-sm p-6" style="color: var(--muted);">
                            No releases yet. <a href="{{ route('admin.releases.create') }}" style="color: var(--accent);">Publish the first one</a>.
                        </p>
                    @else
                        <table class="w-full text-sm" style="border-collapse: collapse;">
                            <thead>
                                <tr style="background: var(--bg-0); color: var(--muted); text-align: left;">
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Version</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Channel</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Size</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Published</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;">Announced</th>
                                    <th style="padding: 0.75rem 1rem; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; font-weight: 500;"></th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach ($releases as $r)
                                    <tr style="border-top: 1px solid var(--border);">
                                        <td style="padding: 0.85rem 1rem; color: var(--fg-hi); font-family: var(--font-mono);">
                                            v{{ $r->version }}
                                            @if ($r->tag)
                                                <span class="font-mono text-[10px] uppercase tracking-wider px-2 py-0.5 rounded-full ml-1.5" style="background: color-mix(in oklab, var(--accent) 15%, transparent); color: var(--accent); border: 1px solid color-mix(in oklab, var(--accent) 30%, transparent);">{{ $r->tag }}</span>
                                            @endif
                                        </td>
                                        <td style="padding: 0.85rem 1rem; color: var(--fg);">{{ $r->channel }}</td>
                                        <td style="padding: 0.85rem 1rem; color: var(--muted); font-family: var(--font-mono); font-size: 12px;">{{ $r->sizeLabel() }}</td>
                                        <td style="padding: 0.85rem 1rem; color: var(--muted); font-size: 12px;">{{ $r->published_at?->format('d M Y H:i') }}</td>
                                        <td style="padding: 0.85rem 1rem; color: var(--muted); font-size: 12px;">
                                            @if ($r->announced_at)
                                                <span style="color: var(--accent);">&check;</span> {{ $r->announced_at->diffForHumans() }}
                                            @else
                                                &mdash;
                                            @endif
                                        </td>
                                        <td style="padding: 0.85rem 1rem; text-align: right;">
                                            <div class="flex items-center justify-end gap-2">
                                                <a href="{{ $r->downloadUrl() }}" target="_blank" rel="noopener"
                                                   class="inline-flex items-center rounded-lg"
                                                   style="padding: 0.35rem 0.7rem; font-size: 0.72rem; font-weight: 500; background: transparent; border: 1px solid var(--border-strong); color: var(--fg);">
                                                    Download
                                                </a>
                                                <x-modal-confirm
                                                    triggerLabel="Delete"
                                                    title="Delete v{{ $r->version }}?"
                                                    description="This removes the release from the database. Tick the checkbox in the form to also delete the installer from R2."
                                                    confirmLabel="Yes, delete"
                                                    variant="danger"
                                                    method="DELETE"
                                                    :action="route('admin.releases.destroy', ['id' => $r->id])">
                                                    <label class="flex items-start gap-2.5 text-xs cursor-pointer mt-2" style="color: var(--muted);">
                                                        <input type="checkbox" name="delete_file" value="1" form="" style="accent-color: var(--accent);" class="mt-0.5 shrink-0 w-4 h-4 rounded cursor-pointer">
                                                        <span>Also delete the .exe from R2 ({{ $r->r2_key }})</span>
                                                    </label>
                                                </x-modal-confirm>
                                            </div>
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
