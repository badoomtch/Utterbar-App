{{-- Talk to AI: a four-tab interactive mock of dictating into the
     popular chat interfaces. Each assistant has its own chrome (URL
     bar, brand glyph, accent colour) so the tabs read as four
     different products, not interchangeable wallpaper.

     Click "Hold to talk" to watch a sample prompt stream into the
     chosen assistant's input, then auto-send and watch the assistant
     reply stream back. Switching tabs resets the stage. The assistant
     replies are written in-character, in the voice of the model name
     they are next to. Pure Alpine, no JS bundles.

     Voice rules: British spelling, no em-dashes, no banned-list
     terms in customer-visible copy. --}}
<div x-data="talkToAi" x-init="init()" class="ub-ai-stage" :style="`--ai-tint: ${current.color};`">

    <div class="ub-ai-tabs">
        <template x-for="(a, i) in assistants" :key="a.id">
            <button type="button"
                    class="ub-ai-tab"
                    :class="{ 'is-active': i === activeIdx }"
                    :style="`--tab-tint: ${a.color};`"
                    @click="setActive(i)">
                <span class="ub-ai-tab-glyph"
                      :style="i === activeIdx
                          ? `background: ${a.color}; border-color: ${a.color}; color: #fff;`
                          : `background: transparent; border-color: var(--border-strong); color: ${a.color};`"
                      x-text="a.initials"></span>
                <span class="ub-ai-tab-meta">
                    <span class="ub-ai-tab-name" x-text="a.name"></span>
                    <span class="ub-ai-tab-org" x-text="a.short"></span>
                </span>
            </button>
        </template>
    </div>

    <div class="ub-ai-conversation" :data-assistant="current.id">
        <div class="ub-ai-urlbar">
            <span class="dots"><i></i><i></i><i></i></span>
            <span class="url">
                <span class="lock">&#x1f512;</span>
                <span x-text="current.url"></span>
            </span>
            <span class="ub-ai-urlbar-brand" :style="`color: ${current.color};`">
                <span class="ub-ai-urlbar-glyph" :style="`background: ${current.color};`" x-text="current.initials"></span>
                <span x-text="current.name"></span>
            </span>
        </div>

        <div class="ub-ai-thread">
            {{-- Empty state: shown until the user clicks the mic. --}}
            <template x-if="phase === 'idle' && !typed">
                <div class="ub-ai-empty">
                    <span class="ub-ai-empty-glyph" :style="`background: ${current.color};`" x-text="current.initials"></span>
                    <p class="ub-ai-empty-title">
                        Talk to <span x-text="current.name"></span>.
                    </p>
                    <p class="ub-ai-empty-sub">
                        Hold the mic. Speak. Watch your words land in <span x-text="current.short"></span>'s box.
                    </p>
                </div>
            </template>

            {{-- User message bubble: appears as the prompt streams in,
                 stays after send. --}}
            <template x-if="typed.length > 0 || phase === 'listening'">
                <div class="ub-ai-msg user">
                    <div class="avatar">You</div>
                    <div class="bubble">
                        <span class="name">You &middot; spoken</span>
                        <span x-text="typed"></span>
                    </div>
                </div>
            </template>

            {{-- Assistant reply: streams in after auto-send. --}}
            <template x-if="phase === 'replying' || phase === 'done'">
                <div class="ub-ai-msg assistant" :data-font="current.bubbleFont">
                    <div class="avatar"
                         :style="`background: ${current.color}; color: #fff;`"
                         x-text="current.initials"></div>
                    <div class="bubble">
                        <span class="name" :style="`color: ${current.color};`" x-text="current.name"></span>
                        <span x-text="reply"></span>
                        <template x-if="phase === 'replying'"><span class="ub-ai-cursor"></span></template>
                    </div>
                </div>
            </template>
        </div>

        <div class="ub-ai-input-wrap" :class="{ 'is-listening': phase === 'listening' }">
            <div class="ub-ai-input-text" :data-placeholder="current.placeholder">
                <span x-text="(phase === 'listening' || (typed && phase !== 'replying' && phase !== 'done')) ? typed : ''"></span>
                <template x-if="phase === 'listening'"><span class="ub-ai-cursor"></span></template>
            </div>
            <button type="button"
                    class="ub-ai-send-btn"
                    :style="`background: ${current.color};`"
                    aria-label="Send"
                    x-text="current.send"></button>
            <button type="button"
                    class="ub-ai-mic-btn"
                    :class="{ 'is-listening': phase === 'listening' }"
                    @click="phase === 'listening' ? cancel() : startListening()">
                <span class="led"></span>
                <span x-text="phase === 'listening' ? 'Listening...' : 'Hold to talk'"></span>
                <span class="kbd">Ctrl Space</span>
            </button>
        </div>
    </div>
