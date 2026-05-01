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
            The public installer is not code-signed. End users see a Windows SmartScreen warning on first install. For any Business or Enterprise order where SmartScreen warnings are unacceptable on a fleet rollout, we sign the installer at order time. The signing-cert cost is folded into the quote, not invented as a separate line. Mention &ldquo;signed installer&rdquo; in your enquiry and we&rsquo;ll include it.
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
            <tr>
                <td class="py-3 pr-4" style="color: var(--fg-hi); font-weight: 500; vertical-align: top;">Licence cache</td>
                <td class="py-3" style="font-family: var(--font-mono); font-size: 0.85rem;">Windows Credential Manager + %APPDATA%\Utterbar\licence.json (DPAPI-encrypted)</td>
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
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">utterbar.com</td>
                <td class="py-3" style="color: var(--muted);">App launch, if auto-update is on.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="color: var(--fg-hi);">Licence activate / heartbeat</td>
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">utterbar.com</td>
                <td class="py-3" style="color: var(--muted);">Once on activation; periodic heartbeat thereafter.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="color: var(--fg-hi);">Model download</td>
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">github.com / dl.utterbar.com</td>
                <td class="py-3" style="color: var(--muted);">First run, and any time the user installs an additional model.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="color: var(--fg-hi);">Pricing lookup</td>
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">utterbar.com</td>
                <td class="py-3" style="color: var(--muted);">When the in-app upgrade modal opens.</td>
            </tr>
            <tr>
                <td class="py-3 pr-4" style="color: var(--fg-hi);">Update binary</td>
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">dl.utterbar.com</td>
                <td class="py-3" style="color: var(--muted);">When the user clicks Install on the update toast.</td>
            </tr>
        </tbody>
    </table>
    <p class="mt-5 text-base leading-relaxed" style="color: var(--fg);">
        That is the full list. There is no telemetry, no analytics ping, no error-reporting beacon, no opt-out. The app does not phone home for activation status, so a temporary network outage does not break dictation.
    </p>

    <h2 class="mt-14 ub-docs-h2">Air-gapped deployments</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Utterbar can run on a fully air-gapped machine. The model files have to reach the machine once (sneakernet, internal mirror, golden image). After that, dictation works indefinitely without network.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Air-gapped <em>activation</em> (sign a request file offline, ingest a signed activation file) is supported on request for Business and Enterprise customers. Mention it in your enquiry and we will include the activation files with the quote.
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
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">RequireLicence</td>
                <td class="py-3 pr-4" style="color: var(--muted);">REG_DWORD</td>
                <td class="py-3" style="color: var(--muted);">0 or 1. When 1, the app refuses to dictate unless a Pro licence is active on the machine. The free tier becomes unavailable. Use when only the paid SKU is approved on managed devices.</td>
            </tr>
            <tr style="border-bottom: 1px solid var(--border);">
                <td class="py-3 pr-4" style="font-family: var(--font-mono); font-size: 0.85rem;">BlockHistory</td>
                <td class="py-3 pr-4" style="color: var(--muted);">REG_DWORD</td>
                <td class="py-3" style="color: var(--muted);">0 or 1. When 1, no new history rows are persisted. Existing rows are not deleted &mdash; the policy is forward-only. Use for environments where transcript retention is out of scope.</td>
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
        More keys land per customer request. The pattern is small (one new field, one registry read, one apply rule), so adding one more is a same-day change for us. Tell us what you need pinned in the enquiry.
    </p>

    <h2 class="mt-14 ub-docs-h2">Licensing</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        A licence is a single key. One licence covers N seats, where N is the seat count you bought. Each seat consumes one device-activation slot, identified by a SHA-256 hash of the device&rsquo;s fingerprint (machine GUID + hostname + primary MAC). Activations include a friendly device name the user picks at install time.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Licence keys are verified locally with Ed25519 signatures baked into the desktop app. The app does not phone home for activation status. If our servers go down, your existing installs keep working.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Today, the licence is administered through your <a href="{{ route('account') }}" style="color: var(--accent);">account</a> on utterbar.com (sign in with the email the licence was issued to). The account view shows every active device, when it was last seen, and lets you deactivate any seat. For team rollouts at 25+ seats we work with you directly to mint and rotate licences as your headcount changes.
    </p>

    <h3 id="licence-delivery-for-managed-fleets" class="mt-10 ub-docs-h3" style="font-size: 1.25rem; font-weight: 600; color: var(--fg-hi); letter-spacing: -0.015em;">Licence delivery for managed fleets</h3>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        For fleet rollouts where the end user shouldn&rsquo;t need to paste a licence key into the app, the licence can be delivered as a signed file on disk. Drop a single binary at:
    </p>
    <pre class="mt-4 p-4 rounded-input text-sm overflow-x-auto" style="background: var(--bg-2); color: var(--fg); font-family: var(--font-mono); line-height: 1.55;">%APPDATA%\Utterbar\licence.bin</pre>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Utterbar reads the file at startup, verifies the Ed25519 signature against the public keys baked into the desktop app, and treats the result as the active Pro licence on that machine. The user never sees an activation dialog. Removal is &ldquo;delete the file&rdquo;.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        How to obtain the file:
    </p>
    <ul class="mt-3 space-y-2 text-base leading-relaxed" style="color: var(--fg);">
        <li>Sign in to the admin panel on utterbar.com and open the licence detail page for the seat you want to deploy.</li>
        <li>Click <strong style="color: var(--fg-hi);">Download licence.bin</strong>. The file served is the same Ed25519-signed blob the desktop app already verifies offline; no separate trust model.</li>
        <li>Rename the download to <code class="font-mono text-sm" style="color: var(--accent);">licence.bin</code> and ship it to the target machine via your MDM, GPO, or whatever fleet pipeline you already run for app config files.</li>
    </ul>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Behaviour notes worth knowing before you deploy:
    </p>
    <ul class="mt-3 space-y-2 text-base leading-relaxed" style="color: var(--fg);">
        <li><strong style="color: var(--fg-hi);">Managed wins.</strong> If a verified <code class="font-mono text-sm" style="color: var(--accent);">licence.bin</code> is present, it overrides any user-pasted licence cache on the same machine. The user can&rsquo;t accidentally downgrade a managed deployment by pasting a personal key.</li>
        <li><strong style="color: var(--fg-hi);">Managed-but-invalid fails closed.</strong> If the file is present but corrupt, the wrong length, or signed with a key the app doesn&rsquo;t recognise, the machine reports as Free, a warning lands in the app log, and paste-flow activation is also blocked. We deliberately do <em>not</em> silently fall back to whatever the user happened to paste, AND we don&rsquo;t let the user paste a personal key as an override &mdash; an admin who deployed a broken file gets a clear &ldquo;no Pro&rdquo; signal until they fix or remove the file. Fix is &ldquo;push a valid file&rdquo; or &ldquo;delete the file&rdquo;, never &ldquo;ask the user to enter a key&rdquo;.</li>
        <li><strong style="color: var(--fg-hi);">Activation / deactivation are disabled in-app.</strong> The Settings &rarr; Licence panel renders a short &ldquo;Managed by your administrator&rdquo; notice instead of the activation form. Removal is &ldquo;delete the file&rdquo;; you don&rsquo;t need the user to click anything.</li>
        <li><strong style="color: var(--fg-hi);">No DPAPI wrapper.</strong> The file is the raw 96-byte signed blob. The bytes aren&rsquo;t secret in the cryptographic sense (the signature, not encryption, is what makes them trustworthy), so we don&rsquo;t machine-bind the licence. That&rsquo;s also what lets you stamp it into a golden image.</li>
    </ul>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Combined with <code class="font-mono text-sm" style="color: var(--accent);">RequireLicence</code> in the policy table above, this gives you a managed fleet where Utterbar runs as Pro on every machine, the free tier is unreachable, and no one ever sees an activation prompt.
    </p>

    <h3 id="air-gapped-activation" class="mt-10 ub-docs-h3" style="font-size: 1.25rem; font-weight: 600; color: var(--fg-hi); letter-spacing: -0.015em;">Air-gapped activation</h3>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        For machines that cannot reach <code class="font-mono text-sm" style="color: var(--accent);">utterbar.com</code> at all &mdash; air-gapped lab kit, restricted-network admin VMs, classified-environment workstations &mdash; Utterbar supports a fully offline activation flow. Six steps, no network from the offline machine:
    </p>
    <ol class="mt-4 space-y-2 list-decimal list-inside text-base leading-relaxed" style="color: var(--fg);">
        <li>On the offline machine, open <strong style="color: var(--fg-hi);">Settings &rarr; Licence</strong> and click <strong style="color: var(--fg-hi);">Export activation request</strong>. Utterbar writes a small JSON file (around 250 bytes) containing the machine fingerprint, device name, app version and an RFC 3339 timestamp.</li>
        <li>Hand-carry the file to your IT administrator. Email, USB stick, internal share &mdash; whatever your network policy allows. The file is not a secret; it carries no licence material, only the machine&rsquo;s identity.</li>
        <li>The administrator signs in to the admin panel at <code class="font-mono text-sm" style="color: var(--accent);">utterbar.com/admin</code>, opens the licence detail page, and uses the <strong style="color: var(--fg-hi);">Air-gapped activation</strong> upload form to submit the JSON. The server validates the format, counts the device against <code class="font-mono text-sm" style="color: var(--accent);">max_devices</code>, and records an activation row marked <code class="font-mono text-sm" style="color: var(--accent);">offline</code>.</li>
        <li>The administrator clicks the existing <strong style="color: var(--fg-hi);">Download licence.bin</strong> button on the same page. The file served is the same 96-byte Ed25519-signed blob every other holder of this licence gets.</li>
        <li>Send the <code class="font-mono text-sm" style="color: var(--accent);">licence.bin</code> back to the offline machine via the same out-of-band channel.</li>
        <li>On the offline machine, open <strong style="color: var(--fg-hi);">Settings &rarr; Licence</strong> again and click <strong style="color: var(--fg-hi);">Import licence.bin</strong>. Utterbar verifies the signature offline, copies the file to <code class="font-mono text-sm" style="color: var(--accent);">%APPDATA%\Utterbar\licence.bin</code>, and asks you to restart. Restart Utterbar; the next launch picks the file up via the same managed-licence loader the fleet flow uses.</li>
    </ol>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        Limitations worth being explicit about &mdash; this version does the smallest useful thing, and your security team should know the edges:
    </p>
    <ul class="mt-3 space-y-2 text-base leading-relaxed" style="color: var(--fg);">
        <li><strong style="color: var(--fg-hi);">The licence.bin is not machine-bound in this version.</strong> The file the admin sends back is the same bytes for every device on the licence. A user who copies the file to a second offline machine could technically activate it there too. Treat the admin upload as <em>audit and max-device intent</em>, not as a cryptographic binding. If you need hard one-machine-one-file enforcement, that&rsquo;s a future v3 payload format we haven&rsquo;t shipped yet &mdash; mention it in your enquiry.</li>
        <li><strong style="color: var(--fg-hi);">Offline devices do not heartbeat.</strong> The activation row carries <code class="font-mono text-sm" style="color: var(--accent);">last_heartbeat = null</code> permanently. The admin licence page renders a small <code class="font-mono text-sm" style="color: var(--accent);">offline</code> chip alongside such rows so you can tell at a glance which devices are reachable via heartbeat and which aren&rsquo;t.</li>
        <li><strong style="color: var(--fg-hi);">Revocation only takes effect on devices that contact the server.</strong> If you revoke the licence in the admin panel, online devices flip to Free on their next heartbeat (a 410 from <code class="font-mono text-sm" style="color: var(--accent);">/api/licence/heartbeat</code>). Air-gapped devices that never contact the server keep running until you replace or remove their <code class="font-mono text-sm" style="color: var(--accent);">licence.bin</code> on disk.</li>
        <li><strong style="color: var(--fg-hi);">The request file is not signed.</strong> Nothing in it is a secret &mdash; the fingerprint already leaks via the existing online activation path. The administrator&rsquo;s upload-validation step is the trust gate, not a request-side signature.</li>
        <li><strong style="color: var(--fg-hi);">Idempotent uploads.</strong> Re-uploading the same JSON for the same device is a no-op &mdash; useful when an admin is unsure whether the first upload landed.</li>
    </ul>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        Combined with <code class="font-mono text-sm" style="color: var(--accent);">BlockTelemetry</code> and <code class="font-mono text-sm" style="color: var(--accent);">BlockHistory</code> in the policy table above, the air-gapped flow gives you a Pro Utterbar install whose only network call is the one the IT admin makes from a different machine when they upload your activation request.
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
        <li>No SAML or SSO. Licences are keys; you don&rsquo;t need user accounts on our side.</li>
        <li>No SOC 2 report. Independent audit is on the table for serious Enterprise contracts; ask in the enquiry.</li>
        <li>No cloud admin dashboard for IT. Licence admin is via the account portal; bulk seat management for 25+ seats happens by email today and will land as a customer surface when there&rsquo;s a real customer asking for it.</li>
        <li>No mobile clients. Windows-native only.</li>
    </ul>

    <h2 class="mt-14 ub-docs-h2">Talk to us</h2>
    <p class="mt-4 text-base leading-relaxed" style="color: var(--fg);">
        If your IT team has a question this page doesn&rsquo;t cover, reply to the quote thread or email <a href="mailto:{{ config('site.email.contact') }}" style="color: var(--accent);">{{ config('site.email.contact') }}</a>. The technical answers come from the same person who wrote the code.
    </p>
    <p class="mt-3 text-base leading-relaxed" style="color: var(--fg);">
        For pricing and sizing, the <a href="{{ route('business') }}" style="color: var(--accent);">business page</a> has the volume tiers; the <a href="{{ route('business.contact') }}" style="color: var(--accent);">enquiry form</a> takes ten minutes and gets you a quote within one working day.
    </p>
</x-layouts.docs>
