{{-- AppMock: clickable mock of the Utterbar shell. Five sidebar
     panes (Dashboard / History / Vocabulary / Modes / Settings),
     each rendered with real DOM rather than a static screenshot, so
     the user can click around and feel how the real app is laid out.

     Voice rules: British spelling, no em-dashes, no banned-word-list
     terms in customer copy. Sample data is illustrative; numbers are
     not telemetry from a real session. The titlebar shows the active
     pane name without an em-dash separator. --}}
<div x-data="appMock" x-init="init()" class="ub-appmock">

    <div class="ub-appmock-titlebar">
        <div class="lights"><span></span><span></span><span></span></div>
        <div class="title">Utterbar &middot; <span x-text="panes[activeIdx]"></span></div>
    </div>

    <div class="ub-appmock-sidebar">
        <div class="section-label">Workspace</div>
        <template x-for="(p, i) in panes" :key="p">
            <div class="ub-appmock-nav"
                :class="{ 'is-active': i === activeIdx }"
                 @click="activeIdx = i">
                <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                    <g x-show="p === 'Dashboard'"><rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/></g>
                    <g x-show="p === 'History'"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></g>
                    <g x-show="p === 'Vocabulary'"><path d="M4 7h12M4 12h12M4 17h8"/><path d="M19 7l2 2-2 2"/></g>
                    <g x-show="p === 'Modes'"><path d="M4 6h12M4 12h16M4 18h8"/></g>
                    <g x-show="p === 'Settings'"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1A1.7 1.7 0 0 0 9 19.4a1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1A1.7 1.7 0 0 0 4.6 9a1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/></g>
                </svg>
                <span x-text="p"></span>
            </div>
        </template>
    </div>

    <div class="ub-appmock-content">

        {{-- Dashboard pane --}}
        <template x-if="activeIdx === 0">
            <div class="ub-appmock-pane">
                <h4>Today</h4>
                <p class="muted">36 sessions, all on-device</p>
                <div class="ub-appmock-stats">
                    <div class="stat">
                        <div class="label">Words dictated</div>
                        <div class="value cyan" x-text="wordsToday.toLocaleString()"></div>
                        <div class="delta">+312 vs avg</div>
                    </div>
                    <div class="stat">
                        <div class="label">Sessions</div>
                        <div class="value">36</div>
                        <div class="delta">across 9 apps</div>
                    </div>
                    <div class="stat">
                        <div class="label">Time saved</div>
                        <div class="value">42 min</div>
                        <div class="delta">vs typing</div>
                    </div>
                </div>
                <div style="margin-top: 22px;">
                    <div class="ub-appmock-section-label">Recent</div>
                    <div class="ub-appmock-list">
                        <template x-for="(h, i) in history.slice(0, 3)" :key="i">
                            <div class="ub-appmock-list-item" :class="{ 'is-active': i === 0 }">
                                <span class="time" x-text="h.time"></span>
                                <span class="body" x-text="h.body"></span>
                                <span class="meta" x-text="h.meta"></span>
                            </div>
                        </template>
                    </div>
                </div>
            </div>
        </template>

        {{-- History pane --}}
        <template x-if="activeIdx === 1">
            <div class="ub-appmock-pane">
                <h4>History</h4>
                <p class="muted" x-text="`${history.length} sessions today, stored on this machine`"></p>
                <div class="ub-appmock-list" style="margin-top: 18px;">
                    <template x-for="(h, i) in history" :key="i">
                        <div class="ub-appmock-list-item" :class="{ 'is-active': i === 0 }">
                            <span class="time" x-text="h.time"></span>
                            <span class="body" x-text="h.body"></span>
                            <span class="meta" x-text="h.meta"></span>
                        </div>
                    </template>
                </div>
            </div>
        </template>

        {{-- Vocabulary pane --}}
        <template x-if="activeIdx === 2">
            <div class="ub-appmock-pane">
                <h4>Vocabulary</h4>
                <p class="muted" x-text="`${vocab.length} replacements, applied after transcription`"></p>
                <div style="margin-top: 18px;">
                    <template x-for="(v, i) in vocab" :key="i">
                        <div class="ub-appmock-vocab-row">
                            <span class="from" x-text="`&quot;${v.from}&quot;`"></span>
                            <span class="arrow">&rarr;</span>
                            <span class="to" x-text="v.to"></span>
                        </div>
                    </template>
                </div>
            </div>
        </template>

        {{-- Modes pane --}}
        <template x-if="activeIdx === 3">
            <div class="ub-appmock-pane">
                <h4>Modes</h4>
                <p class="muted">Switch by hotkey or in the bar</p>
                <div class="ub-appmock-list" style="margin-top: 18px;">
                    <template x-for="(m, i) in modes" :key="i">
                        <div class="ub-appmock-list-item is-2col"
                             :class="{ 'is-active': m.active }">
                            <div>
                                <div class="mode-name" x-text="m.name"></div>
                                <div class="mode-sub" x-text="m.sub"></div>
                            </div>
                            <span class="meta" x-text="m.active ? 'ACTIVE' : ''"></span>
                        </div>
                    </template>
                </div>
            </div>
        </template>

        {{-- Settings pane --}}
        <template x-if="activeIdx === 4">
            <div class="ub-appmock-pane">
                <h4>Settings</h4>
                <p class="muted">Hotkey, microphone, models, privacy</p>
                <div class="ub-appmock-list" style="margin-top: 18px;">
                    <template x-for="(row, i) in settings" :key="i">
                        <div class="ub-appmock-list-item is-kv">
                            <span class="time" x-text="row.k"></span>
                            <span class="body" x-text="row.v"></span>
                        </div>
                    </template>
                </div>
            </div>
        </template>

    </div>