</div>

@once
@push('scripts')
<script>
    document.addEventListener('alpine:init', () => {
        window.Alpine.data('talkToAi', () => ({
            // Each assistant has visibly distinct chrome.
            assistants: [
                {
                    id: 'claude',  name: 'Claude',  short: 'Anthropic', initials: '✱', color: '#D97757',
                    url: 'claude.ai/new',
                    placeholder: 'Reply to Claude...',
                    send: 'Send',
                    bubbleFont: 'serif',
                    reply: "Yes, here's a draft. I've mirrored Marcus's framing on margin risk and pulled the three figures from yesterday's call. Want me to soften the close?",
                },
                {
                    id: 'chatgpt', name: 'ChatGPT', short: 'OpenAI',    initials: '✦', color: '#10A37F',
                    url: 'chatgpt.com',
                    placeholder: 'Message ChatGPT',
                    send: '↑',
                    bubbleFont: 'sans',
                    reply: "Sure! Here is a tight reply that acknowledges the variance, restates the new £4.2M forecast, and proposes the Friday review you mentioned.\n\nWant me to tighten the tone or add the Q2 context?",
                },
                {
                    id: 'gemini',  name: 'Gemini',  short: 'Google',    initials: '◈', color: '#4F8DF7',
                    url: 'gemini.google.com',
                    placeholder: 'Ask Gemini',
                    send: '→',
                    bubbleFont: 'sans',
                    reply: "Drafted. The reply leads with the revised number, references the Q2 over-delivery, and ends with two clarifying questions for Marcus on commit timing.",
                },
                {
                    id: 'copilot', name: 'Copilot', short: 'Microsoft', initials: '◆', color: '#0E62F0',
                    url: 'copilot.microsoft.com',
                    placeholder: 'Ask me anything...',
                    send: '▶',
                    bubbleFont: 'sans',
                    reply: "Here's a concise draft. I kept it on-record, used your usual sign-off, and surfaced the three Q3 figures straight from the call notes.",
                },
            ],
            prompt: "Hey, could you draft a tight reply to Marcus about the Q3 forecast revision, and pull in the figures from yesterday's call?",

            activeIdx: 0,
            phase: 'idle', // idle | listening | replying | done
            typed: '',
            reply: '',
            _typeTimer: null,
            _replyTimer: null,
            _typeI: 0,
            _replyI: 0,

            get current() { return this.assistants[this.activeIdx]; },

            init() {
                // Stage starts idle. No timers run until the user clicks Hold to talk.
            },
            destroy() { this.cancel(); },

            setActive(i) {
                this.cancel();
                this.activeIdx = i;
            },

            cancel() {
                if (this._typeTimer)  clearInterval(this._typeTimer);
                if (this._replyTimer) clearInterval(this._replyTimer);
                this._typeTimer = this._replyTimer = null;
                this._typeI = 0;
                this._replyI = 0;
                this.typed = '';
                this.reply = '';
                this.phase = 'idle';
            },

            startListening() {
                this.cancel();
                this.phase = 'listening';
                this._typeTimer = setInterval(() => {
                    if (this._typeI >= this.prompt.length) {
                        clearInterval(this._typeTimer);
                        this._typeTimer = null;
                        // Auto-send 600ms after the prompt finishes streaming.
                        setTimeout(() => this.sendPrompt(), 600);
                        return;
                    }
                    this._typeI += Math.random() > 0.65 ? 2 : 1;
                    this.typed = this.prompt.slice(0, Math.min(this._typeI, this.prompt.length));
                }, 30);
            },

            sendPrompt() {
                this.phase = 'replying';
                const replyText = this.current.reply;
                this._replyTimer = setInterval(() => {
                    if (this._replyI >= replyText.length) {
                        clearInterval(this._replyTimer);
                        this._replyTimer = null;
                        this.phase = 'done';
                        return;
                    }
                    this._replyI += Math.random() > 0.5 ? 2 : 1;
                    this.reply = replyText.slice(0, Math.min(this._replyI, replyText.length));
                }, 18);
            },
        }));
    });
</script>
@endpush
@endonce
