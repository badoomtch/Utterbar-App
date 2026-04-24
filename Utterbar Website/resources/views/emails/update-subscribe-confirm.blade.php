<!DOCTYPE html>
<html lang="en-GB">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="color-scheme" content="dark">
    <meta name="supported-color-schemes" content="dark">
    <title>Confirm your Utterbar release emails</title>
</head>
<body style="margin:0; padding:0; background:#0A0E17; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; color:#E8E8EB; line-height:1.65;">
    <table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style="background:#0A0E17;">
        <tr>
            <td align="center" style="padding:40px 16px;">
                <table role="presentation" width="580" cellpadding="0" cellspacing="0" border="0" style="max-width:580px; width:100%; background:#10151F; border-radius:18px;">
                    <tr>
                        <td style="padding:32px 32px 8px 32px;">
                            <a href="https://utterbar.com" style="text-decoration:none; border:0; outline:none;">
                                <img src="https://utterbar.com/brand/png/utterbar-logo-dark-transparent-1024.png"
                                     alt="Utterbar" width="220" height="64"
                                     style="display:block; border:0; outline:none; text-decoration:none; -ms-interpolation-mode:bicubic; max-width:220px; height:auto;" />
                            </a>
                        </td>
                    </tr>

                    <tr>
                        <td style="padding:28px 32px 0 32px;">
                            <p style="margin:0; font-family:'SF Mono',Menlo,Consolas,monospace; font-size:12px; letter-spacing:0.08em; color:#22D3EE; text-transform:uppercase;">
                                &#9679; RELEASE UPDATES &middot; ONE-CLICK CONFIRM
                            </p>
                        </td>
                    </tr>

                    <tr>
                        <td style="padding:10px 32px 6px 32px;">
                            <h1 style="margin:0; font-size:28px; line-height:1.15; font-weight:800; letter-spacing:-0.02em; color:#ffffff;">
                                One click to confirm.
                            </h1>
                        </td>
                    </tr>

                    <tr>
                        <td style="padding:16px 32px 0 32px;">
                            <p style="margin:0 0 14px 0; font-size:16px; color:#E6EEF6;">
                                You (or someone using <strong style="color:#ffffff;">{{ $email }}</strong>) asked to hear about new Utterbar releases.
                            </p>
                            <p style="margin:0 0 14px 0; font-size:15px; color:#8A8A94;">
                                Confirm the subscription and we&rsquo;ll email only when there&rsquo;s a release worth installing. No weekly nags, no drip campaign.
                            </p>
                        </td>
                    </tr>

                    <tr>
                        <td align="center" style="padding:22px 32px 0 32px;">
                            <a href="{{ $confirmUrl }}" style="display:inline-block; padding:12px 28px; border-radius:999px; background:#ffffff; color:#0A0E17; font-size:15px; font-weight:600; text-decoration:none; border:2px solid #22D3EE; box-shadow:0 0 0 3px rgba(34,211,238,0.18); line-height:1;">
                                <span style="display:inline-block; width:8px; height:8px; border-radius:50%; background:#22D3EE; vertical-align:middle; margin-right:10px; box-shadow:0 0 10px #22D3EE;">&nbsp;</span><span style="vertical-align:middle;">Confirm &amp; subscribe</span>
                            </a>
                        </td>
                    </tr>

                    <tr>
                        <td style="padding:28px 32px 0 32px;">
                            <div style="height:1px; background:rgba(255,255,255,0.08); line-height:1px; font-size:1px;">&nbsp;</div>
                        </td>
                    </tr>

                    <tr>
                        <td style="padding:20px 32px 28px 32px;">
                            <p style="margin:0 0 10px 0; font-size:12px; color:#7A8A99;">
                                Didn&rsquo;t ask for this? Ignore the email &mdash; no subscription happens without the click above.
                                Or <a href="{{ $unsubscribeUrl }}" style="color:#22D3EE;">remove {{ $email }} immediately</a>.
                            </p>
                            <p style="margin:0; font-size:12px; color:#7A8A99; font-family:'SF Mono',Menlo,Consolas,monospace; letter-spacing:0.02em;">
                                utterbar.com &middot; hello@utterbar.com
                            </p>
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
</body>
</html>
