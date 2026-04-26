<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Models\ContactMessage;
use Illuminate\Http\Request;

/**
 * Read-only admin listing for the contact form. Messages are also mirrored
 * to email at submission time, so this page exists mainly to dig up old
 * threads without trawling the inbox.
 */
class ContactController extends Controller
{
    public function index(Request $request)
    {
        $q = trim((string) $request->query('q', ''));

        $messages = ContactMessage::query()
            ->when($q !== '', fn ($qb) => $qb->where(function ($w) use ($q) {
                $w->where('email', 'like', "%{$q}%")
                  ->orWhere('name', 'like', "%{$q}%")
                  ->orWhere('subject', 'like', "%{$q}%")
                  ->orWhere('message', 'like', "%{$q}%");
            }))
            ->orderByDesc('id')
            ->paginate(25)
            ->withQueryString();

        return view('pages.admin.contacts.index', [
            'messages' => $messages,
            'q'        => $q,
        ]);
    }

    public function show(int $id)
    {
        $message = ContactMessage::findOrFail($id);
        return view('pages.admin.contacts.show', ['message' => $message]);
    }
}
