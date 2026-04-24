@props([
    'title' => 'Confirm',
    'description' => null,
    'confirmLabel' => 'Confirm',
    'cancelLabel'  => 'Cancel',
    'action',
    'method' => 'POST',
    'variant' => 'danger',   // danger | accent
    'triggerLabel' => 'Open',
    'triggerClass' => '',
    'triggerStyle' => '',
])

@php
    // Button colour for the confirm action. Danger = red; accent = cyan.
    $confirmBg = $variant === 'accent' ? 'var(--accent)' : 'var(--danger)';
    $confirmFg = $variant === 'accent' ? 'var(--bg-0)'   : '#ffffff';
@endphp

<div x-data="{ open: false }" class="inline-block">
    <button type="button" @click="open = true"
            class="{{ $triggerClass ?: 'shrink-0 inline-flex items-center rounded-lg' }}"
            style="{{ $triggerStyle ?: 'padding: 0.35rem 0.7rem; font-size: 0.72rem; font-weight: 500; background: transparent; border: 1px solid var(--border-strong); color: var(--muted);' }}">
        {{ $triggerLabel }}
    </button>

    <template x-teleport="body">
        <div x-show="open" x-cloak
             @keydown.escape.window="open = false"
             role="dialog" aria-modal="true"
             style="position: fixed; inset: 0; z-index: 60; display: flex; align-items: center; justify-content: center; padding: 1rem;">

            {{-- Backdrop --}}
            <div x-show="open"
                 x-transition.opacity.duration.150ms
                 @click="open = false"
                 style="position: absolute; inset: 0; background: rgba(5, 8, 13, 0.72); backdrop-filter: blur(6px); -webkit-backdrop-filter: blur(6px);"></div>

            {{-- Card --}}
            <div x-show="open"
                 x-transition:enter="transition ease-out duration-200"
                 x-transition:enter-start="opacity-0 transform scale-95"
                 x-transition:enter-end="opacity-100 transform scale-100"
                 x-transition:leave="transition ease-in duration-150"
                 x-transition:leave-start="opacity-100 transform scale-100"
                 x-transition:leave-end="opacity-0 transform scale-95"
                 class="ub-card relative"
                 style="width: 100%; max-width: 440px; padding: 1.75rem 1.75rem 1.5rem;">

                <h3 style="color: var(--fg-hi); font-size: 1.25rem; font-weight: 600; letter-spacing: -0.015em; line-height: 1.25;">
                    {{ $title }}
                </h3>

                @if ($description)
                    <p class="mt-3 text-sm" style="color: var(--muted); line-height: 1.55;">
                        {!! $description !!}
                    </p>
                @endif

                {{-- Custom body content (optional) --}}
                @if (trim($slot->toHtml()))
                    <div class="mt-4">
                        {{ $slot }}
                    </div>
                @endif

                <div class="mt-6 flex items-center justify-end gap-2.5">
                    <button type="button" @click="open = false"
                            class="inline-flex items-center rounded-lg"
                            style="padding: 0.55rem 1rem; font-size: 0.875rem; font-weight: 500;
                                   background: transparent; border: 1px solid var(--border-strong); color: var(--fg);">
                        {{ $cancelLabel }}
                    </button>

                    <form method="POST" action="{{ $action }}" class="inline">
                        @csrf
                        @if (strtoupper($method) !== 'POST')
                            @method($method)
                        @endif
                        <button type="submit"
                                class="inline-flex items-center rounded-lg"
                                style="padding: 0.55rem 1.1rem; font-size: 0.875rem; font-weight: 600; letter-spacing: -0.005em;
                                       background: {{ $confirmBg }}; color: {{ $confirmFg }}; border: 1px solid {{ $confirmBg }};">
                            {{ $confirmLabel }}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    </template>
</div>
