<!DOCTYPE html>
<html lang="en-GB">
<head>
    <meta charset="utf-8">
    <title>Business enquiry · {{ $enquiry->company }}</title>
</head>
<body style="margin:0; padding:0; background:#0A0E17; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; color:#E8E8EB;">
    @php
        $opts = \App\Http\Controllers\BusinessEnquiryController::REQUIREMENT_OPTIONS;
        $picked = (array) ($enquiry->requirements ?? []);
    @endphp
    <table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style="background:#0A0E17;">
        <tr>
            <td align="center" style="padding:32px 16px;">
                <table role="presentation" width="620" cellpadding="0" cellspacing="0" border="0" style="max-width:620px; width:100%; background:#10151F; border-radius:14px;">
                    <tr>
                        <td style="padding:28px 32px;">
                            <p style="margin:0; font-family: 'SF Mono', Menlo, monospace; font-size:11px; letter-spacing:0.08em; color:#22D3EE; text-transform:uppercase;">
                                Utterbar &middot; business enquiry
                            </p>
                            <h1 style="margin:12px 0 0 0; font-size:22px; font-weight:700; letter-spacing:-0.015em; color:#ffffff;">
                                {{ $enquiry->company }}
                            </h1>
                            <p style="margin:6px 0 0 0; font-size:14px; color:#8A8A94;">
                                {{ (int) $enquiry->seat_count }} seats &middot; {{ $enquiry->tierLabel() }}
                            </p>
                        </td>
                    </tr>
                    <tr><td style="padding:0 32px;"><div style="height:1px; background:rgba(255,255,255,0.08); line-height:1px; font-size:1px;">&nbsp;</div></td></tr>
                    <tr>
                        <td style="padding:24px 32px;">
                            <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%" style="font-size:14px; line-height:1.55;">
                                <tr>
                                    <td style="padding:6px 0; color:#8A8A94; width:90px;">From</td>
                                    <td style="padding:6px 0; color:#E8E8EB;">{{ $enquiry->name }} &lt;{{ $enquiry->email }}&gt;</td>
                                </tr>
                                @if($enquiry->role)
                                <tr>
                                    <td style="padding:6px 0; color:#8A8A94;">Role</td>
                                    <td style="padding:6px 0; color:#E8E8EB;">{{ $enquiry->role }}</td>
                                </tr>
                                @endif
                                <tr>
                                    <td style="padding:6px 0; color:#8A8A94;">Sent</td>
                                    <td style="padding:6px 0; color:#E8E8EB;">{{ $enquiry->created_at->format('D, j M Y · H:i') }}</td>
                                </tr>
                                <tr>
                                    <td style="padding:6px 0; color:#8A8A94;">IP</td>
                                    <td style="padding:6px 0; color:#E8E8EB;">{{ $enquiry->ip ?: '—' }}</td>
                                </tr>
                            </table>
                        </td>
                    </tr>
                    <tr><td style="padding:0 32px;"><div style="height:1px; background:rgba(255,255,255,0.08); line-height:1px; font-size:1px;">&nbsp;</div></td></tr>
                    <tr>
                        <td style="padding:24px 32px;">
                            <p style="margin:0 0 8px 0; font-family: 'SF Mono', Menlo, monospace; font-size:10px; letter-spacing:0.14em; color:#8A8A94; text-transform:uppercase;">Use case</p>
                            <div style="font-size:15px; line-height:1.6; color:#E8E8EB; white-space:pre-wrap;">{{ $enquiry->use_case }}</div>
                        </td>
                    </tr>
                    @if(count($picked) > 0)
                    <tr><td style="padding:0 32px;"><div style="height:1px; background:rgba(255,255,255,0.08); line-height:1px; font-size:1px;">&nbsp;</div></td></tr>
                    <tr>
                        <td style="padding:24px 32px;">
                            <p style="margin:0 0 8px 0; font-family: 'SF Mono', Menlo, monospace; font-size:10px; letter-spacing:0.14em; color:#8A8A94; text-transform:uppercase;">Requirements</p>
                            <ul style="margin:0; padding-left:18px; font-size:14px; line-height:1.7; color:#E8E8EB;">
                                @foreach($picked as $key)
                                    <li>{{ $opts[$key] ?? $key }}</li>
                                @endforeach
                            </ul>
                        </td>
                    </tr>
                    @endif
                    @if($enquiry->message)
                    <tr><td style="padding:0 32px;"><div style="height:1px; background:rgba(255,255,255,0.08); line-height:1px; font-size:1px;">&nbsp;</div></td></tr>
                    <tr>
                        <td style="padding:24px 32px;">
                            <p style="margin:0 0 8px 0; font-family: 'SF Mono', Menlo, monospace; font-size:10px; letter-spacing:0.14em; color:#8A8A94; text-transform:uppercase;">Additional notes</p>
                            <div style="font-size:15px; line-height:1.6; color:#E8E8EB; white-space:pre-wrap;">{{ $enquiry->message }}</div>
                        </td>
                    </tr>
                    @endif
                    <tr><td style="padding:0 32px;"><div style="height:1px; background:rgba(255,255,255,0.08); line-height:1px; font-size:1px;">&nbsp;</div></td></tr>
                    <tr>
                        <td style="padding:20px 32px 28px 32px;">
                            <p style="margin:0; font-size:13px; color:#8A8A94; line-height:1.55;">
                                Reply directly to this email; it goes to {{ $enquiry->email }}.<br>
                                Admin view: <a href="{{ url('/admin/business/' . $enquiry->id) }}" style="color:#22D3EE;">{{ url('/admin/business/' . $enquiry->id) }}</a>
                            </p>
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
</body>
</html>
