<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/**
 * Lead capture for the /business path.
 *
 * Kept separate from `contact_messages` on purpose. Sales leads have
 * different fields (seat count, deployment notes, compliance), get
 * different operator handling (longer-form reply, possibly a custom
 * licence mint), and want to stay visible in admin without being
 * drowned by general support / contact volume.
 */
return new class extends Migration {
    public function up(): void
    {
        Schema::create('business_enquiries', function (Blueprint $t) {
            $t->id();
            // Submitter identity. `name` is split into first/last on the
            // form but stored as one string; the form rejects empty.
            $t->string('name', 120);
            $t->string('email', 191);
            $t->string('company', 160);
            $t->string('role', 120)->nullable();

            // Sizing the deal. seat_count maps to a tier (3-24 = Team,
            // 25-249 = Business, 250+ = Enterprise). Stored as int so
            // operator filters / sorts work; also kept as a free-text
            // line on the message body if the buyer added context.
            $t->unsignedInteger('seat_count');

            // Free-text use case. Useful so the operator can spot
            // patterns (call centre x N, legal x N) without hand-tagging.
            $t->text('use_case');

            // Optional compliance / deployment requirements. Bullets,
            // not free-form prose. The form lets the buyer tick what
            // applies; serialised as JSON for simplicity (a future
            // dedicated reporting view can split it back out).
            $t->json('requirements')->nullable();

            // Optional message body for anything not covered by the
            // structured fields.
            $t->text('message')->nullable();

            // Diagnostic/anti-abuse fields, mirrors contact_messages.
            $t->string('ip', 45)->nullable();
            $t->string('user_agent', 255)->nullable();

            // Operator workflow. Default 'new', flipped to 'contacted'
            // / 'closed_won' / 'closed_lost' from the admin view.
            $t->string('status', 24)->default('new');

            // Best-effort delivery flag, mirrors contact_messages.
            $t->timestamp('emailed_at')->nullable();

            $t->timestamps();

            $t->index('email');
            $t->index('status');
            $t->index('created_at');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('business_enquiries');
    }
};
