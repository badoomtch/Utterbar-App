@props(['wave' => false])

@php
    // The hover-reveal waveform is opt-in. Pass `:wave="true"` on the
    // home-page hero brand plate (and anywhere else the brand stands
    // alone large enough to carry the affordance). The animation is
    // pure SVG <animate>, so it costs no JS, and the parent uses
    // group-hover via the .ub-wm-hoverwrap wrapper for the reveal.
@endphp

@if ($wave)
    <span {{ $attributes->merge(['class' => 'ub-wm ub-wm-hoverwrap']) }} tabindex="0">
        <span class="ub-wm-utter">Utter</span><span class="ub-wm-bar">bar</span><svg class="ub-wm-wave" viewBox="0 0 60 24" aria-hidden="true">
            <defs>
                <linearGradient id="ub-wm-wave-grad" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" stop-color="#00e5ff"/>
                    <stop offset="100%" stop-color="#00a3ff"/>
                </linearGradient>
            </defs>
            <g fill="url(#ub-wm-wave-grad)">
                <rect x="2" y="9" width="3" height="6" rx="1.5"><animate attributeName="height" values="6;14;6" dur="1.1s" begin="0s" repeatCount="indefinite"/><animate attributeName="y" values="9;5;9" dur="1.1s" begin="0s" repeatCount="indefinite"/></rect>
                <rect x="10" y="6" width="3" height="12" rx="1.5"><animate attributeName="height" values="12;20;12" dur="1.0s" begin="0.1s" repeatCount="indefinite"/><animate attributeName="y" values="6;2;6" dur="1.0s" begin="0.1s" repeatCount="indefinite"/></rect>
                <rect x="18" y="3" width="3" height="18" rx="1.5"><animate attributeName="height" values="18;22;18" dur="0.9s" begin="0.05s" repeatCount="indefinite"/><animate attributeName="y" values="3;1;3" dur="0.9s" begin="0.05s" repeatCount="indefinite"/></rect>
                <rect x="26" y="1" width="3" height="22" rx="1.5"><animate attributeName="height" values="22;14;22" dur="1.2s" begin="0.2s" repeatCount="indefinite"/><animate attributeName="y" values="1;5;1" dur="1.2s" begin="0.2s" repeatCount="indefinite"/></rect>
                <rect x="34" y="3" width="3" height="18" rx="1.5"><animate attributeName="height" values="18;10;18" dur="1.0s" begin="0.15s" repeatCount="indefinite"/><animate attributeName="y" values="3;7;3" dur="1.0s" begin="0.15s" repeatCount="indefinite"/></rect>
                <rect x="42" y="6" width="3" height="12" rx="1.5"><animate attributeName="height" values="12;18;12" dur="0.95s" begin="0.25s" repeatCount="indefinite"/><animate attributeName="y" values="6;3;6" dur="0.95s" begin="0.25s" repeatCount="indefinite"/></rect>
                <rect x="50" y="9" width="3" height="6" rx="1.5"><animate attributeName="height" values="6;14;6" dur="1.1s" begin="0.3s" repeatCount="indefinite"/><animate attributeName="y" values="9;5;9" dur="1.1s" begin="0.3s" repeatCount="indefinite"/></rect>
            </g>
        </svg>
    </span>
@else
    <span {{ $attributes->merge(['class' => 'ub-wm']) }}><span class="ub-wm-utter">Utter</span><span class="ub-wm-bar">bar</span></span>
@endif
