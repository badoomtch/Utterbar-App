<?php

namespace App\Http\Controllers;

use App\Mail\WaitlistConfirmation;
use App\Models\WaitlistEntry;
use App\Services\Notify\AdminNotify;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cookie;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Mail;

class WaitlistController extends Controller
{
    public function store(Request $request)
    {
        $data = $request->validate([
            'email'    => ['required', 'email:rfc', 'max:255'],
            'use_case' => ['nullable', 'string', 'max:280'],
            'hp'       => ['nullable', 'size:0'],
            'consent'  => ['accepted'],
        ], [
            'consent.accepted' => 'Please tick the consent box so we know you\'re happy to receive a confirmation email.',
            'hp.size'          => 'Bot detected.',
        ]);

        $email = strtolower(trim($data['email']));

        $existing = WaitlistEntry::where('email', $email)->first();
        $alreadyOnList = (bool) $existing;

        if (! $existing) {
            $entry = WaitlistEntry::create([
                'email'      => $email,
                'use_case'   => $data['use_case'] ?? null,
                'ip'         => $request->ip(),
                'user_agent' => substr((string) $request->userAgent(), 0, 255),
            ]);

            try {
                Mail::to($email)->send(new WaitlistConfirmation());
            } catch (\Throwable $e) {
                Log::warning('Waitlist confirmation email failed', [
                    'email' => $email,
                    'error' => $e->getMessage(),
                ]);
            }

            AdminNotify::newWaitlist($entry, WaitlistEntry::count());
        }

        $message = $alreadyOnList
            ? "You're already on the list — nothing to do."
            : "You're in. Check your inbox for the confirmation.";

        $cookie = Cookie::make('ub_waitlist', '1', 60 * 24 * 365, '/', null, true, false, false, 'Lax');

        if ($request->wantsJson() || $request->ajax()) {
            return response()
                ->json(['ok' => true, 'message' => $message, 'already' => $alreadyOnList])
                ->withCookie($cookie);
        }

        return back()
            ->with('waitlist_status', $message)
            ->with('waitlist_already', $alreadyOnList)
            ->withCookie($cookie);
    }
}
