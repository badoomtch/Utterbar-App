UTTERBAR · BUSINESS ENQUIRY
{{ $enquiry->company }} · {{ (int) $enquiry->seat_count }} seats · {{ $enquiry->tierLabel() }}

---

From:    {{ $enquiry->name }} <{{ $enquiry->email }}>
Company: {{ $enquiry->company }}@if($enquiry->role) · {{ $enquiry->role }}@endif
Sent:    {{ $enquiry->created_at->format('D, j M Y · H:i') }}
IP:      {{ $enquiry->ip ?: '—' }}

---

USE CASE

{{ $enquiry->use_case }}

---

REQUIREMENTS

@php
    $opts = \App\Http\Controllers\BusinessEnquiryController::REQUIREMENT_OPTIONS;
    $picked = (array) ($enquiry->requirements ?? []);
@endphp
@if(count($picked) === 0)
None ticked.
@else
@foreach($picked as $key)
- {{ $opts[$key] ?? $key }}
@endforeach
@endif

@if($enquiry->message)
---

ADDITIONAL NOTES

{{ $enquiry->message }}

@endif
---

Reply directly to this email. It goes to {{ $enquiry->email }}.
Admin view: {{ url('/admin/business/' . $enquiry->id) }}
