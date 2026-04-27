<?php

namespace App\Http\Controllers;

use App\Mail\BusinessEnquirySubmission;
use App\Models\BusinessEnquiry;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Mail;

/**
 * Lead capture for the /business path.
 *
 * Mirrors the structure of ContactController (validate + create + email
 * + admin listing) but with the typed B2B fields. Stays separate from
 * the general contact form on purpose: sales leads need different
 * operator handling and shouldn't get drowned by support volume.
 */
class BusinessEnquiryController extends Controller
{
    /**
     * The structured "what does your team need" list. Buyers tick what
     * applies, the form serialises the picked subset into the
     * `requirements` column. Anything not on the list goes in the
     * free-text `message` field instead.
     *
     * Keep these short and IT-buyer-flavoured: each one is a thing an
     * IT desk would actually ask before approving a deployment.
     */
    public const REQUIREMENT_OPTIONS = [
        'silent_install'      => 'Silent install via SCCM, Intune, or similar',
        'air_gapped'          => 'Air-gapped network (no outbound internet)',
        'group_policy'        => 'Centrally managed settings (group policy)',
        'compliance'          => 'Compliance-bounded environment (legal, healthcare, finance)',
        'volume_invoice'      => 'Pay by invoice / purchase order, not card',
        'priority_support'    => 'Priority support / named contact',
        'soak_window'         => 'Held-back update channel for internal testing',
        'data_retention'      => 'Specific data-retention or audit requirements',
    ];

    public function show(Request $request)
    {
        return view('pages.business-contact', [
            'requirementOptions' => self::REQUIREMENT_OPTIONS,
        ]);
    }

    public function store(Request $request)
    {
        $data = $request->validate([
            'name'         => ['required', 'string', 'max:120'],
            'email'        => ['required', 'email:rfc', 'max:191'],
            'company'      => ['required', 'string', 'max:160'],
            'role'         => ['nullable', 'string', 'max:120'],
            'seat_count'   => ['required', 'integer', 'min:1', 'max:100000'],
            'use_case'     => ['required', 'string', 'min:5', 'max:2000'],
            'requirements' => ['array'],
            'requirements.*' => ['string', 'in:' . implode(',', array_keys(self::REQUIREMENT_OPTIONS))],
            'message'      => ['nullable', 'string', 'max:5000'],
            // Honeypot + consent, mirroring ContactController.
            'hp'           => ['nullable', 'size:0'],
            'consent'      => ['accepted'],
        ], [
            'consent.accepted' => 'Please tick the consent box to continue.',
            'hp.size'          => 'Bot detected.',
            'use_case.min'     => 'A sentence or two on what your team would use this for.',
            'seat_count.min'   => 'Number of seats your team needs.',
        ]);

        $entry = BusinessEnquiry::create([
            'name'         => trim($data['name']),
            'email'        => strtolower(trim($data['email'])),
            'company'      => trim($data['company']),
            'role'         => isset($data['role']) ? trim($data['role']) : null,
            'seat_count'   => (int) $data['seat_count'],
            'use_case'     => $data['use_case'],
            'requirements' => array_values($data['requirements'] ?? []),
            'message'      => $data['message'] ?? null,
            'ip'           => $request->ip(),
            'user_agent'   => substr((string) $request->userAgent(), 0, 255),
            'status'       => 'new',
        ]);

        try {
            Mail::to(config('site.email.contact'))->send(new BusinessEnquirySubmission($entry));
            $entry->update(['emailed_at' => now()]);
        } catch (\Throwable $e) {
            Log::warning('Business enquiry email failed', [
                'id'    => $entry->id,
                'error' => $e->getMessage(),
            ]);
        }

        $message = "Got it. I'll get back to you within one working day with a quote and the deployment notes for your environment.";

        if ($request->wantsJson() || $request->ajax()) {
            return response()->json(['ok' => true, 'message' => $message]);
        }

        return redirect()->route('business.contact')->with('business_status', $message);
    }
}
