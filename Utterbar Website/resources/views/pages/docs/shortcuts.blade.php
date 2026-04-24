<x-layouts.docs title="Shortcuts">
    <h1 class="ub-docs-h1">Every <x-hl>hotkey</x-hl>, your way.</h1>
    <p class="ub-lead mt-6">Seven hotkey slots. One default binding. The rest are unbound on install &mdash; pick what your fingers already know.</p>

    <div class="mt-10 ub-card p-0 overflow-hidden">
        <table class="w-full text-sm">
            <thead>
                <tr style="background: var(--bg-2); color: var(--fg-hi);">
                    <th class="text-left p-3 font-semibold">Slot</th>
                    <th class="text-left p-3 font-semibold">Default</th>
                    <th class="text-left p-3 font-semibold">Trigger style</th>
                </tr>
            </thead>
            <tbody>
                @foreach ([
                    ['ToggleRecording', ['Ctrl','Space'], 'Toggle'],
                    ['PushToTalk',      null,            'PushToTalk'],
                    ['HoldToTalk',      null,            'HoldToTalk'],
                    ['CancelRecording', null,            'Toggle'],
                    ['CycleMode',       null,            'Toggle'],
                    ['OpenSettings',    null,            'Toggle'],
                    ['OpenHistory',     null,            'Toggle'],
                ] as [$slot, $keys, $style])
                    <tr class="border-t" style="border-color: var(--border);">
                        <td class="p-3 align-middle font-mono text-xs" style="color: var(--fg-hi);">{{ $slot }}</td>
                        <td class="p-3 align-middle">
                            @if ($keys)
                                <x-kbd-chip :keys="$keys" />
                            @else
                                <span class="font-mono text-xs" style="color: var(--muted);">unbound</span>
                            @endif
                        </td>
                        <td class="p-3 align-middle" style="color: var(--muted);">{{ $style }}</td>
                    </tr>
                @endforeach
            </tbody>
        </table>
    </div>

    <p class="mt-6 text-sm" style="color: var(--muted);">
        Every slot is rebindable from Settings &rarr; Shortcuts. Per-mode hotkey overrides live in the mode editor (Pro).
    </p>

    <div class="mt-8">
        <h2 class="text-2xl">Trigger styles</h2>
        <ul class="mt-4 space-y-3 text-sm">
            <li class="flex gap-3"><span style="color: var(--accent);"><x-icon name="check" size="16" stroke="2" /></span><span><strong style="color: var(--fg-hi);">Toggle</strong> &mdash; <span style="color: var(--muted);">press once to start, press again to stop.</span></span></li>
            <li class="flex gap-3"><span style="color: var(--accent);"><x-icon name="check" size="16" stroke="2" /></span><span><strong style="color: var(--fg-hi);">PushToTalk</strong> &mdash; <span style="color: var(--muted);">press-and-release to capture a single burst, useful for short messages.</span></span></li>
            <li class="flex gap-3"><span style="color: var(--accent);"><x-icon name="check" size="16" stroke="2" /></span><span><strong style="color: var(--fg-hi);">HoldToTalk</strong> &mdash; <span style="color: var(--muted);">records while held, stops on release. Walkie-talkie style.</span></span></li>
        </ul>
    </div>
</x-layouts.docs>
