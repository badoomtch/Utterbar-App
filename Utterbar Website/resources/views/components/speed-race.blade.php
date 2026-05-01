{{-- Speed race: looping side-by-side typing-vs-Utterbar comparison.
     Pure Alpine. The "race" is two character-rate counters, one slow
     (the typing track) and one ~3x faster (the Utterbar track). Both
     advance through the same script and reset together when both
     finish, with a brief pause between scripts so the user can read
     the final state. WPM badges update live.
     British spelling, no em-dashes, no banned words. --}}
<div x-data="speedRace" class="ub-speed-card">
    <p class="ub-eyebrow">The maths</p>
    <h3 class="ub-speed-title">
        Speak about <x-hl>3 times faster</x-hl> than you can type.
    </h3>
    <p class="ub-speed-lead">
        Long prompts, fuller context, fewer sore wrists. Watch the same sentence finish two ways.
    </p>

    <div class="ub-speed-grid">
        <div class="ub-speed-track typing">
            <div class="ub-speed-track-label">
                <span class="name">Typing &middot; 60 wpm</span>
                <span class="wpm" x-text="`${typingPct}%`"></span>
            </div>
            <div class="ub-speed-track-text" x-text="typed"></div>
            <div class="ub-speed-progress">
                <div class="fill" :style="`width: ${typingPct}%`"></div>
            </div>
        </div>
        <div class="ub-speed-track voice">
            <div class="ub-speed-track-label">
                <span class="name">Utterbar &middot; 180 wpm</span>
                <span class="wpm" x-text="`${voicePct}%`"></span>
            </div>
            <div class="ub-speed-track-text" x-text="voice"></div>
            <div class="ub-speed-progress">
                <div class="fill" :style="`width: ${voicePct}%`"></div>
            </div>
        </div>
    </div>

    <button type="button" class="ub-speed-restart" @click="nextScript()">
        &#x21bb; next sentence
    </button>
</div>

@once
@push('scripts')
<script>
    document.addEventListener('alpine:init', () => {
        window.Alpine.data('speedRace', () => ({
            scripts: [
                "Tell me about the proposal you sent yesterday and pull in the latest figures from the Q3 forecast call.",
                "Summarise the three things that went wrong in the launch retro and propose owners for each follow-up.",
                "Draft a calm but firm reply to the supplier explaining why we are pausing the order until the audit clears.",
            ],
            scriptIdx: 0,
            typed: '',
            voice: '',
            _typedI: 0,
            _voiceI: 0,
            _timer: null,
            get current() { return this.scripts[this.scriptIdx]; },
            get typingPct() {
                const total = this.current.length || 1;
                return Math.min(100, Math.round((this._typedI / total) * 100));
            },
            get voicePct() {
                const total = this.current.length || 1;
                return Math.min(100, Math.round((this._voiceI / total) * 100));
            },
            init() {
                this.run();
            },
            destroy() {
                if (this._timer) clearInterval(this._timer);
            },
            nextScript() {
                this.scriptIdx = (this.scriptIdx + 1) % this.scripts.length;
                this.run();
            },
            run() {
                if (this._timer) clearInterval(this._timer);
                this._typedI = 0;
                this._voiceI = 0;
                this.typed = '';
                this.voice = '';
                this._timer = setInterval(() => {
                    const total = this.current.length;
                    // Typing: ~60 wpm, advances 0 or 1 char per 100ms tick,
                    // weighted slightly slow so the race reads as deliberate.
                    if (this._typedI < total) {
                        this._typedI += Math.random() > 0.7 ? 0 : 1;
                        this.typed = this.current.slice(0, this._typedI);
                    }
                    // Voice: ~180 wpm, advances 2 or 3 chars per tick.
                    if (this._voiceI < total) {
                        this._voiceI += Math.random() > 0.5 ? 3 : 2;
                        this.voice = this.current.slice(0, Math.min(this._voiceI, total));
                    }
                    if (this._typedI >= total && this._voiceI >= total) {
                        clearInterval(this._timer);
                        this._timer = null;
                        // Pause briefly on the finished state, then loop.
                        setTimeout(() => {
                            this.scriptIdx = (this.scriptIdx + 1) % this.scripts.length;
                            this.run();
                        }, 1800);
                    }
                }, 100);
            },
        }));
    });
</script>
@endpush
@endonce
