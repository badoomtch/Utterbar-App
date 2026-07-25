{{--
    Cookie consent banner — minimal UK GDPR / PECR compliant prompt.

    Gating model:
      • Until the user has chosen, NO non-essential cookies / tags fire
        (the gtag.js block in layouts/app.blade.php checks this cookie
        before loading).
      • "Accept" sets `utterbar_consent = granted` for 12 months.
      • "Essential only" sets `utterbar_consent = essential` for 12
        months. Same effect as no choice (no GA), but stops the banner
        nagging the user on every page.
      • Footer "Cookies" link clears the cookie and reloads, which
        reopens the banner so the user can revise.

    Cookie name + format chosen so the GA gate can be a one-line check
    in the head — Blade template reads $_COOKIE directly, no PHP class
    or middleware needed.
--}}
<div
    x-data="cookieConsent()"
    x-init="init()"
    x-cloak
    x-show="visible"
    x-transition:enter="transition ease-out duration-200"
    x-transition:enter-start="opacity-0 translate-y-4"
    x-transition:enter-end="opacity-100 translate-y-0"
    role="dialog"
    aria-labelledby="cookie-consent-title"
    aria-describedby="cookie-consent-body"
    class="fixed left-4 right-4 bottom-4 z-50 mx-auto max-w-2xl rounded-xl border p-5 shadow-lg"
    style="background: var(--bg-1); border-color: var(--border);"
>
    <div class="flex flex-col gap-3 md:flex-row md:items-start md:gap-5">
        <div class="flex-1">
            <p id="cookie-consent-title" class="text-sm font-medium" style="color: var(--fg-hi);">
                Cookies on utterbar.com
            </p>
            <p id="cookie-consent-body" class="mt-1.5 text-xs leading-relaxed" style="color: var(--muted);">
                We use essential cookies so this site works (form security and your consent choice). With your permission we'd also load Google Analytics so we can see which pages are useful. Your choice is saved for 12 months.
                <a href="{{ route('legal.privacy') }}" style="color: var(--accent);">Privacy policy</a>.
            </p>
        </div>
        <div class="flex shrink-0 items-center gap-2">
            <button
                type="button"
                @click="setChoice('essential')"
                class="text-xs px-3 py-2 rounded-md transition-colors"
                style="color: var(--muted); border: 1px solid var(--border);"
            >
                Essential only
            </button>
            {{-- Padding matches the .ub-btn-cta design defaults
                 proportionally — the button has an absolute-positioned
                 cyan LED dot at left:1.05rem, so left padding must be
                 ≥1.85rem or the dot covers the first letter ("A" of
                 Accept got eaten in the original tighter version). --}}
            <button
                type="button"
                @click="setChoice('granted')"
                class="ub-btn ub-btn-cta"
                style="font-size: 12px; padding: 0.5rem 1rem 0.5rem 1.85rem;"
            >
                Accept
            </button>
        </div>
    </div>
</div>

@push('scripts')
<script>
function cookieConsent() {
    return {
        visible: false,
        init() {
            this.visible = !this.readChoice();
        },
        readChoice() {
            const m = document.cookie.match(/(?:^|; )utterbar_consent=([^;]+)/);
            return m ? decodeURIComponent(m[1]) : null;
        },
        setChoice(value) {
            // 12 months. SameSite=Lax is the right default for a single-
            // origin first-party cookie like this — no cross-site posts
            // need to read it.
            const oneYear = 60 * 60 * 24 * 365;
            document.cookie = 'utterbar_consent=' + encodeURIComponent(value)
                + '; Max-Age=' + oneYear
                + '; Path=/; SameSite=Lax; Secure';
            this.visible = false;
            // Reload so the gtag block in <head> picks up the new
            // consent on the next page render. Cleaner than dynamically
            // injecting the GA script after the fact.
            if (value === 'granted') {
                window.location.reload();
            }
        },
    };
}

// Footer "Cookies" revisit link — clears the cookie + reloads.
window.utterbarRevisitCookieConsent = function () {
    document.cookie = 'utterbar_consent=; Max-Age=0; Path=/; SameSite=Lax; Secure';
    window.location.reload();
};
</script>
@endpush
