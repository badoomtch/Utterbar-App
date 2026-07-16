<?php

namespace App\Http\Controllers;

use App\Mail\LicenceDelivery;
use App\Models\Activation;
use App\Models\Licence;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Mail;

class AccountController extends Controller
{
    public function show(Request $request)
    {
        /** @var \App\Models\User $user */
        $user = Auth::user();

        $licences = Licence::where('email', $user->email)
            ->with(['activations' => fn ($q) => $q->orderByDesc('last_heartbeat')])
            ->orderByDesc('id')
            ->get();

        return view('pages.account', [
            'user'     => $user,
            'licences' => $licences,
        ]);
    }

    /**
     * POST /account/licences/{id}/resend — resend the licence email for a
     * licence owned by the signed-in user.
     */
    public function resendLicence(Request $request, int $id): RedirectResponse
    {
        $licence = $this->ownedLicenceOr404($id);

        try {
            Mail::to($licence->email)->send(new LicenceDelivery($licence));
        } catch (\Throwable $e) {
            return redirect()->route('account')->with('flash', [
                'type' => 'error',
                'text' => 'Email delivery failed. Try again in a minute, or contact hello@ddraig.dev.',
            ]);
        }

        return redirect()->route('account')->with('flash', [
            'type' => 'ok',
            'text' => "Licence email resent to {$licence->email}.",
        ]);
    }

    /**
     * POST /account/licences/{id}/devices/{fingerprint}/revoke
     * Remove a device activation from the signed-in user's licence.
     */
    public function deactivateDevice(Request $request, int $id, string $fingerprint): RedirectResponse
    {
        $licence = $this->ownedLicenceOr404($id);

        $deleted = Activation::where('licence_id', $licence->id)
            ->where('fingerprint_hash', strtolower($fingerprint))
            ->delete();

        return redirect()->route('account')->with('flash', [
            'type' => $deleted ? 'ok' : 'error',
            'text' => $deleted
                ? 'Device deactivated. The slot is now free.'
                : 'Device wasn\'t found. Probably already deactivated.',
        ]);
    }

    private function ownedLicenceOr404(int $id): Licence
    {
        /** @var \App\Models\User $user */
        $user = Auth::user();
        $licence = Licence::find($id);

        if (!$licence || strcasecmp($licence->email, $user->email) !== 0) {
            abort(404);
        }

        return $licence;
    }
}
