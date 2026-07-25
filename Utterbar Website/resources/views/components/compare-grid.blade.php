{{-- Comparison grid: cloud dictation vs Utterbar across the
     decision-shaped axes (privacy, offline, pricing model, account,
     latency, languages).

     Two careful copy choices:
     - Pricing column on the cloud side reads "Subscription, ongoing"
       rather than naming a £/month band, which would be a competitive
       claim we would have to maintain.
     - Latency reads "Round-trip to a server" vs "On-device, no
       round-trip" rather than naming a millisecond figure we have not
       measured. The asymmetry is the point.

     British spelling, no em-dashes. --}}
<div class="ub-compare">

    <div class="ub-compare-row head">
        <div class="label">Capability</div>
        <div class="their">Cloud dictation</div>
        <div class="ours">Utterbar</div>
    </div>

    <div class="ub-compare-row">
        <div class="label">Audio leaves your PC</div>
        <div class="their">
            <x-icon name="check" class="ub-compare-check" size="16" stroke="2" />
            Always
        </div>
        <div class="ours">
            <x-icon name="x" class="ub-compare-x" size="16" stroke="2" />
            Never
        </div>
    </div>

    <div class="ub-compare-row">
        <div class="label">Works offline</div>
        <div class="their">
            <x-icon name="x" class="ub-compare-x" size="16" stroke="2" />
            Needs internet
        </div>
        <div class="ours">
            <x-icon name="check" class="ub-compare-check" size="16" stroke="2" />
            Yes, once installed
        </div>
    </div>

    <div class="ub-compare-row">
        <div class="label">Pricing model</div>
        <div class="their">Subscription, ongoing</div>
        <div class="ours">Free &amp; open source</div>
    </div>

    <div class="ub-compare-row">
        <div class="label">Account required</div>
        <div class="their">
            <x-icon name="check" class="ub-compare-check" size="16" stroke="2" />
            Always
        </div>
        <div class="ours">
            <x-icon name="x" class="ub-compare-x" size="16" stroke="2" />
            Never
        </div>
    </div>

    <div class="ub-compare-row">
        <div class="label">Latency</div>
        <div class="their">Round-trip to a server</div>
        <div class="ours">On-device, no round-trip</div>
    </div>

    <div class="ub-compare-row">
        <div class="label">Languages</div>
        <div class="their">Cloud-bound</div>
        <div class="ours">30+ on-device</div>
    </div>

</div>
