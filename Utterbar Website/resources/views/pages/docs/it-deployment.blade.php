<x-layouts.docs
    title="IT deployment"
    description="What an IT team needs to know before rolling out Utterbar across a fleet. Silent install, network egress, file locations, group policy, code-signing status. Honest answers.">

    <h1 class="ub-docs-h1">For your <x-hl>IT team</x-hl>.</h1>
    <p class="ub-lead mt-6">What Utterbar is, where it stores things, what it talks to, and how to deploy it. Written for the person reviewing the install before it touches a fleet. Honest about what we have and what we don&rsquo;t.</p>

    <h2 class="mt-14 ub-docs-h2">System requirements</h2>
    <ul class="mt-4 space-y-2 text-base leading-relaxed" style="color: var(--fg);">
        <li>Windows 10 22H2 or Windows 11, x86_64.</li>
        <li>WebView2 runtime (pre-installed on Windows 11; auto-bootstrapped by the installer on Windows 10).</li>
        <li>4 GB RAM minimum, 8 GB or more recommended.</li>
        <li>~500 MB disk for the app and the default speech model.</li>
        <li>Optional: a DirectX 12-capable GPU (any vendor) for GPU acceleration. Not required.</li>
    </ul>

    <h2 class="mt-14 ub-docs-h2">Installer</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Two installer formats ship for every release: NSIS (.exe) and MSI. Both are per-user installers (<code class="font-mono text-sm" style="color: var(--accent);">installMode: currentUser</code>): no admin rights required, no system services, no driver installs.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        The NSIS installer supports silent install via the standard <code class="font-mono text-sm" style="color: var(--accent);">/S</code> flag. The MSI is built for SCCM, Intune, PDQ Deploy, or any tool that prefers MSI. Pick whichever fits your packaging pipeline.
    </p>

    <div class="mt-6 ub-card" style="padding: 1rem 1.25rem;">
        <p class="text-sm" style="color: var(--muted); line-height: 1.55;">
            <strong style="color: var(--fg-hi);">Code-signing status, plainly.</strong>
            The public installer is not code-signed. End users see a Windows SmartScreen warning on first install. If your fleet requires a signed binary, Utterbar is open source, so you can build and sign it yourself with your own organisation&rsquo;s certificate from the same source we ship.
        </p>
    </div>

    <h3 id="installer-return-codes" class="mt-10 ub-docs-h3" style="font-size: 1.25rem; font-weight: 600; color: var(--fg-hi); letter-spacing: -0.015em;">Installer return codes</h3>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        The NSIS installer follows standard NSIS exit-code conventions. The codes below are what fleet-deployment tools (SCCM, Intune, PDQ, Microsoft Store) can rely on:
    </p>
    <table class="mt-4 w-full text-sm" style="color: var(--fg);">
        <thead>
            <tr style="border-bottom: 1px solid var(--border-strong);">
                <th class="py-2 pr-4 text-left ub-eyebrow" style="color: var(--muted);">Code</th>
                <th class="py-2 text-left ub-eyebrow" style="color: var(--muted);">Meaning</th>
            </tr>
        </thead>
        <tbody>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="font-family: var(--font-mono); color: var(--fg-hi);">0</td>
                <td class="py-3" style="color: var(--muted);">Install completed successfully.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="font-family: var(--font-mono); color: var(--fg-hi);">1223</td>
                <td class="py-3" style="color: var(--muted);">User cancelled the install (standard <code class="text-xs">ERROR_CANCELLED</code>).</td>
            </tr>
            <tr>
                <td class="py-3 pr-4" style="font-family: var(--font-mono); color: var(--fg-hi);">non-zero (other)</td>
                <td class="py-3" style="color: var(--muted);">Generic install failure. NSIS does not distinguish between disk-full, file-system-permission, or anti-virus quarantine; they all bubble up as a non-zero exit. Investigate via the system Event Log if a deployment tool reports a failure.</td>
            </tr>
        </tbody>
    </table>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--muted); font-size: 0.95rem;">
        The installer never requires a reboot, never prompts for elevation (per-user install), never registers a system service, and never makes a network call during install. Everything that touches the network happens after the user runs the app for the first time.
    </p>

    <h2 class="mt-14 ub-docs-h2">File locations</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Everything Utterbar writes lives under <code class="font-mono text-sm" style="color: var(--accent);">%APPDATA%\Utterbar\</code> per Windows user. The app does not write to <code class="font-mono text-sm" style="color: var(--accent);">Program Files</code>, <code class="font-mono text-sm" style="color: var(--accent);">ProgramData</code>, or anywhere outside the user&rsquo;s profile.
    </p>
    <table class="mt-5 w-full text-sm" style="color: var(--fg);">
        <tbody>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="color: var(--fg-hi); font-weight: 500; vertical-align: top;">Settings</td>
                <td class="py-3" style="font-family: var(--font-mono); font-size: 0.85rem;">%APPDATA%\Utterbar\config\settings.json</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="color: var(--fg-hi); font-weight: 500; vertical-align: top;">Speech models</td>
                <td class="py-3" style="font-family: var(--font-mono); font-size: 0.85rem;">%APPDATA%\Utterbar\models\</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="color: var(--fg-hi); font-weight: 500; vertical-align: top;">History database</td>
                <td class="py-3" style="font-family: var(--font-mono); font-size: 0.85rem;">%APPDATA%\Utterbar\data\history.db</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="color: var(--fg-hi); font-weight: 500; vertical-align: top;">Recordings (opt-in)</td>
                <td class="py-3" style="font-family: var(--font-mono); font-size: 0.85rem;">%APPDATA%\Utterbar\recordings\</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="color: var(--fg-hi); font-weight: 500; vertical-align: top;">Logs</td>
                <td class="py-3" style="font-family: var(--font-mono); font-size: 0.85rem;">%APPDATA%\Utterbar\logs\</td>
            </tr>
        </tbody>
    </table>

    <h2 class="mt-14 ub-docs-h2">What network access the app does and does not need</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Utterbar transcribes on the device. Audio, transcripts, vocabulary, and history all stay on the user&rsquo;s machine and never leave it. The app does make a small, fully enumerated set of outbound network calls. None of them carry transcribed content.
    </p>
    <table class="mt-5 w-full text-sm" style="color: var(--fg);">
        <thead>
            <tr style="border-bottom: 1px solid var(--border-strong);">
                <th class="py-2 pr-4 text-left ub-eyebrow" style="color: var(--muted);">Call</th>
                <th class="py-2 pr-4 text-left ub-eyebrow" style="color: var(--muted);">Endpoint</th>
                <th class="py-2 text-left ub-eyebrow" style="color: var(--muted);">When</th>
            </tr>
        </thead>
        <tbody>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="color: var(--fg-hi);">Update check</td>
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">api.github.com</td>
                <td class="py-3" style="color: var(--muted);">App launch, if auto-update is on.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="color: var(--fg-hi);">Model download</td>
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">github.com</td>
                <td class="py-3" style="color: var(--muted);">First run, and any time the user installs an additional model.</td>
            </tr>
            <tr>
                <td class="py-3 pr-4" style="color: var(--fg-hi);">Update binary</td>
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">github.com</td>
                <td class="py-3" style="color: var(--muted);">When the user clicks Install on the update toast.</td>
            </tr>
        </tbody>
    </table>
    <p class="mt-5 text-base leading-relaxed" style="color: var(--fg);">
        That is the full list. There is no telemetry, no analytics ping, no error-reporting beacon, no opt-out. Utterbar is free and never checks a licence, so a temporary network outage does not break dictation.
    </p>

    <h2 class="mt-14 ub-docs-h2">Air-gapped deployments</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Utterbar can run on a fully air-gapped machine. The model files have to reach the machine once (sneakernet, internal mirror, golden image). After that, dictation works indefinitely without network. There is no activation step: every feature is on by default.
    </p>

    <h2 class="mt-14 ub-docs-h2">Group policy / centrally managed settings</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        A small set of settings can be pinned by an IT admin via the Windows registry, and will override the per-user value in <code class="font-mono text-sm" style="color: var(--accent);">settings.json</code>. The keys live under:
    </p>
    <pre class="mt-4 p-4 rounded-input text-sm overflow-x-auto" style="background: var(--bg-2); color: var(--fg); font-family: var(--font-mono); line-height: 1.55;">HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Utterbar
