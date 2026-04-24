<x-layouts.app title="EULA">
    <x-section eyebrow="Legal" title="End-user licence agreement">
        <article class="max-w-3xl space-y-5 leading-relaxed" style="color: var(--fg);">
            <p class="font-mono text-sm" style="color: var(--muted);">Last updated: {{ date('j F Y') }} · Placeholder — full EULA ships inside the 1.0 installer.</p>

            <h3>Licence grant</h3>
            <p>Utterbar grants you a personal, non-transferable, non-exclusive licence to install and use the Utterbar desktop application on any Windows machine you control, subject to the tier you've licensed (Free, Pro, or Team).</p>

            <h3>Restrictions</h3>
            <ul class="list-disc pl-5 space-y-1.5" style="color: var(--muted);">
                <li>No reverse-engineering except where permitted by law.</li>
                <li>No redistribution of the installer without written consent.</li>
                <li>No removal of attribution from the packaged model licences.</li>
            </ul>

            <h3>Third-party components</h3>
            <p>Utterbar includes NVIDIA Parakeet TDT (CC-BY-4.0), sherpa-onnx (Apache 2.0), and Tauri (Apache 2.0 / MIT). Their licences are reproduced in full in <code class="font-mono px-1 rounded" style="background: var(--bg-2); color: var(--accent);">LICENSE.txt</code> inside the installer.</p>

            <h3>Termination</h3>
            <p>This licence ends automatically if you breach it. On termination, uninstall the app; the local data is yours to keep or delete.</p>

            <h3>Liability</h3>
            <p>To the maximum extent permitted by law, Utterbar's total liability under this licence is limited to the amount you paid for it in the 12 months preceding the claim.</p>
        </article>
    </x-section>
</x-layouts.app>