</div>

@once
@push('scripts')
<script>
    document.addEventListener('alpine:init', () => {
        window.Alpine.data('appMock', () => ({
            panes: ['Dashboard', 'History', 'Vocabulary', 'Modes', 'Settings'],
            activeIdx: 0,
            wordsToday: 4824,
            _statTimer: null,

            history: [
                { time: '14:08', body: 'Tell me about the proposal Marcus sent yesterday and pull in...', meta: 'Claude · 38w' },
                { time: '13:51', body: "Reply to the supplier explaining we're pausing the order...", meta: 'Mail · 52w' },
                { time: '13:30', body: 'Summarise the launch retro: three things, owners, dates.',     meta: 'Linear · 44w' },
                { time: '12:14', body: 'Notes from the quick chat with Priya about the rollout plan.', meta: 'Notes · 81w' },
                { time: '11:02', body: 'Draft Slack to the team: Friday review moved to Monday.',       meta: 'Slack · 22w' },
                { time: '10:38', body: 'Voice memo: idea for the onboarding email sequence.',           meta: 'Notes · 96w' },
            ],
            vocab: [
                { from: 'j parry',  to: 'Jay Parry' },
                { from: 'rdt',      to: 'RDT' },
                { from: 'claude',   to: 'Claude' },
                { from: 'tooltip',  to: 'Tooltip' },
                { from: 'directml', to: 'DirectML' },
            ],
            modes: [
                { name: 'Plain',      sub: 'Verbatim, no edits',         active: true  },
                { name: 'Chat voice', sub: 'Punctuation, fillers cleaned' },
                { name: 'Email',      sub: 'Cleanup, greeting fix' },
                { name: 'Code',       sub: 'Symbols, no prose punctuation' },
                { name: 'Meeting',    sub: 'Verbatim, capitalised speakers' },
            ],
            settings: [
                { k: 'HOTKEY',     v: 'Ctrl + Space' },
                { k: 'MICROPHONE', v: 'Realtek HD Audio (default)' },
                { k: 'ENGINE',     v: 'Parakeet · CPU' },
                { k: 'HISTORY',    v: 'On-device, 30 days' },
                { k: 'TELEMETRY',  v: 'Disabled' },
            ],

            init() {
                // Slowly drift the words-today counter so the dashboard
                // feels alive without burning CPU.
                this._statTimer = setInterval(() => {
                    this.wordsToday = 4824 + Math.floor(Math.random() * 320);
                }, 1700);
            },
            destroy() {
                if (this._statTimer) clearInterval(this._statTimer);
                this._statTimer = null;
            },
        }));
    });
</script>
@endpush
@endonce
