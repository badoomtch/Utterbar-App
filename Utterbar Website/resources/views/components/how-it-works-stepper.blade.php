@php
    $steps = [
        [
            'n' => '1',
            'title' => 'Press Ctrl + Space',
            'body' => 'The Utterbar wakes up. The LED pulses cyan. Works anywhere, on any monitor.',
            'kbd' => ['Ctrl', 'Space'],
        ],
        [
            'n' => '2',
            'title' => 'Speak naturally',
            'body' => 'The waveform moves. A local Parakeet model listens. Nothing leaves your PC.',
            'kbd' => null,
        ],
        [
            'n' => '3',
            'title' => 'Let go',
            'body' => 'Your text appears in the app under your cursor. No paste dialog. No copy step.',
            'kbd' => null,
        ],
    ];
@endphp

<div class="grid grid-cols-1 md:grid-cols-3 gap-5">
    @foreach ($steps as $step)
        <div class="ub-card h-full">
            <div class="flex items-baseline justify-between">
                <span class="font-mono text-xs tracking-widest" style="color: var(--accent);">STEP {{ $step['n'] }}</span>
                @if ($step['kbd'])
                    <x-kbd-chip :keys="$step['kbd']" />
                @endif
            </div>
            <h3 class="mt-4 text-xl">{{ $step['title'] }}</h3>
            <p class="mt-2 text-sm leading-relaxed" style="color: var(--muted);">{{ $step['body'] }}</p>
        </div>
    @endforeach
</div>
