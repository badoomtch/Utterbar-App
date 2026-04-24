@props(['items' => []])

<div x-data="{ open: null }" class="relative">
    @foreach ($items as $i => $item)
        @php $isLast = $i === count($items) - 1; @endphp
        <div class="ub-faq-item" :class="open === {{ $i }} ? 'is-open' : ''">
            <button type="button"
                    @click="open === {{ $i }} ? open = null : open = {{ $i }}"
                    :aria-expanded="(open === {{ $i }}).toString()"
                    aria-controls="faq-panel-{{ $i }}"
                    class="ub-faq-trigger">
                <span class="ub-faq-num font-mono">{{ str_pad((string)($i + 1), 2, '0', STR_PAD_LEFT) }}</span>
                <span class="ub-faq-q">{{ $item['q'] }}</span>
                <span class="ub-faq-chev" :class="{ 'is-open': open === {{ $i }} }">
                    <x-icon name="chevron-down" size="20" stroke="2" />
                </span>
            </button>
            <div x-show="open === {{ $i }}" x-collapse id="faq-panel-{{ $i }}">
                <div class="ub-faq-answer">
                    <p>{!! $item['a'] !!}</p>
                </div>
            </div>
            @if (!$isLast)
                <div class="ub-faq-sep"></div>
            @endif
        </div>
    @endforeach
</div>
