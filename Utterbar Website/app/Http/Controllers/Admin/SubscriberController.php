<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Models\UpdateSubscriber;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;

/**
 * Admin-side view of the `update_subscribers` table (people who ticked
 * "email me when there's a new release" in the app). Currently the only
 * way to remove a subscriber was the signed unsubscribe URL in a sent
 * email — this page lets an operator remove one by hand.
 */
class SubscriberController extends Controller
{
    public function index(Request $request)
    {
        $q      = trim((string) $request->query('q', ''));
        $status = (string) $request->query('status', '');

        $subs = UpdateSubscriber::query()
            ->when($q !== '', fn ($qb) => $qb->where('email', 'like', "%{$q}%"))
            ->when($status === 'confirmed',    fn ($qb) => $qb->whereNotNull('confirmed_at')->whereNull('unsubscribed_at'))
            ->when($status === 'unconfirmed',  fn ($qb) => $qb->whereNull('confirmed_at')->whereNull('unsubscribed_at'))
            ->when($status === 'unsubscribed', fn ($qb) => $qb->whereNotNull('unsubscribed_at'))
            ->orderByDesc('id')
            ->paginate(50)
            ->withQueryString();

        return view('pages.admin.subscribers.index', [
            'subs'   => $subs,
            'q'      => $q,
            'status' => $status,
        ]);
    }

    public function unsubscribe(Request $request, int $id): RedirectResponse
    {
        $sub = UpdateSubscriber::findOrFail($id);

        if (!$sub->unsubscribed_at) {
            $sub->unsubscribed_at = now();
            $sub->save();
        }

        Log::info('admin unsubscribed subscriber', [
            'subscriber_id' => $sub->id,
            'email'         => $sub->email,
            'actor'         => optional($request->user())->email,
        ]);

        return redirect()->route('admin.subscribers.index')->with('flash', [
            'type' => 'ok',
            'text' => "Unsubscribed {$sub->email}.",
        ]);
    }
}
