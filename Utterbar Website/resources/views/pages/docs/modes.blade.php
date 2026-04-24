<x-layouts.docs title="Modes">
    <h1 class="ub-docs-h1">Modes that <x-hl>match</x-hl> the job.</h1>
    <p class="ub-lead mt-6">A mode bundles a post-processing prompt and a formatting rule-set. Change mode and the output shape changes without you touching anything else.</p>

    <div class="mt-10 space-y-6">
        <div>
            <h2 class="text-2xl">The six built-in modes</h2>
            <div class="mt-5 ub-card p-0 overflow-hidden">
                <table class="w-full text-sm">
                    <thead>
                        <tr style="background: var(--bg-2); color: var(--fg-hi);">
                            <th class="text-left p-3 font-semibold">Mode</th>
                            <th class="text-left p-3 font-semibold">Post-processing</th>
                            <th class="text-left p-3 font-semibold">What it's for</th>
                        </tr>
                    </thead>
                    <tbody>
                        @foreach ([
                            ['Voice-to-text', 'None (pass-through)',          'Just types what you said. Default.'],
                            ['Message',       'Deterministic template',       'Casual tone, sentence cap, no filler.'],
                            ['Email',         'LLM (deterministic fallback)', 'Polite, well-formatted email.'],
                            ['Note',          'LLM (deterministic fallback)', 'Bullets (dash style default).'],
                            ['Meeting',       'Deterministic, preserves \\n', 'Verbatim capture; keeps line breaks.'],
                            ['Super',         'LLM + context',                 'Uses active-app / selection / clipboard, if permitted.'],
                        ] as [$name, $proc, $use])
                            <tr class="border-t" style="border-color: var(--border);">
                                <td class="p-3 align-top font-medium" style="color: var(--fg-hi);">{{ $name }}</td>
                                <td class="p-3 align-top" style="color: var(--muted);">{{ $proc }}</td>
                                <td class="p-3 align-top" style="color: var(--fg);">{{ $use }}</td>
                            </tr>
                        @endforeach
                    </tbody>
                </table>
            </div>
        </div>

        <div>
            <h2 class="text-2xl">LLM post-processing</h2>
            <p class="mt-3" style="color: var(--muted);">Email, Note and Super call the local llama.cpp pipeline on Pro. Builds compiled without the llama feature use a deterministic fallback that gets you 80% of the way there. Either way, no cloud call.</p>
        </div>

        <div>
            <h2 class="text-2xl">Custom modes (Pro)</h2>
            <p class="mt-3" style="color: var(--muted);">Clone a built-in, edit the prompt, set a per-mode hotkey, and decide which context the mode is allowed to read (active-app / selection / clipboard).</p>
        </div>
    </div>

    <div class="mt-12 ub-card"><p class="text-sm" style="color: var(--muted);">Full per-mode guides with examples land with 1.0.</p></div>
</x-layouts.docs>
