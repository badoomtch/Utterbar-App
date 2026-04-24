<x-layouts.docs title="FAQ">
    <h1 class="ub-docs-h1">Questions we <x-hl>always</x-hl> get.</h1>
    <p class="ub-lead mt-6">The exhaustive version of the home-page FAQ.</p>

    @php
        $items = [
            ['q' => 'Does Utterbar send my voice to the cloud?', 'a' => 'No. Audio is captured, transcribed, and binned on your PC.'],
            ['q' => 'Which Windows versions are supported?', 'a' => 'Windows 10 22H2 and Windows 11, x86_64. WebView2 is required (already on Windows 11; auto-bootstrapped on Windows 10).'],
            ['q' => 'Do I need a GPU?', 'a' => 'No. The default int8 Parakeet build (~460 MB) runs comfortably on CPU. Pro unlocks an fp16 build (~1.16 GB) that runs on any DX12 GPU via DirectML, or any NVIDIA card via CUDA.'],
            ['q' => 'What languages are supported?', 'a' => 'English only today. Multilingual is on the roadmap.'],
            ['q' => 'Does it work offline?', 'a' => 'Completely, once the model is downloaded on first run.'],
            ['q' => 'Can I use it in video-call and chat apps?', 'a' => 'Yes, for typing into the chat box. Utterbar never injects audio into a call.'],
            ['q' => 'What happens to my data when I uninstall?', 'a' => '<code>%APPDATA%\\Utterbar\\</code> is left in place so you can re-install without losing history. Delete the folder yourself for a clean wipe.'],
            ['q' => 'Will there be a macOS version?', 'a' => 'After Windows 1.0 ships. Mac already has decent local-only options; I\'d rather make Windows properly good first.'],
            ['q' => 'Will there be a Linux version?', 'a' => 'On the roadmap under "Later". No ETA.'],
            ['q' => 'Can I audit the code?', 'a' => 'Utterbar isn\'t open source (the paid features stay paid that way), but security researchers and enterprise buyers can request a source review under NDA. Email <a href="mailto:' . config('site.email.contact') . '" style="color: var(--accent);">' . config('site.email.contact') . '</a>.'],
            ['q' => 'How do I export my transcription history?', 'a' => 'Settings &rarr; History &rarr; Export. Produces a zip of every transcription (and audio, if you retained it).'],
            ['q' => 'Can I use it commercially?', 'a' => 'Yes, once Pro is licensed. Free is for personal use.'],
        ];
    @endphp

    <div class="mt-8"><x-faq-accordion :items="$items" /></div>
</x-layouts.docs>
