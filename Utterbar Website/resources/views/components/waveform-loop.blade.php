@props(['size' => 'lg'])

@php
    $sizes = [
        'sm' => 260,
        'md' => 360,
        'lg' => 520,
    ];
    $w = $sizes[$size] ?? $sizes['lg'];
@endphp

<div class="relative w-full flex items-center justify-center">
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="30 15 110 95"
         color-interpolation="linearRGB"
         style="width: {{ $w }}px; max-width: 100%; height: auto; overflow: visible; background: transparent;
                filter:
                    drop-shadow(0 0  6px rgba(34, 211, 238, 0.55))
                    drop-shadow(0 0 14px rgba(34, 211, 238, 0.38))
                    drop-shadow(0 0 28px rgba(34, 211, 238, 0.22))
                    drop-shadow(0 0 56px rgba(34, 211, 238, 0.12));"
         aria-hidden="true" focusable="false">
        <defs>
            {{-- Waveform cyan: 5-stop arc through cyan → sky → azure, linearRGB interpolation to kill banding --}}
            <linearGradient id="wf-cyan" x1="0%" y1="0%" x2="0%" y2="100%" color-interpolation="linearRGB">
                <stop offset="0%"   stop-color="#00f6ff" />
                <stop offset="25%"  stop-color="#00e2fb" />
                <stop offset="50%"  stop-color="#00ccf5" />
                <stop offset="75%"  stop-color="#00b5ed" />
                <stop offset="100%" stop-color="#00a3ff" />
            </linearGradient>

            {{-- Pill slate: 4-stop deep slate for richer body, no visible banding on the dark fill --}}
            <linearGradient id="wf-pill" x1="0%" y1="0%" x2="0%" y2="100%" color-interpolation="linearRGB">
                <stop offset="0%"   stop-color="#3a4a64" />
                <stop offset="40%"  stop-color="#2a3a52" />
                <stop offset="75%"  stop-color="#18243a" />
                <stop offset="100%" stop-color="#0f1827" />
            </linearGradient>
            <linearGradient id="wf-rim" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%"   stop-color="#7a8da8" stop-opacity="0.9" />
                <stop offset="60%"  stop-color="#4b5c77" stop-opacity="0.6" />
                <stop offset="100%" stop-color="#334155" stop-opacity="0.4" />
            </linearGradient>

            {{-- LED glow: huge region, stacked merge for a real radial halo --}}
            <filter id="wf-glow" x="-300%" y="-800%" width="700%" height="1700%" color-interpolation-filters="linearRGB">
                <feGaussianBlur stdDeviation="2.8" result="blur" />
                <feMerge>
                    <feMergeNode in="blur" />
                    <feMergeNode in="blur" />
                    <feMergeNode in="SourceGraphic" />
                </feMerge>
            </filter>

            {{-- Pill drop-shadow inside the SVG so it rides the shape, not the bbox --}}
            <filter id="wf-shadow" x="-10%" y="-10%" width="120%" height="130%" color-interpolation-filters="linearRGB">
                <feDropShadow dx="0" dy="4" stdDeviation="4" flood-color="#000000" flood-opacity="0.4"/>
            </filter>
        </defs>

        <rect x="35" y="22" width="90" height="26" rx="13" fill="url(#wf-pill)" filter="url(#wf-shadow)" stroke="url(#wf-rim)" stroke-width="1.4" />
        <rect x="72" y="32" width="16" height="6" rx="3" fill="#00f6ff" filter="url(#wf-glow)" class="wf-led"/>

        <g fill="url(#wf-cyan)" class="wf-bars">
            <rect x="41.5" y="69" width="5" height="8"  rx="2.5" style="transform-origin: 44px 73px;  animation: wf-b 3.2s ease-in-out infinite; animation-delay: 0ms;"/>
            <rect x="50.5" y="64" width="5" height="18" rx="2.5" style="transform-origin: 53px 73px;  animation: wf-b 3.2s ease-in-out infinite; animation-delay: 200ms;"/>
            <rect x="59.5" y="59" width="5" height="28" rx="2.5" style="transform-origin: 62px 73px;  animation: wf-b 3.2s ease-in-out infinite; animation-delay: 400ms;"/>
            <rect x="68.5" y="54" width="5" height="38" rx="2.5" style="transform-origin: 71px 73px;  animation: wf-b 3.2s ease-in-out infinite; animation-delay: 580ms;"/>
            <path d="M 77.5 51.5 A 2.5 2.5 0 0 1 82.5 51.5 V 94 L 77.5 101 Z" style="transform-origin: 80px 76px; animation: wf-b 3.2s ease-in-out infinite; animation-delay: 720ms;"/>
            <rect x="86.5" y="54" width="5" height="38" rx="2.5" style="transform-origin: 89px 73px;  animation: wf-b 3.2s ease-in-out infinite; animation-delay: 580ms;"/>
            <rect x="95.5" y="59" width="5" height="28" rx="2.5" style="transform-origin: 98px 73px;  animation: wf-b 3.2s ease-in-out infinite; animation-delay: 400ms;"/>
            <rect x="104.5" y="64" width="5" height="18" rx="2.5" style="transform-origin: 107px 73px; animation: wf-b 3.2s ease-in-out infinite; animation-delay: 200ms;"/>
            <rect x="113.5" y="69" width="5" height="8"  rx="2.5" style="transform-origin: 116px 73px; animation: wf-b 3.2s ease-in-out infinite; animation-delay: 0ms;"/>
        </g>

        <style>
            @keyframes wf-b {
                0%, 100% { transform: scaleY(0.55); }
                50%      { transform: scaleY(1); }
            }
            .wf-led { animation: wf-led 3.6s ease-in-out infinite; }
            @keyframes wf-led {
                0%, 100% { opacity: 0.85; }
                50%      { opacity: 1; }
            }
            @media (prefers-reduced-motion: reduce) {
                .wf-bars rect, .wf-bars path, .wf-led { animation: none !important; }
            }
        </style>
    </svg>
</div>
