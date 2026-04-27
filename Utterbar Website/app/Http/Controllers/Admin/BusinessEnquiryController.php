<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\BusinessEnquiryController as PublicBusinessEnquiryController;
use App\Http\Controllers\Controller;
use App\Models\BusinessEnquiry;
use Illuminate\Http\Request;

/**
 * Admin listing for /business enquiries. Sales leads need different
 * triage than the general contact form (longer reply window, possibly
 * a custom licence mint), and they shouldn't sit in the same list as
 * support questions about the floating bar.
 */
class BusinessEnquiryController extends Controller
{
    public function index(Request $request)
    {
        $q = trim((string) $request->query('q', ''));
        $status = (string) $request->query('status', '');

        $enquiries = BusinessEnquiry::query()
            ->when($q !== '', fn ($qb) => $qb->where(function ($w) use ($q) {
                $w->where('email', 'like', "%{$q}%")
                  ->orWhere('name', 'like', "%{$q}%")
                  ->orWhere('company', 'like', "%{$q}%")
                  ->orWhere('use_case', 'like', "%{$q}%")
                  ->orWhere('message', 'like', "%{$q}%");
            }))
            ->when(in_array($status, ['new', 'contacted', 'closed_won', 'closed_lost'], true),
                fn ($qb) => $qb->where('status', $status))
            ->orderByDesc('id')
            ->paginate(25)
            ->withQueryString();

        return view('pages.admin.business.index', [
            'enquiries' => $enquiries,
            'q'         => $q,
            'status'    => $status,
        ]);
    }

    public function show(int $id)
    {
        $enquiry = BusinessEnquiry::findOrFail($id);
        return view('pages.admin.business.show', [
            'enquiry'            => $enquiry,
            'requirementOptions' => PublicBusinessEnquiryController::REQUIREMENT_OPTIONS,
        ]);
    }

    public function updateStatus(Request $request, int $id)
    {
        $data = $request->validate([
            'status' => ['required', 'in:new,contacted,closed_won,closed_lost'],
        ]);
        $enquiry = BusinessEnquiry::findOrFail($id);
        $enquiry->update(['status' => $data['status']]);

        return redirect()
            ->route('admin.business.show', $enquiry->id)
            ->with('admin_status', "Status set to {$data['status']}.");
    }
}
