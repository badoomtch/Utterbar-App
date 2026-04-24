@props([
    'label',
    'value',
    'accent' => false,
    'class' => null,
])

@php
    $valueColor = 'var(--fg-hi)';
    if ($accent)              $valueColor = 'var(--accent)';
    if ($class === 'danger')  $valueColor = 'var(--danger)';
@endphp

<div class="ub-card" style="padding: 1.1rem 1.25rem;">
    <p class="ub-eyebrow" style="font-size: 10.5px;">{{ $label }}</p>
    <p class="mt-2" style="color: {{ $valueColor }}; font-size: 1.75rem; font-weight: 600; letter-spacing: -0.02em; line-height: 1.1; font-family: var(--font-mono);">
        {{ $value }}
    </p>
</div>
