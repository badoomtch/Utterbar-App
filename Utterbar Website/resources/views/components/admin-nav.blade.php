@props(['active' => 'dashboard'])

@php
    $items = [
        ['key' => 'dashboard',   'label' => 'Dashboard',   'href' => route('admin.index')],
        ['key' => 'licences',    'label' => 'Licences',    'href' => route('admin.licences.index')],
        ['key' => 'releases',    'label' => 'Releases',    'href' => route('admin.releases.index')],
        ['key' => 'subscribers', 'label' => 'Subscribers', 'href' => route('admin.subscribers.index')],
        ['key' => 'contacts',    'label' => 'Contacts',    'href' => route('admin.contacts.index')],
        ['key' => 'analytics',   'label' => 'Analytics',   'href' => route('admin.analytics.index')],
        ['key' => 'tips',        'label' => 'Tips',        'href' => route('admin.tips.index')],
    ];
@endphp

<nav class="mb-10 flex flex-wrap items-center gap-1 border-b" style="border-color: var(--border);">
    @foreach ($items as $item)
        @php $isActive = $item['key'] === $active; @endphp
        <a href="{{ $item['href'] }}"
           class="inline-flex items-center px-4 py-3 text-sm font-medium transition-colors"
           style="{{ $isActive
                ? 'color: var(--fg-hi); border-bottom: 2px solid var(--accent); margin-bottom: -1px;'
                : 'color: var(--muted); border-bottom: 2px solid transparent; margin-bottom: -1px;' }}">
            {{ $item['label'] }}
        </a>
    @endforeach
</nav>
