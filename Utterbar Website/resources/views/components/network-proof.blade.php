{{-- Network proof: a fake activity monitor showing three lanes.
     The point is the asymmetry: mic input bounces, on-device CPU
     ticks along, network output stays flat. Live transcript fills on
     the left while the network log stays empty on the right.

     Click the start button to begin. Idles by default so the page
     does not run a recurring timer for visitors who never reach this
     section. Stops itself after ~28s if left running, to avoid CPU
     burn on background tabs.

     British spelling, no em-dashes, no banned words. --}}
<div x-data="networkProof" class="ub-proof-monitor" x-init="init()">

    <div class="ub-proof-head">
        <span class="title">// session.live &middot; Activity Monitor</span>
        <span class="ub-proof-status">
            <span class="led" :class="{ 'is-on': recording }"></span>
            <span x-text="recording ? `recording &middot; ${secs.toFixed(1)}s` : 'idle'"></span>
        </span>
    </div>

    <div class="ub-proof-lanes">
        {{-- Mic lane: bouncy waveform --}}
        <div class="ub-proof-lane is-active" :class="{ 'is-live': recording }" style="--lane-color: var(--accent);">
            <div class="ub-proof-lane-head">
                <div class="ub-proof-lane-title">
                    <span class="ub-proof-lane-led"></span>
                    <span class="ub-proof-lane-label">Microphone</span>
                    <span class="ub-proof-lane-sub">System audio in</span>
                </div>
                <span class="ub-proof-lane-unit" x-text="recording ? '-12 dB peak' : 'standby'"></span>
            </div>
            <div class="ub-proof-lane-graph">
                <svg viewBox="0 0 100 36" preserveAspectRatio="none">
                    <line x1="0" y1="35.5" x2="100" y2="35.5" stroke="currentColor" stroke-width="0.5" stroke-opacity="0.55" />
                    <path :d="path(mic)" stroke="currentColor" stroke-width="0.7" fill="none" />
                </svg>
            </div>
        </div>

        {{-- On-device transcribe lane: steady moderate oscillation --}}
        <div class="ub-proof-lane is-active" :class="{ 'is-live': recording }" style="--lane-color: #A78BFA;">
            <div class="ub-proof-lane-head">
                <div class="ub-proof-lane-title">
                    <span class="ub-proof-lane-led"></span>
                    <span class="ub-proof-lane-label">On-device transcribe</span>
                    <span class="ub-proof-lane-sub">Parakeet &middot; CPU</span>
                </div>
                <span class="ub-proof-lane-unit" x-text="recording ? '12% util' : '0% util'"></span>
            </div>
            <div class="ub-proof-lane-graph">
                <svg viewBox="0 0 100 36" preserveAspectRatio="none">
                    <line x1="0" y1="35.5" x2="100" y2="35.5" stroke="currentColor" stroke-width="0.5" stroke-opacity="0.55" />
                    <path :d="path(cpu)" stroke="currentColor" stroke-width="0.7" fill="none" />
                </svg>
            </div>
        </div>

        {{-- Network out lane: dashed flatline. The whole point. --}}
        <div class="ub-proof-lane is-silent" style="--lane-color: var(--fg-dim);">
            <div class="ub-proof-lane-head">
                <div class="ub-proof-lane-title">
                    <span class="ub-proof-lane-led"></span>
                    <span class="ub-proof-lane-label">Network out</span>
                    <span class="ub-proof-lane-sub">all interfaces</span>
                </div>
                <span class="ub-proof-lane-unit">0 B/s</span>
            </div>
            <div class="ub-proof-lane-graph">
                <svg viewBox="0 0 100 36" preserveAspectRatio="none">
                    <line x1="0" y1="35.5" x2="100" y2="35.5" stroke="currentColor" stroke-width="0.7" stroke-dasharray="2,2" />
                </svg>
                <div class="ub-proof-lane-flatlabel">FLATLINE. Nothing leaves the machine.</div>
            </div>
        </div>
    </div>

    <div class="ub-proof-split">
        <div class="ub-proof-transcript">
            <div class="ub-proof-transcript-head">
                <span>Transcript buffer</span>
                <span class="ub-muted">RAM only, cleared on release</span>
            </div>
            <div class="ub-proof-transcript-body">
                <template x-if="transcript.length === 0">
                    <div class="ub-proof-transcript-empty"
                         x-text="recording ? 'Listening...' : 'Start monitoring to see transcription appear here as you speak.'">
                    </div>
                </template>
                <template x-for="(line, i) in transcript" :key="i">
                    <div class="ub-proof-line" :class="line.final ? 'is-final' : 'is-partial'">
                        <span class="ub-proof-line-marker" x-text="line.final ? '✓' : '...'"></span>
                        <span class="ub-proof-line-text" x-text="line.text"></span>
                    </div>
                </template>
            </div>
        </div>

        <div class="ub-proof-netlog">
            <div class="ub-proof-netlog-head">
                <span>Network requests</span>
                <span class="ub-muted">filter: all</span>
            </div>
            <div class="ub-proof-netlog-body">
                <div class="ub-proof-netlog-headers">
                    <span>time</span><span>request</span><span>status</span><span>bytes</span>
                </div>
                <div class="ub-proof-netlog-empty">
                    <div class="ub-proof-netlog-big">No requests<span class="ub-accent">.</span></div>
                    <div class="ub-proof-netlog-small">Audio captured, transcribed, discarded. All on this machine.</div>
                    <div class="ub-proof-netlog-stat">
                        <span><b>0</b> connections</span>
                        <span><b>0 B</b> sent</span>
                        <span><b>0 B</b> received</span>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <div class="ub-proof-foot">
        <button type="button"
                class="ub-proof-action-btn"
                :class="{ 'is-recording': recording }"
                @click="recording ? stop() : start()">
            <span class="led"></span>
            <span x-text="recording ? '&#x25a0; stop monitoring' : '&#x25b6; start monitoring &amp; dictate'"></span>
        </button>
        <span class="ub-proof-foot-meta">
            Open your real Activity Monitor alongside it. Watch the network row stay still.
        </span>
    </div>
