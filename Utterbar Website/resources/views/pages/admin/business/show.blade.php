<x-layouts.app title="Admin · Business enquiry" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-4xl mx-auto">

                <x-admin-nav active="business" />

                @if (session('admin_status'))
                    <div class="mb-4 p-3 rounded-lg text-sm" style="background: color-mix(in oklab, var(--accent) 12%, transparent); color: var(--accent); border: 1px solid color-mix(in oklab, var(--accent) 30%, transparent);">
                        {{ session('admin_status') }}
                    </div>
                @endif

                <div class="mb-6">
                    <a href="{{ route('admin.business.index') }}" class="text-sm" style="color: var(--muted);">&larr; Back to enquiries</a>
                </div>

                <div class="ub-card mb-6" style="padding: 1.75rem 2rem;">
                    <div class="flex items-baseline justify-between flex-wrap gap-3">
                        <div>
                            <p class="ub-eyebrow" style="color: var(--accent);">{{ $enquiry->tierLabel() }}</p>
                            <h1 class="mt-2" style="font-size: 1.75rem; color: var(--fg-hi); letter-spacing: -0.02em;">{{ $enquiry->company }}</h1>
                            <p class="mt-2 text-sm" style="color: var(--muted);">
                                {{ $enquiry->name }} &middot;
                                <a href="mailto:{{ $enquiry->email }}" style="color: var(--accent);">{{ $enquiry->email }}</a>
                                @if($enquiry->role) &middot; {{ $enquiry->role }}@endif
                            </p>
                        </div>
                        <div class="text-right">
                            <div class="font-mono" style="font-size: 1.5rem; color: var(--fg-hi);">{{ number_format($enquiry->seat_count) }}</div>
                            <p class="text-xs" style="color: var(--muted); letter-spacing: 0.06em; text-transform: uppercase;">seats</p>
                        </div>
                    </div>
                    <div class="mt-5 pt-5 text-sm" style="color: var(--muted); border-top: 1px solid var(--border);">
                        Sent {{ $enquiry->created_at->format('D, j M Y · H:i') }}
                        @if($enquiry->emailed_at) &middot; emailed {{ $enquiry->emailed_at->format('H:i') }} @endif
                        @if($enquiry->ip) &middot; IP {{ $enquiry->ip }} @endif
                    </div>
                </div>

                <div class="ub-card mb-6" style="padding: 1.75rem 2rem;">
                    <p class="ub-eyebrow mb-3">Use case</p>
                    <p style="color: var(--fg); line-height: 1.6; white-space: pre-wrap;">{{ $enquiry->use_case }}</p>
                </div>

                @php $picked = (array) ($enquiry->requirements ?? []); @endphp
                @if(count($picked) > 0)
                    <div class="ub-card mb-6" style="padding: 1.75rem 2rem;">
                        <p class="ub-eyebrow mb-3">Requirements</p>
                        <ul class="space-y-2">
                            @foreach($picked as $key)
                                <li style="color: var(--fg);">
                                    <span style="color: var(--accent);">&middot;</span>
                                    {{ $requirementOptions[$key] ?? $key }}
                                </li>
                            @endforeach
                        </ul>
                    </div>
                @endif

                @if($enquiry->message)
                    <div class="ub-card mb-6" style="padding: 1.75rem 2rem;">
                        <p class="ub-eyebrow mb-3">Additional notes</p>
                        <p style="color: var(--fg); line-height: 1.6; white-space: pre-wrap;">{{ $enquiry->message }}</p>
                    </div>
                @endif

                <div class="ub-card" style="padding: 1.75rem 2rem;">
                    <p class="ub-eyebrow mb-3">Status</p>
                    <p class="mb-4 text-sm" style="color: var(--muted);">Currently: <strong style="color: var(--fg-hi);">{{ str_replace('_', ' ', $enquiry->status) }}</strong></p>
                    <form method="POST" action="{{ route('admin.business.status', $enquiry->id) }}" class="flex flex-wrap items-center gap-3">
                        @csrf
                        <select name="status" class="px-3 py-2 rounded-lg text-sm"
                                style="background: var(--bg-0); border: 1px solid var(--border); color: var(--fg-hi);">
                            <option value="new" {{ $enquiry->status === 'new' ? 'selected' : '' }}>New</option>
                            <option value="contacted" {{ $enquiry->status === 'contacted' ? 'selected' : '' }}>Contacted</option>
                            <option value="closed_won" {{ $enquiry->status === 'closed_won' ? 'selected' : '' }}>Closed: won</option>
                            <option value="closed_lost" {{ $enquiry->status === 'closed_lost' ? 'selected' : '' }}>Closed: lost</option>
                        </select>
                        <button type="submit" class="ub-btn ub-btn-ghost">Update status</button>
                    </form>
                </div>

            </div>
        </div>
    </section>
</x-layouts.app>
