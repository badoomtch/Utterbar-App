<x-layouts.app title="Sign in" :no-index="true">
    <section class="pt-20 md:pt-28 pb-24">
        <div class="ub-container">
            <div class="max-w-md mx-auto">
                <h1 class="ub-fg-hi text-center" style="font-size: clamp(2.25rem, 4vw, 3rem); letter-spacing: -0.035em; line-height: 1.05;">
                    Sign in.
                </h1>
                <p class="ub-lead mt-5 text-center" style="font-size: 1rem;">
                    We&rsquo;ll email you a one-tap sign-in link. No passwords, ever.
                </p>

                <form method="POST" action="{{ route('sign-in.send') }}" class="mt-8 ub-card" style="padding: 1.75rem;">
                    @csrf

                    <label for="si-email" class="ub-eyebrow mb-2 block">Your email</label>
                    <input id="si-email" name="email" type="email" required autocomplete="email" autofocus
                           value="{{ old('email') }}"
                           placeholder="you@example.com"
                           class="w-full rounded-xl px-4 py-3"
                           style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">

                    @error('email')
                        <p class="mt-3 text-sm" style="color: var(--danger);">{{ $message }}</p>
                    @enderror

                    <button type="submit" class="ub-btn ub-btn-cta w-full mt-5">
                        Send me a sign-in link
                    </button>
                </form>

                <p class="mt-8 text-center text-sm" style="color: var(--muted);">
                    No account yet?
                    <a href="{{ route('pricing') }}" style="color: var(--accent);">Buy Utterbar Pro</a>
                    and we&rsquo;ll set one up for you automatically.
                </p>
            </div>
        </div>
    </section>
</x-layouts.app>
