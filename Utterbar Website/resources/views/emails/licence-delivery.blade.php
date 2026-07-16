<!DOCTYPE html>
<html lang="en-GB">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="color-scheme" content="dark">
    <meta name="supported-color-schemes" content="dark">
    <title>Your Utterbar Pro licence</title>
    <style>
        @media (prefers-color-scheme: light) {
            body { background:#0B0F14 !important; }
        }
    </style>
</head>
<body style="margin:0; padding:0; background:#0A0E17; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; color:#E8E8EB; line-height:1.65;">

    <table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style="background:#0A0E17;">
        <tr>
            <td align="center" style="padding:40px 16px;">
                <table role="presentation" width="580" cellpadding="0" cellspacing="0" border="0" style="max-width:580px; width:100%; background:#10151F; border-radius:18px;">

                    {{-- Brand header — PNG because email clients strip <svg> --}}
                    <tr>
                        <td style="padding:32px 32px 8px 32px;">
                            <a href="https://utterbar.com" style="text-decoration:none; border:0; outline:none;">
                                <img src="https://utterbar.com/brand/png/utterbar-logo-dark-transparent-1024.png"
                                     alt="Utterbar"
                                     width="220"
                                     height="64"
                                     style="display:block; border:0; outline:none; text-decoration:none; -ms-interpolation-mode:bicubic; max-width:220px; height:auto;" />
                            </a>
                        </td>
                    </tr>

                    {{-- Status strip --}}
                    <tr>
                        <td style="padding:28px 32px 0 32px;">
                            <p style="margin:0; font-family:'SF Mono',Menlo,Consolas,monospace; font-size:12px; letter-spacing:0.08em; color:#22D3EE; text-transform:uppercase;">
                                &#9679; PRO &middot; LIFETIME &middot; ACTIVATED {{ optional($issuedAt)->format('Y-m-d') }}
                            </p>
                        </td>
                    </tr>

                    {{-- Headline --}}
                    <tr>
                        <td style="padding:10px 32px 6px 32px;">
                            <h1 style="margin:0; font-size:28px; line-height:1.15; font-weight:800; letter-spacing:-0.02em; color:#ffffff;">
                                You&rsquo;re in.
                            </h1>
                        </td>
                    </tr>

                    {{-- Body --}}
                    <tr>
                        <td style="padding:16px 32px 0 32px;">
                            <p style="margin:0 0 14px 0; font-size:16px; color:#E6EEF6;">
                                Thanks for buying Utterbar Pro. Here&rsquo;s your licence.
                            </p>
                            <p style="margin:0 0 14px 0; font-size:15px; color:#8A8A94;">
                                Open Utterbar &rarr; Settings &rarr; Licence, paste the key, and you&rsquo;re done. Good on up to {{ $maxDevices }} devices.
                            </p>
                        </td>
                    </tr>

                    {{-- Licence key box --}}
                    <tr>
                        <td style="padding:18px 32px 0 32px;">
                            <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%">
                                <tr>
                                    <td align="center" style="background:#0A0E17; border:1px solid rgba(34,211,238,0.35); border-radius:12px; padding:20px 16px;">
                                        <p style="margin:0 0 6px 0; font-family:'SF Mono',Menlo,Consolas,monospace; font-size:11px; letter-spacing:0.12em; color:#8A8A94; text-transform:uppercase;">
                                            your licence key
                                        </p>
                                        <p style="margin:0; font-family:'SF Mono',Menlo,Consolas,monospace; font-size:20px; font-weight:600; letter-spacing:0.08em; color:#ffffff; word-break:break-all;">
                                            {{ $displayKey }}
                                        </p>
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>

                    {{-- One-click deep link — centered, inline dot (email-safe, no position:absolute) --}}
                    <tr>
                        <td align="center" style="padding:22px 32px 0 32px;">
                            <a href="utterbar://licence/{{ $deepLinkKey }}" style="display:inline-block; padding:12px 24px; border-radius:999px; background:#ffffff; color:#0A0E17; font-size:14px; font-weight:600; text-decoration:none; border:2px solid #22D3EE; box-shadow:0 0 0 3px rgba(34,211,238,0.18); line-height:1;">
                                <span style="display:inline-block; width:8px; height:8px; border-radius:50%; background:#22D3EE; vertical-align:middle; margin-right:10px; box-shadow:0 0 10px #22D3EE;">&nbsp;</span><span style="vertical-align:middle;">Paste into Utterbar</span>
                            </a>
                            <p style="margin:12px 0 0 0; font-size:12px; color:#7A8A99;">
                                Or copy-paste the key above into Settings &rarr; Licence.
                            </p>
                        </td>
                    </tr>

                    {{-- Signature block --}}
                    <tr>
                        <td style="padding:24px 32px 0 32px;">
                            <table role="presentation" cellpadding="0" cellspacing="0" border="0" style="border-left:3px solid #22D3EE; padding-left:14px;">
                                <tr>
                                    <td>
                                        <p style="margin:0; font-size:16px; color:#ffffff; font-weight:600;">Jay</p>
                                        <p style="margin:2px 0 0 0; font-size:13px; color:#7A8A99;">Building Utterbar</p>
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>

                    @if (! empty($isVolume))
                    {{-- Volume-only block: short note pointing IT teams at
                         the licence.bin fleet drop. Single-user emails skip
                         this entirely so the existing flow is unchanged. --}}
                    <tr>
                        <td style="padding:24px 32px 0 32px;">
                            <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%" style="background:#0A0E17; border:1px solid rgba(255,255,255,0.08); border-radius:12px;">
                                <tr>
                                    <td style="padding:18px 18px;">
                                        <p style="margin:0 0 8px 0; font-family:'SF Mono',Menlo,Consolas,monospace; font-size:11px; letter-spacing:0.12em; color:#22D3EE; text-transform:uppercase;">
                                            For your IT team
                                        </p>
                                        <p style="margin:0 0 10px 0; font-size:14px; color:#E6EEF6;">
                                            This is a volume licence covering <strong>{{ $maxDevices }} devices</strong>.
                                            Each device activates by pasting the key once, the same as a single-user purchase.
                                        </p>
                                        <p style="margin:0; font-size:13px; color:#8A8A94;">
                                            If you&rsquo;d rather deploy across a fleet without each user pasting the key,
                                            sign in at <a href="https://utterbar.com/admin" style="color:#22D3EE; text-decoration:none;">utterbar.com/admin</a>,
                                            open this licence, and download a signed <code style="color:#E6EEF6;">licence.bin</code>.
                                            Drop it at <code style="color:#E6EEF6;">%APPDATA%\Utterbar\licence.bin</code> on each managed machine
                                            and Utterbar treats it as the active Pro licence at startup, no activation prompt.
                                            Full doc: <a href="https://utterbar.com/docs/it-deployment" style="color:#22D3EE; text-decoration:none;">utterbar.com/docs/it-deployment</a>.
                                        </p>
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>
                    @endif

                    {{-- Manage account CTA --}}
                    <tr>
                        <td style="padding:24px 32px 0 32px;">
                            <p style="margin:0 0 8px 0; font-size:13px; color:#8A8A94;">
                                Need this again later? Manage your licences and devices at
                                <a href="https://utterbar.com/account" style="color:#22D3EE; text-decoration:none;">utterbar.com/account</a>.
                                We&rsquo;ll email a sign-in link to <span style="color:#E6EEF6;">{{ $email }}</span>.
                            </p>
                        </td>
                    </tr>

                    {{-- Divider --}}
                    <tr>
                        <td style="padding:28px 32px 0 32px;">
                            <div style="height:1px; background:rgba(255,255,255,0.08); line-height:1px; font-size:1px;">&nbsp;</div>
                        </td>
                    </tr>

                    {{-- Footer --}}
                    <tr>
                        <td style="padding:20px 32px 28px 32px;">
                            <p style="margin:0 0 10px 0; font-size:13px; color:#7A8A99;">
                                Lost this email later? Visit <a href="https://utterbar.com/licence" style="color:#22D3EE; text-decoration:none;">utterbar.com/licence</a>, enter {{ $email }}, and we&rsquo;ll send a magic link to manage your licence and devices.
                            </p>
                            <p style="margin:0; font-size:12px; color:#7A8A99; font-family:'SF Mono',Menlo,Consolas,monospace; letter-spacing:0.02em;">
                                utterbar.com &middot; hello@ddraig.dev
                            </p>
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
</body>
</html>
