@props(['withWordmark' => true, 'size' => 'md'])

@php
    $sizes = [
        'sm'  => ['h' => 28,  'markW' => 57,  'font' => 20],
        'md'  => ['h' => 36,  'markW' => 73,  'font' => 24],
        'nav' => ['h' => 42,  'markW' => 85,  'font' => 28],
        'lg'  => ['h' => 56,  'markW' => 113, 'font' => 40],
        'xl'  => ['h' => 88,  'markW' => 178, 'font' => 62],
    ];
    $s = $sizes[$size] ?? $sizes['md'];
@endphp

<span {{ $attributes->merge(['class' => 'inline-flex items-center gap-3']) }} aria-label="Utterbar">
    {{-- Glyph: normalised 0,0 → 120,60 with 2px padding for the white stroke --}}
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="-2 -2 124 64" width="{{ $s['markW'] }}" height="{{ $s['h'] }}" color-interpolation="linearRGB" aria-hidden="true" focusable="false">
        <defs>
            <linearGradient id="ub-pill-{{ $size }}" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%"   stop-color="#223042" />
                <stop offset="100%" stop-color="#111827" />
            </linearGradient>
            <linearGradient id="ub-wave-{{ $size }}" x1="0%" y1="0%" x2="0%" y2="100%" color-interpolation="linearRGB">
                <stop offset="0%"   stop-color="#00f6ff" />
                <stop offset="25%"  stop-color="#00e2fb" />
                <stop offset="50%"  stop-color="#00ccf5" />
                <stop offset="75%"  stop-color="#00b5ed" />
                <stop offset="100%" stop-color="#00a3ff" />
            </linearGradient>
        </defs>

        {{-- Pill body + crisp white border --}}
        <rect x="0" y="0" width="120" height="60" rx="30"
              fill="url(#ub-pill-{{ $size }})"
              stroke="#ffffff" stroke-width="2" />

        {{-- White hole-punch on the left --}}
        <circle cx="20" cy="30" r="8" fill="#ffffff" />

        {{-- Internal waveform with central speech-bubble tail (pointing down-right) --}}
        <g fill="url(#ub-wave-{{ $size }})">
            <rect x="44"  y="25" width="4" height="10" rx="2" />
            <rect x="54"  y="21" width="4" height="18" rx="2" />
            <rect x="64"  y="16" width="4" height="28" rx="2" />
            <path d="M 74 10 A 2 2 0 0 1 78 10 V 50 L 74 44 Z" />
            <rect x="84"  y="16" width="4" height="28" rx="2" />
            <rect x="94"  y="21" width="4" height="18" rx="2" />
            <rect x="104" y="25" width="4" height="10" rx="2" />
        </g>
    </svg>

    @if ($withWordmark)
        <span class="ub-wm leading-none select-none" style="font-size: {{ $s['font'] }}px; font-weight: 600; letter-spacing: -0.04em;">
            <span class="ub-wm-utter">Utter</span><span class="ub-wm-bar">bar</span>
        </span>
    @endif
</span>
