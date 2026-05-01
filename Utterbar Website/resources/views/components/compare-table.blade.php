@props([
    // Array of comparison rows. Each row:
    //   topic     – section label (e.g. 'Where transcription happens')
    //   utterbar  – HTML/text describing the Utterbar position
    //   competitor – HTML/text describing the competitor position
    //   edge      – one of 'utterbar' | 'competitor' | 'tie'
    'rows' => [],
    // The competitor's display name. Rendered in the column footer label.
    'competitor' => 'Competitor',
    // Optional sourced-from line shown below the table.
    'sourceUrl' => null,
    'sourceLabel' => null,
    'sourceDate' => null,
])

{{--
  <x-compare-table> — DRY wrapper around the Utterbar-vs-X comparison
  table that the original Wispr Flow page invented inline. Every
  /compare/<competitor> deep dive uses this so the visual treatment,
  edge highlighting, and per-row label markup stay in one place.

  The "edge" field drives the colour treatment per row:
    • utterbar    — Utterbar cell foregrounded (cyan check, fg-hi text);
                    competitor cell muted.
    • competitor  — competitor cell foregrounded; Utterbar cell muted.
    • tie         — both cells get a neutral check.

  Pass HTML inside the `utterbar` / `competitor` strings (e.g. &pound;,
  &rsquo;) and use {!! ... !!} via the `<...>` helper — the component
  uses {!! !!} so callers can include entities and inline markup.
--}}

<div class="rounded-2xl overflow-hidden" style="background: var(--bg-1); border: 1px solid var(--border);">
    @foreach ($rows as $i => $row)
        @php
            $edge = $row['edge'] ?? 'tie';
            $utterStrong = $edge === 'utterbar';
            $compStrong = $edge === 'competitor';
        @endphp
        <div class="grid grid-cols-1 md:grid-cols-12 gap-4 md:gap-6 p-5 md:p-6"
             style="{{ $i > 0 ? 'border-top: 1px solid var(--border);' : '' }}">
            <div class="md:col-span-3">
                <p class="font-mono text-xs uppercase tracking-wider" style="color: var(--muted); letter-spacing: 0.12em;">{{ $row['topic'] }}</p>
            </div>
            <div class="md:col-span-4 lg:col-span-4">
                <div class="flex items-start gap-2.5">
                    @if ($utterStrong)
                        <span class="mt-0.5 shrink-0" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.4" /></span>
                    @elseif ($edge === 'tie')
                        <span class="mt-0.5 shrink-0" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span>
                    @else
                        <span class="mt-0.5 shrink-0" style="color: var(--fg-dim); opacity: 0.6;"><x-icon name="check" size="16" stroke="1.6" /></span>
                    @endif
                    <span style="color: {{ $utterStrong ? 'var(--fg-hi)' : 'var(--fg)' }};">{!! $row['utterbar'] !!}</span>
                </div>
                <p class="font-mono text-[10px] uppercase tracking-wider mt-2 ml-6" style="color: var(--muted);">Utterbar</p>
            </div>
            <div class="md:col-span-5">
                <div class="flex items-start gap-2.5">
                    @if ($compStrong)
                        <span class="mt-0.5 shrink-0" style="color: var(--accent);"><x-icon name="check" size="16" stroke="2.4" /></span>
                    @elseif ($edge === 'tie')
                        <span class="mt-0.5 shrink-0" style="color: var(--muted);"><x-icon name="check" size="16" stroke="2" /></span>
                    @else
                        <span class="mt-0.5 shrink-0" style="color: var(--fg-dim); opacity: 0.6;"><x-icon name="check" size="16" stroke="1.6" /></span>
                    @endif
                    <span style="color: {{ $compStrong ? 'var(--fg-hi)' : 'var(--muted)' }};">{!! $row['competitor'] !!}</span>
                </div>
                <p class="font-mono text-[10px] uppercase tracking-wider mt-2 ml-6" style="color: var(--muted);">{{ $competitor }}</p>
            </div>
        </div>
    @endforeach
</div>

@if ($sourceUrl && $sourceLabel)
    <p class="mt-4 text-xs" style="color: var(--muted);">
        {{ $competitor }} data sourced from <a href="{{ $sourceUrl }}" target="_blank" rel="noopener" style="color: var(--accent);">{{ $sourceLabel }}</a>@if ($sourceDate), verified {{ $sourceDate }}@endif. We&rsquo;ll keep this in sync as their plans change.
    </p>
@endif
