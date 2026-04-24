<!DOCTYPE html>
<html lang="en-GB">
<head>
    <meta charset="utf-8">
    <title>{{ $submission->subject ?: 'New contact message' }}</title>
</head>
<body style="margin:0; padding:0; background:#0A0E17; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; color:#E8E8EB;">
    <table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style="background:#0A0E17;">
        <tr>
            <td align="center" style="padding:32px 16px;">
                <table role="presentation" width="580" cellpadding="0" cellspacing="0" border="0" style="max-width:580px; width:100%; background:#10151F; border-radius:14px;">
                    <tr>
                        <td style="padding:28px 32px;">
                            <p style="margin:0; font-family: 'SF Mono', Menlo, monospace; font-size:11px; letter-spacing:0.08em; color:#8A8A94; text-transform:uppercase;">
                                Utterbar &middot; contact
                            </p>
                            <h1 style="margin:12px 0 0 0; font-size:22px; font-weight:700; letter-spacing:-0.015em; color:#ffffff;">
                                {{ $submission->subject ?: 'New contact message' }}
                            </h1>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding:0 32px;">
                            <div style="height:1px; background:rgba(255,255,255,0.08); line-height:1px; font-size:1px;">&nbsp;</div>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding:24px 32px;">
                            <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%">
                                <tr>
                                    <td style="padding-bottom:10px; font-size:13px; color:#8A8A94; width:90px;">From</td>
                                    <td style="padding-bottom:10px; font-size:14px; color:#E8E8EB;">
                                        {{ $submission->name ? $submission->name . ' ' : '' }}&lt;<a href="mailto:{{ $submission->email }}" style="color:#22D3EE; text-decoration:none;">{{ $submission->email }}</a>&gt;
                                    </td>
                                </tr>
                                <tr>
                                    <td style="padding-bottom:10px; font-size:13px; color:#8A8A94;">Sent</td>
                                    <td style="padding-bottom:10px; font-size:14px; color:#E8E8EB; font-family:'SF Mono', Menlo, monospace;">
                                        {{ $submission->created_at->format('D, j M Y · H:i') }}
                                    </td>
                                </tr>
                                <tr>
                                    <td style="font-size:13px; color:#8A8A94; vertical-align:top;">IP</td>
                                    <td style="font-size:12px; color:#5A5A64; font-family:'SF Mono', Menlo, monospace;">
                                        {{ $submission->ip ?: '—' }}
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding:0 32px;">
                            <div style="height:1px; background:rgba(255,255,255,0.08); line-height:1px; font-size:1px;">&nbsp;</div>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding:24px 32px 32px 32px;">
                            <p style="margin:0 0 10px 0; font-family: 'SF Mono', Menlo, monospace; font-size:11px; letter-spacing:0.08em; color:#8A8A94; text-transform:uppercase;">
                                Message
                            </p>
                            <div style="font-size:15px; color:#E8E8EB; line-height:1.6; white-space:pre-wrap; word-wrap:break-word;">{{ $submission->message }}</div>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding:0 32px 28px 32px;">
                            <p style="margin:0; font-family: 'SF Mono', Menlo, monospace; font-size:11px; color:#5A5A64; letter-spacing:0.04em;">
                                Reply directly to this email &mdash; it goes to {{ $submission->email }}.
                            </p>
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
</body>
</html>
