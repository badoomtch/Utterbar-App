UTTERBAR · CONTACT
{{ $submission->subject ?: 'New contact message' }}

---

From: {{ $submission->name ? $submission->name . ' ' : '' }}<{{ $submission->email }}>
Sent: {{ $submission->created_at->format('D, j M Y · H:i') }}
IP:   {{ $submission->ip ?: '-' }}

---

{{ $submission->message }}

---

Reply directly to this email. It goes to {{ $submission->email }}.