HKEY_CURRENT_USER\SOFTWARE\Policies\Utterbar     (HKLM wins on conflict)</pre>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Currently pinnable values:
    </p>
    <table class="mt-4 w-full text-sm" style="color: var(--fg);">
        <thead>
            <tr style="border-bottom: 1px solid var(--border-strong);">
                <th class="py-2 pr-4 text-left ub-eyebrow" style="color: var(--muted);">Value</th>
                <th class="py-2 pr-4 text-left ub-eyebrow" style="color: var(--muted);">Type</th>
                <th class="py-2 text-left ub-eyebrow" style="color: var(--muted);">Effect</th>
            </tr>
        </thead>
        <tbody>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">UpdateChannel</td>
                <td class="py-3 pr-4" style="color: var(--muted);">REG_SZ</td>
                <td class="py-3" style="color: var(--muted);"><code class="text-xs">stable</code> or <code class="text-xs">beta</code>. Pins the channel; user UI becomes read-only.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">CheckUpdatesAutomatically</td>
                <td class="py-3 pr-4" style="color: var(--muted);">REG_DWORD</td>
                <td class="py-3" style="color: var(--muted);">0 or 1. Pins whether auto-update checks happen at launch.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">BlockModelDownloads</td>
                <td class="py-3 pr-4" style="color: var(--muted);">REG_DWORD</td>
                <td class="py-3" style="color: var(--muted);">0 or 1. When 1, the catalogue refuses any new model download. Use for environments where models must come from your internal mirror.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">BlockSystemAudio</td>
                <td class="py-3 pr-4" style="color: var(--muted);">REG_DWORD</td>
                <td class="py-3" style="color: var(--muted);">0 or 1. When 1, forces the capture source back to <code class="text-xs">Microphone</code> on every settings read. Use to disable system / loopback audio capture (call recording, meeting transcription) for compliance.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">BlockHistory</td>
                <td class="py-3 pr-4" style="color: var(--muted);">REG_DWORD</td>
                <td class="py-3" style="color: var(--muted);">0 or 1. When 1, no new history rows are persisted. Existing rows are not deleted. The policy is forward-only. Use for environments where transcript retention is out of scope.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">BlockHistoryExport</td>
                <td class="py-3 pr-4" style="color: var(--muted);">REG_DWORD</td>
                <td class="py-3" style="color: var(--muted);">0 or 1. When 1, the &ldquo;export history as JSON&rdquo; command refuses. History stays visible in the UI; only the off-device copy is disabled. <code class="text-xs">BlockHistory</code> implies this.</td>
            </tr>
            <tr>
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">BlockTelemetry</td>
                <td class="py-3 pr-4" style="color: var(--muted);">REG_DWORD</td>
                <td class="py-3" style="color: var(--muted);">0 or 1. When 1, the local on-device analytics rollup (Dashboard counters: words, dictations, time saved) does not bump on a successful dictation. There is no remote telemetry; this gates the local SQLite rollup only.</td>
            </tr>
        </tbody>
    </table>
    <div class="mt-5 ub-card" style="padding: 1rem 1.25rem;">
        <p class="text-sm" style="color: var(--muted); line-height: 1.55;">
            <strong style="color: var(--fg-hi);">Restart to apply.</strong>
            The policy is read once at app start. After pushing a change to the registry, the user must close and reopen Utterbar before the new value takes effect. This is the same contract Windows itself has for most managed policies, and matches what an MDM-pushed change typically expects.
        </p>
    </div>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        More keys land per request. The pattern is small (one new field, one registry read, one apply rule), so adding one more is a same-day change for us. Tell us what you need pinned by email, or open an issue.
    </p>

    <h2 class="mt-14 ub-docs-h2">Diagnostic info on the user&rsquo;s machine</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Settings &gt; Advanced &gt; Diagnostics shows the version, OS, CPU thread count, ONNX backend feature flag, and the on-disk paths for logs, the database, and models. Useful for the first-line IT support call. Logs are rotated automatically; the newest is always at <code class="font-mono text-sm" style="color: var(--accent);">%APPDATA%\Utterbar\logs\latest.log</code>.
    </p>

    <h2 class="mt-14 ub-docs-h2">What we do NOT have</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Saying so plainly:
    </p>
    <ul class="mt-4 space-y-2 text-base leading-relaxed" style="color: var(--fg);">
        <li>No SAML or SSO. There are no accounts on our side; nothing to sign in to.</li>
        <li>No SOC 2 report. The app is open source, so you can audit exactly what it does yourself.</li>
        <li>No cloud admin dashboard for IT. Deployment is driven entirely by the installer flags and registry policies documented above.</li>
        <li>No mobile clients. Windows-native only.</li>
    </ul>

    <h2 class="mt-14 ub-docs-h2">Talk to us</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        If your IT team has a question this page doesn&rsquo;t cover, email <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a>. The technical answers come from the same person who wrote the code.
    </p>
</x-layouts.docs>