</div>

@once
@push('scripts')
<script>
    document.addEventListener('alpine:init', () => {
        window.Alpine.data('networkProof', () => ({
            recording: false,
            secs: 0,
            mic: Array.from({ length: 56 }, () => 0),
            cpu: Array.from({ length: 56 }, () => 0.04),
            transcript: [],
            _phraseI: 0,
            _charI: 0,
            _laneTimer: null,
            _secsTimer: null,
            _transTimer: null,
            _autoStop: null,
            phrases: [
                "Hey Marcus, on the Q3 forecast revision",
                "I want to flag the variance against our",
                "original commit. We are trending around",
                "four point two million, not three eight,",
                "and that is mostly the enterprise renewals",
                "landing earlier than expected. Can we",
                "block fifteen minutes Friday to walk through",
                "the assumptions before the board pre-read?",
            ],
            init() {
                // The monitor is intentionally inert until the user
                // clicks Start. No timer runs on idle visitors.
            },
            destroy() { this.stop(); },
            path(data) {
                const W = 100, H = 36;
                const step = W / (data.length - 1);
                return data.map((v, i) => {
                    const cmd = i === 0 ? 'M' : 'L';
                    return `${cmd}${(i * step).toFixed(2)},${(H - v * H).toFixed(2)}`;
                }).join(' ');
            },
            start() {
                this.recording = true;
                this.secs = 0;
                this.mic = Array.from({ length: 56 }, () => 0);
                this.cpu = Array.from({ length: 56 }, () => 0.04);
                this.transcript = [];
                this._phraseI = 0;
                this._charI = 0;

                this._secsTimer = setInterval(() => { this.secs += 0.1; }, 100);

                this._laneTimer = setInterval(() => {
                    this.mic = [...this.mic.slice(1), 0.25 + Math.random() * 0.7];
                    this.cpu = [...this.cpu.slice(1), 0.08 + Math.random() * 0.10];
                }, 80);

                this._transTimer = setInterval(() => {
                    const phrase = this.phrases[this._phraseI % this.phrases.length];
                    this._charI += 2;
                    const partial = phrase.slice(0, this._charI);
                    const next = this.transcript.slice();
                    if (next.length === 0 || next[next.length - 1].final) {
                        next.push({ text: partial, final: false });
                    } else {
                        next[next.length - 1] = { text: partial, final: false };
                    }
                    if (this._charI >= phrase.length) {
                        next[next.length - 1].final = true;
                        this._phraseI += 1;
                        this._charI = 0;
                    }
                    this.transcript = next.slice(-6);
                }, 60);

                // Auto-stop after 28s so a forgotten browser tab
                // does not keep these timers running indefinitely.
                this._autoStop = setTimeout(() => this.stop(), 28000);
            },
            stop() {
                this.recording = false;
                if (this._secsTimer)  clearInterval(this._secsTimer);
                if (this._laneTimer)  clearInterval(this._laneTimer);
                if (this._transTimer) clearInterval(this._transTimer);
                if (this._autoStop)   clearTimeout(this._autoStop);
                this._secsTimer = this._laneTimer = this._transTimer = this._autoStop = null;
            },
        }));
    });
</script>
@endpush
@endonce
