@props(['variant' => 'compact'])

@php
    // Hide the whole surface when Stripe tipping isn't configured (dev,
    // preview environments, or before the dashboard Price has been set).
    // Buttons that 404 are worse than no buttons.
    $enabled = (bool) config('services.stripe.enabled')
        && (bool) config('services.stripe.tip_price_id');
@endphp

@if ($enabled)
{{--
    Tip-jar surface.

    Designed to stand OUT, not blend in. The CSS variables `--accent`
    drive a soft glow ring + cyan-tinted background gradient so this
    block reads as "this is special, not just another section". On hover
    each £ pill flashes the accent border + a tiny scale lift so the
    interaction is unambiguous (previously the buttons had no hover
    affordance at all and felt dead).

    The four buttons each post a hidden `amount` value to /tip/checkout,
    which redirects straight to Stripe Checkout. No JS required for the
    basic flow. The "Custom" button posts no amount and Stripe shows
    its own amount picker.

    Heart icon (no SVG dep — single emoji glyph) gives the eye a third
    pull alongside the eyebrow and the cyan ring without spending more
    of the §1.6 cyan budget than the existing ring already costs.
--}}
<div class="ub-tip-jar relative overflow-hidden rounded-2xl p-6 md:p-7"
     style="
        background:
            radial-gradient(120% 80% at 0% 0%, color-mix(in oklab, var(--accent) 14%, transparent), transparent 60%),
            linear-gradient(180deg, color-mix(in oklab, var(--accent) 6%, var(--bg-1)) 0%, var(--bg-1) 100%);
        border: 1px solid color-mix(in oklab, var(--accent) 35%, var(--border));
        box-shadow:
            0 0 0 1px color-mix(in oklab, var(--accent) 25%, transparent) inset,
            0 12px 40px -12px color-mix(in oklab, var(--accent) 30%, transparent);
     ">
    {{-- Atmospheric corner wash, subtle — gives the card depth without
         repeating the radial gradient on the body. --}}
    <div aria-hidden="true" class="absolute inset-0 pointer-events-none"
         style="background: radial-gradient(80% 60% at 100% 100%, color-mix(in oklab, var(--accent) 10%, transparent), transparent 65%);"></div>

    <div class="relative flex flex-col gap-5 md:flex-row md:items-center md:justify-between">
        <div class="flex-1">
            <p class="ub-eyebrow" style="color: var(--accent);">
                Like it? Send a tip
            </p>
            <p class="mt-3 text-base font-medium leading-snug" style="color: var(--fg-hi); letter-spacing: -0.005em;">
                Utterbar is free. If it's earning its keep, a few quid keeps the next release moving.
            </p>
            <p class="mt-2 text-xs leading-relaxed" style="color: var(--muted);">
                Voluntary &middot; no perks &middot; no licence &middot; opens Stripe in your browser
            </p>
        </div>
        <div class="flex flex-wrap items-center gap-2 shrink-0">
            @foreach ([300, 500, 1000] as $pence)
                <form method="POST" action="{{ route('tip.checkout') }}">
                    @csrf
                    <input type="hidden" name="amount" value="{{ $pence }}">
                    <button type="submit" class="ub-tip-pill">
                        &pound;{{ number_format($pence / 100, 0) }}
                    </button>
                </form>
            @endforeach
            <form method="POST" action="{{ route('tip.checkout') }}">
                @csrf
                <button type="submit"
                        class="ub-btn ub-btn-cta"
                        style="font-size: 12px; padding: 0.5rem 1.1rem 0.5rem 1.95rem;">
                    Custom &rarr;
                </button>
            </form>
        </div>
    </div>
</div>
@endif

{{-- Tip-pill styles. Inlined here rather than in app.css so the whole
     tip-jar visual lives in one file — easy to edit without bouncing
     between Blade and the global stylesheet. The styles are scoped via
     `.ub-tip-pill` so they don't leak. --}}
@once
@push('head')
<style>
    .ub-tip-pill {
        font-family: var(--font-mono);
        font-size: 0.875rem;
        font-weight: 600;
        letter-spacing: 0.02em;
        color: var(--fg-hi);
        background: var(--bg-0);
        border: 1px solid color-mix(in oklab, var(--accent) 25%, var(--border));
        border-radius: 0.5rem;
        padding: 0.5rem 0.95rem;
        cursor: pointer;
        transition:
            transform 160ms ease,
            border-color 160ms ease,
            background 160ms ease,
            color 160ms ease,
            box-shadow 220ms ease;
    }

    .ub-tip-pill:hover {
        color: var(--accent);
        border-color: color-mix(in oklab, var(--accent) 75%, transparent);
        background: color-mix(in oklab, var(--accent) 10%, var(--bg-0));
        transform: translateY(-1px);
        box-shadow:
            0 0 0 3px color-mix(in oklab, var(--accent) 18%, transparent),
            0 6px 18px -8px color-mix(in oklab, var(--accent) 50%, transparent);
    }

    .ub-tip-pill:active {
        transform: translateY(0);
        box-shadow:
            0 0 0 2px color-mix(in oklab, var(--accent) 25%, transparent);
    }

    .ub-tip-pill:focus-visible {
        outline: 2px solid var(--accent);
        outline-offset: 2px;
    }
</style>
@endpush
@endonce
