<?php

namespace App\Http\Controllers;

use App\Mail\ContactSubmission;
use App\Models\ContactMessage;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Mail;

class ContactController extends Controller
{
    public function store(Request $request)
    {
        $data = $request->validate([
            'name'    => ['nullable', 'string', 'max:120'],
            'email'   => ['required', 'email:rfc', 'max:255'],
            'subject' => ['nullable', 'string', 'max:200'],
            'message' => ['required', 'string', 'min:5', 'max:5000'],
            'hp'      => ['nullable', 'size:0'],
            'consent' => ['accepted'],
        ], [
            'consent.accepted' => 'Please tick the consent box to continue.',
            'hp.size'          => 'Bot detected.',
            'message.min'      => 'Write a bit more than that.',
            'message.max'      => 'Keep it under 5,000 characters.',
        ]);

        $entry = ContactMessage::create([
            'name'       => $data['name'] ?? null,
            'email'      => strtolower(trim($data['email'])),
            'subject'    => $data['subject'] ?? null,
            'message'    => $data['message'],
            'ip'         => $request->ip(),
            'user_agent' => substr((string) $request->userAgent(), 0, 255),
        ]);

        try {
            Mail::to(config('site.email.contact'))->send(new ContactSubmission($entry));
            $entry->update(['emailed_at' => now()]);
        } catch (\Throwable $e) {
            Log::warning('Contact email failed', [
                'id'    => $entry->id,
                'error' => $e->getMessage(),
            ]);
        }

        $message = "Got it. I'll reply from hello@ddraig.dev within a day or two.";

        if ($request->wantsJson() || $request->ajax()) {
            return response()->json(['ok' => true, 'message' => $message]);
        }

        return back()->with('contact_status', $message);
    }
}
