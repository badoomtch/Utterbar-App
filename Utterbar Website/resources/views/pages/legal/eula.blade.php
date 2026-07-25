{{-- noIndex while this page is a placeholder. The real EULA ships inside
     the 1.0 installer; until then, the page is here for anyone who wants
     to read the licence terms, but we don't want it indexed as canonical
     legal text. Re-index when the full EULA lands. --}}
<x-layouts.app title="EULA" :noIndex="true">
    <x-section eyebrow="Legal" title="End-user licence agreement">
        <article class="max-w-3xl space-y-5 leading-relaxed" style="color: var(--fg);">
            <p class="font-mono text-sm" style="color: var(--muted);">Last updated: {{ date('j F Y') }} · Placeholder, full EULA ships inside the 1.0 installer.</p>

            <h3>Licence grant</h3>
            <p>Utterbar is free and open source. You may install and use the Utterbar desktop application on any Windows machine you control, for personal or commercial use. Your use of the source code is governed by the open-source licence included with it.</p>

            <h3>Restrictions</h3>
            <ul class="list-disc pl-5 space-y-1.5" style="color: var(--muted);">
                <li>No removal of attribution from the packaged model and component licences.</li>
                <li>Redistribution and modification are governed by the open-source licence included with the source code.</li>
            </ul>

            <h3>Third-party components</h3>
            <p>Utterbar includes NVIDIA Parakeet TDT (CC-BY-4.0), sherpa-onnx (Apache 2.0), and Tauri (Apache 2.0 / MIT). Their licences are reproduced in full in <code class="font-mono px-1 rounded" style="background: var(--bg-2); color: var(--accent);">LICENSE.txt</code> inside the installer.</p>

            <h3>Warranty and liability</h3>
            <p>Utterbar is provided &ldquo;as is&rdquo;, without warranty of any kind. To the maximum extent permitted by law, and because it is provided free of charge, Utterbar&rsquo;s total liability arising from your use of the app is limited to &pound;50.</p>

            <h3>Termination</h3>
            <p>You may stop using Utterbar at any time. On termination, uninstall the app; the local data is yours to keep or delete.</p>
        </article>
    </x-section>
</x-layouts.app>
